//! Dark fantasy decorations: drifting embers, a vignette, fading rules and engraved title bars.

use super::pixel::mix;
use super::sprites::{self, Sprite};
use super::style::{label_font, Palette};
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::TAU;
use std::time::Duration;

const EMBER_FRAME: Duration = Duration::from_millis(50);
const EMBER_COUNT: u32 = 40;
const ICON_SCALE: f32 = 2.0;

/// Sparks rising through `rect`, swaying and flickering out as they climb.
pub fn paint_embers(ui: &egui::Ui, rect: Rect, palette: &Palette) {
    let time = ui.input(|input| input.time) as f32;
    let painter = ui.painter();
    let travel = rect.height() + 40.0;
    for index in 0..EMBER_COUNT {
        let random = |salt: u32| noise(index, salt);
        let speed = 14.0 + random(1) * 34.0;
        let progress = (time * speed / travel + random(2)).fract();
        let sway = (time * (0.5 + random(3)) + random(4) * TAU).sin() * (4.0 + random(5) * 16.0);
        let center = egui::pos2(
            rect.left() + random(0) * rect.width() + sway,
            rect.bottom() + 20.0 - progress * travel,
        );
        let flicker = 0.7 + 0.3 * (time * (4.0 + random(6) * 7.0)).sin();
        let strength = (1.0 - progress) * (progress * 8.0).min(1.0) * flicker;
        let color = mix(palette.hot, palette.gold, random(7) * 0.6);
        let radius = 1.0 + random(8) * 1.6;
        painter.circle_filled(center, radius * 3.5, fade(color, strength * 0.18));
        painter.circle_filled(center, radius, fade(color, strength));
    }
    ui.ctx().request_repaint_after(EMBER_FRAME);
}

/// Window edges sinking into the palette's vignette color; painted only, so input passes through.
pub fn paint_vignette(ctx: &egui::Context, palette: &Palette) {
    // Strength per ring, from the window edge inward; the steep first step keeps the middle clear.
    const RINGS: [(f32, f32); 3] = [(0.0, 1.0), (0.3, 0.35), (1.0, 0.0)];
    let layer = egui::LayerId::new(egui::Order::Debug, egui::Id::new("vignette"));
    let screen = ctx.content_rect();
    let depth = screen.width().min(screen.height()) * 0.16;
    let mut mesh = Mesh::default();
    for (inset, strength) in RINGS {
        let ring = screen.shrink(inset * depth);
        let color = fade(palette.vignette, strength);
        for corner in [
            ring.left_top(),
            ring.right_top(),
            ring.right_bottom(),
            ring.left_bottom(),
        ] {
            mesh.colored_vertex(corner, color);
        }
    }
    for ring in 0..RINGS.len() as u32 - 1 {
        let (outer, inner) = (ring * 4, ring * 4 + 4);
        for side in 0..4 {
            let next = (side + 1) % 4;
            mesh.add_triangle(outer + side, outer + next, inner + next);
            mesh.add_triangle(outer + side, inner + next, inner + side);
        }
    }
    ctx.layer_painter(layer).add(mesh);
}

/// A horizontal bar across `rect` that is solid in the middle and fades out at both ends.
pub fn fading_rule(painter: &egui::Painter, rect: Rect, color: Color32) {
    let mut mesh = Mesh::default();
    for (x, strength) in [
        (rect.left(), 0.0),
        (rect.center().x, 1.0),
        (rect.right(), 0.0),
    ] {
        mesh.colored_vertex(egui::pos2(x, rect.top()), fade(color, strength));
        mesh.colored_vertex(egui::pos2(x, rect.bottom()), fade(color, strength));
    }
    for start in [0, 2] {
        mesh.add_triangle(start, start + 1, start + 2);
        mesh.add_triangle(start + 1, start + 2, start + 3);
    }
    painter.add(mesh);
}

