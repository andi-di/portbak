use super::pixel::{self, PixelButton, PixelCheckbox, Tone};
use super::sprites::{self, Sprite};
use super::style::{label_font, Palette};
use super::widgets::{
    alert, caption, centered_combo, centered_row, local_time, path_picker, retention_warnings,
    DATE_MINUTE, DATE_TIME,
};
use super::{App, CachedHistory, HistoryMode};
use crate::model::{IntervalUnit, Job, Schedule};
use chrono::Utc;
use eframe::egui;
use uuid::Uuid;

const DEFAULT_MAX_COUNT: usize = 10;
const MAX_COUNT_LIMIT: usize = 100_000;
const BYTES_PER_GIB: f64 = 1_073_741_824.0;
const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;
const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const CONTENT_MAX_WIDTH: f32 = 880.0;

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
    pub(super) fn show_job_editor(&mut self, ctx: &egui::Context) {
        let permissions = self.selected.map(|id| Permissions {
            busy: self.is_busy(id),
            idle: self.is_idle(),
            locked: self.scheduling_paused(),
            backup_running: self.active == Some(id) && !self.restoring,
        });
        let can_add = !self.scheduling_paused();
        let selected = self.selected;
        let summaries = &self.history_cache;
        let jobs = &mut self.config.jobs;
        let mut action = None;
        let mut disabled = false;
        let mut add = false;
        let frame = egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(22));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            pixel::paint_scenery(ui);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_max_width(CONTENT_MAX_WIDTH.min(ui.available_width()));
                let job = jobs.iter_mut().find(|job| Some(job.id) == selected);
                let (Some(job), Some(permissions)) = (job, permissions) else {
                    add = welcome(ui, can_add);
                    return;
                };
                let was_enabled = job.enabled;
                let summary = summaries
                    .get(&job.id)
                    .filter(|summary| summary.target == job.target);
                action = job_editor(ui, job, &permissions, summary).map(|action| (job.id, action));
                disabled = was_enabled && !job.enabled;
            });
        });
        if add {
            self.add_job();
        }
        if let (true, Some(id)) = (disabled, selected) {
            self.remove_pending(id);
        }
        if let Some((id, action)) = action {
            self.apply(id, action, ctx);
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
        sprites::show(ui, sprites::FLOPPY, 10.0, palette.hot);
        ui.add_space(24.0 - bob);
        pixel::display_label(ui, "INSERT DISK", 24.0, palette.gold);
        ui.add_space(14.0);
        ui.heading("Your folders. Safely backed up.");
        ui.label(
            egui::RichText::new("Create a backup job and select source and destination folders.")
                .color(palette.dim),
        );
        ui.add_space(22.0);
        let start = PixelButton::new("New job")
            .sprite(sprites::PLUS)
            .tone(Tone::Accent)
            .min_size(egui::vec2(240.0, 52.0));
        let clicked = ui.add_enabled(can_add, start).clicked();
        ui.add_space(14.0);
        let prompt = if pixel::blink(ui, 0.5) {
            "PRESS START"
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
    job_banner(ui, job, permissions.locked);
    if !job.enabled && permissions.backup_running {
        alert(
            ui,
            palette.gold,
            "Automatic backups disabled; current backup will finish.",
        );
    }
    ui.add_space(14.0);
    ui.add_enabled_ui(!permissions.busy, |ui| {
        pixel::section(ui, "Folders", sprites::FOLDER, palette.accent, |ui| {
            folder_settings(ui, job)
        });
        pixel::section(ui, "Scheduling", sprites::CLOCK, palette.hot, |ui| {
            schedule_settings(ui, job)
        });
        pixel::section(ui, "Retention", sprites::SHIELD, palette.good, |ui| {
            retention_settings(ui, job, summary)
        });
    });
    pixel::section(ui, "Status", sprites::MONITOR, palette.gold, |ui| {
        status_panel(ui, job, permissions)
    })
}

/// The job's floppy, name, state lamp and the power switch for automatic backups.
fn job_banner(ui: &mut egui::Ui, job: &mut Job, locked: bool) {
    let palette = Palette::of(ui);
    ui.horizontal(|ui| {
        let tint = if job.enabled {
            palette.good
        } else {
            palette.metal
        };
        sprites::show(ui, sprites::FLOPPY, 5.0, tint);
        ui.add_space(6.0);
        let activation_width = 320.0;
        ui.allocate_ui(
            egui::vec2(ui.available_width() - activation_width, 60.0),
            |ui| {
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(egui::RichText::new(&job.name).heading()).truncate());
                    ui.horizontal(|ui| {
                        let (color, text) = if job.enabled {
                            (palette.good, "Automatic backups enabled")
                        } else {
                            (palette.dim, "Automatic backups disabled")
                        };
                        pixel::led(ui, color, job.enabled);
                        ui.label(egui::RichText::new(text).color(color));
                    });
                });
            },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            activation_button(ui, job, locked);
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
        .sprite(sprites::POWER)
        .tone(tone)
        .min_size(egui::vec2(300.0, 52.0));
    let can_toggle = (job.enabled || job.has_runnable_schedule()) && !locked;
    if ui.add_enabled(can_toggle, button).clicked() {
        job.enabled = !job.enabled;
        job.reschedule(Utc::now());
    }
}

fn folder_settings(ui: &mut egui::Ui, job: &mut Job) {
    caption(ui, "Job name");
    ui.add(
        egui::TextEdit::singleline(&mut job.name)
            .desired_width(ui.available_width().min(420.0))
            .margin(egui::vec2(8.0, 4.0)),
    );
    ui.add_space(6.0);
    path_picker(ui, "Source folder", &mut job.source);
    ui.add_space(6.0);
    path_picker(ui, "Destination folder", &mut job.target);
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(PixelCheckbox::new(&mut job.zip, "Compress as ZIP"));
        sprites::show(ui, sprites::PACKAGE, 2.0, Palette::of(ui).gold);
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
        let modes: [(Schedule, &str, Sprite); 3] = [
            (Schedule::Interval, "Interval", sprites::REWIND),
            (Schedule::Daily, "Daily", sprites::SUN),
            (Schedule::Weekdays, "Weekdays", sprites::CLOCK),
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
        readout(ui, "Next check", &next, palette.accent);
        if let Some(remaining) = job.countdown(Utc::now()) {
            ui.add_space(24.0);
            readout(ui, "Next check in", &remaining, palette.good);
        }
    });
}

