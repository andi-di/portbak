mod history;
mod navigation;
pub(crate) mod platform;
mod rendering;
mod scenery;
mod theme;
mod views;
mod worker;
use history::HistoryState;
use navigation::Page;
use worker::{Worker, WorkerEvent};

use crate::model::{Config, Job};
use crate::persistence;
use eframe::egui;
use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};
use uuid::Uuid;

const AUTOSAVE_DELAY: Duration = Duration::from_millis(600);
const TICK_INTERVAL: Duration = Duration::from_millis(500);

pub struct App {
    config: Config,
    path: PathBuf,
    page: Page,
    worker: Worker,
    history: HistoryState,
    settings_tab: views::settings_view::SettingsTab,
    error: String,
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
        Self {
            page: Page::default(),
            config,
            path,
            worker: Worker::default(),
            history: HistoryState::default(),
            settings_tab: views::settings_view::SettingsTab::default(),
            error: String::new(),
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

    /// Keep destructive actions and protection changes separate from backup execution.
    fn scheduling_paused(&self) -> bool {
        self.worker.restoring
            || self
                .history
                .view
                .as_ref()
                .is_some_and(|history| history.saving || history.confirming_restore)
    }

    fn can_add_job(&self) -> bool {
        !self.worker.restoring
            && !self
                .history
                .view
                .as_ref()
                .is_some_and(|history| history.saving)
    }

    fn is_busy(&self, id: Uuid) -> bool {
        self.worker.active == Some(id)
            || self.worker.queue.contains(&id)
            || self.scheduling_paused()
    }

    /// Returns whether the configuration is now saved; failures stay dirty and are retried.
    fn persist(&mut self) -> bool {
        self.last_save = Instant::now();
        match persistence::save(&self.path, &self.config.for_storage()) {
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
        self.page = Page::JobSettings(job.id);
        self.history.view = None;
        self.config.jobs.push(job);
    }

    fn remove_job(&mut self, id: Uuid) {
        self.history.cache.remove(&id);
        self.history.requests.remove(&id);
        let index = self.config.jobs.iter().position(|job| job.id == id);
        self.config.jobs.retain(|job| job.id != id);
        if self.page.selected_job() == Some(id) {
            self.page = Page::for_job(
                index
                    .and_then(|index| self.config.jobs.get(index.saturating_sub(1)))
                    .map(|job| job.id),
            );
        }
    }

    fn job_activation_changed(&mut self, id: Uuid, was_enabled: bool) {
        let Some(job) = self.job(id) else {
            return;
        };
        let enabled = job.enabled;
        let check_on_enable = job.check_on_enable;
        if was_enabled && !enabled {
            self.remove_pending(id);
        } else if !was_enabled && enabled && check_on_enable {
            self.enqueue(id);
        }
    }

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.worker.rx.try_recv() {
            self.handle(event, ctx);
        }
        self.queue_due_jobs();
        self.start_next_queued(ctx);
        if self.dirty && self.last_save.elapsed() > AUTOSAVE_DELAY {
            self.persist();
        }
        ctx.request_repaint_after(TICK_INTERVAL);
    }

    fn handle(&mut self, event: WorkerEvent, ctx: &egui::Context) {
        match event {
            WorkerEvent::Deleted {
                request,
                result,
                error,
            } => {
                if let Some(history) = self.pending_history(request) {
                    history.saving = false;
                    history.error = error.unwrap_or_default();
                }
                self.history_loaded(request, result);
            }

            WorkerEvent::Progress(message) => self.worker.progress = message,
            WorkerEvent::Finished { job, result } => self.finish_operation(job, result, ctx),
            WorkerEvent::HistoryLoaded { request, result } => self.history_loaded(request, result),
            WorkerEvent::ProtectionSaved {
                request,
                name,
                protected,
                result,
            } => {
                if protected
                    && result.is_ok()
                    && self
                        .history
                        .view
                        .as_ref()
                        .is_some_and(|history| history.request == request)
                {
                    scenery::saya::trigger(ctx, scenery::saya::Action::ProtectionEnabled);
                }
                self.protection_saved(request, &name, protected, result);
            }
            WorkerEvent::SummaryLoaded {
                job,
                target,
                request,
                records,
            } => self.summary_loaded(job, target, request, records),
        }
    }
}

fn describe(error: anyhow::Error) -> String {
    format!("{error:#}")
}

#[cfg(test)]
mod tests {
    use super::history::HistoryViewState;
    use super::*;
    use crate::model::Schedule;
    use crate::{engine, model::ScheduleSettings};
    use chrono::Utc;
    use fs2::FileExt;
    use std::fs::OpenOptions;
    use std::{collections::VecDeque, sync::mpsc};

