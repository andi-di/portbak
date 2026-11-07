use crate::app::rendering::glass::{self, Tint};
use crate::app::rendering::pixel::{self, PixelButton, PixelCheckbox, Tone};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, label_font, Finish, Palette, Skin};
use crate::app::views::widgets::{
    alert, caption, display_path, format_bytes, local_time, retention_warnings, DATE, TIME,
};
use crate::app::{records_of, App, BackupHistory};
use crate::engine::history::Backup;
use chrono::{Local, TimeZone};
use eframe::egui;

const LIST_HEIGHT: f32 = 300.0;
/// Changes are applied after the history state is no longer borrowed.
#[derive(Default)]
struct HistoryResponse {
    restore: Option<String>,
    protection: Option<(String, bool)>,
}

impl App {
    pub(in crate::app) fn show_backups_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let Some(id) = self.selected else {
            return;
        };
        let destination_missing = self
            .job(id)
            .is_some_and(|job| job.target.as_os_str().is_empty());
        self.open_history(id, &ctx);
        let frame =
            egui::Frame::central_panel(&ctx.global_style()).inner_margin(egui::Margin::same(22));
        let mut response = HistoryResponse::default();
        egui::CentralPanel::default()
            .frame(frame)
            .show_inside(ui, |ui| {
                pixel::paint_scenery(ui);
                ui.add_enabled_ui(
                    !self
                        .backup_history
                        .as_ref()
                        .is_some_and(|history| history.saving),
                    |ui| {
                        super::editor::job_tabs(ui, &mut self.backups_tab);
                    },
                );
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(history) = &mut self.backup_history {
                        if history.job.id == id {
                            response = history_contents(ui, history);
                        } else {
                            ui.label("Finishing protection changes for the previous job ...");
                        }
                    } else if destination_missing {
                        ui.label("Choose a destination folder in Job settings to view backups.");
                    } else {
                        ui.label(
                            "Backups will be available when active and queued operations finish.",
                        );
                    }
                });
            });
        if let Some((name, protected)) = response.protection {
            self.save_protection(name, protected, &ctx);
        }
        if let Some(name) = response.restore {
            if let Some(history) = &mut self.backup_history {
                history.selected = Some(name);
                history.confirming_restore = true;
            }
        }
        self.show_restore_confirmation(&ctx);
    }

    fn show_restore_confirmation(&mut self, ctx: &egui::Context) {
        let Some(history) = &mut self.backup_history else {
            return;
        };
        if !history.confirming_restore {
            return;
        }
        let response = restore_confirmation(ctx, history, &self.error);
        if response.1 {
            history.confirming_restore = false;
        } else if response.0 {
            self.confirm_restore(ctx);
        }
    }

    pub(in crate::app) fn close_inactive_history(&mut self) {
        let visible = self.backups_tab && !self.settings_open;
        if self.backup_history.as_ref().is_some_and(|history| {
            !history.saving && (!visible || self.selected != Some(history.job.id))
        }) {
            self.backup_history = None;
        }
    }
}

fn history_contents(ui: &mut egui::Ui, dialog: &mut BackupHistory) -> HistoryResponse {
    let palette = Palette::of(ui);
    let mut response = HistoryResponse::default();
    ui.heading(&dialog.job.name);
    ui.label(
        "Lock backups to protect them from retention deletion. Use a backup's restore button to replace the source contents.",
    );
    match &dialog.backups {
        None => {
            ui.label("Loading backup history ...");
        }
        Some(Err(error)) => alert(ui, palette.bad, error),
        Some(Ok(backups)) => {
            if backups.is_empty() {
                ui.label("No saved backups for this job.");
            }
            response = backup_list(
                ui,
                backups,
                &mut dialog.selected,
                dialog.saving,
                dialog.source_display.is_some(),
            );
            retention_warnings(ui, &dialog.job, &records_of(backups));
        }
    }
    if dialog.saving {
        ui.label("Saving protection ...");
    }
    if !dialog.error.is_empty() {
        alert(ui, palette.bad, &dialog.error);
    }
    response
}

/// Returns confirmation and cancellation separately so dismissal never starts a restore.
fn restore_confirmation(
    ctx: &egui::Context,
    history: &BackupHistory,
    app_error: &str,
) -> (bool, bool) {
    let response = restore_modal(ctx, history, app_error);
    let (confirm, cancel) = response.inner.inner;
    (confirm, cancel || response.should_close())
}

