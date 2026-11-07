use crate::app::rendering::pixel::{self, PixelButton, PixelCheckbox, Tone};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, label_font, Finish, Palette, Skin};
use crate::app::views::widgets::{
    alert, caption, centered_row, local_time, path_picker, retention_warnings, DATE_MINUTE,
    DATE_TIME,
};
use crate::app::{App, CachedHistory, HistoryMode};
use crate::model::{IntervalUnit, Job, Schedule, MAX_JOB_NAME_LENGTH};
use chrono::Utc;
use eframe::egui;
use uuid::Uuid;

const DEFAULT_MAX_COUNT: usize = 10;
const MAX_COUNT_LIMIT: usize = 100_000;
const BYTES_PER_GIB: f64 = 1_073_741_824.0;
const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;
const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Everything that decides when a job runs; a change restarts its schedule.
type ScheduleKey = (bool, Schedule, u32, u32, [bool; 7], u32, IntervalUnit);

#[derive(Clone, Copy)]
enum JobAction {
    Check,
    Restore,
    Manage,
    Remove,
}

/// What the editor may offer, decided by the app before the job is borrowed mutably.
struct Permissions {
    /// The job is running, queued, or another operation blocks editing.
    busy: bool,
    /// History and restore need an idle worker and an empty queue.
    idle: bool,
    /// Activation cannot change during a restore or while history is open.
    locked: bool,
    backup_running: bool,
}

impl App {
    pub(in crate::app) fn show_job_editor(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let permissions = self.selected.map(|id| Permissions {
            busy: self.is_busy(id),
            idle: self.is_idle(),
            locked: self.scheduling_paused(),
            backup_running: self.active == Some(id) && !self.restoring,
        });
        let can_add = !self.scheduling_paused();
        let selected = self.selected;
        let summaries = &self.history_cache;
        let app_dir = self
            .path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let jobs = &mut self.config.jobs;
        let mut action = None;
        let mut activation = None;
        let mut add = false;
        let frame =
            egui::Frame::central_panel(&ctx.global_style()).inner_margin(egui::Margin::same(22));
        egui::CentralPanel::default()
            .frame(frame)
            .show_inside(ui, |ui| {
                pixel::paint_scenery(ui);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let job = jobs.iter_mut().find(|job| Some(job.id) == selected);
                    let (Some(job), Some(permissions)) = (job, permissions) else {
                        add = welcome(ui, can_add);
                        return;
                    };
                    let was_enabled = job.enabled;
                    let previous = job.clone();
                    let summary = summaries
                        .get(&job.id)
                        .filter(|summary| summary.target == job.target);
                    action =
                        job_editor(ui, job, &permissions, summary).map(|action| (job.id, action));
                    if !permissions.busy {
                        if let Err(error) = job.update_auto_destination(&previous, app_dir) {
                            self.error =
                                format!("Could not update automatic destination: {error:#}");
                        }
                    }
                    if was_enabled != job.enabled {
                        activation = Some((job.id, was_enabled));
                    }
                });
            });
        if add {
            self.add_job();
        }
        if let Some((id, was_enabled)) = activation {
            self.job_activation_changed(id, was_enabled);
        }
        if let Some((id, action)) = action {
            self.apply(id, action, &ctx);
        }
    }

    fn apply(&mut self, id: Uuid, action: JobAction, ctx: &egui::Context) {
        match action {
            JobAction::Check => self.enqueue(id),
            JobAction::Restore => self.open_history(id, HistoryMode::Restore, ctx),
            JobAction::Manage => self.open_history(id, HistoryMode::Manage, ctx),
            JobAction::Remove => self.remove_job(id),
        }
    }
}

