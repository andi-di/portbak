//! Liquid Glass wallpaper: soft fields of color drifting under luminous ribbons, in the manner of
//! iOS 26 wallpapers. The ribbons' crisp edges give the glass something to visibly bend.

use crate::app::rendering::glass;
use crate::app::rendering::pixel::shaded_polygon;
use crate::app::rendering::shaders::{Effect, Recipe};
use eframe::egui::{self, Color32, Pos2, Rect};
use std::time::Duration;

/// egui shortens repaint delays by the predicted frame time, so this animates at about 25 fps.
const FRAME: Duration = Duration::from_millis(40);

static WALLPAPER: Recipe = Recipe {
    name: "liquid_wallpaper",
    fragment: FRAGMENT_SHADER,
};

/// The colors of one wallpaper: a vertical base gradient, five drifting fields and the ribbons.
struct Scheme {
    top: Color32,
    bottom: Color32,
    fields: [Color32; 5],
    ribbon: Color32,
}

const DAY: Scheme = Scheme {
    top: Color32::from_rgb(0xd6, 0xe6, 0xff),
    bottom: Color32::from_rgb(0xf8, 0xe8, 0xf6),
    fields: [
        Color32::from_rgb(0x4f, 0xa3, 0xff),
        Color32::from_rgb(0xa9, 0x8a, 0xff),
        Color32::from_rgb(0xff, 0x9c, 0xc8),
        Color32::from_rgb(0xff, 0xc4, 0x7e),
        Color32::from_rgb(0x6a, 0xe2, 0xd6),
    ],
    ribbon: Color32::from_rgb(0xff, 0xff, 0xff),
};

const NIGHT: Scheme = Scheme {
    top: Color32::from_rgb(0x03, 0x05, 0x14),
    bottom: Color32::from_rgb(0x0e, 0x04, 0x1c),
    fields: [
        Color32::from_rgb(0x12, 0x46, 0xe0),
        Color32::from_rgb(0x6a, 0x1e, 0xd8),
        Color32::from_rgb(0xd0, 0x1c, 0x78),
        Color32::from_rgb(0xe0, 0x62, 0x10),
        Color32::from_rgb(0x00, 0x9c, 0xd8),
    ],
    ribbon: Color32::from_rgb(0x9c, 0xc8, 0xff),
};

/// The wallpaper behind every panel; paint it before any panel so they cover it.
pub fn paint(ctx: &egui::Context) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen = ctx.content_rect();
    let night = ctx.global_style().visuals.dark_mode;
    let scheme = if night { &NIGHT } else { &DAY };
    let effect = Effect::of(ctx, &WALLPAPER);
    if effect.available() {
        let time = ctx.input(|input| input.time) as f32;
        painter.add(effect.shape(screen, screen, move |target| {
            target.set("u_time", &[time]);
            target.set("u_night", &[if night { 1.0 } else { 0.0 }]);
            for (name, color) in [
                ("u_top", scheme.top),
                ("u_bottom", scheme.bottom),
                ("u_ribbon", scheme.ribbon),
            ]
            .into_iter()
            .chain(
                ["u_field0", "u_field1", "u_field2", "u_field3", "u_field4"]
                    .into_iter()
                    .zip(scheme.fields),
            ) {
                target.set(name, &rgb(color));
            }
        }));
        ctx.request_repaint_after(FRAME);
    } else {
        painter.extend(still(screen, scheme));
    }
}

/// The wallpaper without the shader: the gradient with the fields as soft glows.
fn still(screen: Rect, scheme: &Scheme) -> Vec<egui::Shape> {
    let corners = [
        screen.left_top(),
        screen.right_top(),
        screen.right_bottom(),
        screen.left_bottom(),
    ];
    let mut shapes = vec![shaded_polygon(&corners, |point| {
        let t = (point.y - screen.top()) / screen.height().max(1.0);
        scheme.top.lerp_to_gamma(scheme.bottom, t)
    })];
    let spots = [
        (0.2, 0.25),
        (0.75, 0.2),
        (0.85, 0.75),
        (0.3, 0.8),
        (0.5, 0.5),
    ];
    for ((x, y), color) in spots.into_iter().zip(scheme.fields) {
        let center = Pos2::new(
            screen.left() + screen.width() * x,
            screen.top() + screen.height() * y,
        );
        let radius = screen.width().max(screen.height()) * 0.4;
        shapes.push(glass::radial_glow(
            center,
            radius,
            color.gamma_multiply(0.7),
        ));
    }
    shapes
}

fn rgb(color: Color32) -> [f32; 3] {
    [color.r(), color.g(), color.b()].map(|channel| f32::from(channel) / 255.0)
}

/// Five blurred fields wander on Lissajous paths through a slowly warping space; three ribbons
/// ripple across, each a soft glowing band around a sharp bright filament. A little noise keeps
/// the gradients from banding.
const FRAGMENT_SHADER: &str = r#"
uniform float u_time;
uniform float u_night;
uniform vec3 u_top;
uniform vec3 u_bottom;
uniform vec3 u_ribbon;
uniform vec3 u_field0;
uniform vec3 u_field1;
uniform vec3 u_field2;
uniform vec3 u_field3;
uniform vec3 u_field4;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 pixel = v_uv * u_size;
    float aspect = u_size.x / max(u_size.y, 1.0);
    // y runs down from the top, x is scaled so fields stay round.
    vec2 uv = vec2(v_uv.x * aspect, 1.0 - v_uv.y);
    float t = u_time * 0.06;
    vec3 color = mix(u_top, u_bottom, smoothstep(0.0, 1.0, uv.y));

    vec2 warp = uv + 0.09 * vec2(sin(uv.y * 3.1 + t * 2.3), cos(uv.x * 2.7 - t * 1.9));
    vec3 fields[5] = vec3[5](u_field0, u_field1, u_field2, u_field3, u_field4);
    for (int i = 0; i < 5; i++) {
        float k = float(i);
        vec2 middle = vec2(
            aspect * (0.5 + 0.42 * sin(t * (0.7 + 0.13 * k) + k * 1.7)),
            0.5 + 0.40 * cos(t * (0.5 + 0.11 * k) + k * 2.3));
        float reach = 0.34 + 0.08 * sin(k * 1.3 + t);
        float d = length(warp - middle) / reach;
        color = mix(color, fields[i], exp(-d * d * 1.3) * 0.88);
    }

    for (int j = 0; j < 3; j++) {
        float k = float(j);
        float y = 0.22 + 0.28 * k
            + 0.09 * sin(warp.x * (1.5 + 0.45 * k) + t * 3.0 + k * 2.1)
            + 0.035 * sin(warp.x * 4.3 - t * 2.2 + k);
        float across = warp.y - y;
        float band = exp(-pow(across / (0.055 + 0.02 * k), 2.0));
        float filament = exp(-pow(across / 0.0035, 2.0));
        float sheen = exp(-pow((across - 0.03) / 0.012, 2.0));
        float strength = mix(0.55, 0.4, u_night);
        color = mix(color, u_ribbon, clamp(band * strength * 0.6, 0.0, 1.0));
        color += u_ribbon * (filament * 0.55 + sheen * 0.18) * strength;
    }

    color += (hash(pixel + fract(u_time)) - 0.5) / 255.0;
    out_color = vec4(clamp(color, 0.0, 1.0), 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_still_wallpaper_covers_the_screen_with_every_field() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 960.0));
        for scheme in [&DAY, &NIGHT] {
            let shapes = still(screen, scheme);
            assert_eq!(shapes.len(), 1 + scheme.fields.len());
            assert_eq!(shapes[0].visual_bounding_rect(), screen);
        }
    }
}
