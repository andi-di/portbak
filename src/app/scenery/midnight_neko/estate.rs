//! Asymmetric estate layers leave the central workspace open.

use super::{draw, draw_moving, noise, texture, Sprite, SpritePose};
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
            match index % 3 {
                0 => Sprite::Cloud,
                1 => Sprite::CloudVariant,
                _ => Sprite::CloudBank,
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
    for layer in 0..4 {
        let phase = layer as f32 * 1.9;
        let center = egui::pos2(
            screen.center().x + (time * 0.025 + phase).sin() * screen.width() * 0.12,
            screen.bottom() - 18.0 - layer as f32 * 24.0 + (time * 0.13 + phase).sin() * 5.0,
        );
        let width = screen.width() * (1.0 + layer as f32 * 0.06);
        let breathe = 0.85 + 0.15 * (time * 0.17 + phase).sin();
        let color = if dark {
            Color32::from_rgb(166, 187, 230).gamma_multiply(0.15 * breathe)
        } else {
            Color32::from_rgb(100, 117, 155).gamma_multiply(0.12 * breathe)
        };
        paint_mist(
            painter,
            Rect::from_center_size(center, egui::vec2(width, width * 0.12)),
            color,
            time,
            layer,
        );
    }
}

fn paint_mist(painter: &egui::Painter, rect: Rect, color: Color32, time: f32, layer: u32) {
    let Some(texture) = texture(painter, Sprite::Mist) else {
        return;
    };
    const COLUMNS: u32 = 24;
    let mut mesh = egui::Mesh::with_texture(texture.id());
    let phase = layer as f32 * 1.9;
    for v in [0.0, 1.0] {
        for column in 0..=COLUMNS {
            let u = column as f32 / COLUMNS as f32;
            let wave = (u * 10.0 + time * 0.22 + phase).sin() * 5.0
                + (u * 17.0 - time * 0.14 + phase).sin() * 2.5;
            let edge = (u * 8.0).min((1.0 - u) * 8.0).min(1.0);
            mesh.vertices.push(egui::epaint::Vertex {
                pos: egui::pos2(
                    rect.left() + u * rect.width(),
                    rect.top() + v * rect.height() + wave,
                ),
                uv: egui::pos2(if layer.is_multiple_of(2) { u } else { 1.0 - u }, v),
                color: color.gamma_multiply(edge),
            });
        }
    }
    for column in 0..COLUMNS {
        mesh.add_triangle(column, column + 1, column + COLUMNS + 1);
        mesh.add_triangle(column + 1, column + COLUMNS + 2, column + COLUMNS + 1);
    }
    painter.add(mesh);
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
