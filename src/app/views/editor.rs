use crate::app::rendering::pixel::{self, Hint, PixelButton, PixelCheckbox, Tone};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, Palette, Skin};
use crate::app::views::widgets::{
    alert, caption, centered_row, gib_field, local_time, page_frame, page_scroll_area, path_picker,
    retention_warnings, DATE_TIME,
};
use crate::app::{App, CachedHistory};
use crate::model::{IntervalUnit, Job, Schedule, MAX_JOB_NAME_LENGTH};
use chrono::Utc;
use eframe::egui;
use uuid::Uuid;

const DEFAULT_MAX_COUNT: usize = 10;
const MAX_COUNT_LIMIT: usize = 100_000;
const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;
const WEEKDAY_LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Everything that decides when a job runs; a change restarts its schedule.
type ScheduleKey = (bool, Schedule, u32, u32, [bool; 7], u32, IntervalUnit);

#[derive(Clone, Copy)]
enum JobAction {
    Check,
    Remove,
}

/// What the editor may offer, decided by the app before the job is borrowed mutably.
struct Permissions {
    /// The job is running, queued, or another operation blocks editing.
    busy: bool,
    /// Activation cannot change during a restore or while the Backups tab is open.
    locked: bool,
    /// Running, restoring, or queued work for this job.
    activity: Option<&'static str>,
}

