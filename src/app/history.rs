use super::worker::Operation;
use super::{describe, App, WorkerEvent};
use crate::{
    engine::{self, history::Backup},
    model::{Job, Record},
};
use chrono::Utc;
use eframe::egui;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub(super) struct LoadedHistory {
    pub(super) backups: Vec<Backup>,
    /// Canonical source path, shown so the user sees exactly which folder gets replaced.
    pub(super) source: Option<PathBuf>,
}

/// Backup records of one job, valid only while the job keeps this destination.
pub(super) struct HistorySummary {
    pub(super) target: PathBuf,
    pub(super) records: Result<Vec<Record>, String>,
}

pub(super) struct PendingSummary {
    pub(super) target: PathBuf,
    pub(super) request: Uuid,
}

pub(super) struct HistoryViewState {
    /// Ignore asynchronous results from a previous history view.
    pub(super) request: Uuid,
    pub(super) job: Job,
    pub(super) backups: Option<Result<Vec<Backup>, String>>,
    pub(super) selected: Option<String>,
    pub(super) source_display: Option<PathBuf>,
    pub(super) saving: bool,
    pub(super) confirming_restore: bool,
    pub(super) error: String,
}

impl HistoryViewState {
    pub(super) fn new(job: Job) -> Self {
        Self {
            request: Uuid::new_v4(),
            job,
            backups: None,
            selected: None,
            source_display: None,
            saving: false,
            confirming_restore: false,
            error: String::new(),
        }
    }

    /// The selected backup, if it can be restored right now.
    pub(super) fn restorable_selection(&self) -> Option<String> {
        if self.saving || self.source_display.is_none() {
            return None;
        }
        let name = self.selected.as_ref()?;
        let backups = self.backups.as_ref()?.as_ref().ok()?;
        backups
            .iter()
            .any(|backup| backup.available && backup.record.name == *name)
            .then(|| name.clone())
    }
}

#[derive(Default)]
pub(super) struct HistoryState {
    pub(super) view: Option<HistoryViewState>,
    pub(super) cache: HashMap<Uuid, HistorySummary>,
    pub(super) requests: HashMap<Uuid, PendingSummary>,
}

impl App {
    pub(super) fn open_history(&mut self, id: Uuid, ctx: &egui::Context) {
        if !self.worker.is_idle() || self.history.view.is_some() {
            return;
        }
        let Some(job) = self.job(id).cloned() else {
            return;
        };
        if job.target.as_os_str().is_empty() {
            return;
        }
        let dialog = HistoryViewState::new(job.clone());
        let request = dialog.request;
        self.history.view = Some(dialog);
        self.worker
            .spawn_task(ctx, move || WorkerEvent::HistoryLoaded {
                request,
                result: load_history(&job).map_err(describe),
            });
    }

    /// Disables the job and saves that before touching the source, so a crash cannot re-enable it.
    pub(super) fn confirm_restore(&mut self, ctx: &egui::Context) {
        if !self.worker.is_idle() {
            return;
        }
        let Some(dialog) = &self.history.view else {
            return;
        };
        let Some(name) = dialog.restorable_selection() else {
            return;
        };
        let id = dialog.job.id;
        let Some(job) = self.job_mut(id) else {
            return;
        };
        job.enabled = false;
        job.reschedule(Utc::now());
        let job = job.clone();
        if !self.persist() {
            return;
        }
        self.history.view = None;
        self.worker.start(job, Operation::Restore(name), ctx);
    }

    pub(super) fn delete_backup(&mut self, name: String, ctx: &egui::Context) {
        if !self.worker.is_idle() {
            return;
        }
        let Some(history) = &mut self.history.view else {
            return;
        };
        if history.saving {
            return;
        }
        history.saving = true;
        history.error.clear();
        let request = history.request;
        let job = history.job.clone();
        self.worker.spawn_task(ctx, move || {
            let deletion = engine::history::delete(&job, &name);
            let result = load_history(&job).map_err(describe);
            WorkerEvent::Deleted {
                request,
                error: deletion.err().map(describe),
                result,
            }
        });
    }

    pub(super) fn save_protection(&mut self, name: String, protected: bool, ctx: &egui::Context) {
        let Some(dialog) = &mut self.history.view else {
            return;
        };
        if dialog.saving {
            return;
        }
        dialog.saving = true;
        dialog.error.clear();
        let request = dialog.request;
        let job = dialog.job.clone();
        self.worker.spawn_task(ctx, move || {
            let result = engine::history::set_protected(&job, &name, protected).map_err(describe);
            WorkerEvent::ProtectionSaved {
                request,
                name,
                protected,
                result,
            }
        });
    }

