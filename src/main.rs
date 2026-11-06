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
    History(
        Uuid,
        Result<(Vec<engine::restore::Backup>, PathBuf), String>,
    ),
}

struct RestoreDialog {
    request: Uuid,
    job: Job,
    backups: Option<Result<Vec<engine::restore::Backup>, String>>,
    selected: Option<String>,
    source_display: Option<PathBuf>,
}

enum Operation {
    Backup,
    Restore(String),
}

struct App {
    config: Config,
    path: PathBuf,
    selected: Option<Uuid>,
    queue: VecDeque<Uuid>,
    active: Option<Uuid>,
    restoring: bool,
    restore_dialog: Option<RestoreDialog>,
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
            restoring: false,
            restore_dialog: None,
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
        if !self.restoring
            && self.restore_dialog.is_none()
            && self.active != Some(id)
            && !self.queue.contains(&id)
        {
            self.queue.push_back(id);
        }
    }

    fn remove_pending(&mut self, id: Uuid) {
        self.queue.retain(|queued| *queued != id);
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
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let (ptx, prx) = mpsc::channel();
            let forward = tx.clone();
            let pctx = ctx.clone();
            let relay = std::thread::spawn(move || {
                for message in prx {
                    let _ = forward.send(Event::Progress(message));
                    pctx.request_repaint();
                }
            });
            let result = std::panic::catch_unwind(|| match operation {
                Operation::Backup => engine::run(&job, &ptx),
                Operation::Restore(name) => engine::restore::run(&job, &name, &ptx),
            })
            .map_err(|_| {
                "Unexpected error in worker; check for restore recovery folders beside the source"
                    .to_string()
            })
            .and_then(|result| result.map_err(|e| format!("{e:#}")));
            drop(ptx);
            let _ = relay.join();
            let _ = tx.send(Event::Done(job.id, result));
            ctx.request_repaint();
        });
    }

    fn open_restore(&mut self, id: Uuid, ctx: &egui::Context) {
        if self.active.is_some() || !self.queue.is_empty() || self.restore_dialog.is_some() {
            return;
        }
        let Some(job) = self.config.jobs.iter().find(|job| job.id == id).cloned() else {
            return;
        };
        let request = Uuid::new_v4();
        self.restore_dialog = Some(RestoreDialog {
            request,
            job: job.clone(),
            backups: None,
            selected: None,
            source_display: None,
        });
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<_> {
                let backups = engine::restore::list(&job)?;
                let source = std::fs::canonicalize(&job.source)?;
                Ok((backups, source))
            })()
            .map_err(|error| format!("{error:#}"));
            let _ = tx.send(Event::History(request, result));
            ctx.request_repaint();
        });
    }

    fn confirm_restore(&mut self, ctx: &egui::Context) {
        if self.active.is_some() || !self.queue.is_empty() {
            return;
        }
        let Some(dialog) = &self.restore_dialog else {
            return;
        };
        let Some(name) = dialog.selected.clone() else {
            return;
        };
        let valid = dialog
            .backups
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .is_some_and(|backups| {
                backups
                    .iter()
                    .any(|backup| backup.available && backup.record.name == name)
            });
        if !valid {
            return;
        }
        let id = dialog.job.id;
        let Some(job) = self.config.jobs.iter_mut().find(|job| job.id == id) else {
            return;
        };
        job.enabled = false;
        job.reschedule(Utc::now());
        let job = job.clone();
        self.dirty = true;
        self.persist();
        if self.dirty {
            return;
        }
        self.restore_dialog = None;
        self.start_worker(job, Operation::Restore(name), ctx);
    }

    fn show_restore(&mut self, ctx: &egui::Context) {
        let mut cancel = false;
        let mut confirm = false;
        if let Some(dialog) = &mut self.restore_dialog {
            egui::Window::new("Restore backup")
                .collapsible(false).resizable(true).default_width(650.0)
                .max_height(520.0)
                .vscroll(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading(&dialog.job.name);
                    ui.label("Choose a saved version. Close programs that write to this folder before restoring.");
                    match &dialog.backups {
                        None => { ui.spinner(); ui.label("Loading backup history …"); }
                        Some(Err(error)) => { ui.colored_label(egui::Color32::LIGHT_RED, error); }
                        Some(Ok(backups)) => {
                            if backups.is_empty() { ui.label("No saved backups for this job."); }
                            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                                for backup in backups {
                                    let record = &backup.record;
                                    let label = format!("{} · {} · {} bytes{}\n{}",
                                        record.created.with_timezone(&chrono::Local).format("%d.%m.%Y %H:%M:%S"),
                                        if backup.zip { "ZIP" } else { "Folder" }, record.bytes,
                                        if backup.available { "" } else { " · unavailable" }, record.name);
                                    if ui.add_enabled(backup.available, egui::Button::new(label).selected(dialog.selected.as_deref() == Some(&record.name))).clicked() {
                                        dialog.selected = Some(record.name.clone());
                                    }
                                }
                            });
                        }
                    }
                    ui.separator();
                    ui.label(format!("Source folder to replace: {}", dialog.source_display.as_ref().unwrap_or(&dialog.job.source).display()));
                    if let Some(name) = &dialog.selected { ui.label(format!("Selected backup: {name}")); }
                    ui.colored_label(egui::Color32::LIGHT_RED,
                        "The ENTIRE current contents of this source folder will be replaced, including additional files, hidden entries and subfolders.");
                    ui.label("Automatic backups for this job will remain disabled after restoration.");
                    if !self.error.is_empty() { ui.colored_label(egui::Color32::LIGHT_RED, &self.error); }
                    centered_row(ui, |ui| {
                        cancel = ui.button("Cancel").clicked();
                        confirm = ui.add_enabled(dialog.selected.is_some(), egui::Button::new(
                            egui::RichText::new("Replace contents and restore").strong().color(egui::Color32::WHITE))
                            .fill(egui::Color32::from_rgb(170, 35, 45))).clicked();
                    });
                });
        }
        if cancel {
            self.restore_dialog = None;
        }
        if confirm {
            self.confirm_restore(ctx);
        }
    }

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(e) = self.rx.try_recv() {
            match e {
                Event::Progress(s) => self.progress = s,
                Event::History(request, result) => {
                    if let Some(dialog) = &mut self.restore_dialog {
                        if dialog.request == request {
                            dialog.backups = Some(result.map(|(backups, source)| {
                                dialog.source_display = Some(source);
                                backups
                            }));
                        }
                    }
                }
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
                    self.restoring = false;

                    self.progress.clear();

                    self.dirty = true;

                    self.persist();
                }
            }
        }

        let now = Utc::now();

        let mut due = vec![];

        for j in self
            .config
            .jobs
            .iter_mut()
            .filter(|_| !self.restoring && self.restore_dialog.is_none())
        {
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

        if self.active.is_none() && self.restore_dialog.is_none() {
            if let Some(id) = self.queue.pop_front() {
                if let Some(job) = self.config.jobs.iter().find(|j| j.id == id).cloned() {
                    self.start_worker(job, Operation::Backup, ctx);
                }
            }
        }

        if self.dirty && self.last_save.elapsed() > Duration::from_millis(600) {
            self.persist()
        }

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

fn centered_row<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let height =
        ui.text_style_height(&egui::TextStyle::Heading) + ui.spacing().button_padding.y * 2.0;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        egui::Layout::left_to_right(egui::Align::Center),
        contents,
    )
}

