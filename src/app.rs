pub(crate) mod platform;
mod rendering;
mod scenery;
mod theme;
mod views;

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

/// Backup records of one job, valid only while the job keeps this destination.
struct CachedHistory {
    target: PathBuf,
    records: Result<Vec<Record>, String>,
}

struct PendingSummary {
    target: PathBuf,
    request: Uuid,
}

struct BackupHistory {
    /// Ignore asynchronous results from a previous history view.
    request: Uuid,
    job: Job,
    backups: Option<Result<Vec<Backup>, String>>,
    selected: Option<String>,
    source_display: Option<PathBuf>,
    saving: bool,
    confirming_restore: bool,
    error: String,
}

impl BackupHistory {
    fn new(job: Job) -> Self {
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
    fn restorable_selection(&self) -> Option<String> {
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
    disabled_during_backup: bool,
    backup_history: Option<BackupHistory>,
    backups_tab: bool,
    settings_open: bool,
    settings_tab: views::settings_view::SettingsTab,
    history_cache: HashMap<Uuid, CachedHistory>,
    history_requests: HashMap<Uuid, PendingSummary>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    error: String,
    progress: String,
    dirty: bool,
    last_save: Instant,
    /// Held for the app's lifetime so a second instance cannot share the configuration.
    _lock: File,
}

impl App {
    pub fn new(mut config: Config, path: PathBuf, lock: File) -> Self {
        if let Some(app_dir) = path.parent() {
            config.resolve_destinations(app_dir);
        }
        let (tx, rx) = mpsc::channel();
        Self {
            selected: config.jobs.first().map(|job| job.id),
            config,
            path,
            queue: VecDeque::new(),
            active: None,
            restoring: false,
            disabled_during_backup: false,
            backup_history: None,
            backups_tab: false,
            settings_open: false,
            settings_tab: views::settings_view::SettingsTab::default(),
            history_cache: HashMap::new(),
            history_requests: HashMap::new(),
            tx,
            rx,
            error: String::new(),
            progress: String::new(),
            dirty: false,
            last_save: Instant::now(),
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

    /// Restoration and browsing backups must not race with scheduled backups.
    fn scheduling_paused(&self) -> bool {
        self.restoring || self.backup_history.is_some()
    }

    fn can_add_job(&self) -> bool {
        !self.restoring
            && !self
                .backup_history
                .as_ref()
                .is_some_and(|history| history.saving)
    }

    fn is_busy(&self, id: Uuid) -> bool {
        self.active == Some(id) || self.queue.contains(&id) || self.scheduling_paused()
    }

    /// Returns whether the configuration is now saved; failures stay dirty and are retried.
    fn persist(&mut self) -> bool {
        self.last_save = Instant::now();
        match engine::save(&self.path, &self.config.for_storage()) {
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
        if !self.can_add_job() {
            return;
        }
        let job = self.config.job_defaults.new_job();
        self.selected = Some(job.id);
        self.settings_open = false;
        self.backups_tab = false;
        self.backup_history = None;
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

    fn job_activation_changed(&mut self, id: Uuid, was_enabled: bool) {
        let Some(job) = self.job(id) else {
            return;
        };
        let enabled = job.enabled;
        let check_on_enable = job.check_on_enable;
        if self.active == Some(id) && !self.restoring {
            self.disabled_during_backup = was_enabled && !enabled;
        }
        if was_enabled && !enabled {
            self.remove_pending(id);
        } else if !was_enabled && enabled && check_on_enable {
            self.enqueue(id);
        }
    }

    fn remove_pending(&mut self, id: Uuid) {
        self.queue.retain(|queued| *queued != id);
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
        self.disabled_during_backup = false;
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

    fn open_history(&mut self, id: Uuid, ctx: &egui::Context) {
        if !self.is_idle() || self.backup_history.is_some() {
            return;
        }
        let Some(job) = self.job(id).cloned() else {
            return;
        };
        if job.target.as_os_str().is_empty() {
            return;
        }
        let dialog = BackupHistory::new(job.clone());
        let request = dialog.request;
        self.backup_history = Some(dialog);
        self.spawn_task(ctx, move || Event::HistoryLoaded {
            request,
            result: load_history(&job).map_err(describe),
        });
    }

    /// Disables the job and saves that before touching the source, so a crash cannot re-enable it.
    fn confirm_restore(&mut self, ctx: &egui::Context) {
        if !self.is_idle() {
            return;
        }
        let Some(dialog) = &self.backup_history else {
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
        self.backup_history = None;
        self.start_worker(job, Operation::Restore(name), ctx);
    }

    fn save_protection(&mut self, name: String, protected: bool, ctx: &egui::Context) {
        let Some(dialog) = &mut self.backup_history else {
            return;
        };
        if dialog.saving {
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
        self.disabled_during_backup = false;
        self.progress.clear();
        self.persist();
        self.reload_summary(id, ctx);
    }

    fn pending_history(&mut self, request: Uuid) -> Option<&mut BackupHistory> {
        self.backup_history
            .as_mut()
            .filter(|dialog| dialog.request == request)
    }

    fn history_loaded(&mut self, request: Uuid, result: Result<LoadedHistory, String>) {
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

    fn protection_saved(
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
        if self.active.is_some() || self.backup_history.is_some() {
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

fn load_history(job: &Job) -> anyhow::Result<LoadedHistory> {
    let backups = engine::history::list(job)?;
    let source = std::fs::canonicalize(&job.source)
        .ok()
        .filter(|source| source.is_dir());
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

    #[test]
    fn add_job_copies_current_defaults_and_selects_a_fresh_job() {
        let mut app = app_with(Config::default(), PathBuf::new());
        app.add_job();
        let first = app.config.jobs[0].clone();
        assert!(first.zip && first.auto_destination && first.rolling);
        app.config.job_defaults.zip = false;
        app.config.job_defaults.interval_value = 15;
        app.add_job();
        assert!(app.config.jobs[0] == first);
        let second = &app.config.jobs[1];
        assert!(!second.zip);
        assert_eq!(second.interval_value, 15);
        assert_eq!(app.selected, Some(second.id));
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn job_check_on_enable_is_independent_of_template_changes() {
        let mut config = Config::default();
        config.job_defaults.check_on_enable = true;
        let mut app = app_with(config, PathBuf::new());
        app.add_job();
        let id = app.selected.unwrap();
        app.config.job_defaults.check_on_enable = false;
        let job = app.job_mut(id).unwrap();
        assert!(job.check_on_enable);
        let saved: Job = serde_json::from_str(&serde_json::to_string(job).unwrap()).unwrap();
        assert!(saved.check_on_enable && saved.auto_destination);
        job.enabled = true;
        job.reschedule(Utc::now());
        app.job_activation_changed(id, false);
        assert_eq!(app.queue, VecDeque::from([id]));
        app.queue.clear();
        app.job_mut(id).unwrap().check_on_enable = false;
        app.config.job_defaults.check_on_enable = true;
        app.job_activation_changed(id, false);
        assert!(app.queue.is_empty());
    }

    #[test]
    fn enabling_can_queue_one_immediate_check_without_changing_future_due_time() {
        let mut config = Config::default();
        config.job_defaults.check_on_enable = true;
        let config: Config =
            serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        let mut app = app_with(config, PathBuf::new());
        app.add_job();
        let id = app.selected.unwrap();
        let job = app.job_mut(id).unwrap();
        job.enabled = true;
        job.reschedule(Utc::now());
        let next = job.next_run;
        app.job_activation_changed(id, false);
        app.job_activation_changed(id, false);
        assert_eq!(app.queue, VecDeque::from([id]));
        assert_eq!(app.job(id).unwrap().next_run, next);
        app.queue.clear();
        app.job_activation_changed(id, true);
        assert!(app.queue.is_empty());
        app.active = Some(id);
        app.job_activation_changed(id, false);
        assert!(app.queue.is_empty());
        app.active = None;
        app.job_mut(id).unwrap().check_on_enable = false;
        app.job_activation_changed(id, false);
        assert!(app.queue.is_empty());
        app.queue.push_back(id);
        app.job_mut(id).unwrap().enabled = false;
        app.job_activation_changed(id, true);
        assert!(app.queue.is_empty());
    }

    #[test]
    fn moving_app_and_backups_preserves_backup_history_and_restore() {
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join("original");
        let moved = root.path().join("moved");
        let source = root.path().join("source");
        std::fs::create_dir(&original).unwrap();
        std::fs::create_dir(&source).unwrap();
        std::fs::write(source.join("file"), "original contents").unwrap();
        let mut job = Job {
            source: source.clone(),
            ..crate::model::JobDefaults::default().new_job()
        };
        job.set_auto_destination(&original).unwrap();
        engine::run(&job, &mpsc::channel().0).unwrap();
        let manual = Job {
            target: root.path().join("manual"),
            ..Job::default()
        };
        let manual_target = manual.target.clone();
        let config = Config {
            jobs: vec![job.clone(), manual],
            ..Config::default()
        };
        let mut app = app_with(config, original.join("portbak.json"));
        assert!(app.persist());
        let saved: Config = engine::load(&app.path).unwrap();
        assert_eq!(
            saved.jobs[0].target,
            PathBuf::from("backups").join(job.target.file_name().unwrap())
        );
        assert_eq!(saved.jobs[1].target, manual_target);
        drop(app);
        std::fs::rename(&original, &moved).unwrap();
        let path = moved.join("portbak.json");
        let config = engine::load(&path).unwrap();
        let app = app_with(config, path);
        let job = &app.config.jobs[0];
        assert!(job
            .target
            .starts_with(std::fs::canonicalize(&moved).unwrap()));
        let backups = engine::history::list(job).unwrap();
        assert_eq!(backups.len(), 1);
        assert!(!engine::run(job, &mpsc::channel().0).unwrap().created);
        std::fs::write(source.join("file"), "changed").unwrap();
        engine::restore::run(job, &backups[0].record.name, &mpsc::channel().0).unwrap();
        assert_eq!(
            std::fs::read_to_string(source.join("file")).unwrap(),
            "original contents"
        );
        assert_eq!(app.config.jobs[1].target, manual_target);
        let mut legacy = Config {
            jobs: vec![Job {
                target: original
                    .join("backups")
                    .join(job.target.file_name().unwrap()),
                ..job.clone()
            }],
            ..Config::default()
        };
        legacy.resolve_destinations(&moved);
        assert_eq!(legacy.jobs[0].target, job.target);
    }

    /// Ticks until `done` holds or five seconds pass.
    fn tick_until(app: &mut App, context: &egui::Context, done: impl Fn(&App) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !done(app) && Instant::now() < deadline {
            app.tick(context);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn dialog(app: &App) -> &BackupHistory {
        app.backup_history.as_ref().unwrap()
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
        let mut dialog = BackupHistory::new(job.clone());
        dialog.source_display = Some(job.source.clone());
        dialog.backups = Some(Ok(engine::history::list(&job).unwrap()));
        dialog.selected = Some(name.to_owned());
        app.backup_history = Some(dialog);
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
    fn backup_history_pauses_scheduling_and_cancel_changes_nothing() {
        let (_directory, mut app, name) = restore_fixture();
        let due = app.config.jobs[0].next_run;
        select_restore(&mut app, &name);
        let context = egui::Context::default();
        app.tick(&context);
        app.enqueue(app.config.jobs[0].id);
        assert!(app.active.is_none());
        assert!(app.queue.is_empty());
        app.backups_tab = false;
        app.close_inactive_history();
        assert!(app.backup_history.is_none());
        assert!(app.config.jobs[0].enabled);
        assert_eq!(app.config.jobs[0].next_run, due);
    }

    #[test]
    fn new_job_leaves_backups_and_preserves_existing_job() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.backups_tab = true;
        let original = app.config.jobs[0].clone();
        assert!(app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 2);
        assert!(app.config.jobs[0] == original);
        assert_eq!(app.selected, Some(app.config.jobs[1].id));
        assert!(!app.backups_tab);
        assert!(app.backup_history.is_none());
        assert!(!app.scheduling_paused());
    }

    #[test]
    fn backup_history_without_destination_does_not_start_filesystem_work() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].target.clear();
        let context = egui::Context::default();
        app.open_history(app.config.jobs[0].id, &context);
        assert!(app.backup_history.is_none());
        assert!(app.rx.try_recv().is_err());
        assert!(app.error.is_empty());
        assert!(!app.scheduling_paused());
    }

    #[test]
    fn new_job_waits_for_restore_and_protection_save() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.backup_history.as_mut().unwrap().saving = true;
        assert!(!app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 1);
        app.backup_history.as_mut().unwrap().saving = false;
        app.restoring = true;
        assert!(!app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 1);
    }

    #[test]
    fn leaving_history_waits_for_protection_save_and_discards_stale_results() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        let request = dialog(&app).request;
        app.backup_history.as_mut().unwrap().saving = true;
        app.selected = None;
        app.close_inactive_history();
        assert!(app.backup_history.is_some());
        app.protection_saved(request, &name, true, Ok(()));
        app.close_inactive_history();
        assert!(app.backup_history.is_none());
        app.history_loaded(request, Err("Stale response".into()));
        assert!(app.backup_history.is_none());
        assert!(app.config.jobs[0].enabled);
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
        app.backup_history.as_mut().unwrap().selected = None;
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        app.backup_history.as_mut().unwrap().selected = Some(name);
        app.active = Some(Uuid::new_v4());
        app.confirm_restore(&context);
        assert!(app.config.jobs[0].enabled);
        app.active = None;
        app.path = app.path.join("missing/config.json");
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        assert!(app.backup_history.is_some());
        assert!(!app.error.is_empty());
    }

    #[test]
    fn manager_saves_protection_and_reports_errors_without_changing_activation() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        let context = egui::Context::default();
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
        app.open_history(app.config.jobs[0].id, &context);
        tick_until(&mut app, &context, |app| dialog(app).backups.is_some());
        assert!(dialog(&app).backups.as_ref().unwrap().is_ok());
        assert!(app.active.is_none());
        assert!(app.config.jobs[0].enabled);
    }

    #[test]
    fn disabled_backup_notice_requires_disabling_during_a_running_check() {
        let (_directory, mut app, _name) = restore_fixture();
        let id = app.config.jobs[0].id;
        app.config.jobs[0].enabled = false;
        app.active = Some(id);
        assert!(!app.disabled_during_backup);
        app.config.jobs[0].enabled = true;
        app.job_activation_changed(id, false);
        assert!(!app.disabled_during_backup);
        app.config.jobs[0].enabled = false;
        app.job_activation_changed(id, true);
        assert!(app.disabled_during_backup);
        app.config.jobs[0].enabled = true;
        app.job_activation_changed(id, false);
        assert!(!app.disabled_during_backup);
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
        let _ = context.run_ui(egui::RawInput::default(), |ui| {
            app.show_job_editor(ui);
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