    /// Loads the retention summary unless one for the current destination exists or is pending.
    pub(super) fn ensure_summary(&mut self, id: Uuid, ctx: &egui::Context) {
        let cached = self.job(id).is_some_and(|job| {
            self.history
                .cache
                .get(&id)
                .is_some_and(|cache| cache.target == job.target)
        });
        if !cached {
            self.reload_summary(id, ctx);
        }
    }

    pub(super) fn reload_summary(&mut self, id: Uuid, ctx: &egui::Context) {
        let Some(job) = self.job(id).cloned() else {
            return;
        };
        let pending = self
            .history
            .requests
            .get(&id)
            .is_some_and(|pending| pending.target == job.target);
        if job.target.as_os_str().is_empty() || pending {
            return;
        }
        let request = Uuid::new_v4();
        let target = job.target.clone();
        self.history.requests.insert(
            id,
            PendingSummary {
                target: target.clone(),
                request,
            },
        );
        self.worker
            .spawn_task(ctx, move || WorkerEvent::SummaryLoaded {
                job: id,
                target,
                request,
                records: engine::history::list(&job)
                    .map(|backups| records_of(&backups))
                    .map_err(describe),
            });
    }

    /// Fresh records supersede any summary still being loaded.
    pub(super) fn store_summary(&mut self, id: Uuid, target: PathBuf, records: Vec<Record>) {
        self.history.requests.remove(&id);
        self.history.cache.insert(
            id,
            HistorySummary {
                target,
                records: Ok(records),
            },
        );
    }

    pub(super) fn pending_history(&mut self, request: Uuid) -> Option<&mut HistoryViewState> {
        self.history
            .view
            .as_mut()
            .filter(|dialog| dialog.request == request)
    }

    pub(super) fn history_loaded(&mut self, request: Uuid, result: Result<LoadedHistory, String>) {
        let Some(dialog) = self.pending_history(request) else {
            return;
        };
        let loaded = match result {
            Ok(loaded) => loaded,
            Err(error) => {
                dialog.backups = Some(Err(error));
                return;
            }
        };
        let records = records_of(&loaded.backups);
        dialog.source_display = loaded.source;
        dialog.backups = Some(Ok(loaded.backups));
        let (id, target) = (dialog.job.id, dialog.job.target.clone());
        self.store_summary(id, target, records);
    }

    pub(super) fn protection_saved(
        &mut self,
        request: Uuid,
        name: &str,
        protected: bool,
        result: Result<(), String>,
    ) {
        let Some(dialog) = self.pending_history(request) else {
            return;
        };
        dialog.saving = false;
        if let Err(error) = result {
            dialog.error = error;
            return;
        }
        let Some(Ok(backups)) = &mut dialog.backups else {
            return;
        };
        if let Some(backup) = backups.iter_mut().find(|backup| backup.record.name == name) {
            backup.record.protected = protected;
        }
        let records = records_of(backups);
        let (id, target) = (dialog.job.id, dialog.job.target.clone());
        self.store_summary(id, target, records);
    }

    pub(super) fn summary_loaded(
        &mut self,
        id: Uuid,
        target: PathBuf,
        request: Uuid,
        records: Result<Vec<Record>, String>,
    ) {
        let current = self
            .history
            .requests
            .get(&id)
            .is_some_and(|pending| pending.request == request);
        if !current {
            return;
        }
        self.history.requests.remove(&id);
        if self.job(id).is_some_and(|job| job.target == target) {
            self.history
                .cache
                .insert(id, HistorySummary { target, records });
        }
    }

    pub(in crate::app) fn close_inactive_history(&mut self) {
        let visible = self.page.is_backups();
        if self.history.view.as_ref().is_some_and(|history| {
            !history.saving && (!visible || self.page.selected_job() != Some(history.job.id))
        }) {
            self.history.view = None;
        }
    }
}

fn load_history(job: &Job) -> anyhow::Result<LoadedHistory> {
    let backups = engine::history::list(job)?;
    let source = std::fs::canonicalize(&job.source)
        .ok()
        .filter(|source| source.is_dir());
    Ok(LoadedHistory { backups, source })
}

pub(super) fn records_of(backups: &[Backup]) -> Vec<Record> {
    backups.iter().map(|backup| backup.record.clone()).collect()
}