fn centered_combo<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let height = ui
        .text_style_height(&egui::TextStyle::Button)
        .max(ui.spacing().icon_width)
        + ui.spacing().button_padding.y * 2.0;
    ui.allocate_ui_with_layout(
        egui::vec2(ui.spacing().combo_width, height),
        egui::Layout::top_down(egui::Align::Min),
        contents,
    )
}

fn path_picker(ui: &mut egui::Ui, label: &str, path: &mut PathBuf) {
    ui.label(label);

    centered_row(ui, |ui| {
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

            centered_row(ui, |ui| {
                ui.heading("portbak");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    centered_combo(ui, |ui| {
                        egui::ComboBox::from_id_salt("theme")
                            .selected_text(&self.config.theme)
                            .show_ui(ui, |ui| {
                                for t in ["System", "Light", "Dark"] {
                                    ui.selectable_value(&mut self.config.theme, t.into(), t);
                                }
                            })
                    });
                });
            });

            ui.add_space(10.0);
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.0);

            if self.active.is_some() {
                centered_row(ui, |ui| {
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

                if ui
                    .add_enabled(
                        !self.restoring && self.restore_dialog.is_none(),
                        egui::Button::new("New job"),
                    )
                    .clicked()
                {
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
                                if self.restoring {
                                    "Restore running"
                                } else {
                                    "Backup running"
                                }
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
        let mut restore = None;
        let mut disabled = None;

        egui::CentralPanel::default().show(ctx,|ui|{

            egui::ScrollArea::vertical().show(ui,|ui|{

                if let Some(j)=self.config.jobs.iter_mut().find(|j|Some(j.id)==self.selected){

                    ui.add_space(10.0);
ui.heading(&j.name);
ui.add_space(10.0);

                    let busy=self.active==Some(j.id)||self.queue.contains(&j.id)||self.restoring||self.restore_dialog.is_some();

                    let can_enable = j.schedule != Schedule::Weekdays || j.weekdays.iter().any(|day| *day);
                    let (label, color) = if j.enabled {
                        ("Disable automatic backups", egui::Color32::from_rgb(170, 35, 45))
                    } else {
                        ("Enable automatic backups", egui::Color32::from_rgb(25, 120, 60))
                    };
                    let button = egui::Button::new(egui::RichText::new(label).strong().color(egui::Color32::WHITE))
                        .fill(color).min_size(egui::vec2(260.0, 40.0));
                    if ui.add_enabled((can_enable || j.enabled) && !self.restoring && self.restore_dialog.is_none(), button).clicked() {
                        j.enabled = !j.enabled;
                        j.reschedule(Utc::now());
                        if !j.enabled { disabled = Some(j.id); }
                    }
                    if !j.enabled && self.active == Some(j.id) && !self.restoring {
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

                        centered_row(ui, |ui|{
ui.selectable_value(&mut j.schedule,Schedule::Interval,"Interval");
ui.selectable_value(&mut j.schedule,Schedule::Daily,"Daily");
ui.selectable_value(&mut j.schedule,Schedule::Weekdays,"Weekdays");
}
);

                        if j.schedule==Schedule::Interval{
centered_row(ui, |ui|{
ui.label("Every");
ui.add(egui::DragValue::new(&mut j.interval_value).range(1..=j.interval_unit.maximum()));
centered_combo(ui, |ui| egui::ComboBox::from_id_salt("interval_unit")
    .selected_text(j.interval_unit.label())
    .show_ui(ui, |ui| {
        for unit in [IntervalUnit::Seconds, IntervalUnit::Minutes, IntervalUnit::Hours] {
            ui.selectable_value(&mut j.interval_unit, unit, unit.label());
        }
    }));
j.interval_value = j.interval_value.clamp(1, j.interval_unit.maximum());
}
);
}
else{

                            centered_row(ui, |ui|{
ui.label("Local time");
ui.add(egui::DragValue::new(&mut j.hour).range(0..=23).min_decimals(0));
ui.label(":");
ui.add(egui::DragValue::new(&mut j.minute).range(0..=59).min_decimals(0));
}
);

                            if j.schedule==Schedule::Weekdays{
centered_row(ui, |ui|{
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
centered_row(ui, |ui|{
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
centered_row(ui, |ui|{
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

                    centered_row(ui, |ui|{
if ui.add_enabled(!busy,egui::Button::new("Check now")).clicked(){
run=Some(j.id);
}
if ui.add_enabled(!busy && self.active.is_none() && self.queue.is_empty(), egui::Button::new("Restore backup…")).clicked(){
restore=Some(j.id);
}
if ui.add_enabled(!busy,egui::Button::new("Remove job")).clicked(){
remove=Some(j.id);
}
}
);

                    ui.add_space(10.0);
ui.small("Backups are full copies. Links are skipped. Each check reads all file contents. Restore with Restore backup… or by copying/extracting.");

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

        if let Some(id) = restore {
            self.open_restore(id, ctx);
        }
        self.show_restore(ctx);

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
                            "An operation is running. Please wait for it to finish before quitting.",
                        );
                    }

                    centered_row(ui, |ui| {
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
        .join("portbak.json");

    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))?;

    lock.try_lock_exclusive()
        .map_err(|_| anyhow::anyhow!("portbak is already running with this configuration."))?;

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
        "portbak",
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
        let msg = format!("portbak could not start:\n{e:#}");

        eprintln!("{msg}");

        rfd::MessageDialog::new()
            .set_title("portbak")
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
    fn mixed_row_controls_share_vertical_center() {
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
                centered_row(ui, |ui| {
                    let label = ui.label("Every");
                    let value = ui.add(egui::DragValue::new(&mut 60));
                    let unit = centered_combo(ui, |ui| {
                        egui::ComboBox::from_id_salt("unit_test")
                            .selected_text("Minutes")
                            .show_ui(ui, |_| {})
                    })
                    .inner;
                    assert!(
                        (label.rect.center().y - value.rect.center().y).abs() < 0.5,
                        "label {:?}, value {:?}",
                        label.rect,
                        value.rect
                    );
                    assert!(
                        (label.rect.center().y - unit.response.rect.center().y).abs() < 0.5,
                        "label {:?}, unit {:?}",
                        label.rect,
                        unit.response.rect
                    );
                });
            });
        });
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
        let name = engine::restore::list(&job).unwrap()[0].record.name.clone();
        let config = Config {
            jobs: vec![job],
            ..Config::default()
        };
        let app = App::new(
            config,
            directory.path().join("config.json"),
            tempfile::tempfile().unwrap(),
        );
        (directory, app, name)
    }

    fn select_restore(app: &mut App, name: &str) {
        let job = app.config.jobs[0].clone();
        app.restore_dialog = Some(RestoreDialog {
            request: Uuid::new_v4(),
            source_display: Some(job.source.clone()),
            backups: Some(Ok(engine::restore::list(&job).unwrap())),
            job,
            selected: Some(name.to_owned()),
        });
    }

    #[test]
    fn restore_dialog_pauses_scheduling_and_cancel_changes_nothing() {
        let (_directory, mut app, name) = restore_fixture();
        let due = app.config.jobs[0].next_run;
        select_restore(&mut app, &name);
        let context = egui::Context::default();
        app.tick(&context);
        app.enqueue(app.config.jobs[0].id);
        assert!(app.active.is_none());
        assert!(app.queue.is_empty());
        app.restore_dialog = None;
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
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.active.is_some() && Instant::now() < deadline {
            app.tick(&context);
            std::thread::sleep(Duration::from_millis(10));
        }
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
        app.restore_dialog.as_mut().unwrap().selected = None;
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        app.restore_dialog.as_mut().unwrap().selected = Some(name);
        app.active = Some(Uuid::new_v4());
        app.confirm_restore(&context);
        assert!(app.config.jobs[0].enabled);
        app.active = None;
        app.path = app.path.join("missing/config.json");
        app.confirm_restore(&context);
        assert!(app.active.is_none());
        assert!(app.restore_dialog.is_some());
        assert!(!app.error.is_empty());
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
