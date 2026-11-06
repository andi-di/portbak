#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod engine;

mod model;

use chrono::Utc;

use eframe::egui;

use fs2::FileExt;

use model::{Config, IntervalUnit, Job, Schedule};

use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use uuid::Uuid;

enum Event {
    Progress(String),
    Done(Uuid, Result<engine::Outcome, String>),
}

struct App {
    config: Config,
    path: PathBuf,
    selected: Option<Uuid>,
    queue: VecDeque<Uuid>,
    active: Option<Uuid>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    error: String,
    progress: String,
    dirty: bool,
    last_save: Instant,
    closing: bool,
    permit_close: bool,
    _lock: File,
}

impl App {
    fn new(config: Config, path: PathBuf, lock: File) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            selected: config.jobs.first().map(|j| j.id),
            config,
            path,
            queue: VecDeque::new(),
            active: None,
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

    fn persist(&mut self) {
        match engine::save(&self.path, &self.config) {
            Ok(()) => {
                self.dirty = false;

                self.error.clear();
            }

            Err(e) => self.error = e.to_string(),
        }

        self.last_save = Instant::now();
    }

    fn enqueue(&mut self, id: Uuid) {
        if self.active != Some(id) && !self.queue.contains(&id) {
            self.queue.push_back(id);
        }
    }

    fn remove_pending(&mut self, id: Uuid) {
        self.queue.retain(|queued| *queued != id);
    }

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(e) = self.rx.try_recv() {
            match e {
                Event::Progress(s) => self.progress = s,
                Event::Done(id, result) => {
                    if let Some(j) = self.config.jobs.iter_mut().find(|j| j.id == id) {
                        match result {
                            Ok(o) => {
                                if o.created {
                                    j.last_success = Some(Utc::now())
                                }

                                j.status = o.message;
                            }

                            Err(e) => j.status = format!("Error: {e}"),
                        };
                    }

                    self.active = None;

                    self.progress.clear();

                    self.dirty = true;

                    self.persist();
                }
            }
        }

        let now = Utc::now();

        let mut due = vec![];

        for j in &mut self.config.jobs {
            let previous = j.next_run;

            if j.take_due(now) {
                due.push(j.id);
            }

            if previous != j.next_run {
                self.dirty = true;
            }
        }

        for id in due {
            self.enqueue(id)
        }

        if self.active.is_none() {
            if let Some(id) = self.queue.pop_front() {
                if let Some(j) = self.config.jobs.iter().find(|j| j.id == id).cloned() {
                    self.active = Some(id);

                    self.progress = "Preparing backup …".into();

                    let tx = self.tx.clone();

                    let ctx = ctx.clone();

                    std::thread::spawn(move || {
                        let (ptx, prx) = mpsc::channel();

                        let forward = tx.clone();

                        let pctx = ctx.clone();

                        let relay = std::thread::spawn(move || {
                            for s in prx {
                                let _ = forward.send(Event::Progress(s));

                                pctx.request_repaint();
                            }
                        });

                        let result = std::panic::catch_unwind(|| engine::run(&j, &ptx))
                            .map_err(|_| "Unexpected error in backup worker".to_string())
                            .and_then(|result| result.map_err(|e| format!("{e:#}")));

                        drop(ptx);

                        let _ = relay.join();

                        let _ = tx.send(Event::Done(id, result));

                        ctx.request_repaint();
                    });
                }
            }
        }

        if self.dirty && self.last_save.elapsed() > Duration::from_millis(600) {
            self.persist()
        }

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn path_picker(ui: &mut egui::Ui, label: &str, path: &mut PathBuf) {
    ui.label(label);

    ui.horizontal(|ui| {
        let mut s = path.to_string_lossy().into_owned();

        if ui
            .add(egui::TextEdit::singleline(&mut s).desired_width(370.0))
            .changed()
        {
            *path = PathBuf::from(s);
        }

        if ui.button("Choose folder …").clicked() {
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                *path = p;
            }
        }
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);

        ctx.set_theme(match self.config.theme.as_str() {
            "Light" => egui::ThemePreference::Light,
            "Dark" => egui::ThemePreference::Dark,
            _ => egui::ThemePreference::System,
        });

        if ctx.input(|i| i.viewport().close_requested()) && !self.permit_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

            self.closing = true;
        }

