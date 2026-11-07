//! Frutiger Aero bubbles: iridescent soap bubbles rising and wobbling behind and in front of the
//! panels. Small ones shy away from the pointer and big ones pop when it touches them; sparkles
//! twinkle in between.

use super::aquarium;
use super::glass;
use super::ornaments::noise;
use super::pixel::mix;
use super::style::Palette;
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Stroke, Vec2};
use std::collections::HashMap;
use std::f32::consts::TAU;
use std::time::Duration;

/// egui shortens repaint delays by the predicted frame time, so this animates at about 30 fps.
const FRAME: Duration = Duration::from_millis(50);
const BUBBLE_COUNT: u32 = 40;
const SPARKLE_COUNT: u32 = 16;
/// Bubbles smaller than this dodge the pointer; bigger ones pop.
const DODGE_RADIUS: f32 = 11.0;
const DODGE_REACH: f32 = 70.0;
const POP_SECONDS: f32 = 0.45;
const RIM_SEGMENTS: u32 = 40;
const FRONT_EVERY: u32 = 4;

/// When each popped bubble burst, keyed by bubble and by which trip up the window it was on.
#[derive(Clone, Default)]
struct Pops(HashMap<(u32, u32), f32>);

/// A bubble behind the panels that a fish can chase and gulp.
pub struct Prey {
    /// The bubble and its trip up the window, as recorded when it pops.
    pub key: (u32, u32),
    pub center: Pos2,
    pub radius: f32,
}

/// A bubble's place on one frame.
struct Bubble {
    center: Pos2,
    radius: f32,
    /// Counts the bubble's trips up the window, so a popped bubble comes back on its next trip.
    trip: u32,
    /// Fades the bubble in at the bottom and out at the top.
    strength: f32,
    tint: Color32,
}

/// Paints the bubbles and sparkles and keeps the window repainting so they move smoothly.
pub fn paint(ctx: &egui::Context, palette: &Palette) {
    let behind = ctx.layer_painter(egui::LayerId::background());
    // Every few bubbles drift in front of the panels; painted only, so clicks pass through.
    let front = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("aero_bubbles_front"),
    ));
    let screen = ctx.content_rect();
    let time = ctx.input(|input| input.time) as f32;
    let pointer = ctx.input(|input| input.pointer.hover_pos());
    let id = pops_id();
    let mut pops: Pops = ctx.data_mut(|data| data.get_temp(id).unwrap_or_default());
    pops.0.retain(|_, popped| time - *popped < 60.0);
    let wakes = aquarium::wakes(ctx);
    for index in 0..BUBBLE_COUNT {
        let painter = if index % FRONT_EVERY == 0 {
            &front
        } else {
            &behind
        };
        let mut bubble = place(index, screen, time, palette);
        if let Some(popped) = pops.0.get(&(index, bubble.trip)) {
            pop(painter, &bubble, time - popped);
            continue;
        }
        if let Some(pointer) = pointer {
            let distance = pointer.distance(bubble.center);
            if bubble.radius >= DODGE_RADIUS && distance < bubble.radius {
                pops.0.insert((index, bubble.trip), time);
                continue;
            }
            if bubble.radius < DODGE_RADIUS && distance < DODGE_REACH {
                let away = (bubble.center - pointer).normalized();
                bubble.center += away * (DODGE_REACH - distance) * 0.6;
            }
        }
        if bubble.radius < DODGE_RADIUS && index % FRONT_EVERY != 0 {
            make_way(&mut bubble, &wakes);
        }
        paint_bubble(painter, &bubble, time, index);
    }
    sparkles(&behind, screen, palette, time);
    ctx.data_mut(|data| data.insert_temp(id, pops));
    ctx.request_repaint_after(FRAME);
}

/// Big bubbles behind the panels that are still whole, for the fish to hunt.
pub fn prey(ctx: &egui::Context, palette: &Palette) -> Vec<Prey> {
    let screen = ctx.content_rect();
    let time = ctx.input(|input| input.time) as f32;
    let pops: Pops = ctx.data_mut(|data| data.get_temp(pops_id()).unwrap_or_default());
    (0..BUBBLE_COUNT)
        .filter(|index| index % FRONT_EVERY != 0)
        .map(|index| (index, place(index, screen, time, palette)))
        .filter(|(index, bubble)| {
            bubble.radius >= DODGE_RADIUS
                && bubble.strength > 0.5
                && !pops.0.contains_key(&(*index, bubble.trip))
        })
        .map(|(index, bubble)| Prey {
            key: (index, bubble.trip),
            center: bubble.center,
            radius: bubble.radius,
        })
        .collect()
}