fn restore_modal(
    ctx: &egui::Context,
    history: &BackupHistory,
    app_error: &str,
) -> egui::ModalResponse<egui::scroll_area::ScrollAreaOutput<(bool, bool)>> {
    let mut confirm = false;
    let mut cancel = false;
    egui::Modal::new(egui::Id::new("confirm_backup_restore")).show(ctx, |ui| {
        ui.set_width((ctx.content_rect().width() - 64.0).clamp(240.0, 760.0));
        egui::ScrollArea::vertical()
            .min_scrolled_height((ctx.content_rect().height() - 64.0).clamp(120.0, 480.0))
            .max_height((ctx.content_rect().height() - 64.0).max(120.0))
            .show(ui, |ui| {
                ui.heading("Restore backup?");
                restore_details(ui, history, app_error);
                ui.horizontal(|ui| {
                    cancel = ui.add(PixelButton::new("Cancel")).clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let restore = PixelButton::new("Replace contents and restore")
                            .sprite(Sprite::Rewind)
                            .tone(Tone::Danger);
                        confirm = ui
                            .add_enabled(history.restorable_selection().is_some(), restore)
                            .clicked();
                    });
                });
                (confirm, cancel)
            })
    })
}

fn restore_details(ui: &mut egui::Ui, dialog: &BackupHistory, app_error: &str) {
    let palette = Palette::of(ui);
    let source = dialog.source_display.as_ref().unwrap_or(&dialog.job.source);
    caption(ui, "Source folder to replace");
    sunken(ui, |ui| {
        ui.horizontal(|ui| {
            sprites::show(ui, Sprite::Folder, 2.0, palette.gold);
            ui.add(egui::Label::new(display_path(source)).wrap());
        });
    });
    if let Some(name) = &dialog.selected {
        ui.add(egui::Label::new(format!("Selected backup: {name}")).wrap());
    }
    egui::Frame::new()
        .fill(pixel::mix(palette.bad, palette.surface, 0.8))
        .stroke(egui::Stroke::new(2.0_f32, palette.bad))
        .corner_radius(style::control_radius(ui.ctx()))
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
            "Close programs that write to this folder before restoring. Automatic backups for this job will remain disabled after restoration.",
        )
        .color(palette.dim),
    );
    if !app_error.is_empty() {
        alert(ui, palette.bad, app_error);
    }
}

fn sunken<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    style::well(ui.ctx())
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

/// Returns the action requested from the backup list.
fn backup_list(
    ui: &mut egui::Ui,
    backups: &[Backup],
    selected: &mut Option<String>,
    saving: bool,
    source_available: bool,
) -> HistoryResponse {
    let mut response = HistoryResponse::default();
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
                    let row = selection_row(ui, backup, selected, saving, source_available);
                    if row.protection.is_some() {
                        response.protection = row.protection;
                    }
                    if row.restore.is_some() {
                        response.restore = row.restore;
                    }
                }
            });
    });
    response
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
    let mut size = egui::Rect::NOTHING;
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
                let text_width = (ui.available_width() - 144.0).max(40.0);
                ui.allocate_ui(egui::vec2(text_width, 0.0), |ui| {
                    size = backup_text(ui, backup, selected);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), trailing)
                    .inner
            })
            .inner
        });
    let rect = frame.response.rect;
    let id = ui.id().with(&backup.record.name);
    let selection_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2((rect.right() - 144.0).max(rect.left()), rect.bottom()),
    );
    let response = ui.interact(selection_rect, id, sense);
    let exact = format!("{} bytes", backup.record.bytes);
    pixel::tooltip(ui, size, id.with("size"), &exact);
    let enabled = ui.is_enabled();
    let hovered = enabled && response.hovered();
    if Skin::of(ui.ctx()).finish == Finish::Glass {
        let tint = match (selected, hovered) {
            (true, _) => Some(Tint::Accent),
            (false, true) => Some(Tint::Hover),
            (false, false) => None,
        };
        if let Some(tint) = tint {
            ui.painter()
                .set(background, glass::tile(ui.ctx(), rect, tint));
        }
        return (response, frame.inner);
    }
    let fill = match (selected, hovered) {
        (true, _) => palette.accent,
        (false, true) => palette.raised,
        (false, false) => egui::Color32::TRANSPARENT,
    };
    let radius = style::control_radius(ui.ctx());
    ui.painter()
        .set(background, egui::Shape::rect_filled(rect, radius, fill));
    (response, frame.inner)
}

/// The backup's time, kind and size; returns where the size is, for its exact-bytes tooltip.
fn backup_text(ui: &mut egui::Ui, backup: &Backup, selected: bool) -> egui::Rect {
    let palette = Palette::of(ui);
    let record = &backup.record;
    let (ink, dim) = if selected {
        (palette.accent_ink, palette.accent_ink)
    } else {
        (palette.ink, palette.dim)
    };
    let mut size = egui::Rect::NOTHING;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(local_time(record.created, TIME))
                    .font(label_font(ui.ctx()))
                    .color(ink),
            );
            let kind = if backup.zip { "ZIP" } else { "Folder" };
            size = ui
                .label(
                    egui::RichText::new(format!("{kind} · {}", format_bytes(record.bytes)))
                        .color(ink),
                )
                .rect;
            if !backup.available {
                pixel::badge(ui, "Unavailable", palette.bad);
            }
            if record.protected {
                pixel::badge(ui, "Protected", palette.good);
            }
        });
        ui.add(egui::Label::new(egui::RichText::new(&record.name).small().color(dim)).wrap());
    });
    size
}

