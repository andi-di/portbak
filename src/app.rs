mod editor;
mod history_view;
mod pixel;
mod settings_view;
mod skins;
mod sprites;
mod style;
mod ui;
mod widgets;

use crate::engine::{self, history::Backup, Outcome};
use crate::model::{Config, Job, Record};
use chrono::Utc;
use eframe::egui;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    panic::{self, AssertUnwindSafe},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use uuid::Uuid;

const AUTOSAVE_DELAY: Duration = Duration::from_millis(600);
const TICK_INTERVAL: Duration = Duration::from_millis(500);

enum Event {
    Progress(String),
    Finished {
        job: Uuid,
        result: Result<Outcome, String>,
    },
    HistoryLoaded {
        request: Uuid,
        result: Result<LoadedHistory, String>,
    },
    ProtectionSaved {
        request: Uuid,
        name: String,
        protected: bool,
        result: Result<(), String>,
    },
    SummaryLoaded {
        job: Uuid,
        target: PathBuf,
        request: Uuid,
        records: Result<Vec<Record>, String>,
    },
}

struct LoadedHistory {
    backups: Vec<Backup>,
    /// Canonical source path, shown so the user sees exactly which folder gets replaced.
    source: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq)]
enum HistoryMode {
    Restore,
    Manage,
}

/// Backup records of one job, valid only while the job keeps this destination.
struct CachedHistory {
    target: PathBuf,
    records: Result<Vec<Record>, String>,
}

struct PendingSummary {
    target: PathBuf,
    request: Uuid,
}

struct HistoryDialog {
    /// Identifies this dialog so results for a closed or reopened dialog are ignored.
    request: Uuid,
    job: Job,
    backups: Option<Result<Vec<Backup>, String>>,
    selected: Option<String>,
    source_display: Option<PathBuf>,
    mode: HistoryMode,
    saving: bool,
    error: String,
}

impl HistoryDialog {
    fn new(job: Job, mode: HistoryMode) -> Self {
        Self {
            request: Uuid::new_v4(),
            job,
            backups: None,
            selected: None,
            source_display: None,
            mode,
            saving: false,
            error: String::new(),
        }
    }

