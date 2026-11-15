//! Short, bounded accents shared by the sidebar and gallery.

use super::Expression;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

pub(super) fn duration(expression: Expression) -> Option<f64> {
    match expression {
        Expression::Love => Some(super::LOVE_HEART_SECONDS + super::LOVE_HEARTS[2].3),
        Expression::Excited => Some(1.8),
        _ => None,
    }
}

pub(super) fn paint(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f64) {
    match expression {
        Expression::Love => super::paint_love_hearts(ui, rect, elapsed),
        Expression::Excited => excited(ui, rect, elapsed as f32),
        _ => {}
    }
}

fn envelope(progress: f32) -> f32 {
    (std::f32::consts::PI * progress.clamp(0.0, 1.0))
        .sin()
        .powf(0.7)
}

fn excited(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let at = |x, y| rect.min + rect.size() * Vec2::new(x, y);
    if let Some(texture) = super::texture(
        ui.ctx(),
        "excited-sparkles",
        include_bytes!("../../../../assets/themes/moonveil/overlays/excited-sparkles.png"),
    ) {
        for (x, y, size, delay, angle) in [
            (0.23, 0.24, 0.19, 0.0, -0.16),
            (0.79, 0.19, 0.23, 0.18, 0.13),
            (0.12, 0.48, 0.15, 0.42, -0.10),
            (0.88, 0.43, 0.17, 0.62, 0.18),
        ] {
            let progress = (elapsed - delay) / 1.15;
            if !(0.0..1.0).contains(&progress) {
                continue;
            }
            let strength = envelope(progress);
            let center = at(x, y - progress * 0.035);
            let side = rect.width() * size * (0.72 + strength * 0.28);
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                Rect::from_center_size(center, Vec2::splat(side)),
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE.gamma_multiply(strength),
            );
            mesh.rotate(
                egui::emath::Rot2::from_angle(angle * (progress - 0.5)),
                center,
            );
            painter.add(mesh);
        }
    }

    // Exact centers of the existing pupil stars in excited.png (512 × 512).
    for (x, y, phase) in [(225.0, 162.0, 0.0), (293.0, 162.0, 0.22)] {
        let strength = envelope(elapsed / 1.8) * (0.72 + 0.28 * (elapsed * 12.0 + phase).cos());
        let center = at(x / 512.0, y / 512.0);
        let radius = rect.width() / 512.0 * (2.3 + 1.2 * strength);
        let color = Color32::from_rgb(255, 250, 224).gamma_multiply(strength);
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(center, color);
        for index in 0..8 {
            let angle = index as f32 * std::f32::consts::FRAC_PI_4;
            let length = if index % 2 == 0 {
                radius
            } else {
                radius * 0.22
            };
            mesh.colored_vertex(center + Vec2::angled(angle) * length, color);
            mesh.add_triangle(0, index + 1, (index + 1) % 8 + 1);
        }
        painter.add(mesh);
        painter.circle_filled(
            center,
            radius * 0.19,
            Color32::WHITE.gamma_multiply(strength),
        );
    }
}
