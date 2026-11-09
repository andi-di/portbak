//! Matte surfaces, fine silver ornaments and restrained crimson focus accents.

use crate::app::rendering::pixel::mix;
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, Palette};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};

macro_rules! asset {
    ($name:literal, $dark:expr) => {
        (
            $name,
            if $dark {
                include_bytes!(concat!(
                    "../../../assets/themes/midnight-neko/icons/dark/",
                    $name,
                    ".png"
                ))
                .as_slice()
            } else {
                include_bytes!(concat!(
                    "../../../assets/themes/midnight-neko/icons/light/",
                    $name,
                    ".png"
                ))
                .as_slice()
            },
        )
    };
}

pub fn icon(painter: &egui::Painter, rect: Rect, sprite: Sprite, _tint: Color32) {
    let dark = painter.ctx().global_style().visuals.dark_mode;
    let (name, bytes) = icon_asset(sprite, dark);
    paint_icon(
        painter,
        rect,
        name,
        bytes,
        dark,
        Palette::of_ctx(painter.ctx()).surface,
    );
}

pub fn foreground(fill: Color32) -> Color32 {
    let luminance = super::modern::luminance(fill);
    if (luminance + 0.05) / 0.05 > 1.05 / (luminance + 0.05) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

pub fn icon_on(painter: &egui::Painter, rect: Rect, sprite: Sprite, fill: Color32) {
    let dark = foreground(fill) == Color32::WHITE;
    let (name, bytes) = icon_asset(sprite, dark);
    paint_icon(painter, rect, name, bytes, dark, fill);
}

pub fn job_icon(painter: &egui::Painter, rect: Rect, active: bool) {
    let dark = painter.ctx().global_style().visuals.dark_mode;
    let (name, bytes) = if active {
        asset!("job-active", dark)
    } else {
        asset!("job-inactive", dark)
    };
    paint_icon(
        painter,
        rect,
        name,
        bytes,
        dark,
        Palette::of_ctx(painter.ctx()).surface,
    );
    let radius = rect.width() * 0.16;
    let center = rect.right_bottom() - egui::vec2(radius + 1.0, radius + 10.0);
    let burgundy = Color32::from_rgb(103, 29, 54);
    let ivory = Color32::from_rgb(246, 232, 226);
    painter.circle_filled(center, radius, burgundy);
    if active {
        painter.add(egui::Shape::convex_polygon(
            vec![
                center + egui::vec2(-radius * 0.34, -radius * 0.52),
                center + egui::vec2(radius * 0.56, 0.0),
                center + egui::vec2(-radius * 0.34, radius * 0.52),
            ],
            ivory,
            Stroke::NONE,
        ));
    } else {
        for x in [-0.30, 0.30] {
            painter.rect_filled(
                Rect::from_center_size(
                    center + Vec2::X * radius * x,
                    egui::vec2(radius * 0.29, radius * 1.02),
                ),
                0,
                ivory,
            );
        }
    }
}

fn contrast(a: Color32, b: Color32) -> f32 {
    let a = super::modern::luminance(a);
    let b = super::modern::luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn silver_finish(image: &mut image::RgbaImage, light_silver: bool, fill: Color32) {
    let mask = image.clone();
    let opaque = |x: i32, y: i32| {
        x >= 0
            && y >= 0
            && x < mask.width() as i32
            && y < mask.height() as i32
            && mask.get_pixel(x as u32, y as u32)[3] != 0
    };
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        if pixel[3] == 0 {
            continue;
        }
        let (x, y) = (x as i32, y as i32);
        // Inset bevels preserve the original silhouette and its transparent gaps.
        let upper_edge = !opaque(x - 3, y - 3);
        let lower_edge = !opaque(x + 3, y + 3);
        let diagonal = (x as f32 * 0.35 + y as f32) / mask.height() as f32;
        let reflection = (1.0 - (diagonal - 0.57).abs() / 0.13).clamp(0.0, 1.0);
        let band = ((diagonal - 0.42).abs() * 2.0).min(1.0);
        let inner_bevel = !opaque(x - 6, y - 6);
        let level = if light_silver {
            if upper_edge {
                253.0
            } else if lower_edge {
                155.0
            } else if inner_bevel {
                210.0
            } else {
                209.0 - band * 34.0 + reflection * 35.0
            }
        } else if upper_edge {
            104.0
        } else if lower_edge {
            22.0
        } else if inner_bevel {
            47.0
        } else {
            62.0 - band * 25.0 + reflection * 30.0
        };
        let level = level as u8;
        let mut color = Color32::from_rgb(level, level, level.saturating_add(5));
        let ink = foreground(fill);
        while contrast(color, fill) < 4.5 && color != ink {
            color = mix(color, ink, 0.12);
            if color.r().abs_diff(ink.r()) < 8 {
                color = ink;
            }
        }
        pixel.0[..3].copy_from_slice(&[color.r(), color.g(), color.b()]);
    }
}

fn paint_icon(
    painter: &egui::Painter,
    rect: Rect,
    name: &str,
    bytes: &[u8],
    dark: bool,
    fill: Color32,
) {
    let id = egui::Id::new(("saya_icon_silver", name, dark, fill));
    let cached: Option<Result<egui::TextureHandle, String>> =
        painter.ctx().data_mut(|data| data.get_temp(id));
    let texture = cached.unwrap_or_else(|| {
        let result = image::load_from_memory(bytes)
            .map(|image| {
                let mut rgba = image.to_rgba8();
                if !matches!(name, "job-active" | "job-inactive") {
                    silver_finish(&mut rgba, dark, fill);
                }
                painter.ctx().load_texture(
                    name,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    ),
                    egui::TextureOptions::NEAREST,
                )
            })
            .map_err(|error| format!("Could not load Saya icon {name}: {error}"));
        if let Err(error) = &result {
            eprintln!("{error}");
        }
        painter
            .ctx()
            .data_mut(|data| data.insert_temp(id, result.clone()));
        result
    });
    if let Ok(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

fn icon_asset(sprite: Sprite, dark: bool) -> (&'static str, &'static [u8]) {
    match sprite {
        Sprite::Floppy => asset!("job", dark),
        Sprite::Folder => asset!("choose-folder", dark),
        Sprite::Clock => asset!("schedule", dark),
        Sprite::Shield => asset!("protection", dark),
        Sprite::Trash => asset!("remove-job", dark),
        Sprite::Rewind => asset!("restore", dark),
        Sprite::Play => asset!("check-now", dark),
        Sprite::Chest => asset!("backups", dark),
        Sprite::Package => asset!("zip-backup", dark),
        Sprite::Sun => asset!("light", dark),
        Sprite::Moon => asset!("dark", dark),
        Sprite::Monitor => asset!("open-location", dark),
        Sprite::Warning => asset!("warning", dark),
        Sprite::Power => asset!("automatic", dark),
        Sprite::Check => asset!("checked", dark),
        Sprite::Cursor => asset!("backup-date", dark),
        Sprite::Plus => asset!("new-job", dark),
        Sprite::Gear => asset!("settings", dark),
        Sprite::JobSettings => asset!("job-settings", dark),
        Sprite::Theme => asset!("theme", dark),
        Sprite::FolderBackup => asset!("folder-backup", dark),
        Sprite::Interval => asset!("interval", dark),
        Sprite::Daily => asset!("daily", dark),
        Sprite::Weekdays => asset!("weekdays", dark),
    }
}

pub fn corners(painter: &egui::Painter, rect: Rect, color: Color32) {
    painter.rect_stroke(
        rect,
        3,
        Stroke::new(1.0_f32, color.gamma_multiply(0.55)),
        StrokeKind::Inside,
    );
    let reach = (rect.width().min(rect.height()) * 0.14).min(12.0);
    for (corner, direction) in [
        (rect.left_top(), Vec2::new(1.0, 1.0)),
        (rect.right_top(), Vec2::new(-1.0, 1.0)),
        (rect.left_bottom(), Vec2::new(1.0, -1.0)),
        (rect.right_bottom(), Vec2::new(-1.0, -1.0)),
    ] {
        let start = corner + direction * 4.0;
        painter.add(egui::Shape::line(
            vec![
                start + Vec2::X * direction.x * reach,
                start,
                start + Vec2::Y * direction.y * reach,
            ],
            Stroke::new(1.0_f32, color),
        ));
        diamond(painter, start + direction * 4.0, 2.0, color);
        if rect.width() > 100.0 && rect.height() > 70.0 {
            filigree(painter, start, direction, color);
        }
    }
}

fn filigree(painter: &egui::Painter, origin: Pos2, direction: Vec2, color: Color32) {
    let at = |x, y| origin + egui::vec2(x * direction.x, y * direction.y);
    for swap in [false, true] {
        let point = |x, y| if swap { at(y, x) } else { at(x, y) };
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [
                point(8.0, 0.0),
                point(16.0, 8.0),
                point(23.0, -2.0),
                point(34.0, 1.0),
            ],
            false,
            Color32::TRANSPARENT,
            Stroke::new(1.0_f32, color),
        ));
        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            [
                point(12.0, 2.0),
                point(17.0, -4.0),
                point(23.0, -2.0),
                point(19.0, 2.0),
            ],
            false,
            Color32::TRANSPARENT,
            Stroke::new(0.8_f32, color),
        ));
        diamond(
            painter,
            point(27.0, 1.0),
            1.7,
            Palette::of_ctx(painter.ctx()).hot,
        );
        painter.circle_filled(point(38.0, 1.0), 0.9, color);
    }
}