impl App {
    pub(in crate::app) fn show_job_editor(&mut self, ui: &mut egui::Ui) {
        if self.backups_tab && self.selected.is_some() {
            self.show_backups_tab(ui);
            return;
        }
        let ctx = ui.ctx().clone();
        let permissions = self.selected.map(|id| Permissions {
            busy: self.is_busy(id),
            locked: self.scheduling_paused(),
            activity: self.activity_label(id).filter(|&label| label != "Running"),
        });
        let can_add = self.can_add_job();
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
        egui::CentralPanel::default()
            .frame(page_frame(ui))
            .show_inside(ui, |ui| {
                pixel::paint_scenery(ui);
                page_scroll_area(ui, |ui| {
                    let job = jobs.iter_mut().find(|job| Some(job.id) == selected);
                    let (Some(job), Some(permissions)) = (job, permissions) else {
                        add = welcome(ui, can_add);
                        return;
                    };
                    job_tabs(ui, &mut self.backups_tab);
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
            crate::app::scenery::catgirl::trigger(
                &ctx,
                crate::app::scenery::catgirl::Action::NewJob,
            );
        }
        if let Some((id, was_enabled)) = activation {
            self.job_activation_changed(id, was_enabled);
            if let Some(job) = self.job(id) {
                let action = if job.enabled {
                    crate::app::scenery::catgirl::Action::AutomaticEnabled
                } else {
                    crate::app::scenery::catgirl::Action::AutomaticDisabled
                };
                crate::app::scenery::catgirl::trigger(&ctx, action);
            }
        }
        if let Some((id, action)) = action {
            let reaction = match action {
                JobAction::Check => crate::app::scenery::catgirl::Action::CheckNow,
                JobAction::Remove => crate::app::scenery::catgirl::Action::RemoveJob,
            };
            crate::app::scenery::catgirl::trigger(&ctx, reaction);
            self.apply(id, action);
        }
    }

    fn apply(&mut self, id: Uuid, action: JobAction) {
        match action {
            JobAction::Check => self.enqueue(id),
            JobAction::Remove => self.remove_job(id),
        }
    }
}

/// The empty state: a bobbing floppy and one big button; returns whether it was pressed.
fn welcome(ui: &mut egui::Ui, can_add: bool) -> bool {
    let palette = Palette::of(ui);
    // Liquid glass follows iOS's empty state: a still, secondary symbol and no arcade prompt.
    let liquid = Skin::of(ui.ctx()).finish == style::Finish::Liquid;
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        let bob = if Skin::of(ui.ctx()).effects.hearts || liquid || pixel::blink(ui, 0.6) {
            0.0
        } else {
            6.0
        };
        ui.add_space(bob);
        if Skin::of(ui.ctx()).effects.hearts {
            crate::app::scenery::catgirl::show_welcome(ui);
        } else {
            let symbol = if liquid { palette.dim } else { palette.hot };
            sprites::show(ui, Sprite::Floppy, 10.0, symbol);
        }
        ui.add_space(24.0 - bob);
        let wording = &Skin::of(ui.ctx()).wording;
        let title = if liquid { palette.ink } else { palette.gold };
        pixel::display_label(ui, wording.welcome_title, 24.0, title);
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
        if liquid {
            return clicked;
        }
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
    job_status(ui, job, permissions.activity);
    ui.add_space(14.0);
    // Busy jobs stay read-only without greying out, so quick checks do not flash the form.
    let unchanged = permissions.busy.then(|| job.clone());
    pixel::section(ui, "Folders", Sprite::Folder, palette.accent, |ui| {
        folder_settings(ui, job, permissions.busy)
    });
    pixel::section(ui, "Scheduling", Sprite::Clock, palette.hot, |ui| {
        schedule_settings(ui, job)
    });
    pixel::section(ui, "Retention", Sprite::Shield, palette.good, |ui| {
        retention_settings(ui, job, summary)
    });
    if let Some(unchanged) = unchanged {
        *job = unchanged;
        // Edits are discarded, but a job without a runnable schedule must still stop.
        if job.enabled && !job.has_runnable_schedule() {
            job.enabled = false;
            job.reschedule(Utc::now());
        }
    }
    action
}

/// Keep the identity centered independently of the unequal action groups.
fn job_banner(ui: &mut egui::Ui, job: &mut Job, permissions: &Permissions) -> Option<JobAction> {
    const ACTION_INSET: f32 = 148.0;
    let narrow = ui.available_width() < 620.0;
    let identity_height = if Skin::of(ui.ctx()).finish == style::Finish::Gothic {
        80.0
    } else {
        60.0
    };
    if narrow {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), identity_height),
            egui::Sense::hover(),
        );
        centered_identity(ui, job, rect);
        ui.add_space(8.0);
    }
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), identity_height),
        egui::Sense::hover(),
    );
    let mut action = None;
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            activation_button(ui, job, permissions.locked);
            if job_action_button(
                ui,
                "Check now",
                Sprite::Play,
                Tone::Accent,
                permissions.busy,
            ) {
                action = Some(JobAction::Check);
            }
        },
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            if job_action_button(
                ui,
                "Remove job",
                Sprite::Trash,
                Tone::Danger,
                permissions.busy,
            ) {
                action = Some(JobAction::Remove);
            }
        },
    );
    if !narrow {
        centered_identity(ui, job, rect.shrink2(egui::vec2(ACTION_INSET, 0.0)));
    }
    action
}

/// Busy buttons keep their look and ignore clicks, so quick checks do not flash them.
fn job_action_button(
    ui: &mut egui::Ui,
    label: &str,
    sprite: Sprite,
    tone: Tone,
    busy: bool,
) -> bool {
    let button = PixelButton::new(label)
        .icon_only()
        .sprite(sprite)
        .tone(tone)
        .min_size(egui::vec2(52.0, 52.0));
    let tooltip = if busy {
        "Available when this job's current operation finishes"
    } else {
        label
    };
    ui.add(button).hint(tooltip).clicked() && !busy
}

