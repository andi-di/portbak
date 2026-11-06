use crate::engine;
use crate::model::{Job, Record};
use chrono::{DateTime, Local, Utc};
use eframe::egui;
use std::path::PathBuf;

pub const DANGER: egui::Color32 = egui::Color32::from_rgb(170, 35, 45);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(25, 120, 60);
pub const ERROR_TEXT: egui::Color32 = egui::Color32::LIGHT_RED;
const ENABLED_DOT: egui::Color32 = egui::Color32::from_rgb(40, 200, 90);

pub const DATE_TIME: &str = "%d.%m.%Y %H:%M:%S";
pub const DATE_MINUTE: &str = "%d.%m.%Y %H:%M";

pub fn local_time(time: DateTime<Utc>, format: &str) -> String {
    time.with_timezone(&Local).format(format).to_string()
}

/// A left-to-right row tall enough that labels, buttons and inputs share one vertical center.
pub fn centered_row<R>(
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

/// Gives a combo box a fixed box so it centers like the other controls in a `centered_row`.
pub fn centered_combo<R>(
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

/// Title with a green dot when automatic backups are enabled.
pub fn job_title(
    ui: &mut egui::Ui,
    title: impl FnOnce(&mut egui::Ui) -> egui::Response,
    enabled: bool,
) -> egui::Response {
    ui.horizontal(|ui| {
        let title = title(ui);
        if enabled {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(12.0, title.rect.height()), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.5, ENABLED_DOT);
            response.on_hover_text("Automatic backups enabled");
        }
        title
    })
    .inner
}

pub fn path_picker(ui: &mut egui::Ui, label: &str, path: &mut PathBuf) {
    ui.label(label);
    centered_row(ui, |ui| {
        let mut text = path.to_string_lossy().into_owned();
        if ui
            .add(egui::TextEdit::singleline(&mut text).desired_width(370.0))
            .changed()
        {
            *path = PathBuf::from(text);
        }
        if ui.button("Choose folder …").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                *path = folder;
            }
        }
    });
}

pub fn retention_warnings(ui: &mut egui::Ui, job: &Job, records: &[Record]) {
    for warning in engine::history::warnings(job, records) {
        ui.colored_label(ui.visuals().warn_fg_color, warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_row_controls_share_vertical_center() {
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
                for (style, size) in [
                    (egui::TextStyle::Heading, 22.0),
                    (egui::TextStyle::Body, 14.0),
                    (egui::TextStyle::Button, 14.0),
                ] {
                    ui.style_mut()
                        .text_styles
                        .insert(style, egui::FontId::proportional(size));
                }
                centered_row(ui, |ui| {
                    let label = ui.label("Every");
                    let value = ui.add(egui::DragValue::new(&mut 60));
                    let unit = centered_combo(ui, |ui| {
                        egui::ComboBox::from_id_salt("unit_test")
                            .selected_text("Minutes")
                            .show_ui(ui, |_| {})
                    })
                    .inner;
                    for rect in [value.rect, unit.response.rect] {
                        assert!(
                            (label.rect.center().y - rect.center().y).abs() < 0.5,
                            "label {:?}, control {rect:?}",
                            label.rect
                        );
                    }
                });
            });
        });
    }
}