pub fn pattern(painter: &egui::Painter, rect: Rect) {
    let palette = Palette::of_ctx(painter.ctx());
    let strength = if painter.ctx().global_style().visuals.dark_mode {
        0.13
    } else {
        0.25
    };
    let color = palette.metal.gamma_multiply(strength);
    let mut mesh = egui::Mesh::default();
    let mut row = 0;
    let mut y = rect.top() + 18.0;
    while y < rect.bottom() - 12.0 {
        let mut x = rect.left() + 18.0 + if row % 2 == 0 { 0.0 } else { 18.0 };
        while x < rect.right() - 12.0 {
            let center = egui::pos2(x, y);
            mesh.add_colored_rect(Rect::from_center_size(center, egui::vec2(1.0, 5.0)), color);
            mesh.add_colored_rect(Rect::from_center_size(center, egui::vec2(5.0, 1.0)), color);
            x += 36.0;
        }
        row += 1;
        y += 36.0;
    }
    painter.add(mesh);
}

fn crescent(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    color: Color32,
    background: Color32,
) {
    painter.circle_filled(center, radius, color);
    painter.circle_filled(
        center + egui::vec2(radius * 0.4, -radius * 0.2),
        radius * 0.86,
        background,
    );
}

fn diamond(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![
            center - Vec2::Y * radius,
            center + Vec2::X * radius,
            center + Vec2::Y * radius,
            center - Vec2::X * radius,
        ],
        color,
        Stroke::NONE,
    ));
}