/// The empty state: a bobbing floppy and one big button; returns whether it was pressed.
fn welcome(ui: &mut egui::Ui, can_add: bool) -> bool {
    let palette = Palette::of(ui);
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        let bob = if pixel::blink(ui, 0.6) { 0.0 } else { 6.0 };
        ui.add_space(bob);
        if Skin::of(ui.ctx()).effects.hearts {
            crate::app::scenery::catgirl::show_welcome(ui);
        } else {
            sprites::show(ui, Sprite::Floppy, 10.0, palette.hot);
        }
        ui.add_space(24.0 - bob);
        let wording = &Skin::of(ui.ctx()).wording;
        pixel::display_label(ui, wording.welcome_title, 24.0, palette.gold);
        ui.add_space(14.0);
        ui.heading("Your folders. Safely backed up.");
        ui.label(
            egui::RichText::new("Create a backup job and select source and destination folders.")
                .color(palette.dim),
        );
        ui.add_space(22.0);
        let start = PixelButton::new("New job")
            .sprite(Sprite::Plus)
            .tone(Tone::Accent)
            .min_size(egui::vec2(240.0, 52.0));
        let clicked = ui.add_enabled(can_add, start).clicked();
        ui.add_space(14.0);
        let prompt = if pixel::blink(ui, 0.5) {
            wording.welcome_prompt
        } else {
            " "
        };
        pixel::display_label(ui, prompt, 10.0, palette.hot);
        clicked
    })
    .inner
}

fn job_editor(
    ui: &mut egui::Ui,
    job: &mut Job,
    permissions: &Permissions,
    summary: Option<&CachedHistory>,
) -> Option<JobAction> {
    let palette = Palette::of(ui);
    let action = job_banner(ui, job, permissions);
    if !job.enabled && permissions.backup_running {
        alert(
            ui,
            palette.gold,
            "Automatic backups disabled; current backup will finish.",
        );
    }
    ui.add_space(14.0);
    ui.add_enabled_ui(!permissions.busy, |ui| {
        pixel::section(ui, "Folders", Sprite::Folder, palette.accent, |ui| {
            folder_settings(ui, job)
        });
        pixel::section(ui, "Scheduling", Sprite::Clock, palette.hot, |ui| {
            schedule_settings(ui, job)
        });
        pixel::section(ui, "Retention", Sprite::Shield, palette.good, |ui| {
            retention_settings(ui, job, summary)
        });
    });
    let status_action = pixel::section(ui, "Status", Sprite::Monitor, palette.gold, |ui| {
        status_panel(ui, job, permissions)
    });
    action.or(status_action)
}

/// Keep job actions beside the identity, or below it in narrow windows.
fn job_banner(ui: &mut egui::Ui, job: &mut Job, permissions: &Permissions) -> Option<JobAction> {
    let mut action = None;
    if ui.available_width() < 620.0 {
        ui.horizontal(|ui| job_identity(ui, job));
        ui.add_space(8.0);
        action = job_controls(ui, job, permissions);
    } else {
        ui.horizontal(|ui| {
            ui.allocate_ui(
                egui::vec2((ui.available_width() - 188.0).max(0.0), 60.0),
                |ui| {
                    ui.horizontal(|ui| job_identity(ui, job));
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                action = job_controls(ui, job, permissions);
            });
        });
    }
    action
}

fn job_controls(ui: &mut egui::Ui, job: &mut Job, permissions: &Permissions) -> Option<JobAction> {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        activation_button(ui, job, permissions.locked);
        let mut action = None;
        for (label, sprite, tone, candidate) in [
            ("Check now", Sprite::Play, Tone::Accent, JobAction::Check),
            ("Remove job", Sprite::Trash, Tone::Danger, JobAction::Remove),
        ] {
            let button = PixelButton::new(label)
                .icon_only()
                .sprite(sprite)
                .tone(tone)
                .min_size(egui::vec2(52.0, 52.0));
            if ui
                .add_enabled(!permissions.busy, button)
                .on_hover_text(label)
                .on_disabled_hover_text(label)
                .clicked()
            {
                action = Some(candidate);
            }
        }
        action
    })
    .inner
}

