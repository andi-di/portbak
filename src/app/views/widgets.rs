use crate::app::rendering::pixel::{self, PixelButton};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, Palette};
use crate::engine;
use crate::model::{Job, Record};
use chrono::{DateTime, Local, Utc};
use eframe::egui;
use std::path::{Path, PathBuf};

pub const DATE_TIME: &str = "%d.%m.%Y %H:%M:%S";
pub const DATE: &str = "%a %d.%m.%Y";
pub const TIME: &str = "%H:%M:%S";

pub const PAGE_PADDING: i8 = 22;

pub fn page_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::symmetric(0, PAGE_PADDING))
}

pub fn page_scroll_area() -> egui::ScrollArea {
    egui::ScrollArea::both()
        .content_margin(egui::Margin::symmetric(PAGE_PADDING, 0))
        .auto_shrink([false, true])
}

pub fn local_time(time: DateTime<Utc>, format: &str) -> String {
    time.with_timezone(&Local).format(format).to_string()
}

/// A left-to-right row tall enough that labels, buttons and inputs share one vertical center.
/// It wraps in narrow windows, since an overlong row would push the scroll bar out of view.
pub fn centered_row<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let height = (ui.text_style_height(&egui::TextStyle::Heading)
        + ui.spacing().button_padding.y * 2.0)
        .max(pixel::button_height(ui));
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), height),
        egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
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

pub fn path_picker(ui: &mut egui::Ui, label: &str, path: &mut PathBuf, read_only: bool) -> bool {
    let mut changed = false;
    let error_id = ui.make_persistent_id((label, "open_folder_error"));
    let mut open_error = ui
        .ctx()
        .data_mut(|data| data.get_temp::<String>(error_id))
        .unwrap_or_default();
    caption(ui, label);
    centered_row(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let open = PixelButton::new("Open folder in file manager")
                .sprite(Sprite::Monitor)
                .icon_only();
            if ui
                .add_enabled(!path.as_os_str().is_empty() && path.is_dir(), open)
                .on_hover_text("Open folder in file manager")
                .on_disabled_hover_text("Choose an existing folder first")
                .clicked()
            {
                open_error = crate::app::platform::open_in_file_manager(path)
                    .err()
                    .map_or_else(String::new, |error| {
                        format!("Could not open folder: {error:#}")
                    });
            }
            let choose = PixelButton::new("Choose folder …")
                .sprite(Sprite::Folder)
                .icon_only();
            if ui.add(choose).on_hover_text("Choose folder").clicked() && !read_only {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    *path = folder;
                    changed = true;
                }
            }
            let mut text = display_path(path);
            let editor = egui::TextEdit::singleline(&mut text)
                .desired_width(ui.available_width())
                .margin(egui::vec2(8.0, 4.0))
                .hint_text("Type a path or choose a folder");
            if ui.add(editor).changed() {
                *path = PathBuf::from(text);
                changed = true;
            }
        });
    });
    if changed {
        open_error.clear();
    }
    if !open_error.is_empty() {
        alert(ui, Palette::of(ui).bad, &open_error);
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(error_id, open_error));
    changed
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

const BYTES_PER_GIB: f64 = 1_073_741_824.0;

/// Editable byte limit shown in GiB.
pub fn gib_field(ui: &mut egui::Ui, enabled: bool, bytes: &mut u64) -> egui::Response {
    let mut gib = *bytes as f64 / BYTES_PER_GIB;
    // Clamping a stored value mid-edit makes egui discard the text being typed,
    // so only user input is clamped and bytes are rounded to stay inside the range.
    let editor = egui::DragValue::new(&mut gib)
        .speed(0.1)
        .range(0.001..=100_000.0)
        .clamp_existing_to_range(false)
        .suffix(" GiB");
    let response = ui.add_enabled(enabled, editor);
    if response.changed() {
        *bytes = (gib * BYTES_PER_GIB).round() as u64;
    }
    response
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
    fn typed_sizes_below_one_gib_are_stored_exactly() {
        for (typed, expected) in [("0.5", 0.5), ("0.001", 0.001), ("0.004", 0.004)] {
            let context = egui::Context::default();
            let mut bytes = 10 * 1024 * 1024 * 1024;
            let frame = |events: Vec<egui::Event>, bytes: &mut u64| {
                let input = egui::RawInput {
                    events,
                    ..Default::default()
                };
                let mut rect = egui::Rect::NOTHING;
                let _ = context.run_ui(input, |ui| {
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        rect = gib_field(ui, true, bytes).rect;
                    });
                });
                rect
            };
            let center = frame(Vec::new(), &mut bytes).center();
            let press = |pressed| egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            frame(
                vec![egui::Event::PointerMoved(center), press(true)],
                &mut bytes,
            );
            frame(vec![press(false)], &mut bytes);
            frame(Vec::new(), &mut bytes);
            // One character per frame, like a person typing.
            for character in typed.chars() {
                frame(vec![egui::Event::Text(character.into())], &mut bytes);
            }
            let enter = egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            };
            frame(vec![enter], &mut bytes);
            frame(Vec::new(), &mut bytes);
            assert_eq!(bytes, (expected * BYTES_PER_GIB).round() as u64, "{typed}");
        }
    }

    #[test]
    fn sizes_use_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(10 * 1024 * 1024 * 1024), "10.0 GiB");
    }
}
