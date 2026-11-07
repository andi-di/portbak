//! Slow ash motes, tiny garnet diamonds and a restrained pointer trail.

use crate::app::rendering::ornaments::noise;
use crate::app::rendering::pixel::mix;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::TAU;
use std::time::Duration;

const MOTES: u32 = 90;
const PARTICLE_LIMIT: usize = 48;
const LIFETIME: f32 = 0.7;

#[derive(Clone, Default)]
struct Magic {
    particles: Vec<Particle>,
    last_pointer: Option<Pos2>,
    released: u32,
}

#[derive(Clone)]
struct Particle {
    origin: Pos2,
    velocity: Vec2,
    born: f32,
    seed: u32,
}

fn state_id() -> egui::Id {
    egui::Id::new("midnight_neko_magic")
}

pub fn paint_background(ctx: &egui::Context, palette: &Palette) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen = ctx.content_rect();
    let time = ctx.input(|input| input.time) as f32;
    let mut mesh = egui::Mesh::default();
    for (position, color) in [
        (screen.left_top(), palette.sky),
        (screen.right_top(), palette.sky),
        (screen.right_bottom(), palette.background),
        (screen.left_bottom(), palette.background),
    ] {
        mesh.colored_vertex(position, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
    floating_motes(ctx, &painter, screen, time, palette);
    ctx.request_repaint_after(Duration::from_millis(50));
}

fn floating_motes(
    ctx: &egui::Context,
    behind: &egui::Painter,
    screen: Rect,
    time: f32,
    palette: &Palette,
) {
    let front = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("midnight_neko_ash"),
    ));
    let pointer = ctx.input(|input| input.pointer.hover_pos());
    for index in 0..MOTES {
        let radius = 2.0 + noise(index, 10) * 2.0;
        let progress =
            time * (5.0 + noise(index, 11) * 5.0) / (screen.height() + 12.0) + noise(index, 12);
        let center = egui::pos2(
            screen.left()
                + noise(index, 13) * screen.width()
                + (time * 0.25 + index as f32).sin() * 7.0,
            screen.bottom() + 6.0 - progress.fract() * (screen.height() + 12.0),
        );
        let fade = (progress.fract() * 10.0).min(1.0) * ((1.0 - progress.fract()) * 10.0).min(1.0);
        let distance_fade =
            pointer.map_or(1.0, |point| (point.distance(center) / 70.0).clamp(0.0, 1.0));
        let color = mix(palette.metal, palette.hot, noise(index, 14) * 0.35)
            .gamma_multiply(fade * distance_fade * 0.55);
        let painter = if index % 2 == 0 { &front } else { behind };
        if index % 3 == 0 {
            diamond(painter, center, radius, color);
        } else {
            painter.circle_filled(center, radius * 0.7, color);
        }
    }
}

fn diamond(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![
            center - Vec2::Y * radius,
            center + Vec2::X * radius * 0.65,
            center + Vec2::Y * radius,
            center - Vec2::X * radius * 0.65,
        ],
        color,
        Stroke::NONE,
    ));
}

fn sparkle(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    painter.line_segment(
        [center - Vec2::X * radius, center + Vec2::X * radius],
        Stroke::new(1.5_f32, color),
    );
    painter.line_segment(
        [
            center - Vec2::Y * radius * 1.4,
            center + Vec2::Y * radius * 1.4,
        ],
        Stroke::new(1.5_f32, color),
    );
    painter.circle_filled(center, radius * 0.22, color);
}

fn release(state: &mut Magic, origin: Pos2, time: f32, count: u32) {
    for _ in 0..count {
        let seed = state.released;
        state.released = state.released.wrapping_add(1);
        state.particles.push(Particle {
            origin,
            velocity: Vec2::angled(noise(seed, 81) * TAU) * (12.0 + noise(seed, 82) * 48.0),
            born: time,
            seed,
        });
    }
    let excess = state.particles.len().saturating_sub(PARTICLE_LIMIT);
    state.particles.drain(..excess);
}

fn record_pointer(state: &mut Magic, pointer: Option<Pos2>, pressed: bool, time: f32) {
    state
        .particles
        .retain(|particle| time - particle.born < LIFETIME);
    let Some(pointer) = pointer else {
        state.last_pointer = None;
        return;
    };
    if let Some(previous) = state.last_pointer {
        let distance = previous.distance(pointer);
        if distance >= 12.0 {
            let steps = (distance / 12.0).floor().min(6.0) as u32;
            for step in 1..=steps {
                release(
                    state,
                    previous.lerp(pointer, step as f32 / steps as f32),
                    time,
                    1,
                );
            }
            state.last_pointer = Some(pointer);
        }
    } else {
        state.last_pointer = Some(pointer);
    }
    if pressed {
        release(state, pointer, time, 5);
    }
}

pub fn paint_overlays(ctx: &egui::Context, palette: &Palette) {
    let (time, pointer, pressed) = ctx.input(|input| {
        (
            input.time as f32,
            input.pointer.hover_pos(),
            input.pointer.any_pressed(),
        )
    });
    let mut state: Magic = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    record_pointer(&mut state, pointer, pressed, time);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("midnight_neko_stardust"),
    ));
    for particle in &state.particles {
        let age = (time - particle.born).max(0.0);
        let fade = 1.0 - age / LIFETIME;
        let center = particle.origin + particle.velocity * age + egui::vec2(0.0, -24.0 * age * age);
        let color =
            mix(palette.hot, palette.gold, noise(particle.seed, 83)).gamma_multiply(fade * 0.55);
        let radius = (1.5 + noise(particle.seed, 84) * 2.0) * fade;
        if particle.seed % 4 == 0 {
            diamond(&painter, center, radius, color);
        } else {
            sparkle(&painter, center, radius, color);
        }
    }
    ctx.data_mut(|data| data.insert_temp(state_id(), state));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_trail_is_bounded_expires_and_does_not_bridge_pointer_leaves() {
        let mut state = Magic::default();
        for step in 0..1000 {
            record_pointer(
                &mut state,
                Some(egui::pos2(step as f32 * 10.0, 30.0)),
                true,
                0.0,
            );
        }
        assert_eq!(state.particles.len(), PARTICLE_LIMIT);
        record_pointer(&mut state, None, false, 2.0);
        assert!(state.particles.is_empty());
        record_pointer(&mut state, Some(egui::pos2(300.0, 400.0)), false, 2.1);
        assert!(state.particles.is_empty());
    }
}
