//! Moonlit roses, cursor-aware bats and sparse petal/crescent trails.

use crate::app::rendering::ornaments::noise;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::time::Duration;

const BAT_COUNT: usize = 7;
const PARTICLE_LIMIT: usize = 24;
const LIFETIME: f32 = 0.7;

#[derive(Clone, Copy)]
enum Sprite {
    Garden,
    BatUp,
    BatDown,
    Perched,
    Petal,
    Crescent,
}

impl Sprite {
    fn asset(self) -> (&'static str, &'static [u8]) {
        match self {
            Self::Garden => (
                "garden",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/garden.png"),
            ),
            Self::BatUp => (
                "bat_up",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/bat-up.png"),
            ),
            Self::BatDown => (
                "bat_down",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/bat-down.png"),
            ),
            Self::Perched => (
                "perched",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/bat-perched.png"),
            ),
            Self::Petal => (
                "petal",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/petal.png"),
            ),
            Self::Crescent => (
                "crescent",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/crescent.png"),
            ),
        }
    }
}

#[derive(Clone)]
struct Bat {
    position: Pos2,
    velocity: Vec2,
    perched: bool,
}

#[derive(Clone, Default)]
struct Garden {
    bats: Vec<Bat>,
    particles: Vec<Particle>,
    last_time: Option<f32>,
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

pub fn set_animations(ctx: &egui::Context, enabled: bool) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new("saya_animations"), enabled));
}

fn animated(ctx: &egui::Context) -> bool {
    ctx.data_mut(|data| {
        data.get_temp(egui::Id::new("saya_animations"))
            .unwrap_or(true)
    })
}

fn state_id() -> egui::Id {
    egui::Id::new("saya_garden")
}

fn draw(painter: &egui::Painter, rect: Rect, sprite: Sprite, tint: Color32) {
    let (name, bytes) = sprite.asset();
    let id = egui::Id::new(("saya_scenery", name));
    let cached: Option<Result<egui::TextureHandle, String>> =
        painter.ctx().data_mut(|data| data.get_temp(id));
    let texture = cached.unwrap_or_else(|| {
        let result = image::load_from_memory(bytes)
            .map(|image| {
                let rgba = image.to_rgba8();
                painter.ctx().load_texture(
                    name,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    ),
                    egui::TextureOptions::NEAREST,
                )
            })
            .map_err(|error| format!("Could not load garden sprite {name}: {error}"));
        if let Err(error) = &result {
            eprintln!("{error}");
        }
        painter
            .ctx()
            .data_mut(|data| data.insert_temp(id, result.clone()));
        result
    });
    if let Ok(texture) = texture {
        painter.image(
            texture.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            tint,
        );
    }
}

fn home(screen: Rect, index: usize) -> Pos2 {
    egui::pos2(
        screen.left() + screen.width() * if index.is_multiple_of(2) { 0.08 } else { 0.92 },
        screen.top() + screen.height() * (0.13 + index as f32 * 0.09),
    )
}

fn steer(bat: &mut Bat, target: Pos2, pointer: Option<Pos2>, pressed: bool, dt: f32, screen: Rect) {
    let mut desired = (target - bat.position).normalized() * 30.0;
    if let Some(pointer) = pointer {
        let away = bat.position - pointer;
        let radius = if pressed { 230.0 } else { 100.0 };
        if away.length() < radius {
            desired = away.normalized() * 120.0;
            if away.length_sq() < 1.0 {
                desired = Vec2::X * 120.0;
            }
            bat.perched = false;
        }
    }
    bat.velocity += (desired - bat.velocity) * (dt * 5.0).min(1.0);
    bat.position += bat.velocity * dt;
    bat.position.x = bat.position.x.clamp(screen.left(), screen.right());
    bat.position.y = bat.position.y.clamp(screen.top(), screen.bottom());
}

