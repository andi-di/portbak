//! Frutiger Aero pointer effects: every click sends a refracting shockwave through the window
//! (plain water rings where the GPU can't) and the moving pointer leaves a trail of tiny rising
//! bubbles. Painted above everything, so input passes through.

use super::ornaments::noise;
use super::pixel::mix;
use super::refraction;
use super::style::Palette;
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::f32::consts::TAU;

const RIPPLE_SECONDS: f32 = refraction::SECONDS;
const RIPPLE_RINGS: u32 = 3;
const RIPPLE_REACH: f32 = 46.0;
/// Pointer travel between two puffs of trail bubbles, picked at random in this range.
const TRAIL_SPACING: (f32, f32) = (8.0, 30.0);
const TRAIL_LIMIT: usize = 60;
/// Share of the pointer's speed a new bubble carries along before water drag slows it.
const TRAIL_CARRY: f32 = 0.12;
const TRAIL_DRAG: f32 = 3.0;
const FILM: [Color32; 3] = [
    Color32::from_rgb(0x40, 0xf0, 0xff),
    Color32::from_rgb(0xff, 0x8a, 0xe8),
    Color32::from_rgb(0xff, 0xe8, 0x70),
];

/// Recent clicks and trail bubbles.
#[derive(Clone, Default)]
struct Splashes {
    ripples: Vec<(Pos2, f32)>,
    trail: Vec<TrailBubble>,
    last_trail: Option<Pos2>,
    /// Pointer travel still needed before the next puff.
    next_spacing: f32,
    /// Bubbles released so far; seeds each new bubble's randomness.
    released: u32,
}

/// One bubble of the pointer trail, with its own size, lifetime and way of rising.
#[derive(Clone)]
struct TrailBubble {
    origin: Pos2,
    born: f32,
    seed: u32,
    /// Velocity inherited from the pointer, in points per second.
    carry: Vec2,
}

impl TrailBubble {
    fn random(&self, salt: u32) -> f32 {
        noise(self.seed, salt + 700)
    }

    /// Mostly tiny bubbles with the odd bigger one.
    fn radius(&self) -> f32 {
        1.2 + self.random(0).powi(2) * 5.5
    }

    fn lifetime(&self) -> f32 {
        0.8 + self.random(1) * 1.2
    }

    /// Where the bubble is `age` seconds after release: pushed along by the pointer until drag
    /// stops it, rising ever faster (bigger bubbles are more buoyant), wobbling and drifting.
    fn position(&self, age: f32) -> Pos2 {
        let glide = (1.0 - (-age * TRAIL_DRAG).exp()) / TRAIL_DRAG;
        let buoyancy = 25.0 + self.radius() * 9.0 + self.random(2) * 20.0;
        let start_speed = 6.0 + self.random(3) * 14.0;
        let rise = start_speed * age + 0.5 * buoyancy * age * age;
        let drift = (self.random(4) - 0.5) * 24.0 * age;
        // The wobble builds up over the first moments instead of jerking sideways at once.
        let wobble = (age * (3.0 + self.random(5) * 6.0) + self.random(6) * TAU).sin()
            * (1.0 + self.random(7) * 4.0)
            * (age / 0.3).min(1.0);
        self.origin + self.carry * glide + Vec2::new(drift + wobble, -rise)
    }
}

pub fn paint(ctx: &egui::Context, palette: &Palette) {
    let id = egui::Id::new("aero_splashes");
    let time = ctx.input(|input| input.time) as f32;
    let mut state: Splashes = ctx.data_mut(|data| data.get_temp(id).unwrap_or_default());
    record(ctx, &mut state, time);
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, id));
    for (center, born) in &state.ripples {
        if refraction::available(ctx) {
            painter.add(refraction::shockwave(ctx, *center, time - born));
        } else {
            ripple(&painter, *center, time - born, palette);
        }
    }
    for bubble in &state.trail {
        trail_bubble(&painter, bubble, time - bubble.born, palette);
    }
    ctx.data_mut(|data| data.insert_temp(id, state));
}

/// Adds this frame's clicks and pointer travel and forgets splashes that have faded.
fn record(ctx: &egui::Context, state: &mut Splashes, time: f32) {
    let (clicked, pointer, velocity) = ctx.input(|input| {
        (
            input.pointer.any_pressed(),
            input.pointer.hover_pos(),
            input.pointer.velocity(),
        )
    });
    state
        .ripples
        .retain(|(_, born)| time - born < RIPPLE_SECONDS);
    state
        .trail
        .retain(|bubble| time - bubble.born < bubble.lifetime());
    let Some(pointer) = pointer else {
        state.last_trail = None;
        return;
    };
    if clicked {
        state.ripples.push((pointer, time));
    }
    let moved = state
        .last_trail
        .is_none_or(|last| last.distance(pointer) > state.next_spacing);
    if moved {
        state.last_trail = Some(pointer);
        release_puff(state, pointer, velocity, time);
    }
}

