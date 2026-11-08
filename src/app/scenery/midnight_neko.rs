//! Moonlit roses, cursor-aware bats and sparse petal/crescent trails.

use crate::app::rendering::ornaments::noise;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::time::Duration;

const BAT_COUNT: usize = 7;
const PARTICLE_LIMIT: usize = 72;
const LIFETIME: f32 = 1.0;

#[derive(Clone, Copy)]
enum Sprite {
    Garden,
    BatDown,
    Perched,
    Petal,
    Crescent,
    PetalRound,
    PetalFolded,
    SilverSpark,
}

impl Sprite {
    fn asset(self) -> (&'static str, &'static [u8]) {
        match self {
            Self::Garden => (
                "garden",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/garden.png"),
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
            Self::PetalRound => (
                "petal_round",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/petal-round.png"),
            ),
            Self::PetalFolded => (
                "petal_folded",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/petal-folded.png"),
            ),
            Self::SilverSpark => (
                "silver_spark",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/silver-spark.png"),
            ),
        }
    }
}

#[derive(Clone)]
struct Bat {
    position: Pos2,
    velocity: Vec2,
    perched: bool,
    facing_left: bool,
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

fn state_id() -> egui::Id {
    egui::Id::new("saya_garden")
}

fn texture(painter: &egui::Painter, sprite: Sprite) -> Option<egui::TextureHandle> {
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
    texture.ok()
}

#[derive(Default)]
struct SpritePose {
    angle: f32,
    mirrored: bool,
    flap: f32,
}

fn draw(painter: &egui::Painter, rect: Rect, sprite: Sprite, tint: Color32) {
    draw_moving(painter, rect, sprite, tint, SpritePose::default());
}

fn sprite_point(rect: Rect, u: f32, v: f32, pose: &SpritePose) -> Pos2 {
    let wing = ((u - 0.5).abs() * 2.0 - 0.2).max(0.0);
    let offset = egui::vec2(
        (u - 0.5) * rect.width(),
        (v - 0.5) * rect.height() + pose.flap * wing * rect.height() * 0.18,
    );
    rect.center() + egui::emath::Rot2::from_angle(pose.angle) * offset
}

fn draw_moving(
    painter: &egui::Painter,
    rect: Rect,
    sprite: Sprite,
    tint: Color32,
    pose: SpritePose,
) {
    let Some(texture) = texture(painter, sprite) else {
        return;
    };
    let mut mesh = egui::Mesh::with_texture(texture.id());
    // Fixed central body columns keep wing motion from stretching the torso.
    let columns = [0.0, 0.4, 0.6, 1.0];
    for v in [0.0, 1.0] {
        for u in columns {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: sprite_point(rect, u, v, &pose),
                uv: egui::pos2(if pose.mirrored { 1.0 - u } else { u }, v),
                color: tint,
            });
        }
    }
    for column in 0..3 {
        mesh.add_triangle(column, column + 1, column + 4);
        mesh.add_triangle(column + 1, column + 5, column + 4);
    }
    painter.add(mesh);
}