    fn app_with(config: Config, path: PathBuf) -> App {
        App::new(config, path, tempfile::tempfile().unwrap())
    }

    #[test]
    fn startup_does_not_select_an_existing_job() {
        let config = Config {
            jobs: vec![Job::default()],
            ..Config::default()
        };
        let app = app_with(config, PathBuf::new());
        assert!(app.page.selected_job().is_none());
    }

    #[test]
    fn removing_jobs_selects_neighbors_and_preserves_other_selection() {
        let jobs: Vec<_> = (0..4).map(|_| Job::default()).collect();
        let ids: Vec<_> = jobs.iter().map(|job| job.id).collect();
        let mut app = app_with(
            Config {
                jobs,
                ..Config::default()
            },
            PathBuf::new(),
        );
        app.page = Page::JobSettings(ids[2]);
        app.remove_job(ids[2]);
        assert_eq!(app.page.selected_job(), Some(ids[1]));
        app.remove_job(ids[3]);
        assert_eq!(app.page.selected_job(), Some(ids[1]));
        app.remove_job(ids[1]);
        assert_eq!(app.page.selected_job(), Some(ids[0]));
        app.remove_job(ids[0]);
        assert_eq!(app.page.selected_job(), None);
        let first = Job::default();
        let second = Job::default();
        app.page = Page::JobSettings(first.id);
        app.config.jobs = vec![first.clone(), second.clone()];
        app.remove_job(first.id);
        assert_eq!(app.page.selected_job(), Some(second.id));
    }