/// Releases one to three bubbles scattered around `pointer` and picks the next spacing.
fn release_puff(state: &mut Splashes, pointer: Pos2, velocity: Vec2, time: f32) {
    let puff = state.released;
    let random = |salt: u32| noise(puff, salt + 800);
    let count = 1 + (random(0) * 2.6) as u32;
    for _ in 0..count {
        if state.trail.len() >= TRAIL_LIMIT {
            break;
        }
        let seed = state.released;
        state.released = state.released.wrapping_add(1);
        let scatter = Vec2::angled(noise(seed, 810) * TAU) * noise(seed, 811) * 9.0;
        state.trail.push(TrailBubble {
            origin: pointer + scatter,
            // Staggered a little so a puff doesn't appear all at once.
            born: time + noise(seed, 813) * 0.08,
            seed,
            carry: velocity * TRAIL_CARRY * (0.5 + noise(seed, 812)),
        });
    }
    let (low, high) = TRAIL_SPACING;
    state.next_spacing = low + random(1) * (high - low);
}

/// Rings spreading out from a click like a drop landing on water, where refraction is unavailable.
fn ripple(painter: &egui::Painter, center: Pos2, age: f32, palette: &Palette) {
    for ring in 0..RIPPLE_RINGS {
        let t = age / RIPPLE_SECONDS - ring as f32 * 0.18;
        if !(0.0..1.0).contains(&t) {
            continue;
        }
        let fade = 1.0 - t;
        let radius = 4.0 + t * RIPPLE_REACH;
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(3.0 * fade, palette.hot.gamma_multiply(0.7 * fade)),
        );
        painter.circle_stroke(
            center,
            radius - 2.0,
            Stroke::new(1.0_f32, Color32::WHITE.gamma_multiply(0.8 * fade)),
        );
    }
}

/// A trail bubble: a faint body and a rim in its own film color with a glint, swelling a little
/// as it rises and fading out at the end of its life.
fn trail_bubble(painter: &egui::Painter, bubble: &TrailBubble, age: f32, palette: &Palette) {
    if age < 0.0 {
        return;
    }
    let t = age / bubble.lifetime();
    let fade = ((1.0 - t) * 3.0).min(1.0) * (age * 12.0).min(1.0);
    let center = bubble.position(age);
    let radius = bubble.radius() * (1.0 + t * 0.35);
    let film = FILM[bubble.seed as usize % FILM.len()];
    let tint = mix(palette.hot, film, 0.5 * bubble.random(8));
    painter.circle_filled(center, radius, tint.gamma_multiply(0.18 * fade));
    // Light rims vanish against a bright sky, so by day they take on the deeper accent blue.
    let edge = if painter.ctx().global_style().visuals.dark_mode {
        mix(tint, Color32::WHITE, 0.35)
    } else {
        mix(tint, palette.accent, 0.55)
    };
    let rim = edge.gamma_multiply(0.9 * fade);
    let width = (radius * 0.25).clamp(0.8, 1.4);
    painter.circle_stroke(center, radius, Stroke::new(width, rim));
    painter.circle_filled(
        center + Vec2::splat(-radius * 0.38),
        (radius * 0.28).max(0.6),
        Color32::WHITE.gamma_multiply(0.9 * fade),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::skins;

    #[test]
    fn clicks_ripple_and_moves_leave_a_fading_trail() {
        let context = egui::Context::default();
        let palette = &skins::default().light;
        let frame = |time: f64, events: Vec<egui::Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let _ = context.run_ui(input, |ui| paint(ui.ctx(), palette));
            context.data_mut(|data| data.get_temp::<Splashes>(egui::Id::new("aero_splashes")))
        };
        let at = egui::pos2(100.0, 100.0);
        frame(0.0, vec![egui::Event::PointerMoved(at)]);
        let state = frame(
            0.1,
            vec![
                egui::Event::PointerMoved(at + Vec2::new(40.0, 0.0)),
                egui::Event::PointerButton {
                    pos: at + Vec2::new(40.0, 0.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .unwrap();
        assert_eq!(state.ripples.len(), 1);
        assert!((2..=6).contains(&state.trail.len()));
        let state = frame(5.0, vec![]).unwrap();
        assert!(state.ripples.is_empty() && state.trail.is_empty());
    }

    #[test]
    fn trail_bubbles_differ_and_rise_faster_and_faster() {
        let bubbles: Vec<TrailBubble> = (0..40)
            .map(|seed| TrailBubble {
                origin: Pos2::ZERO,
                born: 0.0,
                seed,
                carry: Vec2::new(200.0, 0.0),
            })
            .collect();
        let sizes: Vec<f32> = bubbles.iter().map(TrailBubble::radius).collect();
        assert!(sizes.iter().any(|size| *size < 2.5) && sizes.iter().any(|size| *size > 4.0));
        for bubble in &bubbles {
            let heights = [0.0, 0.3, 0.6, 0.9].map(|age| bubble.position(age).y);
            let climbs: Vec<f32> = heights.windows(2).map(|pair| pair[0] - pair[1]).collect();
            assert!(
                climbs.windows(2).all(|pair| pair[1] > pair[0]),
                "{heights:?}"
            );
            // The pointer's push dies away instead of carrying the bubble off.
            let late = bubble.position(5.0).x - bubble.position(4.0).x;
            assert!(late.abs() < 30.0, "{late}");
        }
    }
}