fn job_identity(ui: &mut egui::Ui, job: &Job) {
    let palette = Palette::of(ui);
    let tint = if job.enabled {
        palette.good
    } else {
        palette.metal
    };
    let tint = style::decorative_color(ui.ctx(), tint);
    sprites::show(ui, Sprite::Floppy, 5.0, tint);
    ui.add_space(6.0);
    ui.vertical(|ui| {
        ui.add(egui::Label::new(egui::RichText::new(&job.name).heading()).truncate());
        pixel::status_row(ui, |ui| {
            let (color, text) = if job.enabled {
                (palette.good, "Automatic backups enabled")
            } else {
                (palette.dim, "Automatic backups disabled")
            };
            pixel::led(ui, color, job.enabled);
            pixel::glowing_label(ui, text, color);
        });
    });
}

fn activation_button(ui: &mut egui::Ui, job: &mut Job, locked: bool) {
    let (label, tone) = if job.enabled {
        ("Disable automatic backups", Tone::Danger)
    } else {
        ("Enable automatic backups", Tone::Good)
    };
    let button = PixelButton::new(label)
        .icon_only()
        .sprite(Sprite::Power)
        .tone(tone)
        .min_size(egui::vec2(52.0, 52.0));
    let folders_chosen = !job.source.as_os_str().is_empty() && !job.target.as_os_str().is_empty();
    let can_toggle = (job.enabled || (folders_chosen && job.has_runnable_schedule())) && !locked;
    let tooltip = if locked {
        "Automatic backups cannot be changed during restore or while backup history is open"
    } else if !job.enabled && !folders_chosen {
        "Choose source and destination folders to enable automatic backups"
    } else if !job.enabled && !job.has_runnable_schedule() {
        "Select at least one weekday to enable automatic backups"
    } else {
        label
    };
    if ui
        .add_enabled(can_toggle, button)
        .on_hover_text(tooltip)
        .on_disabled_hover_text(tooltip)
        .clicked()
    {
        job.enabled = !job.enabled;
        job.reschedule(Utc::now());
    }
}

fn folder_settings(ui: &mut egui::Ui, job: &mut Job) {
    caption(ui, "Job name");
    ui.add(
        egui::TextEdit::singleline(&mut job.name)
            .char_limit(MAX_JOB_NAME_LENGTH)
            .desired_width(ui.available_width().min(420.0))
            .margin(egui::vec2(8.0, 4.0)),
    );
    ui.add_space(ui.spacing().item_spacing.y);
    path_picker(ui, "Source folder", &mut job.source);
    ui.add_space(ui.spacing().item_spacing.y);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(
            &mut job.auto_destination,
            "Auto destination folder",
        ));
    });
    if path_picker(ui, "Destination folder", &mut job.target) {
        job.auto_destination = false;
    }
    ui.add_space(ui.spacing().item_spacing.y);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(&mut job.zip, "Compress as ZIP"));
        let tint = style::decorative_color(ui.ctx(), Palette::of(ui).gold);
        sprites::show(ui, Sprite::Package, 2.0, tint);
    });
}

fn schedule_key(job: &Job) -> ScheduleKey {
    (
        job.enabled,
        job.schedule,
        job.hour,
        job.minute,
        job.weekdays,
        job.interval_value,
        job.interval_unit,
    )
}

fn schedule_settings(ui: &mut egui::Ui, job: &mut Job) {
    let palette = Palette::of(ui);
    let before = schedule_key(job);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(
            &mut job.check_on_enable,
            "Check immediately when enabled",
        ));
    });
    schedule_controls(ui, job);
    if schedule_key(job) != before {
        job.reschedule(Utc::now());
    }
    let next = if !job.enabled {
        "disabled".to_owned()
    } else {
        job.next_run.map_or_else(
            || "pending".to_owned(),
            |time| local_time(time, DATE_MINUTE),
        )
    };
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let color = if matches!(Skin::of(ui.ctx()).finish, Finish::Modern | Finish::Gothic) {
            palette.hot
        } else {
            palette.accent
        };
        readout(ui, "Next check", &next, color);
        if let Some(remaining) = job.countdown(Utc::now()) {
            ui.add_space(24.0);
            readout(ui, "Next check in", &remaining, palette.good);
        }
    });
}