/// Pops the bubble with `key` now, e.g. when a fish gulps it.
pub fn burst(ctx: &egui::Context, key: (u32, u32), time: f32) {
    ctx.data_mut(|data| {
        let pops: &mut Pops = data.get_temp_mut_or_default(pops_id());
        pops.0.insert(key, time);
    });
}

fn pops_id() -> egui::Id {
    egui::Id::new("aero_bubble_pops")
}

/// Nudges a small bubble out of the way of any fish swimming through it.
fn make_way(bubble: &mut Bubble, wakes: &[aquarium::Wake]) {
    for wake in wakes {
        let offset = bubble.center - wake.center;
        let reach = wake.reach + bubble.radius + 20.0;
        let distance = offset.length();
        if distance < reach && distance > 0.01 {
            bubble.center += offset / distance * (reach - distance) * 0.8;
        }
    }
}

fn place(index: u32, screen: Rect, time: f32, palette: &Palette) -> Bubble {
    let random = |salt: u32| noise(index, salt);
    let radius = 4.0 + random(1).powi(3) * 46.0;
    let travel = screen.height() + radius * 2.0 + 40.0;
    let speed = 14.0 + random(2) * 30.0 - radius * 0.15;
    let journey = time * speed / travel + random(3);
    let progress = journey.fract();
    let sway = (time * (0.3 + random(4) * 0.6) + random(5) * TAU).sin() * (6.0 + random(6) * 26.0);
    let center = egui::pos2(
        screen.left() + random(0) * screen.width() + sway,
        screen.bottom() + radius - progress * travel,
    );
    let strength = (progress * 5.0).min(1.0) * ((1.0 - progress) * 4.0).min(1.0);
    let tint = if random(7) > 0.5 {
        palette.hot
    } else {
        palette.accent
    };
    Bubble {
        center,
        radius,
        trip: journey.floor() as u32,
        strength,
        tint,
    }
}

/// A soap bubble: a clear body, a rainbow-sheened rim, a curved window reflection and glints.
fn paint_bubble(painter: &egui::Painter, bubble: &Bubble, time: f32, index: u32) {
    let Bubble {
        center,
        radius,
        strength,
        tint,
        ..
    } = *bubble;
    // A gentle wobble, squashing one way then the other.
    let wobble = (time * 3.0 + index as f32).sin() * 0.04;
    let radii = Vec2::new(1.0 + wobble, 1.0 - wobble) * radius;
    painter.add(glass::radial_glow(
        center,
        radius,
        mix(tint, Color32::WHITE, 0.6).gamma_multiply(0.12 * strength),
    ));
    painter.add(rim(center, radii, time + index as f32, strength));
    let reflection = glass::ellipse(
        center + Vec2::new(-0.35, -0.42) * radius,
        Vec2::new(0.38, 0.18) * radius,
        -0.6,
    );
    painter.add(Shape::convex_polygon(
        reflection,
        Color32::WHITE.gamma_multiply(0.75 * strength),
        Stroke::NONE,
    ));
    painter.circle_filled(
        center + Vec2::new(0.5, 0.3) * radius,
        radius * 0.08,
        Color32::WHITE.gamma_multiply(0.6 * strength),
    );
    if radius > 18.0 {
        // Light caught on the far inside wall, a faint crescent along the bottom.
        let arc: Vec<Pos2> = (0..=16)
            .map(|step| {
                let angle = 0.35 + step as f32 / 16.0 * 2.4;
                center + Vec2::angled(angle) * radius * 0.78
            })
            .collect();
        painter.add(Shape::line(
            arc,
            Stroke::new(radius * 0.06, Color32::WHITE.gamma_multiply(0.3 * strength)),
        ));
    }
}

