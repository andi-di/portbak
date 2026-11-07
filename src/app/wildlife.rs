//! Frutiger Aero wildlife: gulls gliding high above and leaves tumbling on the breeze, painted
//! from vector shapes.

use super::glass;
use super::ornaments::noise;
use super::pixel::{mix, shaded_polygon};
use super::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::{PI, TAU};

const GULL_COUNT: u32 = 5;
const LEAF_COUNT: u32 = 6;

/// A turning flat shape's apparent width, kept from collapsing to a line when seen edge-on;
/// `turn` runs from -1 (back) through 0 (edge-on) to 1 (front).
pub fn edge_on(turn: f32) -> f32 {
    const THINNEST: f32 = 0.25;
    let side = if turn < 0.0 { -1.0 } else { 1.0 };
    side * (THINNEST + (1.0 - THINNEST) * turn.abs().min(1.0))
}

/// Seagulls gliding across the upper sky, flapping now and then.
pub fn gulls(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    for index in 0..GULL_COUNT {
        let random = |salt: u32| noise(index, salt + 300);
        let span = random(0) * 18.0 + 16.0;
        let speed = 22.0 + random(1) * 18.0;
        let width = screen.width() + span * 4.0;
        let x = screen.right() + span * 2.0 - (random(2) * width + time * speed) % width;
        let drift = (time * 0.5 + random(3) * TAU).sin() * 10.0;
        let y = screen.top() + screen.height() * (0.08 + random(4) * 0.3) + drift;
        // Bursts of flapping between long glides.
        let flapping = (time * 0.4 + random(5) * TAU).sin() > 0.2;
        let beat = if flapping {
            (time * 8.0 + random(6) * TAU).sin()
        } else {
            0.2
        };
        let body = egui::pos2(x, y);
        for side in [-1.0, 1.0] {
            painter.add(wing(body, span, side, beat, palette));
        }
        let torso = glass::ellipse(body, Vec2::new(span * 0.2, span * 0.08), 0.0);
        painter.add(Shape::convex_polygon(torso, Color32::WHITE, Stroke::NONE));
    }
}

/// One tapered wing curving up from the body and down to a grey tip; `side` is -1 or 1.
fn wing(body: Pos2, span: f32, side: f32, beat: f32, palette: &Palette) -> Shape {
    const STEPS: u32 = 10;
    let elbow = body + Vec2::new(side * span * 0.45, -span * (0.35 + beat * 0.3));
    let tip = body + Vec2::new(side * span, -span * (0.05 - beat * 0.45));
    let tip_color = mix(Color32::WHITE, palette.outline, 0.55);
    let mut mesh = egui::Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let along = body.to_vec2() * (1.0 - t) * (1.0 - t)
            + elbow.to_vec2() * 2.0 * t * (1.0 - t)
            + tip.to_vec2() * t * t;
        let thickness = span * 0.16 * (1.0 - t * 0.85);
        let color = if t > 0.75 { tip_color } else { Color32::WHITE };
        mesh.colored_vertex(along.to_pos2(), color);
        mesh.colored_vertex(along.to_pos2() + Vec2::new(0.0, thickness), color);
        if step > 0 {
            let last = step * 2;
            mesh.add_triangle(last - 2, last - 1, last);
            mesh.add_triangle(last - 1, last + 1, last);
        }
    }
    mesh.into()
}

/// Glossy green leaves tumbling across the window on the breeze, some carrying a dew drop.
pub fn leaves(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32, night: bool) {
    for index in 0..LEAF_COUNT {
        let random = |salt: u32| noise(index, salt + 400);
        let length = 22.0 + random(0) * 22.0;
        let speed = 16.0 + random(1) * 16.0;
        let span = screen.width() + length * 4.0;
        let travelled = (random(2) * span + time * speed) % span;
        let x = screen.left() - length * 2.0 + travelled;
        let fall = travelled / span * screen.height() * 0.5;
        let flutter = (time * (0.7 + random(3)) + random(4) * TAU).sin() * 24.0;
        let y = screen.top() + screen.height() * (0.1 + random(5) * 0.4) + fall + flutter;
        let angle = (time * 0.6 + random(6) * TAU).sin() * 0.8 + random(7) * TAU;
        // Turning over in the wind: the leaf narrows as it shows its edge.
        let flip = edge_on((time * (0.8 + random(8)) + random(9) * TAU).cos());
        let mut lush = mix(palette.good, Color32::from_rgb(0xb0, 0xf0, 0x30), 0.4);
        if night {
            lush = mix(lush, palette.surface, 0.4);
        }
        leaf(
            painter,
            egui::pos2(x, y),
            length,
            angle,
            flip,
            lush,
            random(10) > 0.5,
        );
    }
}