fn selection_row(
    ui: &mut egui::Ui,
    backup: &Backup,
    selected: &mut Option<String>,
    saving: bool,
    source_available: bool,
) -> HistoryResponse {
    let mut restore = None;
    let is_selected = selected.as_deref() == Some(backup.record.name.as_str());
    let (row, change) = backup_row(ui, backup, is_selected, egui::Sense::click(), |ui| {
        let button = PixelButton::new("Restore backup")
            .icon_only()
            .sprite(Sprite::Rewind)
            .tone(Tone::Danger);
        if ui.add_enabled(!saving && backup.available && source_available, button)
            .on_hover_text("Restore this backup")
            .on_disabled_hover_text("Restoration requires an available backup and source folder, with no protection save in progress")
            .clicked() {
                restore = Some(backup.record.name.clone());
            }
        let mut protected = backup.record.protected;
        let enabled = !saving && (backup.available || protected);
        let checkbox = PixelCheckbox::new(&mut protected, "Locked");
        ui.add_enabled(enabled, checkbox)
            .on_hover_text("Protect from retention deletion")
            .changed()
            .then(|| (backup.record.name.clone(), protected))
    });
    if row.clicked() && backup.available && !saving && change.is_none() {
        *selected = (!is_selected).then(|| backup.record.name.clone());
    }
    HistoryResponse {
        restore,
        protection: change,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Record;
    use chrono::Utc;

    #[test]
    fn confirmation_modal_fits_and_escape_cancels_without_confirming() {
        for skin in crate::app::theme::skins::ALL {
            for (theme, screen_size) in [egui::Theme::Light, egui::Theme::Dark]
                .into_iter()
                .flat_map(|theme| {
                    [egui::vec2(800.0, 600.0), egui::vec2(1100.0, 960.0)]
                        .into_iter()
                        .map(move |size| (theme, size))
                })
            {
                let ctx = egui::Context::default();
                crate::app::theme::style::install(&ctx, skin);
                ctx.set_theme(theme);
                let mut history = BackupHistory::new(crate::model::Job {
                    source: r"C:\Users\Someone\Documents\Projects\Important documents".into(),
                    ..Default::default()
                });
                history.source_display = Some(history.job.source.clone());
                history.selected =
                    Some("2026-11-08_18-30-00_Important documents and project files".into());
                for frame in 0..4 {
                    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_size);
                    let mut input = egui::RawInput {
                        screen_rect: Some(screen),
                        ..Default::default()
                    };
                    if frame == 3 {
                        input.events.push(egui::Event::Key {
                            key: egui::Key::Escape,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    let _ = ctx.run_ui(input, |ui| {
                        let response = restore_modal(ui.ctx(), &history, "");
                        if frame >= 2 {
                            assert!(
                                response.inner.content_size.y
                                    <= response.inner.inner_rect.height() + 1.0,
                                "{} {theme:?}: content={:?}, viewport={:?}",
                                skin.id,
                                response.inner.content_size,
                                response.inner.inner_rect
                            );
                        }
                        let (confirm, cancel) = response.inner.inner;
                        let cancel = cancel || response.should_close();
                        assert!(!confirm);
                        assert_eq!(cancel, frame == 3);
                    });
                    if frame >= 2 {
                        let rect = ctx
                            .memory(|memory| {
                                memory.area_rect(egui::Id::new("confirm_backup_restore"))
                            })
                            .unwrap();
                        assert!(
                            screen.contains_rect(rect),
                            "{} {theme:?}: {rect:?}",
                            skin.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn backups_tab_fits_narrow_job_views_in_every_skin() {
        for skin in crate::app::theme::skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let ctx = egui::Context::default();
                crate::app::theme::style::install(&ctx, skin);
                ctx.set_theme(theme);
                let mut history = BackupHistory::new(crate::model::Job::default());
                let name = "2026-11-08_18-30-00_Important documents and project files";
                history.backups = Some(Ok(vec![Backup {
                    record: Record {
                        name: name.into(),
                        hash: String::new(),
                        bytes: 1024,
                        created: Utc::now(),
                        protected: true,
                    },
                    available: true,
                    zip: true,
                }]));
                history.selected = Some(name.into());
                for width in [290.0, 600.0, 900.0] {
                    for _ in 0..3 {
                        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                            ui.set_max_width(width);
                            let output = egui::ScrollArea::vertical().show(ui, |ui| {
                                history_contents(ui, &mut history);
                            });
                            assert!(
                                output.content_size.x <= width + 1.0,
                                "{} {theme:?}: width={width}, content={:?}",
                                skin.id,
                                output.content_size
                            );
                        });
                    }
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