/// The bubble's edge, shimmering through aqua, white, pink and gold as the film thins.
fn rim(center: Pos2, radii: Vec2, phase: f32, strength: f32) -> Shape {
    const FILM: [Color32; 4] = [
        Color32::from_rgb(0x40, 0xf0, 0xff),
        Color32::from_rgb(0xff, 0xff, 0xff),
        Color32::from_rgb(0xff, 0x7a, 0xe0),
        Color32::from_rgb(0xff, 0xe0, 0x50),
    ];
    let mut mesh = Mesh::default();
    for segment in 0..RIM_SEGMENTS {
        let angle = segment as f32 * TAU / RIM_SEGMENTS as f32;
        let hue = (angle / TAU + phase * 0.05).fract() * FILM.len() as f32;
        let from = FILM[hue as usize % FILM.len()];
        let to = FILM[(hue as usize + 1) % FILM.len()];
        let color = mix(from, to, hue.fract()).gamma_multiply(0.7 * strength);
        let direction = Vec2::angled(angle);
        let thickness = (radii.x * 0.18).clamp(1.5, 7.0);
        let outer = direction * radii;
        let inner = outer - direction * thickness;
        mesh.colored_vertex(center + inner, Color32::TRANSPARENT);
        mesh.colored_vertex(center + outer, color);
    }
    for segment in 0..RIM_SEGMENTS {
        let (a, b) = (segment * 2, (segment + 1) % RIM_SEGMENTS * 2);
        mesh.add_triangle(a, a + 1, b + 1);
        mesh.add_triangle(a, b + 1, b);
    }
    mesh.into()
}

/// A burst: a ring flying apart and droplets scattering from where the bubble was.
fn pop(painter: &egui::Painter, bubble: &Bubble, age: f32) {
    if age > POP_SECONDS {
        return;
    }
    let t = age / POP_SECONDS;
    let fade = (1.0 - t) * bubble.strength;
    painter.circle_stroke(
        bubble.center,
        bubble.radius * (1.0 + t * 0.6),
        Stroke::new(2.0_f32, Color32::WHITE.gamma_multiply(fade)),
    );
    for drop in 0..8 {
        let direction = Vec2::angled(drop as f32 * TAU / 8.0 + 0.3);
        let at = bubble.center + direction * bubble.radius * (0.9 + t * 1.2);
        painter.circle_filled(
            at,
            2.5 * (1.0 - t) + 0.5,
            mix(bubble.tint, Color32::WHITE, 0.5).gamma_multiply(fade),
        );
    }
}

/// Little stars of light flaring up and fading here and there.
fn sparkles(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    for index in 0..SPARKLE_COUNT {
        let random = |salt: u32| noise(index, salt + 500);
        let period = 2.5 + random(0) * 4.0;
        let cycle = ((time + random(1) * period) / period).floor() as u32;
        let phase = ((time + random(1) * period) / period).fract();
        let flare = (1.0 - (phase * 4.0 - 1.0).abs()).max(0.0);
        if flare <= 0.0 {
            continue;
        }
        // Each flare lights up somewhere new.
        let spot = |salt: u32| noise(index * 977 + cycle, salt + 600);
        let at = egui::pos2(
            screen.left() + spot(0) * screen.width(),
            screen.top() + spot(1) * screen.height() * 0.9,
        );
        let color = if spot(2) > 0.5 {
            palette.hot
        } else {
            palette.gold
        };
        glass::sparkle(painter, at, (8.0 + random(2) * 12.0) * flare, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::skins;

    #[test]
    fn bubbles_rise_through_the_window_and_fade_at_its_edges() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let palette = &skins::default().light;
        for index in 0..BUBBLE_COUNT {
            let bubble = place(index, screen, 12.5, palette);
            assert!((0.0..=1.0).contains(&bubble.strength));
            assert!(bubble.center.y <= screen.bottom() + bubble.radius);
            assert!(bubble.center.y >= screen.top() - bubble.radius - 40.0);
        }
    }

    #[test]
    fn touching_a_big_bubble_pops_it_until_its_next_trip() {
        let context = egui::Context::default();
        let palette = &skins::default().light;
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let time = 5.0_f32;
        let (index, bubble) = (0..BUBBLE_COUNT)
            .map(|index| (index, place(index, screen, time, palette)))
            .find(|(_, bubble)| bubble.radius >= DODGE_RADIUS && screen.contains(bubble.center))
            .expect("some big bubble is on screen");
        let input = egui::RawInput {
            screen_rect: Some(screen),
            time: Some(time as f64),
            events: vec![egui::Event::PointerMoved(bubble.center)],
            ..Default::default()
        };
        let _ = context.run_ui(input, |ui| paint(ui.ctx(), palette));
        let pops: Pops = context
            .data_mut(|data| data.get_temp(egui::Id::new("aero_bubble_pops")))
            .unwrap();
        assert!(pops.0.contains_key(&(index, bubble.trip)));
    }
}