    #[test]
    fn add_job_copies_current_defaults_and_selects_a_fresh_job() {
        let mut app = app_with(Config::default(), PathBuf::new());
        app.add_job();
        let first = app.config.jobs[0].clone();
        assert!(first.zip && first.auto_destination && first.rolling);
        app.config.job_defaults.zip = false;
        app.config.job_defaults.timing.interval_value = 15;
        app.add_job();
        assert!(app.config.jobs[0] == first);
        let second = &app.config.jobs[1];
        assert!(!second.zip);
        assert_eq!(second.timing.interval_value, 15);
        assert_eq!(app.page.selected_job(), Some(second.id));
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn job_check_on_enable_is_independent_of_template_changes() {
        let mut config = Config::default();
        config.job_defaults.check_on_enable = true;
        let mut app = app_with(config, PathBuf::new());
        app.add_job();
        let id = app.page.selected_job().unwrap();
        app.config.job_defaults.check_on_enable = false;
        let job = app.job_mut(id).unwrap();
        assert!(job.check_on_enable);
        let saved: Job = serde_json::from_str(&serde_json::to_string(job).unwrap()).unwrap();
        assert!(saved.check_on_enable && saved.auto_destination);
        job.enabled = true;
        job.reschedule(Utc::now());
        app.job_activation_changed(id, false);
        assert_eq!(app.worker.queue, VecDeque::from([id]));
        app.worker.queue.clear();
        app.job_mut(id).unwrap().check_on_enable = false;
        app.config.job_defaults.check_on_enable = true;
        app.job_activation_changed(id, false);
        assert!(app.worker.queue.is_empty());
    }

    #[test]
    fn enabling_can_queue_one_immediate_check_without_changing_future_due_time() {
        let mut config = Config::default();
        config.job_defaults.check_on_enable = true;
        let config: Config =
            serde_json::from_str(&serde_json::to_string(&config).unwrap()).unwrap();
        let mut app = app_with(config, PathBuf::new());
        app.add_job();
        let id = app.page.selected_job().unwrap();
        let job = app.job_mut(id).unwrap();
        job.enabled = true;
        job.reschedule(Utc::now());
        let next = job.next_run;
        app.job_activation_changed(id, false);
        app.job_activation_changed(id, false);
        assert_eq!(app.worker.queue, VecDeque::from([id]));
        assert_eq!(app.job(id).unwrap().next_run, next);
        app.worker.queue.clear();
        app.job_activation_changed(id, true);
        assert!(app.worker.queue.is_empty());
        app.worker.active = Some(id);
        app.job_activation_changed(id, false);
        assert!(app.worker.queue.is_empty());
        app.worker.active = None;
        app.job_mut(id).unwrap().check_on_enable = false;
        app.job_activation_changed(id, false);
        assert!(app.worker.queue.is_empty());
        app.worker.queue.push_back(id);
        app.job_mut(id).unwrap().enabled = false;
        app.job_activation_changed(id, true);
        assert!(app.worker.queue.is_empty());
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
        crate::destinations::create_for(&mut job, &original).unwrap();
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
        let saved: Config = persistence::load(&app.path).unwrap();
        assert_eq!(
            saved.jobs[0].target,
            PathBuf::from("backups").join(job.target.file_name().unwrap())
        );
        assert_eq!(saved.jobs[1].target, manual_target);
        drop(app);
        std::fs::rename(&original, &moved).unwrap();
        let path = moved.join("portbak.json");
        let config = persistence::load(&path).unwrap();
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

    fn dialog(app: &App) -> &HistoryViewState {
        app.history.view.as_ref().unwrap()
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
        let mut dialog = HistoryViewState::new(job.clone());
        dialog.source_display = Some(job.source.clone());
        dialog.backups = Some(Ok(engine::history::list(&job).unwrap()));
        dialog.selected = Some(name.to_owned());
        app.history.view = Some(dialog);
    }

    #[test]
    fn disabling_removes_pending_work_and_preserves_active_work() {
        let mut app = app_with(Config::default(), PathBuf::new());
        let active = Uuid::new_v4();
        let queued = Uuid::new_v4();
        let other = Uuid::new_v4();
        app.worker.active = Some(active);
        app.enqueue(queued);
        app.enqueue(other);
        app.remove_pending(queued);
        app.remove_pending(active);
        assert_eq!(app.worker.active, Some(active));
        assert_eq!(app.worker.queue, VecDeque::from([other]));
    }

    #[test]
    fn browsing_history_runs_due_backup_without_input_and_reloads_versions() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.page = Page::Backups(app.config.jobs[0].id);
        let job = app.config.jobs[0].clone();
        std::fs::write(job.source.join("file"), "changed").unwrap();
        let context = egui::Context::default();
        app.tick(&context);
        assert_eq!(app.worker.active, Some(job.id));
        assert!(app.history.view.is_none());
        assert!(app.config.jobs[0].next_run > job.next_run);
        tick_until(&mut app, &context, |app| app.worker.active.is_none());
        assert!(app.worker.active.is_none());
        app.open_history(job.id, &context);
        tick_until(&mut app, &context, |app| dialog(app).backups.is_some());
        assert_eq!(
            dialog(&app)
                .backups
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .len(),
            2
        );
        assert!(app.config.jobs[0].enabled);
    }

    #[test]
    fn minimized_moonveil_window_still_runs_due_backups() {
        let (_directory, mut app, _) = restore_fixture();
        app.config.skin = "moonveil".into();
        let job = app.config.jobs[0].clone();
        std::fs::write(job.source.join("file"), "changed while minimized").unwrap();
        let context = egui::Context::default();
        let mut input = egui::RawInput::default();
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .minimized = Some(true);
        let _ = context.run_ui(input, |_| {
            assert!(!scenery::moonveil::decorations_visible(&context));
            app.tick(&context);
            assert_eq!(app.worker.active, Some(job.id));
        });
        tick_until(&mut app, &context, |app| app.worker.active.is_none());
        assert!(app.worker.active.is_none());
        assert_eq!(engine::history::list(&job).unwrap().len(), 2);
    }

    #[test]
    fn history_mutations_pause_due_backups_until_finished() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        let context = egui::Context::default();
        app.history.view.as_mut().unwrap().saving = true;
        app.tick(&context);
        assert!(app.worker.active.is_none());
        app.history.view.as_mut().unwrap().saving = false;
        app.history.view.as_mut().unwrap().confirming_restore = true;
        app.tick(&context);
        assert!(app.worker.active.is_none());
        app.history.view.as_mut().unwrap().confirming_restore = false;
        app.tick(&context);
        assert!(app.worker.active.is_some());
        tick_until(&mut app, &context, |app| app.worker.active.is_none());
        assert!(app.worker.active.is_none());
    }

    #[test]
    fn new_job_leaves_backups_and_preserves_existing_job() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.page = Page::Backups(app.config.jobs[0].id);
        let original = app.config.jobs[0].clone();
        assert!(app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 2);
        assert!(app.config.jobs[0] == original);
        assert_eq!(app.page.selected_job(), Some(app.config.jobs[1].id));
        assert!(!app.page.is_backups());
        assert!(app.history.view.is_none());
        assert!(!app.scheduling_paused());
    }

