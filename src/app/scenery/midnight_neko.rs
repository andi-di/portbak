//! Candy-cloud wallpaper, glossy red hearts and a light dusting of pointer stardust.

use crate::app::rendering::ornaments::noise;
use crate::app::rendering::pixel::mix;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;
use std::f32::consts::TAU;
use std::time::Duration;

const HEARTS: u32 = 48;
const PARTICLE_LIMIT: usize = 32;
const LIFETIME: f32 = 0.55;

#[derive(Clone, Default)]
struct Magic {
    popped: HashMap<u32, (u64, f32)>,
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
    for index in 0..12 {
        let center = screen.min
            + egui::vec2(
                noise(index, 31) * screen.width(),
                noise(index, 32) * screen.height() + (time * 0.2 + index as f32).sin() * 14.0,
            );
        painter.circle_filled(
            center,
            50.0 + noise(index, 33) * 110.0,
            palette.hot.gamma_multiply(0.035),
        );
    }
    floating_hearts(ctx, &painter, screen, time);
    ctx.request_repaint_after(Duration::from_millis(50));
}

fn floating_hearts(ctx: &egui::Context, behind: &egui::Painter, screen: Rect, time: f32) {
    let front = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("midnight_neko_hearts"),
    ));
    let pointer = ctx.input(|input| input.pointer.hover_pos());
    let mut state: Magic = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    for index in 0..HEARTS {
        let radius = 10.0 + noise(index, 10) * 15.0;
        let travel = screen.height() + radius * 4.0;
        let progress = time * (18.0 + noise(index, 11) * 24.0) / travel + noise(index, 12);
        let trip = progress.floor() as u64;
        let center = egui::pos2(
            screen.left()
                + radius
                + noise(index, 13) * (screen.width() - radius * 2.0).max(0.0)
                + (time * 0.7 + index as f32).sin() * 18.0,
            screen.bottom() + radius * 2.0 - progress.fract() * travel,
        );
        let painter = if index % 3 == 0 { &front } else { behind };
        let color = mix(
            Color32::from_rgb(0xd4, 0x12, 0x3b),
            Color32::from_rgb(0xff, 0x45, 0x58),
            noise(index, 14),
        );
        if let Some((popped_trip, born)) = state.popped.get(&index) {
            if *popped_trip == trip {
                burst(painter, center, radius, time - born, color);
                continue;
            }
        }
        if pointer.is_some_and(|point| point.distance(center) < radius) {
            state.popped.insert(index, (trip, time));
            release(&mut state, center, time, 2);
            continue;
        }
        let fade = (progress.fract() * 12.0).min(1.0) * ((1.0 - progress.fract()) * 12.0).min(1.0);
        glossy_heart(painter, center, radius, color, fade * 0.8);
    }
    ctx.data_mut(|data| data.insert_temp(state_id(), state));
}

fn heart(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, color);
    for step in 0..=40 {
        let angle = step as f32 / 40.0 * TAU;
        let x = angle.sin().powi(3);
        let y = -(13.0 * angle.cos()
            - 5.0 * (2.0 * angle).cos()
            - 2.0 * (3.0 * angle).cos()
            - (4.0 * angle).cos())
            / 16.0;
        mesh.colored_vertex(center + egui::vec2(x, y) * radius, color);
        if step > 0 {
            mesh.add_triangle(0, step, step + 1);
        }
    }
    painter.add(mesh);
}

fn glossy_heart(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, opacity: f32) {
    let depth = Vec2::new(radius * 0.05, radius * 0.12);
    heart(
        painter,
        center + depth * 1.5,
        radius * 1.03,
        Color32::from_black_alpha((opacity * 36.0) as u8),
    );
    heart(
        painter,
        center + depth,
        radius,
        Color32::from_rgb(0x79, 0x08, 0x25).gamma_multiply(opacity),
    );
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(
        center + egui::vec2(-0.22, -0.22) * radius,
        mix(color, Color32::from_rgb(255, 154, 159), 0.6).gamma_multiply(opacity),
    );
    for step in 0..=48 {
        let angle = step as f32 / 48.0 * TAU;
        let x = angle.sin().powi(3);
        let y = -(13.0 * angle.cos()
            - 5.0 * (2.0 * angle).cos()
            - 2.0 * (3.0 * angle).cos()
            - (4.0 * angle).cos())
            / 16.0;
        let shade = ((x + y) * 0.3 + 0.5).clamp(0.0, 1.0);
        let edge = mix(color, Color32::from_rgb(0x74, 0x06, 0x23), shade);
        mesh.colored_vertex(
            center + egui::vec2(x, y) * radius,
            edge.gamma_multiply(opacity),
        );
        if step > 0 {
            mesh.add_triangle(0, step, step + 1);
        }
    }
    painter.add(mesh);
    painter.add(egui::Shape::ellipse_filled(
        center + egui::vec2(-0.42, -0.49) * radius,
        egui::vec2(0.23, 0.12) * radius,
        Color32::WHITE.gamma_multiply(opacity * 0.8),
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

fn burst(painter: &egui::Painter, center: Pos2, radius: f32, age: f32, color: Color32) {
    if !(0.0..0.55).contains(&age) {
        return;
    }
    let fade = 1.0 - age / 0.55;
    for index in 0..8 {
        let direction = Vec2::angled(index as f32 / 8.0 * TAU);
        heart(
            painter,
            center + direction * radius * (1.0 + age * 4.0),
            4.0 * fade,
            color.gamma_multiply(fade),
        );
    }
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
        if distance >= 22.0 {
            let steps = (distance / 22.0).floor().min(4.0) as u32;
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
        release(state, pointer, time, 4);
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
        let color = mix(palette.hot, palette.gold, noise(particle.seed, 83)).gamma_multiply(fade);
        let radius = (2.0 + noise(particle.seed, 84) * 4.0) * fade;
        if particle.seed % 4 == 0 {
            heart(&painter, center, radius, color);
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
