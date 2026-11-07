//! A refraction shockwave for Aero clicks: an expanding lens ring that bends whatever lies under
//! it and splits its colors at the edge.

use super::shaders::{Effect, Recipe};
use eframe::egui::{self, Pos2, Rect, Vec2};

/// How long one shockwave lasts.
pub const SECONDS: f32 = 0.9;
/// How far the ring travels, in points.
const REACH: f32 = 110.0;

static SHOCKWAVE: Recipe = Recipe {
    name: "shockwave",
    fragment: FRAGMENT_SHADER,
};

/// The ring at one moment, in points.
#[derive(Debug, PartialEq)]
struct Wave {
    radius: f32,
    /// Half the width of the bending band.
    band: f32,
    /// How far the band bends what is under it.
    strength: f32,
}

/// The ring `age` seconds after the click: bursting out fast, then slowing, widening and fading.
fn wave(age: f32) -> Option<Wave> {
    let t = age / SECONDS;
    if !(0.0..1.0).contains(&t) {
        return None;
    }
    let ease = 1.0 - (1.0 - t).powi(3);
    Some(Wave {
        radius: 4.0 + ease * REACH,
        band: 12.0 + 20.0 * t,
        strength: 16.0 * (1.0 - t).powf(1.5),
    })
}

/// False once the GPU turned the shader down.
pub fn available(ctx: &egui::Context) -> bool {
    Effect::of(ctx, &SHOCKWAVE).available()
}

/// A shape that refracts everything painted before it around `center`, `age` seconds after the
/// click; nothing once the wave has faded.
pub fn shockwave(ctx: &egui::Context, center: Pos2, age: f32) -> egui::Shape {
    let Some(wave) = wave(age) else {
        return egui::Shape::Noop;
    };
    let reach = wave.radius + wave.band + wave.strength + 2.0;
    let rect = Rect::from_center_size(center, Vec2::splat(reach * 2.0));
    Effect::of(ctx, &SHOCKWAVE).shape(rect, ctx.content_rect(), move |target| {
        target.set("u_center", &target.pixel(center));
        target.set(
            "u_ring",
            &[
                target.pixels(wave.radius),
                target.pixels(wave.band),
                target.pixels(wave.strength),
            ],
        );
    })
}

/// Bends the frame across a ring: pixels inside the band are sampled from further in or out,
/// like light through a lens, with red and blue pulled apart and a faint glint on the crest.
const FRAGMENT_SHADER: &str = r#"
uniform vec2 u_center;
// Radius, half band width and bend strength, in pixels.
uniform vec3 u_ring;

void main() {
    vec2 pixel = v_uv * u_size;
    vec2 offset = pixel - u_center;
    float dist = length(offset);
    vec2 dir = dist > 0.001 ? offset / dist : vec2(0.0);
    float across = (dist - u_ring.x) / u_ring.y;
    float wave = 0.0;
    if (abs(across) < 1.0) {
        wave = sin(across * 3.14159265) * (1.0 - across * across);
    }
    vec2 bend = -dir * wave * u_ring.z;
    vec2 split = dir * abs(wave) * u_ring.z * 0.2;
    float red = frame(pixel + bend + split).r;
    vec4 middle = frame(pixel + bend);
    float blue = frame(pixel + bend - split).b;
    float glint = max(wave, 0.0) * clamp(u_ring.z / 16.0, 0.0, 1.0) * 0.18;
    out_color = vec4(vec3(red, middle.g, blue) + vec3(0.6, 0.9, 1.0) * glint, 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_grows_widens_and_fades_out() {
        let early = wave(0.05).unwrap();
        let late = wave(SECONDS * 0.9).unwrap();
        assert!(late.radius > early.radius && late.band > early.band);
        assert!(late.strength < early.strength);
        assert!(late.radius <= REACH + 4.0);
        assert_eq!(wave(SECONDS), None);
        assert_eq!(wave(-0.1), None);
    }

    #[test]
    fn shockwaves_vanish_when_done() {
        let ctx = egui::Context::default();
        assert!(matches!(
            shockwave(&ctx, Pos2::ZERO, 0.3),
            egui::Shape::Callback(_)
        ));
        assert!(matches!(
            shockwave(&ctx, Pos2::ZERO, SECONDS),
            egui::Shape::Noop
        ));
    }
}
