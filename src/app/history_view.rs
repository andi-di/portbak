use super::pixel::{self, PixelButton, PixelCheckbox, Tone};
use super::sprites::{self, Sprite};
use super::style::{label_font, Palette};
use super::widgets::{alert, caption, format_bytes, local_time, retention_warnings, DATE, TIME};
use super::{records_of, App, HistoryDialog, HistoryMode};
use crate::engine::history::Backup;
use chrono::{Local, TimeZone};
use eframe::egui;

const LIST_HEIGHT: f32 = 300.0;
const DIALOG_WIDTH: f32 = 840.0;
const DIALOG_HEIGHT: f32 = 800.0;

/// What the user asked for in this frame; applied once the dialog is no longer borrowed.
#[derive(Default)]
struct DialogResponse {
    close: bool,
    confirm: bool,
    protection: Option<(String, bool)>,
}

impl App {
    pub(super) fn show_history(&mut self, ctx: &egui::Context) {
        let Some(dialog) = &mut self.history_dialog else {
            return;
        };
        let response = history_window(ctx, dialog, &self.error)
            .map(|output| output.inner)
            .unwrap_or_default();
        if response.close {
            self.history_dialog = None;
        }
        if let Some((name, protected)) = response.protection {
            self.save_protection(name, protected, ctx);
        }
        if response.confirm {
            self.confirm_restore(ctx);
        }
    }
}

fn history_window(
    ctx: &egui::Context,
    dialog: &mut HistoryDialog,
    app_error: &str,
) -> Option<egui::scroll_area::ScrollAreaOutput<DialogResponse>> {
    let palette = Palette::of_ctx(ctx);
    let (title, sprite, band) = match dialog.mode {
        HistoryMode::Manage => ("Manage backups", Sprite::Chest, palette.accent),
        HistoryMode::Restore => ("Restore backup", Sprite::Rewind, palette.hot),
    };
    let window = egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_size([DIALOG_WIDTH, DIALOG_HEIGHT])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
    pixel::dialog(ctx, window, title, sprite, band, |ui| {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| dialog_contents(ui, dialog, app_error))
    })
}

fn dialog_contents(
    ui: &mut egui::Ui,
    dialog: &mut HistoryDialog,
    app_error: &str,
) -> DialogResponse {
    let palette = Palette::of(ui);
    let manage = dialog.mode == HistoryMode::Manage;
    let mut response = DialogResponse::default();
    ui.horizontal(|ui| {
        sprites::show(ui, Sprite::Floppy, 3.0, palette.hot);
        ui.add(egui::Label::new(egui::RichText::new(&dialog.job.name).heading()).truncate());
    });
    ui.label(
        egui::RichText::new(if manage {
            "Protect saved versions from automatic deletion. \
             Protected versions still count toward retention limits."
        } else {
            "Choose a saved version. Close programs that write to this folder before restoring."
        })
        .color(palette.dim),
    );
    ui.add_space(4.0);
    match &dialog.backups {
        None => {
            ui.horizontal(|ui| {
                pixel::marquee(ui, 180.0, palette.accent);
                ui.label("Loading backup history …");
            });
        }
        Some(Err(error)) => alert(ui, palette.bad, error),
        Some(Ok(backups)) => {
            if backups.is_empty() {
                alert(ui, palette.dim, "No saved backups for this job.");
            }
            response.protection =
                backup_list(ui, backups, &mut dialog.selected, manage, dialog.saving);
            if manage {
                retention_warnings(ui, &dialog.job, &records_of(backups));
            }
        }
    }
    ui.add_space(4.0);
    if !manage {
        restore_details(ui, dialog, app_error);
    }
    if dialog.saving {
        ui.horizontal(|ui| {
            pixel::marquee(ui, 120.0, palette.good);
            ui.label("Saving protection …");
        });
    }
    if !dialog.error.is_empty() {
        alert(ui, palette.bad, &dialog.error);
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let close = PixelButton::new(if manage { "Close" } else { "Cancel" });
        response.close = ui.add_enabled(!dialog.saving, close).clicked();
        if !manage {
            let restore = PixelButton::new("Replace contents and restore")
                .sprite(Sprite::Rewind)
                .tone(Tone::Danger);
            response.confirm = ui.add_enabled(dialog.selected.is_some(), restore).clicked();
        }
    });
    response
}

