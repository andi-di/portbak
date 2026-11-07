//! Aero frosted glass: what lies behind a pane, blurred and brightened like Vista's glass, with
//! the pane's rim bending the view inward like a thick bevelled edge.

use crate::app::rendering::shaders::{Effect, Recipe};
use eframe::egui::{self, Rect};

/// How far the blur reaches, in points.
const BLUR: f32 = 10.0;
/// How wide the bending bevel along the rim is, in points.
const BEVEL: f32 = 9.0;

static FROST: Recipe = Recipe {
    name: "frost",
    fragment: FRAGMENT_SHADER,
};

/// A shape that frosts everything painted before it under the rounded `rect`; paint it first
/// in a pane so the pane's tint and reflections lie on top.
pub fn backdrop(ctx: &egui::Context, rect: Rect, radius: f32) -> egui::Shape {
    Effect::of(ctx, &FROST).shape(rect, ctx.content_rect(), move |target| {
        // The bottom-left and top-right corners, since the copy's origin is its bottom left.
        let [left, bottom] = target.pixel(rect.left_bottom());
        let [right, top] = target.pixel(rect.right_top());
        target.set("u_pane", &[left, bottom, right, top]);
        target.set(
            "u_shape",
            &[
                target.pixels(radius),
                target.pixels(BLUR),
                target.pixels(BEVEL),
            ],
        );
    })
}

/// Blurs with taps spiralling out, each averaging four pixels through linear filtering, bends
/// the view near the rim, lifts the colors a little and keeps only the pane's rounded shape,
/// softly antialiased.
const FRAGMENT_SHADER: &str = r#"
// Pane corners in pixels: left, bottom, right, top.
uniform vec4 u_pane;
// Corner radius, blur reach and bevel width, in pixels.
uniform vec3 u_shape;

// Signed distance to the rounded pane outline: negative inside.
float outline(vec2 pixel, vec2 center, vec2 half_size) {
    float radius = min(u_shape.x, min(half_size.x, half_size.y));
    vec2 corner = abs(pixel - center) - (half_size - radius);
    return length(max(corner, 0.0)) + min(max(corner.x, corner.y), 0.0) - radius;
}

void main() {
    vec2 pixel = v_uv * u_size;
    vec2 center = (u_pane.xy + u_pane.zw) * 0.5;
    vec2 half_size = (u_pane.zw - u_pane.xy) * 0.5;
    float distance_in = -outline(pixel, center, half_size);
    float mask = clamp(distance_in + 0.5, 0.0, 1.0);
    if (mask <= 0.0) {
        discard;
    }
    // Thick glass: near the rim the view is pulled in from further inside.
    float rim = clamp(1.0 - distance_in / u_shape.z, 0.0, 1.0);
    vec2 inward = normalize(center - pixel + vec2(0.0001));
    vec2 look = pixel + inward * rim * rim * u_shape.z * 0.8;

    vec3 sum = vec3(0.0);
    const int TAPS = 32;
    for (int tap = 0; tap < TAPS; tap++) {
        float step_out = (float(tap) + 0.5) / float(TAPS);
        float angle = float(tap) * 2.39996;
        vec2 offset = vec2(cos(angle), sin(angle)) * u_shape.y * sqrt(step_out);
        sum += frame(look + offset).rgb;
    }
    vec3 color = sum / float(TAPS);
    // Vista glass glows a little: slightly richer colors, lifted shadows.
    float grey = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(grey), color, 1.2);
    color = color * 0.9 + 0.1;
    out_color = vec4(clamp(color, 0.0, 1.0) * mask, mask);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backdrop_covers_the_pane_on_screen() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let ctx = &ui.ctx().clone();
            let pane = Rect::from_min_size(egui::pos2(20.0, 30.0), egui::vec2(200.0, 100.0));
            let egui::Shape::Callback(callback) = backdrop(ctx, pane, 12.0) else {
                panic!("the backdrop is a paint callback");
            };
            assert_eq!(callback.rect, pane);
        });
    }
}