fn update(state: &mut Garden, screen: Rect, time: f32, pointer: Option<Pos2>, pressed: bool) {
    let dt = state
        .last_time
        .map_or(0.0, |previous| (time - previous).clamp(0.0, 0.05));
    state.last_time = Some(time);
    if state.bats.is_empty() {
        state.bats = (0..BAT_COUNT)
            .map(|index| Bat {
                position: home(screen, index),
                velocity: Vec2::ZERO,
                perched: true,
            })
            .collect();
    }
    for (index, bat) in state.bats.iter_mut().enumerate() {
        let phase = time * 0.2 + index as f32 * 1.7;
        let roost = home(screen, index);
        let resting = (time + index as f32 * 3.1).rem_euclid(24.0) > 15.0;
        let target = if resting {
            roost
        } else if index == 0 && pointer.is_some() {
            pointer.unwrap() + egui::vec2(120.0, -90.0)
        } else {
            egui::pos2(
                screen.center().x + phase.sin() * screen.width() * 0.43,
                screen.center().y + (phase * 0.7).cos() * screen.height() * 0.36,
            )
        };
        bat.perched = resting && bat.position.distance(roost) < 8.0;
        if bat.perched {
            bat.velocity = Vec2::ZERO;
        }
        steer(bat, target, pointer, pressed, dt, screen);
    }
    record_pointer(state, pointer, pressed, time);
}

pub fn paint_background(ctx: &egui::Context, palette: &Palette) {
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0, palette.background);
    if animated(ctx) {
        let (time, pointer, pressed) = ctx.input(|input| {
            (
                input.time as f32,
                input.pointer.hover_pos(),
                input.pointer.any_pressed(),
            )
        });
        let mut state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
        update(&mut state, screen, time, pointer, pressed);
        ctx.data_mut(|data| data.insert_temp(state_id(), state));
        ctx.request_repaint_after(Duration::from_millis(50));
    } else {
        ctx.data_mut(|data| data.remove::<Garden>(state_id()));
    }
    paint_scene(&painter, palette);
}

/// Repeat the same scene behind opaque panel contents, clipped by each painter.
pub fn paint_scene(painter: &egui::Painter, palette: &Palette) {
    let ctx = painter.ctx();
    let screen = ctx.content_rect();
    let dark = ctx.global_style().visuals.dark_mode;
    draw(
        painter,
        screen,
        Sprite::Garden,
        Color32::WHITE.gamma_multiply(if dark { 0.16 } else { 0.12 }),
    );
    let time = ctx.input(|input| input.time) as f32;
    if !animated(ctx) {
        return;
    }
    let state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    for (index, bat) in state.bats.iter().enumerate() {
        let sprite = if bat.perched {
            Sprite::Perched
        } else if (time * 5.0 + index as f32).fract() < 0.5 {
            Sprite::BatUp
        } else {
            Sprite::BatDown
        };
        draw(
            painter,
            Rect::from_center_size(bat.position, Vec2::splat(36.0 + index as f32 * 2.0)),
            sprite,
            Color32::WHITE.gamma_multiply(0.42),
        );
    }
    for index in 0..12 {
        let progress = (time * (0.02 + noise(index, 4) * 0.012) + noise(index, 6)).fract();
        let center = egui::pos2(
            screen.left()
                + noise(index, 7) * screen.width()
                + (time * 0.4 + index as f32).sin() * 12.0,
            screen.top() + progress * screen.height(),
        );
        draw(
            painter,
            Rect::from_center_size(center, Vec2::splat(10.0)),
            Sprite::Petal,
            palette.hot.gamma_multiply(0.3),
        );
    }
}

fn release(state: &mut Garden, origin: Pos2, time: f32, count: u32) {
    for _ in 0..count {
        let seed = state.released;
        state.released = state.released.wrapping_add(1);
        state.particles.push(Particle {
            origin,
            velocity: Vec2::angled(noise(seed, 81) * std::f32::consts::TAU)
                * (12.0 + noise(seed, 82) * 35.0),
            born: time,
            seed,
        });
    }
    let excess = state.particles.len().saturating_sub(PARTICLE_LIMIT);
    state.particles.drain(..excess);
}