fn centered_identity(ui: &mut egui::Ui, job: &Job, rect: egui::Rect) {
    let palette = Palette::of(ui);
    let icon_size = if Skin::of(ui.ctx()).finish == crate::app::theme::style::Finish::Gothic {
        egui::Vec2::splat(80.0)
    } else {
        sprites::size(ui.ctx(), Sprite::Floppy, 5.0)
    };
    let font = egui::TextStyle::Heading.resolve(ui.style());
    let name_width = ui
        .painter()
        .layout_no_wrap(job.display_name().to_owned(), font, palette.ink)
        .size()
        .x;
    let width = (icon_size.x + ui.spacing().item_spacing.x + 6.0 + name_width).min(rect.width());
    let centered = egui::Rect::from_center_size(rect.center(), egui::vec2(width, rect.height()));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(centered)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            if Skin::of(ui.ctx()).finish == crate::app::theme::style::Finish::Gothic {
                let (rect, _) = ui.allocate_exact_size(icon_size, egui::Sense::hover());
                crate::app::scenery::catgirl::paint_emblem(ui, rect);
            } else {
                sprites::show(
                    ui,
                    Sprite::Floppy,
                    5.0,
                    style::job_symbol_color(ui.ctx(), job.enabled),
                );
            }
            ui.add_space(6.0);
            ui.add(egui::Label::new(egui::RichText::new(job.display_name()).heading()).truncate());
        },
    );
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
        "Automatic backups cannot be changed during restore or while the Backups tab is open"
    } else if !job.enabled && !folders_chosen {
        "Choose source and destination folders to enable automatic backups"
    } else if !job.enabled && !job.has_runnable_schedule() {
        "Select at least one weekday to enable automatic backups"
    } else {
        label
    };
    if ui
        .add_enabled(can_toggle, button)
        .hint(tooltip)
        .disabled_hint(tooltip)
        .clicked()
    {
        job.enabled = !job.enabled;
        job.reschedule(Utc::now());
    }
}

fn folder_settings(ui: &mut egui::Ui, job: &mut Job, read_only: bool) {
    caption(ui, "Job name");
    ui.add(
        egui::TextEdit::singleline(&mut job.name)
            .char_limit(MAX_JOB_NAME_LENGTH)
            .desired_width(ui.available_width().min(420.0))
            .margin(egui::vec2(8.0, 4.0)),
    );
    ui.add_space(ui.spacing().item_spacing.y);
    path_picker(ui, "Source folder", &mut job.source, read_only);
    ui.add_space(ui.spacing().item_spacing.y);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(
            &mut job.auto_destination,
            "Auto destination folder",
        ));
    });
    if path_picker(ui, "Destination folder", &mut job.target, read_only) {
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
    let before = schedule_key(job);
    centered_row(ui, |ui| {
        ui.add(PixelCheckbox::new(
            &mut job.check_on_enable,
            "Check immediately when enabled",
        ));
    });
    scheduling_fields(ui, job, true);
    if schedule_key(job) != before {
        job.reschedule(Utc::now());
    }
}

pub(super) fn schedule_controls(ui: &mut egui::Ui, job: &mut Job) {
    scheduling_fields(ui, job, false);
}

fn scheduling_fields(ui: &mut egui::Ui, job: &mut Job, show_next_check: bool) {
    centered_row(ui, |ui| {
        pixel::segmented(
            ui,
            &mut job.schedule,
            &[
                (Schedule::Interval, "Interval", Sprite::Interval),
                (Schedule::Daily, "Daily", Sprite::Daily),
                (Schedule::Weekdays, "Weekdays", Sprite::Weekdays),
            ],
        );
    });
    match job.schedule {
        Schedule::Interval => interval_row(ui, job),
        Schedule::Daily => time_row(ui, job, show_next_check),
        Schedule::Weekdays => {
            weekday_row(ui, job);
            time_row(ui, job, show_next_check);
        }
    }
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
    .hint(label)
    .changed()
}