    /// The selected backup, if it can be restored right now.
    fn restorable_selection(&self) -> Option<String> {
        if self.mode != HistoryMode::Restore || self.saving {
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

enum Operation {
    Backup,
    Restore(String),
}

pub struct App {
    config: Config,
    path: PathBuf,
    selected: Option<Uuid>,
    queue: VecDeque<Uuid>,
    active: Option<Uuid>,
    restoring: bool,
    history_dialog: Option<HistoryDialog>,
    settings_open: bool,
    history_cache: HashMap<Uuid, CachedHistory>,
    history_requests: HashMap<Uuid, PendingSummary>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    error: String,
    progress: String,
    dirty: bool,
    last_save: Instant,
    closing: bool,
    permit_close: bool,
    /// Held for the app's lifetime so a second instance cannot share the configuration.
    _lock: File,
}

impl App {
    pub fn new(config: Config, path: PathBuf, lock: File) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            selected: config.jobs.first().map(|job| job.id),
            config,
            path,
            queue: VecDeque::new(),
            active: None,
            restoring: false,
            history_dialog: None,
            settings_open: false,
            history_cache: HashMap::new(),
            history_requests: HashMap::new(),
            tx,
            rx,
            error: String::new(),
            progress: String::new(),
            dirty: false,
            last_save: Instant::now(),
            closing: false,
            permit_close: false,
            _lock: lock,
        }
    }

    fn job(&self, id: Uuid) -> Option<&Job> {
        self.config.jobs.iter().find(|job| job.id == id)
    }

    fn job_mut(&mut self, id: Uuid) -> Option<&mut Job> {
        self.config.jobs.iter_mut().find(|job| job.id == id)
    }

    /// No worker runs and nothing waits; required before history can be opened or restored.
    fn is_idle(&self) -> bool {
        self.active.is_none() && self.queue.is_empty()
    }

    /// A restore or an open history dialog must not race with scheduled backups.
    fn scheduling_paused(&self) -> bool {
        self.restoring || self.history_dialog.is_some()
    }

    fn is_busy(&self, id: Uuid) -> bool {
        self.active == Some(id) || self.queue.contains(&id) || self.scheduling_paused()
    }

    /// Returns whether the configuration is now saved; failures stay dirty and are retried.
    fn persist(&mut self) -> bool {
        self.last_save = Instant::now();
        match engine::save(&self.path, &self.config) {
            Ok(()) => {
                self.dirty = false;
                self.error.clear();
                true
            }
            Err(error) => {
                self.dirty = true;
                self.error = error.to_string();
                false
            }
        }
    }

    fn add_job(&mut self) {
        let job = Job::default();
        self.selected = Some(job.id);
        self.settings_open = false;
        self.config.jobs.push(job);
    }

    fn remove_job(&mut self, id: Uuid) {
        self.history_cache.remove(&id);
        self.history_requests.remove(&id);
        self.config.jobs.retain(|job| job.id != id);
        self.selected = self.config.jobs.first().map(|job| job.id);
    }

    fn enqueue(&mut self, id: Uuid) {
        if !self.scheduling_paused() && self.active != Some(id) && !self.queue.contains(&id) {
            self.queue.push_back(id);
        }
    }

    fn remove_pending(&mut self, id: Uuid) {
        self.queue.retain(|queued| *queued != id);
    }

    fn quit(&mut self, ctx: &egui::Context) {
        if self.persist() {
            self.permit_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Runs `task` on a background thread and delivers its event to the UI thread.
    fn spawn_task(&self, ctx: &egui::Context, task: impl FnOnce() -> Event + Send + 'static) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            // Sending only fails once the app has shut down.
            let _ = tx.send(task());
            ctx.request_repaint();
        });
    }

    fn start_worker(&mut self, job: Job, operation: Operation, ctx: &egui::Context) {
        self.active = Some(job.id);
        self.restoring = matches!(operation, Operation::Restore(_));
        self.progress = if self.restoring {
            "Preparing restore …"
        } else {
            "Preparing backup …"
        }
        .into();
        let progress_tx = self.tx.clone();
        let progress_ctx = ctx.clone();
        self.spawn_task(ctx, move || {
            let (sender, receiver) = mpsc::channel();
            let relay = std::thread::spawn(move || {
                for message in receiver {
                    let _ = progress_tx.send(Event::Progress(message));
                    progress_ctx.request_repaint();
                }
            });
            let result = run_operation(&job, operation, &sender);
            // Flush all progress before reporting completion.
            drop(sender);
            let _ = relay.join();
            Event::Finished {
                job: job.id,
                result,
            }
        });
    }

    fn open_history(&mut self, id: Uuid, mode: HistoryMode, ctx: &egui::Context) {
        if !self.is_idle() || self.history_dialog.is_some() {
            return;
        }
        let Some(job) = self.job(id).cloned() else {
            return;
        };
        let dialog = HistoryDialog::new(job.clone(), mode);
        let request = dialog.request;
        self.history_dialog = Some(dialog);
        self.spawn_task(ctx, move || Event::HistoryLoaded {
            request,
            result: load_history(&job, mode).map_err(describe),
        });
    }

    /// Disables the job and saves that before touching the source, so a crash cannot re-enable it.
    fn confirm_restore(&mut self, ctx: &egui::Context) {
        if !self.is_idle() {
            return;
        }
        let Some(dialog) = &self.history_dialog else {
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
        self.history_dialog = None;
        self.start_worker(job, Operation::Restore(name), ctx);
    }

    fn save_protection(&mut self, name: String, protected: bool, ctx: &egui::Context) {
        let Some(dialog) = &mut self.history_dialog else {
            return;
        };
        if dialog.mode != HistoryMode::Manage || dialog.saving {
            return;
        }
        dialog.saving = true;
        dialog.error.clear();
        let request = dialog.request;
        let job = dialog.job.clone();
        self.spawn_task(ctx, move || {
            let result = engine::history::set_protected(&job, &name, protected).map_err(describe);
            Event::ProtectionSaved {
                request,
                name,
                protected,
                result,
            }
        });
    }

    /// Loads the retention summary unless one for the current destination exists or is pending.
    fn ensure_summary(&mut self, id: Uuid, ctx: &egui::Context) {
        let cached = self.job(id).is_some_and(|job| {
            self.history_cache
                .get(&id)
                .is_some_and(|cache| cache.target == job.target)
        });
        if !cached {
            self.reload_summary(id, ctx);
        }
    }

    fn reload_summary(&mut self, id: Uuid, ctx: &egui::Context) {
        let Some(job) = self.job(id).cloned() else {
            return;
        };
        let pending = self
            .history_requests
            .get(&id)
            .is_some_and(|pending| pending.target == job.target);
        if job.target.as_os_str().is_empty() || pending {
            return;
        }
        let request = Uuid::new_v4();
        let target = job.target.clone();
        self.history_requests.insert(
            id,
            PendingSummary {
                target: target.clone(),
                request,
            },
        );
        self.spawn_task(ctx, move || Event::SummaryLoaded {
            job: id,
            target,
            request,
            records: engine::history::list(&job)
                .map(|backups| records_of(&backups))
                .map_err(describe),
        });
    }

    /// Fresh records supersede any summary still being loaded.
    fn store_summary(&mut self, id: Uuid, target: PathBuf, records: Vec<Record>) {
        self.history_requests.remove(&id);
        self.history_cache.insert(
            id,
            CachedHistory {
                target,
                records: Ok(records),
            },
        );
    }

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            self.handle(event, ctx);
        }
        self.queue_due_jobs();
        self.start_next_queued(ctx);
        if self.dirty && self.last_save.elapsed() > AUTOSAVE_DELAY {
            self.persist();
        }
        ctx.request_repaint_after(TICK_INTERVAL);
    }

