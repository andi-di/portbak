use super::widgets::{centered_row, local_time, retention_warnings, DANGER, DATE_TIME, ERROR_TEXT};
use super::{records_of, App, HistoryDialog, HistoryMode};
use crate::engine::history::Backup;
use chrono::{Local, TimeZone};
use eframe::egui;

const LIST_HEIGHT: f32 = 220.0;

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
        let app_error = &self.error;
        let mut response = DialogResponse::default();
        let title = match dialog.mode {
            HistoryMode::Manage => "Manage backups",
            HistoryMode::Restore => "Restore backup",
        };
        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_width(650.0)
            .max_height(520.0)
            .vscroll(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| response = dialog_contents(ui, dialog, app_error));
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

fn dialog_contents(
    ui: &mut egui::Ui,
    dialog: &mut HistoryDialog,
    app_error: &str,
) -> DialogResponse {
    let manage = dialog.mode == HistoryMode::Manage;
    let mut response = DialogResponse::default();
    ui.heading(&dialog.job.name);
    ui.label(if manage {
        "Protect saved versions from automatic deletion. \
         Protected versions still count toward retention limits."
    } else {
        "Choose a saved version. Close programs that write to this folder before restoring."
    });
    match &dialog.backups {
        None => {
            ui.spinner();
            ui.label("Loading backup history …");
        }
        Some(Err(error)) => {
            ui.colored_label(ERROR_TEXT, error);
        }
        Some(Ok(backups)) => {
            if backups.is_empty() {
                ui.label("No saved backups for this job.");
            }
            response.protection =
                backup_list(ui, backups, &mut dialog.selected, manage, dialog.saving);
            if manage {
                retention_warnings(ui, &dialog.job, &records_of(backups));
            }
        }
    }
    ui.separator();
    if !manage {
        restore_details(ui, dialog, app_error);
    }
    if dialog.saving {
        ui.label("Saving protection …");
    }
    if !dialog.error.is_empty() {
        ui.colored_label(ERROR_TEXT, &dialog.error);
    }
    centered_row(ui, |ui| {
        let close = egui::Button::new(if manage { "Close" } else { "Cancel" });
        response.close = ui.add_enabled(!dialog.saving, close).clicked();
        if !manage {
            let restore = egui::Button::new(
                egui::RichText::new("Replace contents and restore")
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(DANGER);
            response.confirm = ui.add_enabled(dialog.selected.is_some(), restore).clicked();
        }
    });
    response
}

fn restore_details(ui: &mut egui::Ui, dialog: &HistoryDialog, app_error: &str) {
    let source = dialog.source_display.as_ref().unwrap_or(&dialog.job.source);
    ui.label(format!("Source folder to replace: {}", source.display()));
    if let Some(name) = &dialog.selected {
        ui.label(format!("Selected backup: {name}"));
    }
    ui.colored_label(
        ERROR_TEXT,
        "The ENTIRE current contents of this source folder will be replaced, \
         including additional files, hidden entries and subfolders.",
    );
    ui.label("Automatic backups for this job will remain disabled after restoration.");
    if !app_error.is_empty() {
        ui.colored_label(ERROR_TEXT, app_error);
    }
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

fn backup_label(backup: &Backup) -> String {
    let record = &backup.record;
    let kind = if backup.zip { "ZIP" } else { "Folder" };
    let unavailable = if backup.available {
        ""
    } else {
        " · unavailable"
    };
    let protected = if record.protected {
        " · Protected"
    } else {
        ""
    };
    format!(
        "{} · {kind} · {} bytes{unavailable}{protected}\n{}",
        local_time(record.created, DATE_TIME),
        record.bytes,
        record.name
    )
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
    egui::ScrollArea::vertical()
        .max_height(LIST_HEIGHT)
        .show(ui, |ui| {
            for (backup, first_of_day) in backups.iter().zip(starts) {
                let mut text = egui::RichText::new(backup_label(backup));
                if first_of_day {
                    text = text.strong();
                }
                if manage {
                    if let Some(change) = protection_row(ui, backup, text, first_of_day, saving) {
                        changed = Some(change);
                    }
                } else {
                    selection_row(ui, backup, text, first_of_day, selected);
                }
            }
        });
    changed
}

fn protection_row(
    ui: &mut egui::Ui,
    backup: &Backup,
    text: egui::RichText,
    first_of_day: bool,
    saving: bool,
) -> Option<(String, bool)> {
    let fill = if first_of_day {
        ui.visuals().faint_bg_color
    } else {
        egui::Color32::TRANSPARENT
    };
    egui::Frame::new()
        .fill(fill)
        .inner_margin(4.0)
        .show(ui, |ui| {
            ui.label(text);
            let mut protected = backup.record.protected;
            // Unavailable backups can still be unprotected so retention may drop them.
            let enabled = !saving && (backup.available || protected);
            let checkbox = egui::Checkbox::new(&mut protected, "Protect from automatic deletion");
            ui.add_enabled(enabled, checkbox)
                .changed()
                .then(|| (backup.record.name.clone(), protected))
        })
        .inner
}

fn selection_row(
    ui: &mut egui::Ui,
    backup: &Backup,
    text: egui::RichText,
    first_of_day: bool,
    selected: &mut Option<String>,
) {
    let is_selected = selected.as_deref() == Some(backup.record.name.as_str());
    let mut button = egui::Button::new(text).selected(is_selected);
    if first_of_day && !is_selected {
        button = button.fill(ui.visuals().faint_bg_color);
    }
    if ui.add_enabled(backup.available, button).clicked() {
        *selected = Some(backup.record.name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Record;
    use chrono::Utc;

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
