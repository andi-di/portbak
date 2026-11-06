//! Forged-iron widget faces for engraved skins: chamfered plates and sockets grown with thorns.

use super::pixel::mix;
use super::style::Palette;
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Stroke, Vec2};

/// Room a plate needs on each side for its side thorns.
pub const THORN_REACH: f32 = 8.0;
const CURVE_STEPS: u32 = 6;

/// How a plate should look in its current interaction state.
pub struct PlateLook {
    pub fill: Color32,
    pub enabled: bool,
    pub pressed: bool,
    /// Hover glow from 0 (none) to 1 (full).
    pub glow: f32,
    pub focused: bool,
}

/// A chamfered plate with a gilded inner line, thorns at its ends and edges, and a soft shadow
/// or ember glow. `face` excludes the thorns; `spike` is how far edge thorns reach up and down.
pub fn thorned_plate(painter: &egui::Painter, face: Rect, spike: f32, look: &PlateLook) {
    let palette = Palette::of_ctx(painter.ctx());
    let iron = if look.enabled {
        mix(look.fill, palette.metal, 0.35)
    } else {
        palette.shade
    };
    painter.add(soft_shadow(face, palette));
    if look.glow > 0.0 {
        let glow = egui::Shadow {
            offset: [0, 0],
            blur: 22,
            spread: 2,
            color: mix(look.fill, palette.hot, 0.6).gamma_multiply(0.75 * look.glow),
        };
        painter.add(glow.as_shape(face, 4.0));
    }
    for (base_a, base_b, tip) in plate_thorns(face, spike) {
        thorn(painter, base_a, base_b, tip, iron);
    }
    let chamfer = (face.height() * 0.28).min(9.0);
    let (top, bottom) = match (look.enabled, look.pressed) {
        (false, _) => (palette.surface, palette.surface),
        (true, false) => (
            mix(look.fill, Color32::WHITE, 0.14),
            mix(look.fill, Color32::BLACK, 0.4),
        ),
        (true, true) => (
            mix(look.fill, Color32::BLACK, 0.4),
            mix(look.fill, Color32::BLACK, 0.1),
        ),
    };
    let outline = octagon(face, chamfer);
    gradient_polygon(painter, &outline, face, top, bottom);
    let edge = if look.enabled {
        palette.outline
    } else {
        palette.shade
    };
    painter.add(Shape::closed_line(outline, Stroke::new(1.5_f32, edge)));
    let trim =
        mix(look.fill, palette.gold, 0.55).gamma_multiply(if look.enabled { 0.8 } else { 0.3 });
    painter.add(Shape::closed_line(
        octagon(face.shrink(3.0), (chamfer - 1.5).max(1.0)),
        Stroke::new(1.0_f32, trim),
    ));
    if look.focused {
        painter.add(Shape::closed_line(
            octagon(face.expand(3.0), chamfer + 1.5),
            Stroke::new(2.0_f32, palette.hot),
        ));
    }
}

/// A sunken square socket with clipped corners and a thorn at each corner, for check boxes.
pub fn spiked_socket(painter: &egui::Painter, rect: Rect, border: Color32) {
    let palette = Palette::of_ctx(painter.ctx());
    let center = rect.center();
    let reach = rect.width() * 0.22;
    let iron = mix(border, palette.metal, 0.5);
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let corner = center + Vec2::new(dx, dy) * rect.width() * 0.36;
        let along = Vec2::new(-dy, dx) * rect.width() * 0.12;
        let tip = corner + Vec2::new(dx, dy) * reach * 1.6;
        thorn(painter, corner - along, corner + along, tip, iron);
    }
    let chamfer = rect.width() * 0.26;
    let outline = octagon(rect, chamfer);
    gradient_polygon(
        painter,
        &outline,
        rect,
        palette.sunken,
        mix(palette.sunken, palette.raised, 0.6),
    );
    painter.add(Shape::closed_line(outline, Stroke::new(1.5_f32, border)));
}

/// The soft shadow under raised plates.
fn soft_shadow(face: Rect, palette: &Palette) -> Shape {
    let shadow = egui::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: palette.outline.gamma_multiply(0.6),
    };
    shadow.as_shape(face, 4.0).into()
}

