//! Asymmetric estate layers leave the central workspace open.

use super::{draw, draw_moving, noise, Sprite, SpritePose};
use eframe::egui::{self, Color32, Rect, Vec2};

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

pub(super) fn paint(painter: &egui::Painter) {
    let ctx = painter.ctx();
    let screen = ctx.content_rect();
    let dark = ctx.global_style().visuals.dark_mode;
    let time = ctx.input(|input| input.time) as f32;
    let mountains = Rect::from_min_size(
        egui::pos2(screen.left(), screen.bottom() - screen.height() * 0.42),
        egui::vec2(screen.width(), screen.height() * 0.42),
    );
    draw(
        painter,
        mountains,
        Sprite::Mountains,
        Color32::WHITE.gamma_multiply(if dark { 0.25 } else { 0.16 }),
    );
    let radius = (screen.height() * 0.12).clamp(55.0, 100.0);
    let moon = egui::pos2(screen.right() - radius * 1.7, screen.top() + 145.0 + radius);
    draw(
        painter,
        Rect::from_center_size(moon, Vec2::splat(radius * 2.0)),
        Sprite::Moon,
        Color32::WHITE.gamma_multiply(if dark { 0.48 } else { 0.28 }),
    );
    for index in 0..5 {
        let progress = (time * (0.0025 + noise(index, 220) * 0.0075) + noise(index, 221)).fract();
        let center = egui::pos2(
            screen.left() - 220.0 + progress * (screen.width() + 440.0),
            screen.top() + 120.0 + noise(index, 222) * screen.height() * 0.35,
        );
        let width = 260.0 + noise(index, 223) * 180.0;
        let color = if dark {
            Color32::WHITE.gamma_multiply(0.23)
        } else {
            Color32::from_rgb(132, 120, 145).gamma_multiply(0.23)
        };
        draw_moving(
            painter,
            Rect::from_center_size(center, egui::vec2(width, width * 0.34)),
            if index.is_multiple_of(2) {
                Sprite::Cloud
            } else {
                Sprite::CloudVariant
            },
            color,
            SpritePose {
                mirrored: index.is_multiple_of(2),
                angle: (time * 0.025 + index as f32).sin() * 0.025,
                ..Default::default()
            },
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
        let width = screen.width() * (0.85 + layer as f32 * 0.08);
        let color = if dark {
            Color32::WHITE.gamma_multiply(0.10)
        } else {
            Color32::from_rgb(125, 110, 135).gamma_multiply(0.12)
        };
        draw_moving(
            painter,
            Rect::from_center_size(center, egui::vec2(width, width * 0.15)),
            Sprite::Mist,
            color,
            SpritePose {
                mirrored: layer % 2 == 0,
                angle: (time * 0.015 + layer as f32).sin() * 0.018,
                ..Default::default()
            },
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