        let before = serde_json::to_vec(&self.config).unwrap_or_default();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.heading("Portable Backup");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("theme")
                        .selected_text(&self.config.theme)
                        .show_ui(ui, |ui| {
                            for t in ["System", "Light", "Dark"] {
                                ui.selectable_value(&mut self.config.theme, t.into(), t);
                            }
                        });
                });
            });

            ui.label("Local backups · only when files change · no installation");

            ui.add_space(10.0);
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.0);

            if self.active.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();

                    ui.label(&self.progress);
                });
            } else {
                ui.label("Ready · scheduling runs while the app is open");
            }

            if !self.error.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_RED, &self.error);
            }

            ui.small(format!("Settings: {}", self.path.display()));

            ui.add_space(5.0);
        });

        egui::SidePanel::left("jobs")
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);

                ui.heading("Backup jobs");

                ui.add_space(8.0);

                if ui.button("New job").clicked() {
                    let j = Job::default();

                    self.selected = Some(j.id);

                    self.config.jobs.push(j);
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for j in &self.config.jobs {
                        ui.group(|ui| {
                            if ui
                                .selectable_label(self.selected == Some(j.id), &j.name)
                                .clicked()
                            {
                                self.selected = Some(j.id);
                            }

                            ui.small(if self.active == Some(j.id) {
                                "Backup running"
                            } else if j.enabled {
                                "Schedule enabled"
                            } else {
                                "Schedule disabled"
                            });

                            if self.queue.contains(&j.id) {
                                ui.small("Queued");
                            }

                            if let Some(remaining) = j.countdown(Utc::now()) {
                                ui.small(format!("Next check in: {remaining}"));
                            }
                            ui.small(&j.status);
                        });
                    }
                });
            });

        let mut run = None;

        let mut remove = None;
        let mut disabled = None;

        egui::CentralPanel::default().show(ctx,|ui|{

            egui::ScrollArea::vertical().show(ui,|ui|{

                if let Some(j)=self.config.jobs.iter_mut().find(|j|Some(j.id)==self.selected){

                    ui.add_space(10.0);
ui.heading(&j.name);
ui.add_space(10.0);

                    let busy=self.active==Some(j.id)||self.queue.contains(&j.id);

                    let can_enable = j.schedule != Schedule::Weekdays || j.weekdays.iter().any(|day| *day);
                    let (label, color) = if j.enabled {
                        ("Disable automatic backups", egui::Color32::from_rgb(170, 35, 45))
                    } else {
                        ("Enable automatic backups", egui::Color32::from_rgb(25, 120, 60))
                    };
                    let button = egui::Button::new(egui::RichText::new(label).strong().color(egui::Color32::WHITE))
                        .fill(color).min_size(egui::vec2(260.0, 40.0));
                    if ui.add_enabled(can_enable || j.enabled, button).clicked() {
                        j.enabled = !j.enabled;
                        j.reschedule(Utc::now());
                        if !j.enabled { disabled = Some(j.id); }
                    }
                    if !j.enabled && self.active == Some(j.id) {
                        ui.label("Automatic backups disabled; current backup will finish.");
                    }

                    ui.add_enabled_ui(!busy,|ui|{

                        ui.label("Job name");
ui.text_edit_singleline(&mut j.name);
ui.add_space(8.0);

                        path_picker(ui,"Source folder",&mut j.source);
ui.add_space(8.0);
path_picker(ui,"Destination folder",&mut j.target);

                        ui.add_space(8.0);
ui.checkbox(&mut j.zip,"Compress as ZIP");

                        ui.separator();
ui.heading("Scheduling");

                        let old=serde_json::to_vec(&(j.enabled,&j.schedule,j.hour,j.minute,j.weekdays,j.interval_value,j.interval_unit)).unwrap();

                        ui.horizontal(|ui|{
ui.selectable_value(&mut j.schedule,Schedule::Interval,"Interval");
ui.selectable_value(&mut j.schedule,Schedule::Daily,"Daily");
ui.selectable_value(&mut j.schedule,Schedule::Weekdays,"Weekdays");
}
);

                        if j.schedule==Schedule::Interval{
ui.horizontal(|ui|{
ui.label("Every");
ui.add(egui::DragValue::new(&mut j.interval_value).range(1..=j.interval_unit.maximum()));
egui::ComboBox::from_id_salt("interval_unit")
    .selected_text(j.interval_unit.label())
    .show_ui(ui, |ui| {
        for unit in [IntervalUnit::Seconds, IntervalUnit::Minutes, IntervalUnit::Hours] {
            ui.selectable_value(&mut j.interval_unit, unit, unit.label());
        }
    });
j.interval_value = j.interval_value.clamp(1, j.interval_unit.maximum());
}
);
}
else{

                            ui.horizontal(|ui|{
ui.label("Local time");
ui.add(egui::DragValue::new(&mut j.hour).range(0..=23).min_decimals(0));
ui.label(":");
ui.add(egui::DragValue::new(&mut j.minute).range(0..=59).min_decimals(0));
}
);

                            if j.schedule==Schedule::Weekdays{
ui.horizontal(|ui|{
for (i,s) in ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"].iter().enumerate(){
ui.toggle_value(&mut j.weekdays[i],*s);
}
}
);
if !j.weekdays.iter().any(|v|*v){
ui.colored_label(egui::Color32::LIGHT_RED,"Select at least one weekday");
j.enabled=false;
}
}

                        }

                        if old!=serde_json::to_vec(&(j.enabled,&j.schedule,j.hour,j.minute,j.weekdays,j.interval_value,j.interval_unit)).unwrap(){
j.reschedule(Utc::now());
}

                        ui.label(format!("Next check: {}",if j.enabled{
j.next_run.map(|t|t.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M").to_string()).unwrap_or_else(||"pending".into())}
else{
"disabled".into()}
));

                        if let Some(remaining) = j.countdown(Utc::now()) {
                            ui.label(format!("Next check in: {remaining}"));
                        }

                        ui.separator();
ui.heading("Retention");
ui.checkbox(&mut j.rolling,"Enable rolling retention – delete oldest backups automatically");

                        ui.add_enabled_ui(j.rolling,|ui|{

                            let mut count=j.max_count.is_some();
ui.horizontal(|ui|{
if ui.checkbox(&mut count,"Maximum count").changed(){
j.max_count=if count{
Some(10)}
else{
None}
;
}
if let Some(n)=&mut j.max_count{
ui.add(egui::DragValue::new(n).range(1..=100000));
}
}
);

                            let mut size=j.max_bytes.is_some();
ui.horizontal(|ui|{
if ui.checkbox(&mut size,"Maximum total size").changed(){
j.max_bytes=if size{
Some(10*1024*1024*1024)}
else{
None}
;
}
if let Some(n)=&mut j.max_bytes{
let mut gib=*n as f64/1073741824.0;
if ui.add(egui::DragValue::new(&mut gib).speed(0.1).range(0.001..=100000.0).suffix(" GiB")).changed(){
*n=(gib*1073741824.0)as u64;
}
}
}
);

                        }
);
ui.small("The latest successful backup is always kept.");

                    }
);

                    ui.separator();
ui.label(format!("Last backup: {}",j.last_success.map(|t|t.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M:%S").to_string()).unwrap_or_else(||"–".into())));
ui.label(&j.status);
ui.add_space(8.0);

                    ui.horizontal(|ui|{
if ui.add_enabled(!busy,egui::Button::new("Check now")).clicked(){
run=Some(j.id);
}
if ui.add_enabled(!busy,egui::Button::new("Remove job")).clicked(){
remove=Some(j.id);
}
}
);

                    ui.add_space(10.0);
ui.small("Backups are full copies. Links are skipped. Each check reads all file contents. Restore by copying or extracting.");

                }
else{
ui.add_space(60.0);
ui.heading("Your folders. Safely backed up.");
ui.label("Create a backup job and select source and destination folders.");
}

            }
);

        }
);

        if let Some(id) = disabled {
            self.remove_pending(id);
        }

        if let Some(id) = run {
            self.enqueue(id)
        }

        if let Some(id) = remove {
            self.config.jobs.retain(|j| j.id != id);

            self.selected = self.config.jobs.first().map(|j| j.id);
        }

        if before != serde_json::to_vec(&self.config).unwrap_or_default() {
            self.dirty = true;
        }

        if self.closing {
            egui::Window::new("Quit the app?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Quitting stops scheduling. The app keeps running when minimized.");

                    if self.active.is_some() {
                        ui.label(
                            "A backup is running. Please wait for it to finish before quitting.",
                        );
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Keep running").clicked() {
                            self.closing = false;
                        }

                        if ui
                            .add_enabled(self.active.is_none(), egui::Button::new("Quit"))
                            .clicked()
                        {
                            self.persist();

                            if self.error.is_empty() {
                                self.permit_close = true;

                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    });
                });
        }
    }
}