    fn handle(&mut self, event: Event, ctx: &egui::Context) {
        match event {
            Event::Progress(message) => self.progress = message,
            Event::Finished { job, result } => self.finish_operation(job, result, ctx),
            Event::HistoryLoaded { request, result } => self.history_loaded(request, result),
            Event::ProtectionSaved {
                request,
                name,
                protected,
                result,
            } => self.protection_saved(request, &name, protected, result),
            Event::SummaryLoaded {
                job,
                target,
                request,
                records,
            } => self.summary_loaded(job, target, request, records),
        }
    }

    fn finish_operation(&mut self, id: Uuid, result: Result<Outcome, String>, ctx: &egui::Context) {
        if let Some(job) = self.job_mut(id) {
            match result {
                Ok(outcome) => {
                    if outcome.created {
                        job.last_success = Some(Utc::now());
                    }
                    job.status = outcome.message;
                }
                Err(error) => job.status = format!("Error: {error}"),
            }
        }
        self.active = None;
        self.restoring = false;
        self.progress.clear();
        self.persist();
        self.reload_summary(id, ctx);
    }

    fn open_dialog(&mut self, request: Uuid) -> Option<&mut HistoryDialog> {
        self.history_dialog
            .as_mut()
            .filter(|dialog| dialog.request == request)
    }