fn restore_details(ui: &mut egui::Ui, dialog: &HistoryDialog, app_error: &str) {
    let palette = Palette::of(ui);
    let source = dialog.source_display.as_ref().unwrap_or(&dialog.job.source);
    caption(ui, "Source folder to replace");
    sunken(ui, |ui| {
        ui.horizontal(|ui| {
            sprites::show(ui, Sprite::Folder, 2.0, palette.gold);
            ui.add(egui::Label::new(source.display().to_string()).wrap());
        });
    });
    if let Some(name) = &dialog.selected {
        ui.label(format!("Selected backup: {name}"));
    }
    egui::Frame::new()
        .corner_radius(super::style::radius(ui.ctx(), 6))
        .fill(pixel::mix(palette.bad, palette.surface, 0.8))
        .stroke(egui::Stroke::new(2.0_f32, palette.bad))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            alert(
                ui,
                palette.bad,
                "The ENTIRE current contents of this source folder will be replaced, \
                 including additional files, hidden entries and subfolders.",
            );
        });
    ui.label(
        egui::RichText::new(
            "Automatic backups for this job will remain disabled after restoration.",
        )
        .color(palette.dim),
    );
    if !app_error.is_empty() {
        alert(ui, palette.bad, app_error);
    }
}

fn sunken<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let palette = Palette::of(ui);
    egui::Frame::new()
        .fill(palette.sunken)
        .corner_radius(super::style::radius(ui.ctx(), 6))
        .stroke(egui::Stroke::new(2.0_f32, palette.outline))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            contents(ui)
        })
        .inner
}

/// Marks the first backup of each local calendar day in a newest-first list.
fn day_starts<T: TimeZone>(backups: &[Backup], zone: &T) -> Vec<bool> {
    let mut previous = None;
    backups
        .iter()
        .map(|backup| {
            let date = backup.record.created.with_timezone(zone).date_naive();
            let first = previous != Some(date);
            previous = Some(date);
            first
        })
        .collect()
}

/// Returns a protection change requested in manage mode.
fn backup_list(
    ui: &mut egui::Ui,
    backups: &[Backup],
    selected: &mut Option<String>,
    manage: bool,
    saving: bool,
) -> Option<(String, bool)> {
    let mut changed = None;
    let starts = day_starts(backups, &Local);
    sunken(ui, |ui| {
        egui::ScrollArea::vertical()
            .max_height(LIST_HEIGHT)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (backup, first_of_day) in backups.iter().zip(starts) {
                    if first_of_day {
                        day_header(ui, backup);
                    }
                    if manage {
                        if let Some(change) = protection_row(ui, backup, saving) {
                            changed = Some(change);
                        }
                    } else {
                        selection_row(ui, backup, selected);
                    }
                }
            });
    });
    changed
}

/// A dated divider that opens each calendar day's group.
fn day_header(ui: &mut egui::Ui, backup: &Backup) {
    let palette = Palette::of(ui);
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        sprites::show(ui, Sprite::Cursor, 2.0, palette.gold);
        let date = local_time(backup.record.created, DATE).to_uppercase();
        ui.label(
            egui::RichText::new(date)
                .font(label_font(ui.ctx()))
                .color(palette.gold),
        );
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), egui::Sense::hover());
        let rule = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width(), 2.0));
        ui.painter().rect_filled(rule, 0.0, palette.shade);
    });
}

/// Paints a backup's row behind its contents; returns the row's interaction.
fn backup_row<R>(
    ui: &mut egui::Ui,
    backup: &Backup,
    selected: bool,
    sense: egui::Sense,
    trailing: impl FnOnce(&mut egui::Ui) -> R,
) -> (egui::Response, R) {
    let palette = Palette::of(ui);
    let background = ui.painter().add(egui::Shape::Noop);
    let frame = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (sprite, tint) = if backup.zip {
                    (Sprite::Package, palette.gold)
                } else {
                    (Sprite::Folder, palette.gold)
                };
                sprites::show(ui, sprite, 2.0, tint);
                backup_text(ui, backup, selected);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), trailing)
                    .inner
            })
            .inner
        });
    let rect = frame.response.rect;
    let id = ui.id().with(&backup.record.name);
    let response = ui.interact(rect, id, sense);
    let enabled = ui.is_enabled();
    let fill = match (selected, enabled && response.hovered()) {
        (true, _) => palette.accent,
        (false, true) => palette.raised,
        (false, false) => egui::Color32::TRANSPARENT,
    };
    ui.painter().set(
        background,
        egui::Shape::rect_filled(rect, super::style::radius(ui.ctx(), 6), fill),
    );
    (response, frame.inner)
}

fn backup_text(ui: &mut egui::Ui, backup: &Backup, selected: bool) {
    let palette = Palette::of(ui);
    let record = &backup.record;
    let (ink, dim) = if selected {
        (palette.accent_ink, palette.accent_ink)
    } else {
        (palette.ink, palette.dim)
    };
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(local_time(record.created, TIME))
                    .font(label_font(ui.ctx()))
                    .color(ink),
            );
            let kind = if backup.zip { "ZIP" } else { "Folder" };
            ui.label(
                egui::RichText::new(format!("{kind} · {}", format_bytes(record.bytes))).color(ink),
            )
            .on_hover_text(format!("{} bytes", record.bytes));
            if !backup.available {
                pixel::badge(ui, "Unavailable", palette.bad);
            }
            if record.protected {
                pixel::badge(ui, "Protected", palette.good);
            }
        });
        ui.label(egui::RichText::new(&record.name).small().color(dim));
    });
}

