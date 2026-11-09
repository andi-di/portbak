//! Moonlit roses, cursor-aware bats and sparse petal/crescent trails.

mod cursors;
mod estate;

use crate::app::rendering::ornaments::noise;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use std::time::Duration;

const BAT_COUNT: usize = 12;
const PETAL_COUNT: u32 = 60;
const REACTION_RADIUS: f32 = 180.0;
const CLICK_REACTION_RADIUS: f32 = 320.0;
const PARTICLE_LIMIT: usize = 72;
const LIFETIME: f32 = 1.0;
const CLICK_ESCAPE_SECONDS: f32 = 3.5;
const CLICK_ESCAPE_SPEED: f32 = 400.0;

#[derive(Clone, Copy)]
enum Sprite {
    Castle,
    Moon,
    Cloud,
    CloudVariant,
    Mountains,
    Mist,
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
            Self::Castle => (
                "castle",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/castle.png"),
            ),
            Self::Moon => (
                "moon",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/moon.png"),
            ),
            Self::Cloud => (
                "cloud",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/cloud.png"),
            ),
            Self::CloudVariant => (
                "cloud-variant",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/cloud-variant.png"),
            ),
            Self::Mountains => (
                "mountains",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/mountains.png"),
            ),
            Self::Mist => (
                "mist",
                include_bytes!("../../../assets/themes/midnight-neko/scenery/mist.png"),
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

#[derive(Clone, Default)]
struct Bat {
    position: Pos2,
    velocity: Vec2,
    perched: bool,
    facing_left: bool,
    return_at: f32,
    leave_at: f32,
    cycle: u32,
    seed: u32,
    size: f32,
    burst_seconds: f32,
    escape_target: Pos2,
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
    let dark = painter.ctx().global_style().visuals.dark_mode;
    let phase = if matches!(sprite, Sprite::Castle) {
        painter.ctx().input(|input| (input.time * 0.7) as u32 % 8)
    } else {
        0
    };
    let id = egui::Id::new(("saya_scenery", name, dark, phase));
    let cached: Option<Result<egui::TextureHandle, String>> =
        painter.ctx().data_mut(|data| data.get_temp(id));
    let texture = cached.unwrap_or_else(|| {
        let result = image::load_from_memory(bytes)
            .map(|image| {
                let mut rgba = image.to_rgba8();
                if matches!(sprite, Sprite::BatDown | Sprite::Perched) {
                    for pixel in rgba.pixels_mut() {
                        let red = pixel[0].saturating_sub(pixel[1]) > 25;
                        let color = match (dark, red) {
                            (true, false) => [12, 10, 14],
                            (true, true) => [4, 3, 5],
                            (false, false) => [53, 30, 48],
                            (false, true) => [119, 31, 65],
                        };
                        pixel.0[..3].copy_from_slice(&color);
                    }
                }
                if matches!(sprite, Sprite::Castle) {
                    let height = rgba.height();
                    for (x, y, pixel) in rgba.enumerate_pixels_mut() {
                        if y < height * 2 / 3 && pixel[0].saturating_sub(pixel[1]) > 35 {
                            let flicker = noise(x / 16 + y / 16 * 100, phase) > 0.85;
                            if flicker {
                                pixel[0] = pixel[0].saturating_add(35);
                                pixel[1] = pixel[1].saturating_add(12);
                            }
                        }
                    }
                }
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

fn home(screen: Rect, index: usize, size: f32) -> Pos2 {
    let inset = (size * 0.5).min(screen.width() * 0.5);
    egui::pos2(
        screen.left()
            + inset
            + (screen.width() - inset * 2.0) * (index as f32 + 0.5) / BAT_COUNT as f32,
        screen.top() + (size * 0.36 + 2.0).min(screen.height() * 0.5),
    )
}

fn flight_target(screen: Rect, index: usize, bat: &Bat, time: f32) -> Pos2 {
    if index.is_multiple_of(3) && (time + noise(bat.seed, 51) * 100.0).rem_euclid(53.0) < 9.0 {
        let castle = estate::castle_rect(screen);
        let center = castle.min + castle.size() * egui::vec2(0.45, 0.25);
        let angle = time * 1.4 + noise(bat.seed, 52) * std::f32::consts::TAU;
        return center + egui::vec2(angle.cos() * 75.0, angle.sin() * 28.0);
    }
    // Separate territories prevent scattered bats from gathering into one flock.
    let phase = time * 0.3 + noise(bat.seed, 11) * 6.0;
    let x = (index % 4) as f32 + 0.5 + phase.sin() * 0.3;
    let y = (index / 4) as f32 + 0.5 + (phase * 0.8).cos() * 0.3;
    screen.min + egui::vec2(screen.width() * x / 4.0, screen.height() * y / 3.0)
}

fn postpone_roost(bat: &mut Bat, time: f32) {
    bat.cycle = bat.cycle.wrapping_add(1);
    bat.return_at = time + 14.0 + noise(bat.seed, bat.cycle) * 24.0;
    bat.perched = false;
}

fn steer(bat: &mut Bat, target: Pos2, pointer: Option<Pos2>, pressed: bool, dt: f32, screen: Rect) {
    let mut desired = (target - bat.position).normalized()
        * (target.distance(bat.position) * 2.0).min(if bat.burst_seconds > 0.0 {
            320.0
        } else {
            65.0
        });
    if let Some(pointer) = pointer {
        let away = bat.position - pointer;
        let radius = if pressed {
            CLICK_REACTION_RADIUS
        } else {
            REACTION_RADIUS
        };
        if away.length() < radius {
            desired = away.normalized()
                * if bat.burst_seconds > 0.0 {
                    CLICK_ESCAPE_SPEED
                } else {
                    120.0
                };
            if away.length_sq() < 1.0 {
                desired = Vec2::X
                    * if bat.burst_seconds > 0.0 {
                        CLICK_ESCAPE_SPEED
                    } else {
                        120.0
                    };
            }
            bat.perched = false;
        }
    }
    bat.velocity += (desired - bat.velocity) * (dt * 5.0).min(1.0);
    if bat.velocity.x.abs() > 8.0 {
        bat.facing_left = bat.velocity.x < 0.0;
    }
    bat.position += bat.velocity * dt;
    let size = bat.size.max(48.0);
    let inset = egui::vec2(
        (size * 0.5).min(screen.width() * 0.5),
        (size * 0.36).min(screen.height() * 0.5),
    );
    let bounds = screen.shrink2(inset);
    bat.position.x = bat.position.x.clamp(bounds.left(), bounds.right());
    bat.position.y = bat.position.y.clamp(bounds.top(), bounds.bottom());
}

fn update(state: &mut Garden, screen: Rect, time: f32, pointer: Option<Pos2>, pressed: bool) {
    let dt = state
        .last_time
        .map_or(0.0, |previous| (time - previous).clamp(0.0, 0.05));
    state.last_time = Some(time);
    if state.bats.is_empty() {
        let seed = uuid::Uuid::new_v4().as_u128() as u32;
        state.bats = (0..BAT_COUNT)
            .map(|index| {
                let mut bat = Bat {
                    seed: seed.wrapping_add(index as u32 * 17),
                    ..Default::default()
                };
                bat.size = 40.0 + noise(bat.seed, 27).powf(1.4) * 44.0;
                bat.position = flight_target(screen, index, &bat, time);
                postpone_roost(&mut bat, time - 10.0);
                bat
            })
            .collect();
    }
    let positions: Vec<Pos2> = state.bats.iter().map(|bat| bat.position).collect();
    for (index, bat) in state.bats.iter_mut().enumerate() {
        bat.burst_seconds = (bat.burst_seconds - dt).max(0.0);
        if let Some(pointer) =
            pointer.filter(|pointer| pressed && pointer.distance(bat.position) < bat.size * 0.5)
        {
            let away = bat.position - pointer;
            let direction = if away.length_sq() > 1.0 {
                away.normalized()
            } else {
                Vec2::angled(noise(bat.seed, bat.cycle) * std::f32::consts::TAU)
            };
            let bounds = screen.shrink2(egui::vec2(
                (bat.size * 0.5).min(screen.width() * 0.5),
                (bat.size * 0.5).min(screen.height() * 0.5),
            ));
            bat.escape_target = bounds.clamp(bat.position + direction * screen.size().length());
            bat.burst_seconds = CLICK_ESCAPE_SECONDS;
            bat.velocity = direction * CLICK_ESCAPE_SPEED;
        }
        if bat.burst_seconds > 0.0 && bat.position.distance(bat.escape_target) < 35.0 {
            let x = if bat.position.x < screen.center().x {
                0.9
            } else {
                0.1
            };
            let y = 0.15 + noise(bat.seed, bat.cycle + 72) * 0.7;
            bat.escape_target = screen.min + screen.size() * egui::vec2(x, y);
        }
        let radius = if pressed {
            CLICK_REACTION_RADIUS
        } else {
            REACTION_RADIUS
        };
        let threatened = pointer.is_some_and(|pointer| pointer.distance(bat.position) < radius);
        if threatened {
            postpone_roost(bat, time);
        }
        if bat.perched && time >= bat.leave_at {
            postpone_roost(bat, time);
        }
        let roost = home(screen, index, bat.size);
        let returning = time >= bat.return_at;
        let mut target = if bat.burst_seconds > 0.0 {
            bat.escape_target
        } else if returning {
            roost
        } else {
            flight_target(screen, index, bat, time)
        };
        if returning && !threatened && bat.position.distance(roost) < 5.0 {
            if !bat.perched {
                bat.leave_at = time + 8.0 + noise(bat.seed, bat.cycle + 91) * 18.0;
            }
            bat.perched = true;
            bat.position = roost;
            bat.velocity = Vec2::ZERO;
            continue;
        }
        bat.perched = false;
        for (other, position) in positions.iter().enumerate() {
            let away = bat.position - *position;
            if !returning && other != index && away.length() < 65.0 {
                target += away.normalized() * (65.0 - away.length());
            }
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
    estate::paint(painter);
    let time = ctx.input(|input| input.time) as f32;
    paint_bats(painter);
    for index in 0..PETAL_COUNT {
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

pub fn paint_bats(painter: &egui::Painter) {
    let ctx = painter.ctx();
    let time = ctx.input(|input| input.time) as f32;
    let state: Garden = ctx.data_mut(|data| data.get_temp(state_id()).unwrap_or_default());
    for (index, bat) in state.bats.iter().enumerate() {
        let sprite = if bat.perched {
            Sprite::Perched
        } else {
            Sprite::BatDown
        };
        let rect = Rect::from_center_size(bat.position, Vec2::splat(bat.size));
        let pose = SpritePose {
            mirrored: bat.facing_left,
            flap: if bat.perched {
                0.0
            } else {
                (time * 12.0 + index as f32).sin()
            },
            angle: if bat.perched {
                0.0
            } else {
                (bat.velocity.y * 0.002).clamp(-0.12, 0.12)
            },
        };
        draw_moving(painter, rect, sprite, Color32::WHITE, SpritePose { ..pose });
        let (eyes, y) = if bat.perched {
            ([58.5, 67.5], 92.5)
        } else {
            ([65.0, 71.0], 54.5)
        };
        for x in eyes {
            let u = if bat.facing_left {
                1.0 - x / 128.0
            } else {
                x / 128.0
            };
            let center = sprite_point(rect, u, y / 128.0, &pose);
            painter.rect_filled(
                Rect::from_center_size(center, Vec2::splat(1.5)),
                0,
                Color32::from_rgb(137, 24, 43),
            );
        }
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

pub fn paint_overlays(ctx: &egui::Context, _palette: &Palette) {
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
    // Editing and resize cursors retain their native shapes.
    let cursor = ctx.output(|output| output.cursor_icon);
    let pointer = ctx.input(|input| input.pointer.hover_pos());
    if cursors::paint(&painter, pointer, cursor) {
        ctx.set_cursor_icon(egui::CursorIcon::None);
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
            facing_left: false,
            ..Default::default()
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
    fn bats_react_at_expanded_hover_and_click_distances() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(500.0));
        for (distance, pressed) in [(160.0, false), (280.0, true)] {
            let mut bat = Bat {
                position: egui::pos2(distance, 20.0),
                velocity: Vec2::ZERO,
                perched: true,
                facing_left: false,
                ..Default::default()
            };
            steer(
                &mut bat,
                Pos2::ZERO,
                Some(egui::pos2(0.0, 20.0)),
                pressed,
                0.05,
                screen,
            );
            assert!(bat.velocity.x > 0.0);
            assert!(!bat.perched);
        }
    }

    #[test]
    fn bat_facing_follows_direction_and_wing_motion_keeps_body_fixed() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(300.0));
        let mut bat = Bat {
            position: screen.center(),
            velocity: Vec2::ZERO,
            perched: false,
            facing_left: false,
            ..Default::default()
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
    fn direct_click_gives_a_faster_longer_escape_then_resumes_roaming() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1400.0, 900.0));
        let mut clicked = Garden::default();
        update(&mut clicked, screen, 0.0, None, false);
        clicked.bats[0].position = screen.center();
        let mut hovered = clicked.clone();
        let pointer = Some(screen.center() - Vec2::X * 4.0);
        update(&mut clicked, screen, 0.05, pointer, true);
        update(&mut hovered, screen, 0.05, pointer, false);
        assert!(clicked.bats[0].velocity.length() > hovered.bats[0].velocity.length() * 2.0);
        for step in 2..=20 {
            update(&mut clicked, screen, step as f32 * 0.05, None, false);
        }
        assert!(clicked.bats[0].position.distance(screen.center()) > 150.0);
        for step in 21..=80 {
            update(&mut clicked, screen, step as f32 * 0.05, None, false);
        }
        assert_eq!(clicked.bats[0].burst_seconds, 0.0);
        assert!(screen.contains(clicked.bats[0].position));
    }

    #[test]
    fn bats_roost_scatter_spread_and_return_individually() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
        let mut state = Garden::default();
        update(&mut state, screen, 0.0, None, false);
        assert!(state
            .bats
            .windows(2)
            .any(|pair| pair[0].return_at != pair[1].return_at));
        for bat in &mut state.bats {
            bat.return_at = 0.0;
        }
        for step in 1..=600 {
            update(&mut state, screen, step as f32 * 0.05, None, false);
        }
        // Some bats leave earlier than others; every bat has reached its own roost.
        assert!(state.bats.iter().all(|bat| bat.leave_at > 0.0));
        let pointer = state.bats[0].position;
        update(&mut state, screen, 30.05, Some(pointer), false);
        assert!(!state.bats[0].perched);
        assert!(state.bats[0].return_at >= 44.05);
        for step in 602..=780 {
            update(&mut state, screen, step as f32 * 0.05, None, false);
        }
        assert!(state.bats.iter().all(|bat| screen.contains(bat.position)));
        assert!(state.bats[0].position.distance(pointer) > 50.0);
        for step in 781..=2200 {
            update(&mut state, screen, step as f32 * 0.05, None, false);
        }
        assert!(state.bats[0].leave_at > 44.05);
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