pub(super) fn schedule_controls(ui: &mut egui::Ui, job: &mut Job) {
    centered_row(ui, |ui| {
        let modes: [(Schedule, &str, Sprite); 3] = [
            (Schedule::Interval, "Interval", Sprite::Rewind),
            (Schedule::Daily, "Daily", Sprite::Sun),
            (Schedule::Weekdays, "Weekdays", Sprite::Clock),
        ];
        for (schedule, label, sprite) in modes {
            pixel::choice(ui, &mut job.schedule, schedule, label, Some(sprite));
        }
    });
    match job.schedule {
        Schedule::Interval => interval_row(ui, job),
        Schedule::Daily => time_row(ui, job),
        Schedule::Weekdays => {
            time_row(ui, job);
            weekday_row(ui, job);
        }
    }
}

/// A captioned value in the display face, like a digital readout.
fn readout(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    ui.vertical(|ui| {
        caption(ui, label);
        style::well(ui.ctx()).show(ui, |ui| {
            ui.label(
                egui::RichText::new(value.to_uppercase())
                    .font(label_font(ui.ctx()))
                    .color(color),
            );
        });
    });
}

fn interval_row(ui: &mut egui::Ui, job: &mut Job) {
    let total = job.interval_seconds();
    let mut hours = total / 3600;
    let mut minutes = total / 60 % 60;
    let mut seconds = total % 60;
    centered_row(ui, |ui| {
        ui.label("Every");
        let mut changed = duration_field(ui, &mut hours, 8760, "Hours");
        ui.label(":");
        changed |= duration_field(ui, &mut minutes, 59, "Minutes");
        ui.label(":");
        changed |= duration_field(ui, &mut seconds, 59, "Seconds");
        ui.label("(hh:mm:ss)");
        if changed {
            job.set_interval_seconds(hours * 3600 + minutes * 60 + seconds);
        }
    });
}

fn duration_field(ui: &mut egui::Ui, value: &mut u32, maximum: u32, label: &str) -> bool {
    ui.add(
        egui::DragValue::new(value)
            .range(0..=maximum)
            .custom_formatter(|value, _| format!("{:02}", value as u32)),
    )
    .on_hover_text(label)
    .changed()
}

fn time_row(ui: &mut egui::Ui, job: &mut Job) {
    centered_row(ui, |ui| {
        ui.label("Local time");
        ui.add(
            egui::DragValue::new(&mut job.hour)
                .range(0..=23)
                .min_decimals(0),
        );
        ui.label(":");
        ui.add(
            egui::DragValue::new(&mut job.minute)
                .range(0..=59)
                .min_decimals(0),
        );
    });
}

fn weekday_row(ui: &mut egui::Ui, job: &mut Job) {
    centered_row(ui, |ui| {
        for (selected, label) in job.weekdays.iter_mut().zip(WEEKDAY_LABELS) {
            let day = PixelButton::new(label)
                .selected(*selected)
                .min_size(egui::vec2(58.0, 0.0));
            if ui.add(day).clicked() {
                *selected = !*selected;
            }
        }
    });
    if !job.has_runnable_schedule() {
        alert(ui, Palette::of(ui).bad, "Select at least one weekday");
        job.enabled = false;
    }
}