fn record_pointer(state: &mut Garden, pointer: Option<Pos2>, pressed: bool, time: f32) {
    state
        .particles
        .retain(|particle| time - particle.born < LIFETIME);
    let Some(pointer) = pointer else {
        state.last_pointer = None;
        return;
    };
    if state
        .last_pointer
        .is_some_and(|previous| previous.distance(pointer) > 24.0)
    {
        release(state, pointer, time, 1);
        state.last_pointer = Some(pointer);
    } else if state.last_pointer.is_none() {
        state.last_pointer = Some(pointer);
    }
    if pressed {
        release(state, pointer, time, 5);
    }
}

pub fn paint_overlays(ctx: &egui::Context, palette: &Palette) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("saya_cursor"),
    ));
    if animated(ctx) {
        let state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
        let time = ctx.input(|input| input.time) as f32;
        for particle in &state.particles {
            let age = (time - particle.born).max(0.0);
            let fade = (1.0 - age / LIFETIME).clamp(0.0, 1.0);
            let position =
                particle.origin + particle.velocity * age + egui::vec2(0.0, 12.0 * age * age);
            let sprite = if particle.seed % 3 == 0 {
                Sprite::Crescent
            } else {
                Sprite::Petal
            };
            draw(
                &painter,
                Rect::from_center_size(position, Vec2::splat(12.0)),
                sprite,
                Color32::WHITE.gamma_multiply(fade * 0.65),
            );
        }
    }
    // Retain native text, resize and hand cursors; only replace the plain arrow.
    if ctx.output(|output| output.cursor_icon) == egui::CursorIcon::Default {
        if let Some(pointer) = ctx.input(|input| input.pointer.hover_pos()) {
            ctx.set_cursor_icon(egui::CursorIcon::None);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    pointer,
                    pointer + egui::vec2(1.0, 20.0),
                    pointer + egui::vec2(15.0, 14.0),
                ],
                Color32::from_rgb(241, 208, 202),
                Stroke::new(1.5_f32, Color32::from_rgb(53, 37, 44)),
            ));
            painter.line_segment(
                [
                    pointer + egui::vec2(6.0, 12.0),
                    pointer + egui::vec2(11.0, 23.0),
                ],
                Stroke::new(3.0_f32, palette.accent),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bats_flee_pointer_and_stay_inside_small_windows() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 100.0));
        let mut bat = Bat {
            position: egui::pos2(80.0, 50.0),
            velocity: Vec2::ZERO,
            perched: true,
        };
        steer(
            &mut bat,
            egui::pos2(60.0, 50.0),
            Some(egui::pos2(70.0, 50.0)),
            false,
            0.05,
            screen,
        );
        assert!(bat.velocity.x > 0.0);
        assert!(!bat.perched);
        for _ in 0..200 {
            steer(
                &mut bat,
                egui::pos2(900.0, -900.0),
                None,
                false,
                0.05,
                screen,
            );
        }
        assert!(screen.contains(bat.position));
    }
    #[test]
    fn pointer_trail_is_bounded_expires_and_does_not_bridge_pointer_leaves() {
        let mut state = Garden::default();
        for step in 0..1000 {
            record_pointer(
                &mut state,
                Some(egui::pos2(step as f32 * 30.0, 30.0)),
                true,
                0.0,
            );
        }
        assert_eq!(state.particles.len(), PARTICLE_LIMIT);
        record_pointer(&mut state, None, false, 2.0);
        assert!(state.particles.is_empty());
        record_pointer(&mut state, Some(Pos2::ZERO), false, 2.1);
        assert!(state.particles.is_empty());
    }

    #[test]
    fn disabling_motion_clears_dynamic_state_and_keeps_static_scene() {
        let ctx = egui::Context::default();
        let mut garden = Garden::default();
        release(&mut garden, Pos2::ZERO, 0.0, 5);
        ctx.data_mut(|data| data.insert_temp(state_id(), garden));
        set_animations(&ctx, false);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            paint_background(ui.ctx(), &crate::app::theme::skins::default().dark);
        });
        assert!(!animated(&ctx));
        assert!(ctx
            .data_mut(|data| data.get_temp::<Garden>(state_id()))
            .is_none());
    }
}