/// A dark band with a tinted icon, carved capitals and a diamond-tipped ornament line.
pub fn engraved_title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite, band: Color32) {
    let palette = Palette::of(ui);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, mix(palette.sunken, band, 0.12));
    let rule = Rect::from_min_max(
        egui::pos2(rect.left(), rect.bottom() - 2.0),
        rect.right_bottom(),
    );
    fading_rule(painter, rule, band);
    let icon = sprites::size(ui.ctx(), sprite, ICON_SCALE);
    let icon_pos = egui::pos2(rect.left() + 12.0, rect.center().y - icon.y / 2.0 - 1.0);
    sprites::paint(painter, icon_pos, sprite, ICON_SCALE, band);
    let galley = painter.layout_no_wrap(title.to_uppercase(), label_font(ui.ctx()), palette.ink);
    let text_pos = egui::pos2(
        icon_pos.x + icon.x + 12.0,
        rect.center().y - galley.size().y / 2.0 - 1.0,
    );
    let line_left = text_pos.x + galley.size().x + 16.0;
    painter.galley(text_pos.round(), galley, palette.ink);
    let line_right = rect.right() - 20.0;
    if line_right - line_left < 24.0 {
        return;
    }
    let y = rect.center().y - 1.0;
    let line = Stroke::new(1.0_f32, mix(band, palette.sunken, 0.45));
    painter.hline(line_left..=line_right, y, line);
    diamond(painter, egui::pos2(line_left, y), 3.0, band);
    diamond(painter, egui::pos2(line_right, y), 5.0, band);
}

/// A faceted gem lamp; lit lamps glow softly around the stone.
pub fn diamond_lamp(painter: &egui::Painter, rect: Rect, fill: Color32, lit: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    let center = rect.center();
    let radius = rect.width() / 2.0;
    if lit {
        painter.circle_filled(center, radius * 1.1, fade(fill, 0.25));
    }
    diamond(painter, center, radius, palette.outline);
    diamond(painter, center, radius - 1.5, fill);
    if lit {
        let glint = center + Vec2::new(-radius * 0.25, -radius * 0.25);
        diamond(
            painter,
            glint,
            radius * 0.25,
            mix(fill, Color32::WHITE, 0.7),
        );
    }
}

fn diamond(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let points = [
        Vec2::new(0.0, -radius),
        Vec2::new(radius, 0.0),
        Vec2::new(0.0, radius),
        Vec2::new(-radius, 0.0),
    ]
    .map(|offset| center + offset);
    painter.add(Shape::convex_polygon(points.to_vec(), color, Stroke::NONE));
}

/// `color` with its opacity scaled by `strength` in 0..=1.
pub fn fade(color: Color32, strength: f32) -> Color32 {
    color.gamma_multiply(strength.clamp(0.0, 1.0))
}

/// A stable pseudo-random value in 0..1, so every particle keeps its own path between frames.
pub fn noise(index: u32, salt: u32) -> f32 {
    let mut value = index.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 15;
    value = value.wrapping_mul(0x2c1b_3c6d);
    value ^= value >> 12;
    value = value.wrapping_mul(0x297a_2d39);
    value ^= value >> 15;
    (value >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{skins, style};

    #[test]
    fn noise_is_stable_and_stays_in_range() {
        for index in 0..EMBER_COUNT {
            for salt in 0..9 {
                let value = noise(index, salt);
                assert!((0.0..1.0).contains(&value));
                assert_eq!(value, noise(index, salt));
            }
        }
        assert_ne!(noise(1, 0), noise(2, 0));
    }

    #[test]
    fn ornaments_paint_under_every_skin() {
        let context = egui::Context::default();
        for skin in skins::ALL {
            style::install(&context, skin);
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    paint_embers(ui, ui.clip_rect(), Palette::of(ui));
                    engraved_title_bar(ui, "Folders", Sprite::Folder, Palette::of(ui).accent);
                });
                paint_vignette(ui.ctx(), Palette::of_ctx(ui.ctx()));
            });
        }
    }
}