fn protection_row(ui: &mut egui::Ui, backup: &Backup, saving: bool) -> Option<(String, bool)> {
    let (_, change) = backup_row(ui, backup, false, egui::Sense::hover(), |ui| {
        let mut protected = backup.record.protected;
        // Unavailable backups can still be unprotected so retention may drop them.
        let enabled = !saving && (backup.available || protected);
        let checkbox = PixelCheckbox::new(&mut protected, "Protect from automatic deletion");
        ui.add_enabled(enabled, checkbox)
            .changed()
            .then(|| (backup.record.name.clone(), protected))
    });
    change
}

fn selection_row(ui: &mut egui::Ui, backup: &Backup, selected: &mut Option<String>) {
    let is_selected = selected.as_deref() == Some(backup.record.name.as_str());
    let palette = Palette::of(ui);
    let row = ui.add_enabled_ui(backup.available, |ui| {
        backup_row(ui, backup, is_selected, egui::Sense::click(), |ui| {
            if is_selected {
                sprites::show(ui, Sprite::Check, 3.0, palette.accent_ink);
            }
        })
        .0
    });
    let response = row.inner.on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() {
        *selected = Some(backup.record.name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Record;
    use chrono::Utc;

    #[test]
    fn restore_fits_without_outer_scrolling_before_and_after_selection() {
        for skin in crate::app::skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let ctx = egui::Context::default();
                crate::app::style::install(&ctx, skin);
                ctx.set_theme(theme);
                let job = crate::model::Job {
                    name: "My documents".into(),
                    source: "/home/user/Documents/Projects/Important documents".into(),
                    ..Default::default()
                };
                let name = "2026-11-07_18-30-00_My documents and important project files";
                let backups = (0..100)
                    .map(|index| Backup {
                        record: Record {
                            name: format!("{name}_{index}"),
                            hash: String::new(),
                            bytes: 1024,
                            created: Utc::now(),
                            protected: false,
                        },
                        available: true,
                        zip: false,
                    })
                    .collect();
                let mut dialog = HistoryDialog::new(job, HistoryMode::Restore);
                dialog.backups = Some(Ok(backups));
                let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100.0, 960.0));
                for frame in 0..8 {
                    if frame == 4 {
                        dialog.selected = Some(format!("{name}_0"));
                    }
                    let input = egui::RawInput {
                        screen_rect: Some(screen),
                        ..Default::default()
                    };
                    let mut sizes = None;
                    let _ = ctx.run_ui(input, |ui| {
                        let output = history_window(ui.ctx(), &mut dialog, "").unwrap();
                        sizes = Some((output.content_size, output.inner_rect));
                    });
                    if frame < 2 {
                        continue;
                    }
                    let (content, viewport) = sizes.unwrap();
                    assert!(
                        content.y <= viewport.height() + 1.0,
                        "{} {theme:?} selected={}: content={content:?}, viewport={viewport:?}",
                        skin.id,
                        dialog.selected.is_some()
                    );
                    assert!(
                        content.x <= viewport.width() + 1.0,
                        "{} {theme:?}: horizontal overflow",
                        skin.id
                    );
                    let window = ctx
                        .memory(|memory| memory.area_rect(egui::Id::new("Restore backup")))
                        .unwrap();
                    assert!(
                        screen.contains_rect(window),
                        "{} {theme:?}: window={window:?}",
                        skin.id
                    );
                }
            }
        }
    }

    #[test]
    fn day_markers_use_first_list_entry_and_local_calendar_date() {
        let dates = [
            "2026-10-26T00:15:00Z",
            "2026-10-25T23:15:00Z",
            "2026-10-25T01:30:00Z",
            "2026-10-25T00:30:00Z",
        ];
        let backups: Vec<_> = dates
            .iter()
            .enumerate()
            .map(|(index, date)| Backup {
                record: Record {
                    name: index.to_string(),
                    hash: String::new(),
                    bytes: 0,
                    created: date.parse().unwrap(),
                    protected: false,
                },
                available: true,
                zip: false,
            })
            .collect();
        assert_eq!(
            day_starts(&backups, &chrono_tz::Europe::Berlin),
            [true, false, true, false]
        );
        assert_eq!(day_starts(&backups, &Utc), [true, true, false, false]);
    }
}