fn petal(seed: u32) -> Sprite {
    match seed % 3 {
        0 => Sprite::Petal,
        1 => Sprite::PetalRound,
        _ => Sprite::PetalFolded,
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
    if bat.velocity.x.abs() > 8.0 {
        bat.facing_left = bat.velocity.x < 0.0;
    }
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
                facing_left: false,
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
    {
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
    }
    paint_scene(&painter, palette);
}

/// Repeat the same scene behind opaque panel contents, clipped by each painter.
pub fn paint_scene(painter: &egui::Painter, _palette: &Palette) {
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
    let state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    for (index, bat) in state.bats.iter().enumerate() {
        let sprite = if bat.perched {
            Sprite::Perched
        } else {
            Sprite::BatDown
        };
        draw_moving(
            painter,
            Rect::from_center_size(bat.position, Vec2::splat(36.0 + index as f32 * 2.0)),
            sprite,
            Color32::WHITE.gamma_multiply(0.42),
            SpritePose {
                mirrored: bat.facing_left,
                flap: if bat.perched {
                    0.0
                } else {
                    (time * 12.0 + index as f32).sin()
                },
                angle: (bat.velocity.y * 0.002).clamp(-0.12, 0.12),
            },
        );
    }
    for index in 0..36 {
        let progress = (time * (0.02 + noise(index, 4) * 0.012) + noise(index, 6)).fract();
        let center = egui::pos2(
            screen.left()
                + noise(index, 7) * screen.width()
                + (time * 0.4 + index as f32).sin() * 12.0,
            screen.top() + progress * screen.height(),
        );
        let size = 9.0 + noise(index, 8) * 9.0;
        let tumble = 0.4 + (time * 0.7 + index as f32).cos().abs() * 0.6;
        draw_moving(
            painter,
            Rect::from_center_size(center, egui::vec2(size * tumble, size)),
            petal(index),
            Color32::WHITE.gamma_multiply(0.35 + noise(index, 9) * 0.2),
            SpritePose {
                angle: time * (0.3 + noise(index, 10)) + index as f32,
                mirrored: index.is_multiple_of(2),
                ..Default::default()
            },
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
    if let Some(previous) = state.last_pointer {
        let distance = previous.distance(pointer);
        if distance > 14.0 {
            let steps = (distance / 14.0).floor().min(4.0) as u32;
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
    } else if state.last_pointer.is_none() {
        state.last_pointer = Some(pointer);
    }
    if pressed {
        release(state, pointer, time, 9);
    }
}

pub fn paint_overlays(ctx: &egui::Context, palette: &Palette) {
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("saya_cursor"),
    ));
    {
        let state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
        let time = ctx.input(|input| input.time) as f32;
        for particle in &state.particles {
            let age = (time - particle.born).max(0.0);
            let fade = (1.0 - age / LIFETIME).clamp(0.0, 1.0);
            let position =
                particle.origin + particle.velocity * age + egui::vec2(0.0, 12.0 * age * age);
            let sprite = match particle.seed % 5 {
                0 => Sprite::Crescent,
                1 => Sprite::SilverSpark,
                _ => petal(particle.seed),
            };
            let size = 10.0 + noise(particle.seed, 92) * 8.0;
            draw_moving(
                &painter,
                Rect::from_center_size(position, Vec2::splat(size)),
                sprite,
                Color32::WHITE.gamma_multiply(fade * 0.65),
                SpritePose {
                    angle: noise(particle.seed, 93) * 6.0 + age,
                    mirrored: particle.seed.is_multiple_of(2),
                    ..Default::default()
                },
            );
        }
    }
    // Retain native text, resize and hand cursors; only replace the plain arrow.
    if ctx.output(|output| output.cursor_icon) == egui::CursorIcon::Default {
        if let Some(pointer) = ctx.input(|input| input.pointer.hover_pos()) {
            ctx.set_cursor_icon(egui::CursorIcon::None);
            silver_cursor(&painter, pointer, palette.accent);
        }
    }
}

fn silver_cursor(painter: &egui::Painter, tip: Pos2, accent: Color32) {
    let at = |x, y| tip + egui::vec2(x, y);
    let rim = Color32::from_rgb(40, 30, 34);
    let outline = vec![
        at(0.0, 0.0),
        at(2.0, 27.0),
        at(8.0, 21.0),
        at(13.0, 32.0),
        at(18.0, 29.0),
        at(13.0, 18.0),
        at(22.0, 17.0),
    ];
    let facets = [
        (
            vec![at(0.0, 0.0), at(2.0, 27.0), at(9.0, 16.0)],
            Color32::from_rgb(249, 246, 241),
        ),
        (
            vec![at(0.0, 0.0), at(9.0, 16.0), at(22.0, 17.0)],
            Color32::from_rgb(181, 171, 171),
        ),
        (
            vec![at(2.0, 27.0), at(8.0, 21.0), at(9.0, 16.0)],
            Color32::from_rgb(129, 121, 132),
        ),
        (
            vec![
                at(8.0, 21.0),
                at(13.0, 32.0),
                at(18.0, 29.0),
                at(13.0, 18.0),
                at(9.0, 16.0),
            ],
            Color32::from_rgb(198, 192, 202),
        ),
        (
            vec![at(9.0, 16.0), at(13.0, 18.0), at(22.0, 17.0)],
            Color32::from_rgb(103, 92, 106),
        ),
    ];
    for (points, fill) in facets {
        painter.add(egui::Shape::convex_polygon(points, fill, Stroke::NONE));
    }
    painter.add(egui::Shape::closed_line(outline, Stroke::new(1.4_f32, rim)));
    painter.line_segment(
        [at(1.0, 2.0), at(9.0, 16.0)],
        Stroke::new(1.0_f32, Color32::WHITE),
    );
    painter.add(egui::Shape::convex_polygon(
        vec![
            at(11.0, 21.0),
            at(14.0, 23.0),
            at(14.0, 27.0),
            at(11.0, 25.0),
        ],
        accent,
        Stroke::new(0.8_f32, rim),
    ));
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
            facing_left: false,
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
        assert!(!bat.facing_left);
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
    fn bat_facing_follows_direction_and_wing_motion_keeps_body_fixed() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(300.0));
        let mut bat = Bat {
            position: screen.center(),
            velocity: Vec2::ZERO,
            perched: false,
            facing_left: false,
        };
        steer(&mut bat, Pos2::ZERO, None, false, 0.05, screen);
        steer(&mut bat, Pos2::ZERO, None, false, 0.05, screen);
        assert!(bat.facing_left);
        let rect = Rect::from_center_size(screen.center(), Vec2::splat(40.0));
        for flap in [-1.0, 0.0, 1.0] {
            let pose = SpritePose {
                flap,
                ..Default::default()
            };
            assert_eq!(sprite_point(rect, 0.5, 0.5, &pose), rect.center());
        }
        let a = sprite_point(
            rect,
            0.0,
            0.0,
            &SpritePose {
                flap: 0.0,
                ..Default::default()
            },
        );
        let b = sprite_point(
            rect,
            0.0,
            0.0,
            &SpritePose {
                flap: 0.01,
                ..Default::default()
            },
        );
        assert!(a.distance(b) < 0.1);
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
}