    fn history_loaded(&mut self, request: Uuid, result: Result<LoadedHistory, String>) {
        let Some(dialog) = self.open_dialog(request) else {
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

    fn protection_saved(
        &mut self,
        request: Uuid,
        name: &str,
        protected: bool,
        result: Result<(), String>,
    ) {
        let Some(dialog) = self.open_dialog(request) else {
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

    fn summary_loaded(
        &mut self,
        id: Uuid,
        target: PathBuf,
        request: Uuid,
        records: Result<Vec<Record>, String>,
    ) {
        let current = self
            .history_requests
            .get(&id)
            .is_some_and(|pending| pending.request == request);
        if !current {
            return;
        }
        self.history_requests.remove(&id);
        if self.job(id).is_some_and(|job| job.target == target) {
            self.history_cache
                .insert(id, CachedHistory { target, records });
        }
    }

    fn queue_due_jobs(&mut self) {
        if self.scheduling_paused() {
            return;
        }
        let now = Utc::now();
        let mut due = Vec::new();
        for job in &mut self.config.jobs {
            let previous = job.next_run;
            if job.take_due(now) {
                due.push(job.id);
            }
            self.dirty |= previous != job.next_run;
        }
        for id in due {
            self.enqueue(id);
        }
    }

    fn start_next_queued(&mut self, ctx: &egui::Context) {
        if self.active.is_some() || self.history_dialog.is_some() {
            return;
        }
        let Some(id) = self.queue.pop_front() else {
            return;
        };
        if let Some(job) = self.job(id).cloned() {
            self.start_worker(job, Operation::Backup, ctx);
        }
    }
}

fn run_operation(
    job: &Job,
    operation: Operation,
    progress: &mpsc::Sender<String>,
) -> Result<Outcome, String> {
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| match operation {
        Operation::Backup => engine::run(job, progress),
        Operation::Restore(name) => engine::restore::run(job, &name, progress),
    }));
    match outcome {
        Ok(result) => result.map_err(describe),
        Err(_) => Err(
            "Unexpected error in worker; check for restore recovery folders beside the source"
                .into(),
        ),
    }
}

fn load_history(job: &Job, mode: HistoryMode) -> anyhow::Result<LoadedHistory> {
    let backups = engine::history::list(job)?;
    let source = match mode {
        HistoryMode::Restore => Some(std::fs::canonicalize(&job.source)?),
        HistoryMode::Manage => None,
    };
    Ok(LoadedHistory { backups, source })
}

fn records_of(backups: &[Backup]) -> Vec<Record> {
    backups.iter().map(|backup| backup.record.clone()).collect()
}

fn describe(error: anyhow::Error) -> String {
    format!("{error:#}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Schedule;
    use fs2::FileExt;
    use std::fs::OpenOptions;

    fn app_with(config: Config, path: PathBuf) -> App {
        App::new(config, path, tempfile::tempfile().unwrap())
    }

    /// Ticks until `done` holds or five seconds pass.
    fn tick_until(app: &mut App, context: &egui::Context, done: impl Fn(&App) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !done(app) && Instant::now() < deadline {
            app.tick(context);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn dialog(app: &App) -> &HistoryDialog {
        app.history_dialog.as_ref().unwrap()
    }

    fn restore_fixture() -> (tempfile::TempDir, App, String) {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&target).unwrap();
        std::fs::write(source.join("file"), "saved").unwrap();
        let job = Job {
            source,
            target,
            enabled: true,
            next_run: Some(Utc::now() - chrono::Duration::seconds(1)),
            last_success: Some(Utc::now()),
            ..Job::default()
        };
        engine::run(&job, &mpsc::channel().0).unwrap();
        let name = engine::history::list(&job).unwrap()[0].record.name.clone();
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let app = app_with(config, directory.path().join("config.json"));
        (directory, app, name)
    }

    fn select_restore(app: &mut App, name: &str) {
        let job = app.config.jobs[0].clone();
        let mut dialog = HistoryDialog::new(job.clone(), HistoryMode::Restore);
        dialog.source_display = Some(job.source.clone());
        dialog.backups = Some(Ok(engine::history::list(&job).unwrap()));
        dialog.selected = Some(name.to_owned());
        app.history_dialog = Some(dialog);
    }

    #[test]
    fn disabling_removes_pending_work_and_preserves_active_work() {
        let mut app = app_with(Config::default(), PathBuf::new());
        let active = Uuid::new_v4();
        let queued = Uuid::new_v4();
        let other = Uuid::new_v4();
        app.active = Some(active);
        app.enqueue(queued);
        app.enqueue(other);
        app.remove_pending(queued);
        app.remove_pending(active);
        assert_eq!(app.active, Some(active));
        assert_eq!(app.queue, VecDeque::from([other]));
    }

    #[test]
    fn history_dialog_pauses_scheduling_and_cancel_changes_nothing() {
        let (_directory, mut app, name) = restore_fixture();
        let due = app.config.jobs[0].next_run;
        select_restore(&mut app, &name);
        let context = egui::Context::default();
        app.tick(&context);
        app.enqueue(app.config.jobs[0].id);
        assert!(app.active.is_none());
        assert!(app.queue.is_empty());
        app.history_dialog = None;
        assert!(app.config.jobs[0].enabled);
        assert_eq!(app.config.jobs[0].next_run, due);
    }

    #[test]
    fn restore_confirmation_disables_job_and_preserves_backup_timestamp() {
        let (_directory, mut app, name) = restore_fixture();
        let timestamp = app.config.jobs[0].last_success;
        let source = app.config.jobs[0].source.clone();
        std::fs::write(source.join("extra"), "remove").unwrap();
        select_restore(&mut app, &name);
        let context = egui::Context::default();
        app.confirm_restore(&context);
        assert!(app.restoring);
        assert!(!app.config.jobs[0].enabled);
        assert!(app.config.jobs[0].next_run.is_none());
        app.enqueue(Uuid::new_v4());
        assert!(app.queue.is_empty());
        tick_until(&mut app, &context, |app| app.active.is_none());
        assert!(app.active.is_none());
        assert!(!app.restoring);
        assert!(app.config.jobs[0].status.starts_with("Restore complete"));
        assert!(!app.config.jobs[0].enabled);
        assert_eq!(app.config.jobs[0].last_success, timestamp);
        assert!(!source.join("extra").exists());
    }

    #[test]
    fn restore_requires_valid_selection_idle_worker_and_saved_disabled_state() {
        let (_directory, mut app, name) = restore_fixture();
        let context = egui::Context::default();
        select_restore(&mut app, &name);
        app.history_dialog.as_mut().unwrap().selected = None;
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        app.history_dialog.as_mut().unwrap().selected = Some(name);
        app.active = Some(Uuid::new_v4());
        app.confirm_restore(&context);
        assert!(app.config.jobs[0].enabled);
        app.active = None;
        app.path = app.path.join("missing/config.json");
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        assert!(app.history_dialog.is_some());
        assert!(!app.error.is_empty());
    }

    #[test]
    fn manager_saves_protection_and_reports_errors_without_changing_activation() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.history_dialog.as_mut().unwrap().mode = HistoryMode::Manage;
        let context = egui::Context::default();
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        app.save_protection(name.clone(), true, &context);
        tick_until(&mut app, &context, |app| !dialog(app).saving);
        assert!(!dialog(&app).saving);
        assert!(
            engine::history::list(&app.config.jobs[0]).unwrap()[0]
                .record
                .protected
        );
        let id = app.config.jobs[0].id;
        assert!(app.history_cache[&id].records.as_ref().unwrap()[0].protected);
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(app.config.jobs[0].target.join(".lock"))
            .unwrap();
        lock.try_lock_exclusive().unwrap();
        app.save_protection(name, false, &context);
        tick_until(&mut app, &context, |app| !dialog(app).saving);
        let dialog = dialog(&app);
        assert!(!dialog.saving);
        assert!(!dialog.error.is_empty());
        assert!(
            dialog.backups.as_ref().unwrap().as_ref().unwrap()[0]
                .record
                .protected
        );
        assert!(app.config.jobs[0].enabled);
        assert!(app.active.is_none());
    }

    #[test]
    fn manager_history_does_not_require_source_folder() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].source.push("missing");
        let context = egui::Context::default();
        app.open_history(app.config.jobs[0].id, HistoryMode::Manage, &context);
        tick_until(&mut app, &context, |app| dialog(app).backups.is_some());
        assert!(dialog(&app).backups.as_ref().unwrap().is_ok());
        assert!(app.active.is_none());
        assert!(app.config.jobs[0].enabled);
    }

    #[test]
    fn manual_checks_can_queue_disabled_jobs() {
        let job = Job::default();
        let id = job.id;
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let mut app = app_with(config, PathBuf::new());
        app.enqueue(id);
        app.enqueue(id);
        assert_eq!(app.queue, VecDeque::from([id]));
        assert!(!app.config.jobs[0].enabled);
    }

    #[test]
    fn clearing_all_weekdays_disables_job_and_drops_pending_check() {
        let job = Job {
            enabled: true,
            schedule: Schedule::Weekdays,
            weekdays: [false; 7],
            ..Job::default()
        };
        let id = job.id;
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let mut app = app_with(config, PathBuf::new());
        app.enqueue(id);
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            app.show_job_editor(ctx);
        });
        assert!(!app.config.jobs[0].enabled);
        assert!(app.config.jobs[0].next_run.is_none());
        assert!(app.queue.is_empty());
    }

    #[test]
    fn summary_for_a_changed_destination_is_discarded() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].enabled = false;
        let id = app.config.jobs[0].id;
        let context = egui::Context::default();
        app.reload_summary(id, &context);
        app.config.jobs[0].target.push("elsewhere");
        tick_until(&mut app, &context, |app| app.history_requests.is_empty());
        assert!(app.history_requests.is_empty());
        assert!(!app.history_cache.contains_key(&id));
    }
}