fn time_row(ui: &mut egui::Ui, job: &mut Job, show_next_check: bool) {
    let readout_height = ui
        .painter()
        .layout_no_wrap(
            "00".into(),
            style::label_font(ui.ctx()),
            Palette::of(ui).ink,
        )
        .size()
        .y
        + style::well(ui.ctx()).total_margin().sum().y;
    let row_height =
        ui.spacing()
            .interact_size
            .y
            .max(if show_next_check { readout_height } else { 0.0 });
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), row_height),
        egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
        |ui| {
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
                    .min_decimals(0)
                    .custom_formatter(|value, _| format!("{:02}", value as u32)),
            );
            if show_next_check {
                ui.add_space(12.0);
                let next = if job.has_runnable_schedule() {
                    local_time(job.next_after(Utc::now()), "%d.%m.%Y %H:%M")
                } else {
                    "Select at least one weekday".into()
                };
                style::well(ui.ctx())
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(next.to_uppercase())
                                .font(style::label_font(ui.ctx()))
                                .color(Palette::of(ui).good),
                        );
                    })
                    .response
                    .hint("Next check according to the current schedule");
            }
        },
    );
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
                    gib_field(ui, true, bytes);
                },
            );
            centered_row(ui, |ui| {
                ui.add(PixelCheckbox::new(
                    &mut job.count_protected,
                    "Count protected backups toward retention limits",
                ))
                .hint("Protected backups are never deleted by retention");
            });
        });
    });
    ui.horizontal_top(|ui| {
        sprites::show(ui, Sprite::Shield, 1.0, palette.good);
        let note = egui::RichText::new(
            "The latest successful backup and protected versions are always kept.",
        )
        .small()
        .color(palette.dim);
        ui.add(egui::Label::new(note).wrap());
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

pub(super) fn job_status(ui: &mut egui::Ui, job: &Job, activity: Option<&str>) {
    let palette = Palette::of(ui);
    let last_backup = job
        .last_success
        .map_or_else(|| "–".to_owned(), |time| local_time(time, DATE_TIME));
    ui.horizontal_wrapped(|ui| {
        if let Some(activity) = activity {
            ui.label(egui::RichText::new(activity).small().color(palette.gold));
            ui.label(egui::RichText::new("·").small().color(palette.dim));
        }
        ui.label(
            egui::RichText::new(format!("Last backup: {last_backup}"))
                .small()
                .color(palette.dim),
        );
        if !job.status.starts_with("Error") {
            ui.label(
                egui::RichText::new(format!("· {}", job.status))
                    .small()
                    .color(palette.dim),
            );
        }
    });
    if job.status.starts_with("Error") {
        alert(ui, palette.bad, &job.status);
    }
}

pub(super) fn job_tabs(ui: &mut egui::Ui, backups: &mut bool) {
    ui.horizontal_wrapped(|ui| {
        pixel::segmented(
            ui,
            backups,
            &[
                (false, "Job settings", Sprite::JobSettings),
                (true, "Backups", Sprite::Chest),
            ],
        );
    });
    ui.add_space(14.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_controls_and_preview_are_centered_in_every_theme() {
        for skin in crate::app::theme::skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let ctx = egui::Context::default();
                style::install(&ctx, skin);
                ctx.set_theme(theme);
                let mut job = Job {
                    schedule: Schedule::Daily,
                    minute: 5,
                    ..Job::default()
                };
                for _ in 0..3 {
                    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                        ui.set_width(800.0);
                        time_row(ui, &mut job, true);
                    });
                    let mut minute_y = None;
                    let mut preview_y = None;
                    for shape in &output.shapes {
                        if let egui::Shape::Text(text) = &shape.shape {
                            let value = text.galley.text();
                            let center = text.pos.y + text.galley.size().y / 2.0;
                            if value == "05" {
                                minute_y = Some(center);
                            }
                            if value.contains('.') && value.contains(':') {
                                preview_y = Some(center);
                            }
                        }
                    }
                    let difference = (minute_y.expect("two-digit minute")
                        - preview_y.expect("next-check readout"))
                    .abs();
                    assert!(
                        difference <= 1.0,
                        "{} {theme:?}: difference={difference}",
                        skin.id
                    );
                }
            }
        }
    }
}
