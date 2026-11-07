use crate::app::rendering::pixel::{self, PixelButton};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, Palette};
use crate::engine;
use crate::model::{Job, Record};
use chrono::{DateTime, Local, Utc};
use eframe::egui;
use std::path::{Path, PathBuf};

pub const DATE_TIME: &str = "%d.%m.%Y %H:%M:%S";
pub const DATE_MINUTE: &str = "%d.%m.%Y %H:%M";
pub const DATE: &str = "%a %d.%m.%Y";
pub const TIME: &str = "%H:%M:%S";

pub fn local_time(time: DateTime<Utc>, format: &str) -> String {
    time.with_timezone(&Local).format(format).to_string()
}

/// A left-to-right row tall enough that labels, buttons and inputs share one vertical center.
pub fn centered_row<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let height = (ui.text_style_height(&egui::TextStyle::Heading)
        + ui.spacing().button_padding.y * 2.0)
        .max(pixel::button_height(ui));
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        egui::Layout::left_to_right(egui::Align::Center),
        contents,
    )
}

/// A dim, upper-case field caption.
pub fn caption(ui: &mut egui::Ui, text: &str) {
    let dim = Palette::of(ui).dim;
    let text = style::cased(ui.ctx(), text);
    ui.label(egui::RichText::new(text).small().color(dim));
}

/// Show ordinary Windows paths while retaining verbatim paths for file operations.
pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        if let Some(Component::Prefix(prefix)) = path.components().next() {
            match prefix.kind() {
                Prefix::VerbatimDisk(_) => return text[4..].to_owned(),
                Prefix::VerbatimUNC(_, _) => return format!(r"\\{}", &text[8..]),
                _ => {}
            }
        }
    }
    text.into_owned()
}

pub fn path_picker(ui: &mut egui::Ui, label: &str, path: &mut PathBuf) {
    caption(ui, label);
    centered_row(ui, |ui| {
        // Right-to-left so the field takes exactly the width the button leaves.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let choose = PixelButton::new("Choose folder …").sprite(Sprite::Folder);
            if ui.add(choose).clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    *path = folder;
                }
            }
            let mut text = display_path(path);
            let editor = egui::TextEdit::singleline(&mut text)
                .desired_width(ui.available_width())
                .margin(egui::vec2(8.0, 4.0))
                .hint_text("Type a path or choose a folder");
            if ui.add(editor).changed() {
                *path = PathBuf::from(text);
            }
        });
    });
}

pub fn alert(ui: &mut egui::Ui, color: egui::Color32, text: &str) {
    ui.horizontal_top(|ui| {
        sprites::show(ui, Sprite::Warning, 2.0, color);
        ui.add(egui::Label::new(egui::RichText::new(text).color(color)).wrap());
    });
}

pub fn retention_warnings(ui: &mut egui::Ui, job: &Job, records: &[Record]) {
    let gold = Palette::of(ui).gold;
    for warning in engine::history::warnings(job, records) {
        alert(ui, gold, &warning);
    }
}

/// Binary-prefixed size, e.g. `1.5 MiB`; exact byte counts stay available on hover.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn display_paths_hide_disk_and_unc_verbatim_prefixes() {
        for (stored, shown) in [
            (r"\\?\C:\app\backups", r"C:\app\backups"),
            (r"\\?\UNC\server\share\folder", r"\\server\share\folder"),
            (r"C:\ordinary", r"C:\ordinary"),
            (r"\\server\share", r"\\server\share"),
            (r"\\?\Volume{example}\folder", r"\\?\Volume{example}\folder"),
        ] {
            let path = PathBuf::from(stored);
            assert_eq!(display_path(&path), shown);
            assert_eq!(path, PathBuf::from(stored));
        }
    }

    #[test]
    fn mixed_row_controls_share_vertical_center() {
        let context = egui::Context::default();
        let _ = context.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| {
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
                    let seconds = ui.add(egui::DragValue::new(&mut 0));
                    for rect in [value.rect, seconds.rect] {
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

    #[test]
    fn sizes_use_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(10 * 1024 * 1024 * 1024), "10.0 GiB");
    }
}