/// Side thorns hooking upward from each end, plus pairs leaning outward from the top and bottom.
fn plate_thorns(face: Rect, spike: f32) -> Vec<(Pos2, Pos2, Pos2)> {
    let (left, right) = (face.left() + 2.0, face.right() - 2.0);
    let middle = face.center().y;
    let half = face.height() * 0.24;
    let mut thorns = vec![
        (
            egui::pos2(left, middle + half),
            egui::pos2(left, middle - half),
            egui::pos2(face.left() - THORN_REACH, middle - half * 0.7),
        ),
        (
            egui::pos2(right, middle - half),
            egui::pos2(right, middle + half),
            egui::pos2(face.right() + THORN_REACH, middle - half * 0.7),
        ),
    ];
    if spike < 2.0 {
        return thorns;
    }
    let lean = spike * 1.1;
    for (fraction, y, reach) in [
        (0.12, face.top() + 2.0, -spike * 1.15),
        (0.3, face.bottom() - 2.0, spike),
    ] {
        for side in [-1.0_f32, 1.0] {
            let x = face.center().x + side * face.width() * (0.5 - fraction);
            let base = 5.0;
            thorns.push((
                egui::pos2(x - base, y),
                egui::pos2(x + base, y),
                egui::pos2(x + side * lean, y + reach),
            ));
        }
    }
    thorns
}

/// A thorn with concave flanks, from the base edge `a`–`b` out to `tip`.
fn thorn(painter: &egui::Painter, a: Pos2, b: Pos2, tip: Pos2, color: Color32) {
    let base = a + (b - a) / 2.0;
    // Pull each flank's control point toward the thorn's axis so the sides curve inward.
    let inward = |from: Pos2| {
        let middle = from + (tip - from) / 2.0;
        let axis_point = base + (tip - base) / 2.0;
        middle + (axis_point - middle) * 0.45
    };
    let mut outline = quadratic(a, inward(a), tip);
    outline.extend(quadratic(tip, inward(b), b).into_iter().skip(1));
    let mut mesh = Mesh::default();
    mesh.colored_vertex(base, color);
    for point in &outline {
        mesh.colored_vertex(*point, color);
    }
    for index in 1..outline.len() as u32 {
        mesh.add_triangle(0, index, index + 1);
    }
    painter.add(mesh);
}

fn quadratic(from: Pos2, control: Pos2, to: Pos2) -> Vec<Pos2> {
    (0..=CURVE_STEPS)
        .map(|step| {
            let t = step as f32 / CURVE_STEPS as f32;
            let (u, v) = (1.0 - t, t);
            (from.to_vec2() * u * u + control.to_vec2() * 2.0 * u * v + to.to_vec2() * v * v)
                .to_pos2()
        })
        .collect()
}

/// The corners of `rect` with each corner cut by `chamfer`, clockwise from the top left.
fn octagon(rect: Rect, chamfer: f32) -> Vec<Pos2> {
    let (l, t, r, b, c) = (
        rect.left(),
        rect.top(),
        rect.right(),
        rect.bottom(),
        chamfer,
    );
    [
        (l + c, t),
        (r - c, t),
        (r, t + c),
        (r, b - c),
        (r - c, b),
        (l + c, b),
        (l, b - c),
        (l, t + c),
    ]
    .map(|(x, y)| egui::pos2(x, y))
    .to_vec()
}

/// Fills a convex polygon with a vertical gradient across `bounds`.
fn gradient_polygon(
    painter: &egui::Painter,
    points: &[Pos2],
    bounds: Rect,
    top: Color32,
    bottom: Color32,
) {
    let shade = |y: f32| {
        mix(
            top,
            bottom,
            ((y - bounds.top()) / bounds.height()).clamp(0.0, 1.0),
        )
    };
    let center = bounds.center();
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, shade(center.y));
    for point in points {
        mesh.colored_vertex(*point, shade(point.y));
    }
    let count = points.len() as u32;
    for index in 0..count {
        mesh.add_triangle(0, index + 1, (index + 1) % count + 1);
    }
    painter.add(mesh);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plates_grow_two_side_thorns_and_edge_thorns_only_with_room() {
        let face = Rect::from_min_size(Pos2::ZERO, egui::vec2(120.0, 32.0));
        assert_eq!(plate_thorns(face, 0.0).len(), 2);
        let thorns = plate_thorns(face, 4.0);
        assert_eq!(thorns.len(), 6);
        for (_, _, tip) in thorns {
            assert!(
                !face.contains(tip),
                "thorn tip {tip:?} stays inside the plate"
            );
        }
    }

    #[test]
    fn octagon_cuts_every_corner() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(40.0, 20.0));
        let points = octagon(rect, 5.0);
        assert_eq!(points.len(), 8);
        assert!(points
            .iter()
            .all(|point| rect.expand(0.01).contains(*point)));
        assert!(!points.contains(&rect.min) && !points.contains(&rect.max));
    }
}
