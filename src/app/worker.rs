use super::history::LoadedHistory;
use super::{describe, scenery, App};
use crate::{
    engine::{self, Outcome},
    model::{Job, Record},
};
use chrono::Utc;
use eframe::egui;
use std::{
    collections::VecDeque,
    panic::{self, AssertUnwindSafe},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub(super) enum WorkerEvent {
    Deleted {
        error: Option<String>,
        request: Uuid,
        result: Result<LoadedHistory, String>,
    },
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

pub(super) enum Operation {
    Backup,
    Restore(String),
}

pub(super) struct Worker {
    pub(super) queue: VecDeque<Uuid>,
    pub(super) active: Option<Uuid>,
    pub(super) restoring: bool,
    pub(super) activity_until: Option<Instant>,
    pub(super) recent_activity: Option<(Uuid, &'static str)>,
    pub(super) progress: String,
    pub(super) tx: mpsc::Sender<WorkerEvent>,
    pub(super) rx: mpsc::Receiver<WorkerEvent>,
}

impl Default for Worker {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            queue: VecDeque::new(),
            active: None,
            restoring: false,
            activity_until: None,
            recent_activity: None,
            progress: String::new(),
            tx,
            rx,
        }
    }
}

impl Worker {
    pub(super) fn is_idle(&self) -> bool {
        self.active.is_none() && self.queue.is_empty()
    }
    /// Running, restoring, or queued work for a job. Finished work stays labelled as long as
    /// the footer activity, so quick checks do not flash the label.
    pub(super) fn activity_label(&self, id: Uuid) -> Option<&'static str> {
        if self.active == Some(id) {
            return Some(if self.restoring {
                "Restoring"
            } else {
                "Running"
            });
        }
        if self.queue.contains(&id) {
            return Some("Queued");
        }
        let recent = self
            .activity_until
            .is_some_and(|until| until > Instant::now());
        self.recent_activity
            .filter(|(job, _)| recent && *job == id)
            .map(|(_, label)| label)
    }

    /// Runs `task` on a background thread and delivers its event to the UI thread.
    pub(super) fn spawn_task(
        &self,
        ctx: &egui::Context,
        task: impl FnOnce() -> WorkerEvent + Send + 'static,
    ) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            // Sending only fails once the app has shut down.
            let _ = tx.send(task());
            ctx.request_repaint();
        });
    }

    pub(super) fn start(&mut self, job: Job, operation: Operation, ctx: &egui::Context) {
        self.active = Some(job.id);
        self.activity_until = Some(Instant::now() + Duration::from_millis(750));
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
                    let _ = progress_tx.send(WorkerEvent::Progress(message));
                    progress_ctx.request_repaint();
                }
            });
            let result = run_operation(&job, operation, &sender);
            // Flush all progress before reporting completion.
            drop(sender);
            let _ = relay.join();
            WorkerEvent::Finished {
                job: job.id,
                result,
            }
        });
    }
}

impl App {
    pub(super) fn enqueue(&mut self, id: Uuid) {
        if !self.scheduling_paused()
            && self.worker.active != Some(id)
            && !self.worker.queue.contains(&id)
        {
            self.worker.queue.push_back(id);
        }
    }

    pub(super) fn remove_pending(&mut self, id: Uuid) {
        self.worker.queue.retain(|queued| *queued != id);
    }

    pub(super) fn finish_operation(
        &mut self,
        id: Uuid,
        result: Result<Outcome, String>,
        ctx: &egui::Context,
    ) {
        use scenery::saya::{self, Action};
        match &result {
            Err(_) => saya::trigger(ctx, Action::Failed),
            Ok(_) if self.worker.restoring => saya::trigger(ctx, Action::RestoreCompleted),
            Ok(_) => {}
        }
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
        self.worker.recent_activity = self.worker.activity_label(id).map(|label| (id, label));
        self.worker.active = None;
        self.worker.restoring = false;
        self.worker.activity_until = Some(Instant::now() + Duration::from_millis(750));
        self.worker.progress.clear();
        self.persist();
        self.reload_summary(id, ctx);
    }

    pub(super) fn queue_due_jobs(&mut self) {
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

    pub(super) fn start_next_queued(&mut self, ctx: &egui::Context) {
        if self.worker.active.is_some() || self.scheduling_paused() {
            return;
        }
        let Some(id) = self.worker.queue.pop_front() else {
            return;
        };
        if let Some(job) = self.job(id).cloned() {
            // Reload the visible history after the worker finishes, including retention changes.
            self.history.view = None;
            self.worker.start(job, Operation::Backup, ctx);
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