/// A captioned value in the display face, like a digital readout.
fn readout(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    ui.vertical(|ui| {
        caption(ui, label);
        let palette = Palette::of(ui);
        egui::Frame::new()
            .fill(palette.sunken)
            .stroke(egui::Stroke::new(2.0_f32, palette.outline))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(value.to_uppercase())
                        .font(label_font(ui.ctx()))
                        .color(color),
                );
            });
    });
}

fn interval_row(ui: &mut egui::Ui, job: &mut Job) {
    centered_row(ui, |ui| {
        ui.label("Every");
        ui.add(
            egui::DragValue::new(&mut job.interval_value).range(1..=job.interval_unit.maximum()),
        );
        centered_combo(ui, |ui| {
            egui::ComboBox::from_id_salt("interval_unit")
                .selected_text(job.interval_unit.label())
                .show_ui(ui, |ui| {
                    for unit in [
                        IntervalUnit::Seconds,
                        IntervalUnit::Minutes,
                        IntervalUnit::Hours,
                    ] {
                        ui.selectable_value(&mut job.interval_unit, unit, unit.label());
                    }
                })
        });
        // A smaller unit can lower the maximum below the current value.
        job.interval_value = job.interval_value.clamp(1, job.interval_unit.maximum());
    });
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
    ui.add(PixelCheckbox::new(
        &mut job.rolling,
        "Enable rolling retention – delete oldest backups automatically",
    ));
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
        });
    });
    ui.horizontal(|ui| {
        sprites::show(ui, sprites::SHIELD, 1.0, palette.good);
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
    readout(ui, "Last backup", &last_backup, palette.gold);
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
            "Check now",
            sprites::PLAY,
            Tone::Accent,
            can_edit,
            JobAction::Check,
        ),
        (
            "Restore backup…",
            sprites::REWIND,
            Tone::Plain,
            can_open_history,
            JobAction::Restore,
        ),
        (
            "Manage backups…",
            sprites::CHEST,
            Tone::Plain,
            can_open_history,
            JobAction::Manage,
        ),
        (
            "Remove job",
            sprites::TRASH,
            Tone::Danger,
            can_edit,
            JobAction::Remove,
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