    #[test]
    fn backup_history_without_destination_does_not_start_filesystem_work() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].target.clear();
        let context = egui::Context::default();
        app.open_history(app.config.jobs[0].id, &context);
        assert!(app.history.view.is_none());
        assert!(app.worker.rx.try_recv().is_err());
        assert!(app.error.is_empty());
        assert!(!app.scheduling_paused());
    }

    #[test]
    fn new_job_waits_for_restore_and_protection_save() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        app.history.view.as_mut().unwrap().saving = true;
        assert!(!app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 1);
        app.history.view.as_mut().unwrap().saving = false;
        app.worker.restoring = true;
        assert!(!app.can_add_job());
        app.add_job();
        assert_eq!(app.config.jobs.len(), 1);
    }

    #[test]
    fn leaving_history_waits_for_protection_save_and_discards_stale_results() {
        let (_directory, mut app, name) = restore_fixture();
        select_restore(&mut app, &name);
        let request = dialog(&app).request;
        app.history.view.as_mut().unwrap().saving = true;
        app.page = Page::Home;
        app.close_inactive_history();
        assert!(app.history.view.is_some());
        app.protection_saved(request, &name, true, Ok(()));
        app.close_inactive_history();
        assert!(app.history.view.is_none());
        app.history_loaded(request, Err("Stale response".into()));
        assert!(app.history.view.is_none());
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
        assert!(app.worker.restoring);
        assert!(!app.config.jobs[0].enabled);
        assert!(app.config.jobs[0].next_run.is_none());
        app.enqueue(Uuid::new_v4());
        assert!(app.worker.queue.is_empty());
        tick_until(&mut app, &context, |app| app.worker.active.is_none());
        assert!(app.worker.active.is_none());
        assert!(!app.worker.restoring);
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
        app.history.view.as_mut().unwrap().selected = None;
        app.confirm_restore(&context);
        assert!(app.worker.active.is_none());
        app.history.view.as_mut().unwrap().selected = Some(name);
        app.worker.active = Some(Uuid::new_v4());
        app.confirm_restore(&context);
        assert!(app.config.jobs[0].enabled);
        app.worker.active = None;
        app.path = app.path.join("missing/config.json");
        app.confirm_restore(&context);
        assert!(app.worker.active.is_none());
        assert!(app.history.view.is_some());
        assert!(!app.error.is_empty());
    }

    #[test]
    fn manager_saves_protection_and_reports_errors_without_changing_activation() {
        let (_directory, mut app, name) = restore_fixture();
        app.config.jobs[0].next_run = Some(Utc::now() + chrono::Duration::hours(1));
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
        assert!(app.history.cache[&id].records.as_ref().unwrap()[0].protected);
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
        assert!(app.worker.active.is_none());
    }

    #[test]
    fn manager_history_does_not_require_source_folder() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].next_run = Some(Utc::now() + chrono::Duration::hours(1));
        app.config.jobs[0].source.push("missing");
        let context = egui::Context::default();
        app.open_history(app.config.jobs[0].id, &context);
        tick_until(&mut app, &context, |app| dialog(app).backups.is_some());
        assert!(dialog(&app).backups.as_ref().unwrap().is_ok());
        assert!(app.worker.active.is_none());
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
        assert_eq!(app.worker.queue, VecDeque::from([id]));
        assert!(!app.config.jobs[0].enabled);
    }

    #[test]
    fn clearing_all_weekdays_disables_job_and_drops_pending_check() {
        let job = Job {
            enabled: true,
            timing: ScheduleSettings {
                mode: Schedule::Weekdays,
                weekdays: [false; 7],
                ..ScheduleSettings::default()
            },
            ..Job::default()
        };
        let id = job.id;
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let mut app = app_with(config, PathBuf::new());
        app.page = Page::JobSettings(id);
        app.enqueue(id);
        let context = egui::Context::default();
        let _ = context.run_ui(egui::RawInput::default(), |ui| {
            app.show_job_editor(ui);
        });
        assert!(!app.config.jobs[0].enabled);
        assert!(app.config.jobs[0].next_run.is_none());
        assert!(app.worker.queue.is_empty());
    }

    #[test]
    fn summary_for_a_changed_destination_is_discarded() {
        let (_directory, mut app, _name) = restore_fixture();
        app.config.jobs[0].enabled = false;
        let id = app.config.jobs[0].id;
        let context = egui::Context::default();
        app.reload_summary(id, &context);
        app.config.jobs[0].target.push("elsewhere");
        tick_until(&mut app, &context, |app| app.history.requests.is_empty());
        assert!(app.history.requests.is_empty());
        assert!(!app.history.cache.contains_key(&id));
    }
}
