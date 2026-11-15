//! Short, bounded accents shared by the sidebar and gallery.

use super::Expression;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

pub(super) fn duration(expression: Expression) -> Option<f64> {
    match expression {
        Expression::Love => Some(super::LOVE_HEART_SECONDS + super::LOVE_HEARTS[2].3),
        Expression::Celebrating => Some(2.4),
        Expression::Excited => Some(1.8),
        _ => None,
    }
}

pub(super) fn paint(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f64) {
    match expression {
        Expression::Love => super::paint_love_hearts(ui, rect, elapsed),
        Expression::Celebrating => confetti(ui, rect, elapsed as f32),
        Expression::Excited => excited(ui, rect, elapsed as f32),
        _ => {}
    }
}

fn envelope(progress: f32) -> f32 {
    (std::f32::consts::PI * progress.clamp(0.0, 1.0))
        .sin()
        .powf(0.7)
}

fn confetti(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let colors = [
        Color32::from_rgb(221, 222, 239),
        Color32::from_rgb(157, 39, 75),
        Color32::from_rgb(228, 150, 175),
        Color32::from_rgb(173, 152, 203),
        Color32::from_rgb(244, 220, 178),
    ];
    let mut mesh = egui::Mesh::default();
    // Deterministic trajectories avoid random jitter and per-frame particle state.
    for index in 0..64 {
        let random = |channel: u32| {
            let mut bits = (index as u32 + 1).wrapping_mul(0x9e37_79b9) ^ channel;
            bits = (bits ^ (bits >> 16)).wrapping_mul(0x7feb_352d);
            bits = (bits ^ (bits >> 15)).wrapping_mul(0x846c_a68b);
            (bits ^ (bits >> 16)) as f32 / u32::MAX as f32
        };
        let phase = random(1) * std::f32::consts::TAU;
        let age = elapsed - random(2) * 1.05;
        if age <= 0.0 {
            continue;
        }
        let sway = 0.012 + random(3) * 0.038;
        let flutter = 2.0 + random(4) * 5.0;
        let x = 0.04
            + random(5) * 0.92
            + ((phase + age * flutter).sin() - phase.sin()) * sway
            + (random(6) - 0.5) * 0.07 * age;
        let y = -0.06 + (0.32 + random(7) * 0.42) * age + (0.07 + random(8) * 0.1) * age * age;
        let center = rect.min + rect.size() * Vec2::new(x, y);
        let opacity = (age * 10.0).min(1.0) * ((2.4 - elapsed) / 0.6).clamp(0.0, 1.0);
        let color = colors[index % colors.len()].gamma_multiply(opacity);
        let spin = (1.5 + random(9) * 5.5) * if random(10) < 0.5 { -1.0 } else { 1.0 };
        let rotation = egui::emath::Rot2::from_angle(phase + age * spin);
        let width = rect.width() * (0.006 + random(11) * 0.01);
        let flip = (age * (4.0 + random(12) * 8.0) + phase).cos();
        let length = width * (1.1 + random(13) * 2.3);
        let base = mesh.vertices.len() as u32;
        // Folded foil has a bright crease and a darker underside as it tumbles.
        for (point, shade) in [
            (Vec2::new(-width * flip, -length), 0.65),
            (Vec2::new(width * flip, -length * 0.7), 0.9),
            (Vec2::new(width * flip * 0.6, length), 0.6),
            (Vec2::new(-width * flip, length * 0.7), 0.8),
            (Vec2::ZERO, 1.0),
        ] {
            mesh.colored_vertex(center + rotation * point, color.gamma_multiply(shade));
        }
        for corner in 0..4 {
            mesh.add_triangle(base + 4, base + corner, base + (corner + 1) % 4);
        }
    }
    painter.add(mesh);
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
        eye_glare(&painter, center, rect.width() / 512.0, elapsed, phase);
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

fn eye_glare(painter: &egui::Painter, center: Pos2, scale: f32, elapsed: f32, phase: f32) {
    let flash = envelope((elapsed - 0.12 - phase * 0.15) / 1.2);
    if flash <= 0.0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    for (extent, color) in [
        (
            Vec2::new(12.0, 3.8),
            Color32::from_rgb(217, 226, 255).gamma_multiply(flash * 0.42),
        ),
        (
            Vec2::new(14.0, 0.9),
            Color32::from_rgb(255, 252, 234).gamma_multiply(flash * 0.85),
        ),
    ] {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, color);
        for index in 0..24 {
            let angle = index as f32 * std::f32::consts::TAU / 24.0;
            let offset = Vec2::angled(angle) * extent * scale * (0.75 + flash * 0.25);
            mesh.colored_vertex(center + offset, Color32::TRANSPARENT);
            mesh.add_triangle(base, base + index + 1, base + (index + 1) % 24 + 1);
        }
    }
    painter.add(mesh);
}