fn leaf(
    painter: &egui::Painter,
    center: Pos2,
    length: f32,
    angle: f32,
    flip: f32,
    color: Color32,
    dew: bool,
) {
    const STEPS: u32 = 12;
    let turn = Vec2::angled(angle);
    let place = |x: f32, y: f32| {
        let local = Vec2::new(x * length, y * length * flip);
        center
            + Vec2::new(
                local.x * turn.x - local.y * turn.y,
                local.x * turn.y + local.y * turn.x,
            )
    };
    let width = |t: f32| 0.24 * (t * PI).sin().powf(0.8);
    let mut outline: Vec<Pos2> = (0..=STEPS)
        .map(|step| {
            let t = step as f32 / STEPS as f32;
            place(t - 0.5, -width(t))
        })
        .collect();
    outline.extend((1..STEPS).rev().map(|step| {
        let t = step as f32 / STEPS as f32;
        place(t - 0.5, width(t))
    }));
    let light = mix(color, Color32::WHITE, 0.35);
    let dark = mix(color, Color32::BLACK, 0.3);
    // Lit from the top of the screen, so the shading stays put as the leaf flips over.
    painter.add(shaded_polygon(&outline, |point| {
        let t = ((point.y - center.y) / length + 0.5).clamp(0.0, 1.0);
        light.lerp_to_gamma(dark, t)
    }));
    painter.add(Shape::closed_line(
        outline,
        Stroke::new(1.0_f32, mix(color, Color32::BLACK, 0.45)),
    ));
    painter.line_segment(
        [place(-0.56, 0.0), place(0.45, 0.0)],
        Stroke::new(1.2_f32, mix(color, Color32::WHITE, 0.55)),
    );
    // Built in the leaf's own frame, so the shine flattens with it and never sticks out.
    let shine: Vec<Pos2> = (0..16)
        .map(|step| {
            let around = Vec2::angled(step as f32 * TAU / 16.0);
            place(-0.05 + around.x * 0.22, -0.1 + around.y * 0.05)
        })
        .collect();
    painter.add(Shape::convex_polygon(
        shine,
        Color32::WHITE.gamma_multiply(0.45 * flip.abs()),
        Stroke::NONE,
    ));
    if dew && flip > 0.3 {
        glass::sphere(
            painter,
            place(0.15, 0.08),
            length * 0.08,
            mix(color, Color32::WHITE, 0.7),
            0.6,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::skins;

    #[test]
    fn turning_shapes_never_collapse_to_a_line() {
        for step in -100..=100 {
            let turn = step as f32 / 100.0;
            let width = edge_on(turn);
            assert!((0.25..=1.0).contains(&width.abs()), "{turn} -> {width}");
            assert!(turn == 0.0 || width.signum() == turn.signum());
        }
    }

    #[test]
    fn wildlife_paints_at_any_time_and_size() {
        let context = egui::Context::default();
        let palette = &skins::default().light;
        for size in [egui::vec2(800.0, 600.0), egui::vec2(1.0, 1.0)] {
            let screen = Rect::from_min_size(Pos2::ZERO, size);
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                let ctx = &ui.ctx().clone();
                let painter = ctx.layer_painter(egui::LayerId::background());
                for time in [0.0_f32, 13.7, 9_999.0] {
                    gulls(&painter, screen, palette, time);
                    leaves(&painter, screen, palette, time, time > 1.0);
                }
            });
        }
    }
}