pub fn lamp(painter: &egui::Painter, rect: Rect, color: Color32) {
    diamond(painter, rect.center(), 4.0, color);
}

pub fn activity(painter: &egui::Painter, rect: Rect, active: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    let time = painter.ctx().input(|input| input.time) as f32;
    let center = egui::pos2(rect.left() + 13.0, rect.center().y);
    let silver = if active {
        palette.metal
    } else {
        mix(palette.metal, palette.sunken, 0.4)
    };
    let arc: Vec<_> = (0..25)
        .map(|step| {
            let angle = 0.65 + step as f32 / 24.0 * 5.0;
            center + Vec2::angled(angle) * 9.0
        })
        .collect();
    painter.add(egui::Shape::line(arc, Stroke::new(1.6_f32, silver)));
    let pulse = if active {
        0.35 + 0.2 * (time * 4.0).sin()
    } else {
        0.05
    };
    diamond(
        painter,
        center,
        5.0,
        mix(palette.accent, palette.hot, pulse),
    );
    painter.line_segment(
        [center - egui::vec2(2.0, 2.0), center - egui::vec2(0.0, 3.5)],
        Stroke::new(1.0_f32, palette.paper),
    );
    let start = egui::pos2(rect.left() + 28.0, rect.center().y);
    let end = egui::pos2(rect.right() - 3.0, rect.center().y);
    painter.line_segment([start, end], Stroke::new(0.8_f32, palette.outline));
    for index in 0..5 {
        diamond(painter, start.lerp(end, index as f32 / 4.0), 1.8, silver);
    }
    if active {
        let orbit = center + Vec2::angled(time * 2.3) * 9.0;
        diamond(painter, orbit, 2.2, palette.paper);
        diamond(
            painter,
            start.lerp(end, (time * 0.45).fract()),
            3.0,
            palette.hot,
        );
        painter
            .ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
    }
}

pub fn control(painter: &egui::Painter, rect: Rect, fill: Color32, focused: bool, enabled: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    painter.rect_filled(rect, 3, fill);
    painter.rect_stroke(
        rect,
        3,
        Stroke::new(1.0_f32, palette.outline),
        StrokeKind::Inside,
    );
    if enabled && fill == palette.accent && rect.width() > 50.0 {
        diamond(
            painter,
            egui::pos2(rect.left() + 5.0, rect.center().y),
            1.5,
            palette.accent_ink,
        );
        diamond(
            painter,
            egui::pos2(rect.right() - 5.0, rect.center().y),
            1.5,
            palette.accent_ink,
        );
    }
    if focused && enabled {
        painter.rect_stroke(
            rect.expand(2.0),
            4,
            Stroke::new(2.0_f32, palette.hot),
            StrokeKind::Outside,
        );
    }
}

