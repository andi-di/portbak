//! Matte surfaces, fine silver ornaments and restrained crimson focus accents.

use crate::app::rendering::pixel::mix;
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, Palette};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};

pub fn icon(painter: &egui::Painter, rect: Rect, sprite: Sprite) {
    let cell = match sprite {
        Sprite::Folder => 0,
        Sprite::Clock => 1,
        Sprite::Shield => 2,
        Sprite::Trash => 3,
        Sprite::Rewind => 4,
        Sprite::Play => 5,
        Sprite::Chest => 6,
        Sprite::Package => 7,
        Sprite::Sun => 8,
        Sprite::Moon => 9,
        Sprite::Monitor => 10,
        Sprite::Warning => 11,
        Sprite::Power => 12,
        Sprite::Check => 13,
        Sprite::Plus => 14,
        Sprite::Gear => 15,
        Sprite::Floppy | Sprite::Cursor => {
            crate::app::scenery::catgirl::paint_icon(painter, rect);
            return;
        }
    };
    let id = egui::Id::new("midnight_neko_icons_v3");
    let cached: Option<Result<egui::TextureHandle, String>> =
        painter.ctx().data_mut(|data| data.get_temp(id));
    let texture = cached.unwrap_or_else(|| {
        let result = image::load_from_memory(include_bytes!(
            "../../../assets/themes/midnight-neko/icons.png"
        ))
        .map(|image| {
            let rgba = image.to_rgba8();
            painter.ctx().load_texture(
                "Midnight Neko icons",
                egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        })
        .map_err(|error| format!("Could not load Gothic icons: {error}"));
        if let Err(error) = &result {
            eprintln!("{error}");
        }
        painter
            .ctx()
            .data_mut(|data| data.insert_temp(id, result.clone()));
        result
    });
    if let Ok(texture) = texture {
        let min = egui::pos2((cell % 4) as f32 / 4.0, (cell / 4) as f32 / 4.0);
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(min, min + Vec2::splat(0.25)),
            Color32::WHITE,
        );
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