fn startup() -> anyhow::Result<()> {
    let path = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Application directory is missing"))?
        .join("portable-backup.json");

    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))?;

    lock.try_lock_exclusive().map_err(|_| {
        anyhow::anyhow!("Portable Backup is already running with this configuration.")
    })?;

    let config: Config = if path.exists() {
        engine::load(&path)?
    } else {
        Config::default()
    };

    anyhow::ensure!(config.version == 1, "Unknown configuration version");

    engine::save(&path, &config)?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 960.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Portable Backup",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.all_styles_mut(|style| {
                style.spacing.item_spacing = egui::vec2(8.0, 5.0);
                style.spacing.button_padding = egui::vec2(10.0, 5.0);
                style
                    .text_styles
                    .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
                style
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
                style
                    .text_styles
                    .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
                style.visuals.override_text_color = Some(if style.visuals.dark_mode {
                    egui::Color32::from_gray(220)
                } else {
                    egui::Color32::from_gray(35)
                });
            });
            Ok(Box::new(App::new(config, path, lock)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

fn main() {
    if let Err(e) = startup() {
        let msg = format!("Portable Backup could not start:\n{e:#}");

        eprintln!("{msg}");

        rfd::MessageDialog::new()
            .set_title("Portable Backup")
            .set_description(&msg)
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabling_removes_pending_work_and_preserves_active_work() {
        let mut app = App::new(
            Config::default(),
            PathBuf::new(),
            tempfile::tempfile().unwrap(),
        );
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
    fn manual_checks_can_queue_disabled_jobs() {
        let job = Job::default();
        let id = job.id;
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let mut app = App::new(config, PathBuf::new(), tempfile::tempfile().unwrap());
        app.enqueue(id);
        app.enqueue(id);
        assert_eq!(app.queue, VecDeque::from([id]));
        assert!(!app.config.jobs[0].enabled);
    }
}