pub fn title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite) {
    let palette = Palette::of(ui);
    let band = mix(palette.surface, palette.accent, 0.075);
    egui::Frame::new()
        .fill(band)
        .inner_margin(egui::Margin::symmetric(16, 11))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                sprites::show(ui, sprite, 2.0, palette.metal);
                ui.label(
                    egui::RichText::new(title)
                        .font(style::label_font(ui.ctx()))
                        .color(palette.ink),
                );
            });
            let rect = ui.min_rect();
            crescent(
                ui.painter(),
                egui::pos2(rect.right() - 8.0, rect.center().y),
                6.0,
                palette.metal,
                band,
            );
            let y = rect.bottom() + 6.0;
            ui.painter().line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(1.0_f32, palette.outline),
            );
            diamond(
                ui.painter(),
                egui::pos2(rect.right() - 4.0, y),
                3.0,
                palette.metal,
            );
            for offset in [-8.0, 0.0, 8.0] {
                diamond(
                    ui.painter(),
                    egui::pos2(rect.center().x + offset, y),
                    if offset == 0.0 { 3.0 } else { 1.5 },
                    palette.hot,
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(a: Color32, b: Color32) -> f32 {
        let luminance = |color: Color32| {
            let linear = |value: u8| {
                let value = f32::from(value) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
        };
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn saya_status_heads_use_identical_artwork_in_both_modes() {
        for (dark, light) in [
            (asset!("job-active", true).1, asset!("job-active", false).1),
            (
                asset!("job-inactive", true).1,
                asset!("job-inactive", false).1,
            ),
        ] {
            let dark = image::load_from_memory(dark).unwrap().to_rgba8();
            let light = image::load_from_memory(light).unwrap().to_rgba8();
            assert_eq!(dark.dimensions(), (128, 128));
            assert_eq!(dark.get_pixel(0, 0).0[3], 0);
            assert!(dark == light);
        }
    }

    #[test]
    fn icon_variants_match_geometry_and_contrast_control_surfaces() {
        let skin = crate::app::theme::skins::find("midnight-neko");
        for sprite in Sprite::ALL {
            let light = image::load_from_memory(icon_asset(sprite, false).1)
                .unwrap()
                .to_rgba8();
            let dark = image::load_from_memory(icon_asset(sprite, true).1)
                .unwrap()
                .to_rgba8();
            assert_eq!(light.dimensions(), dark.dimensions());
            assert!(light.pixels().zip(dark.pixels()).all(|(a, b)| a[3] == b[3]));
            for palette in [&skin.light, &skin.dark] {
                for base in [
                    palette.surface,
                    palette.raised,
                    palette.highlight,
                    palette.accent,
                    palette.good,
                    palette.bad,
                ] {
                    for fill in [
                        base,
                        mix(base, Color32::WHITE, 0.18),
                        mix(base, Color32::BLACK, 0.12),
                    ] {
                        let ink = foreground(fill);
                        let mut image = if ink == Color32::WHITE {
                            dark.clone()
                        } else {
                            light.clone()
                        };
                        silver_finish(&mut image, ink == Color32::WHITE, fill);
                        for pixel in image.pixels().filter(|pixel| pixel[3] != 0) {
                            let color = Color32::from_rgb(pixel[0], pixel[1], pixel[2]);
                            assert!(contrast(fill, color) >= 4.5, "{sprite:?} / {fill:?}");
                        }
                        assert!(contrast(fill, ink) >= 4.5);
                    }
                }
            }
        }
    }

    #[test]
    fn icons_share_resolution_padding_and_visible_extent() {
        let mut names = std::collections::HashSet::new();
        for sprite in Sprite::ALL {
            let (name, bytes) = icon_asset(sprite, true);
            assert!(names.insert(name), "duplicate icon for {sprite:?}");
            let image = image::load_from_memory(bytes).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (128, 128), "{name}");
            let mut bounds = (128, 128, 0, 0);
            for (x, y, pixel) in image.enumerate_pixels() {
                if pixel[3] != 0 {
                    bounds.0 = bounds.0.min(x);
                    bounds.1 = bounds.1.min(y);
                    bounds.2 = bounds.2.max(x + 1);
                    bounds.3 = bounds.3.max(y + 1);
                }
            }
            let (left, top, right, bottom) = bounds;
            assert!(
                left >= 12 && top >= 12 && right <= 116 && bottom <= 116,
                "{name}"
            );
            assert_eq!((right - left).max(bottom - top), 104, "{name}");
            assert!(left.abs_diff(128 - right) <= 1, "{name}");
            assert!(top.abs_diff(128 - bottom) <= 1, "{name}");
        }
    }
}
