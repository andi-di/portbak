//! Shared decorations: fading rules and stable noise for animated scenery.

use eframe::egui::{self, Color32, Mesh, Rect};

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

/// `color` with its opacity scaled by `strength` in 0..=1.
fn fade(color: Color32, strength: f32) -> Color32 {
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

    #[test]
    fn noise_is_stable_and_stays_in_range() {
        for index in 0..40 {
            for salt in 0..9 {
                let value = noise(index, salt);
                assert!((0.0..1.0).contains(&value));
                assert_eq!(value, noise(index, salt));
            }
        }
        assert_ne!(noise(1, 0), noise(2, 0));
    }
}