fn retention_settings(ui: &mut egui::Ui, job: &mut Job, summary: Option<&CachedHistory>) {
    let palette = Palette::of(ui);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(
            &mut job.rolling,
            "Enable rolling retention – delete oldest backups automatically",
        ));
    });
    ui.add_enabled_ui(job.rolling, |ui| {
        ui.indent("limits", |ui| {
            optional_limit(
                ui,
                "Maximum count",
                &mut job.max_count,
                DEFAULT_MAX_COUNT,
                |ui, count| {
                    ui.add(egui::DragValue::new(count).range(1..=MAX_COUNT_LIMIT));
                },
            );
            optional_limit(
                ui,
                "Maximum total size",
                &mut job.max_bytes,
                DEFAULT_MAX_BYTES,
                |ui, bytes| {
                    let mut gib = *bytes as f64 / BYTES_PER_GIB;
                    let editor = egui::DragValue::new(&mut gib)
                        .speed(0.1)
                        .range(0.001..=100_000.0)
                        .suffix(" GiB");
                    if ui.add(editor).changed() {
                        *bytes = (gib * BYTES_PER_GIB) as u64;
                    }
                },
            );
            centered_row(ui, |ui| {
                ui.add(PixelCheckbox::new(
                    &mut job.count_protected,
                    "Count protected backups toward retention limits",
                ))
                .on_hover_text("Protected backups are never deleted by retention");
            });
        });
    });
    ui.horizontal(|ui| {
        sprites::show(ui, Sprite::Shield, 1.0, palette.good);
        ui.label(
            egui::RichText::new(
                "The latest successful backup and protected versions are always kept.",
            )
            .small()
            .color(palette.dim),
        );
    });
    match summary.map(|summary| &summary.records) {
        Some(Ok(records)) => retention_warnings(ui, job, records),
        Some(Err(error)) => {
            alert(
                ui,
                palette.dim,
                &format!("Backup history unavailable: {error}"),
            );
        }
        None => {}
    }
}

/// A checkbox that switches a limit on (starting at `default`) or off, plus its value editor.
fn optional_limit<T>(
    ui: &mut egui::Ui,
    label: &str,
    limit: &mut Option<T>,
    default: T,
    edit: impl FnOnce(&mut egui::Ui, &mut T),
) {
    centered_row(ui, |ui| {
        let mut active = limit.is_some();
        if ui.add(PixelCheckbox::new(&mut active, label)).changed() {
            *limit = active.then_some(default);
        }
        if let Some(value) = limit {
            edit(ui, value);
        }
    });
}

fn status_panel(ui: &mut egui::Ui, job: &Job, permissions: &Permissions) -> Option<JobAction> {
    let palette = Palette::of(ui);
    let last_backup = job
        .last_success
        .map_or_else(|| "–".to_owned(), |time| local_time(time, DATE_TIME));
    let color = style::decorative_color(ui.ctx(), palette.gold);
    readout(ui, "Last backup", &last_backup, color);
    ui.add_space(2.0);
    if job.status.starts_with("Error") {
        alert(ui, palette.bad, &job.status);
    } else {
        ui.add(egui::Label::new(&job.status).wrap());
    }
    ui.add_space(8.0);
    let action = action_buttons(ui, permissions);
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Backups are full copies. Links are skipped. Each check reads all file contents. \
             Restore with Restore backup… or by copying/extracting.",
        )
        .small()
        .color(palette.dim),
    );
    action
}

fn action_buttons(ui: &mut egui::Ui, permissions: &Permissions) -> Option<JobAction> {
    let can_edit = !permissions.busy;
    let can_open_history = can_edit && permissions.idle;
    let buttons = [
        (
            "Restore backup…",
            Sprite::Rewind,
            Tone::Plain,
            can_open_history,
            JobAction::Restore,
        ),
        (
            "Manage backups…",
            Sprite::Chest,
            Tone::Plain,
            can_open_history,
            JobAction::Manage,
        ),
    ];
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
        for (label, sprite, tone, enabled, action) in buttons {
            let button = PixelButton::new(label).sprite(sprite).tone(tone);
            if ui.add_enabled(enabled, button).clicked() {
                clicked = Some(action);
            }
        }
    });
    clicked
}
