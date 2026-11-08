//! Asymmetric estate layers leave the central workspace open.

use super::{draw, noise, Sprite};
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

pub(super) fn castle_rect(screen: Rect) -> Rect {
    let size = (screen.width() * 0.47)
        .clamp(280.0, 680.0)
        .min(screen.width() * 0.72)
        .min(screen.height() * 0.78);
    Rect::from_min_size(
        egui::pos2(screen.right() - size - 8.0, screen.bottom() - size - 16.0),
        Vec2::splat(size),
    )
}

fn cloud(painter: &egui::Painter, center: Pos2, size: Vec2, color: Color32) {
    for (offset, scale) in [(-0.28, 0.65), (0.0, 1.0), (0.30, 0.55)] {
        painter.add(egui::Shape::ellipse_filled(
            center + egui::vec2(size.x * offset, size.y * offset * 0.2),
            size * scale,
            color,
        ));
    }
}

pub(super) fn paint(painter: &egui::Painter) {
    let ctx = painter.ctx();
    let screen = ctx.content_rect();
    let palette = Palette::of_ctx(ctx);
    let dark = ctx.global_style().visuals.dark_mode;
    let time = ctx.input(|input| input.time) as f32;
    let silver = if dark {
        Color32::from_rgb(166, 157, 177)
    } else {
        Color32::from_rgb(80, 69, 89)
    };
    for layer in 0..2 {
        let mut points = vec![screen.left_bottom()];
        for index in 0..18 {
            points.push(egui::pos2(
                screen.left() + screen.width() * index as f32 / 17.0,
                screen.bottom()
                    - screen.height()
                        * (0.12 + noise(index, layer + 201) * 0.15 + layer as f32 * 0.06),
            ));
        }
        for pair in points[1..].windows(2) {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    pair[0],
                    pair[1],
                    egui::pos2(pair[1].x, screen.bottom()),
                    egui::pos2(pair[0].x, screen.bottom()),
                ],
                silver.gamma_multiply(0.055),
                Stroke::NONE,
            ));
        }
    }
    let radius = (screen.height() * 0.12).clamp(55.0, 100.0);
    let moon = egui::pos2(screen.right() - radius * 1.7, screen.top() + 145.0 + radius);
    draw(
        painter,
        Rect::from_center_size(moon, Vec2::splat(radius * 2.0)),
        Sprite::Moon,
        Color32::WHITE.gamma_multiply(if dark { 0.48 } else { 0.28 }),
    );
    for index in 0..5 {
        let progress = (time * (0.002 + noise(index, 220) * 0.001) + noise(index, 221)).fract();
        let center = egui::pos2(
            screen.left() - 220.0 + progress * (screen.width() + 440.0),
            screen.top() + 120.0 + noise(index, 222) * screen.height() * 0.35,
        );
        cloud(
            painter,
            center,
            egui::vec2(
                110.0 + noise(index, 223) * 100.0,
                15.0 + noise(index, 224) * 14.0,
            ),
            crate::app::rendering::pixel::mix(palette.background, silver, 0.22)
                .gamma_multiply(0.35),
        );
    }
    draw(
        painter,
        castle_rect(screen),
        Sprite::Castle,
        Color32::WHITE.gamma_multiply(if dark { 0.62 } else { 0.32 }),
    );
    for layer in 0..3 {
        let center = egui::pos2(
            screen.center().x + (time * 0.035 + layer as f32 * 2.0).sin() * screen.width() * 0.18,
            screen.bottom() - 22.0 - layer as f32 * 25.0,
        );
        cloud(
            painter,
            center,
            egui::vec2(screen.width() * 0.45, 15.0 + layer as f32 * 5.0),
            silver.gamma_multiply(0.025),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estate_stays_on_the_right_inside_resized_windows() {
        for size in [
            egui::vec2(610.0, 480.0),
            egui::vec2(1000.0, 700.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let screen = Rect::from_min_size(egui::pos2(30.0, 20.0), size);
            let castle = castle_rect(screen);
            assert!(screen.contains_rect(castle));
            assert!(castle.left() >= screen.left() + screen.width() * 0.25);
        }
    }
}
