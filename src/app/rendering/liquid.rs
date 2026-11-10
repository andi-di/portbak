//! Liquid glass in the manner of iOS 26: thick clear lenses whose rounded rim bends and splits
//! the light behind them, lightly blurred and saturated, with a specular edge that follows the
//! pointer and a swell when pressed. Without OpenGL 3 the glass falls back to tinted panes.

use crate::app::rendering::glass::{self, Tint};
use crate::app::rendering::modern;
use crate::app::rendering::pixel::FaceLook;
use crate::app::rendering::shaders::{Effect, Recipe};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{self, label_font, Palette};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use std::time::Duration;

static LIQUID: Recipe = Recipe {
    name: "liquid",
    fragment: FRAGMENT_SHADER,
};

static SCROLL_EDGE: Recipe = Recipe {
    name: "liquid_scroll_edge",
    fragment: SCROLL_EDGE_SHADER,
};

/// Where light falls from when the pointer is away: the upper left, as on iOS.
const LIGHT: Vec2 = Vec2::new(-0.55, -0.83);
/// How strongly the pointer pulls the specular edge towards itself.
const POINTER_PULL: f32 = 0.65;
/// How far past a piece of glass the pointer must be, in points, to pull its light fully.
const POINTER_REACH: f32 = 60.0;
/// How long glass takes to brighten under the pointer or settle again.
pub const HOVER_SECONDS: f32 = 0.15;
/// Width of the bright line along the rim, in points.
const RIM: f32 = 1.3;
/// How far pressed glass swells, as a fraction of its height, up to `SWELL_LIMIT` points.
const SWELL: f32 = 0.12;
const SWELL_LIMIT: f32 = 5.0;
const PROGRESS_FRAME: Duration = Duration::from_millis(30);
/// How far into a scroll view its edge blurs content, and how far past it the blur fades out.
const EDGE_REACH: f32 = 30.0;
const EDGE_FADE: f32 = 10.0;
/// The strongest blur right at a scroll edge, in points.
const EDGE_BLUR: f32 = 9.0;
/// The iOS switch green, which stays bright while green text is darkened for legibility.
const SWITCH_GREEN: Color32 = Color32::from_rgb(0x34, 0xc7, 0x59);
const SWITCH_GREEN_DARK: Color32 = Color32::from_rgb(0x30, 0xd1, 0x58);

/// How a piece of liquid glass bends, tints and lights what lies behind it.
#[derive(Clone, Copy)]
pub struct Lens {
    /// Body tint, laid over the view at `tint_strength`.
    pub tint: Color32,
    pub tint_strength: f32,
    /// Blur radius of the view, in points.
    pub blur: f32,
    /// Width of the curved rim, in points.
    pub bezel: f32,
    /// How far the rim bends the view inwards, in points.
    pub depth: f32,
    /// Saturation of the view: 1 keeps it, more makes it vivid.
    pub vibrancy: f32,
    /// Strength of the specular edge.
    pub shine: f32,
    /// How far the rim splits red from blue, relative to `depth`.
    pub dispersion: f32,
    /// Magnification of the view: 1 keeps it, more enlarges it like a loupe.
    pub zoom: f32,
    /// A glow where the pointer presses, from 0 to 1.
    pub touch: Option<(Pos2, f32)>,
    /// The soft shadow below the glass, as an opacity.
    pub shadow: f32,
}

impl Lens {
    /// Regular glass for panes, sidebars, bars and menus.
    pub fn pane(ctx: &egui::Context) -> Self {
        let palette = Palette::of_ctx(ctx);
        let dark = ctx.global_style().visuals.dark_mode;
        Self {
            tint: palette.surface,
            tint_strength: if dark { 0.34 } else { 0.2 },
            blur: 5.0,
            bezel: 26.0,
            depth: 26.0,
            vibrancy: 1.6,
            shine: if dark { 0.42 } else { 0.62 },
            dispersion: 0.45,
            zoom: 1.0,
            touch: None,
            shadow: if dark { 0.38 } else { 0.10 },
        }
    }

    /// Clearer, more strongly bent glass for buttons, cards and other small controls.
    pub fn control(ctx: &egui::Context) -> Self {
        let palette = Palette::of_ctx(ctx);
        let dark = ctx.global_style().visuals.dark_mode;
        Self {
            tint: palette.raised,
            tint_strength: if dark { 0.16 } else { 0.24 },
            blur: 2.0,
            bezel: 16.0,
            depth: 14.0,
            vibrancy: 1.4,
            shine: if dark { 0.55 } else { 0.8 },
            dispersion: 0.6,
            zoom: 1.0,
            touch: None,
            shadow: if dark { 0.32 } else { 0.12 },
        }
    }

    /// The same glass tinted with `color`, like iOS's prominent glass.
    pub fn tinted(self, color: Color32, strength: f32) -> Self {
        Self {
            tint: color,
            tint_strength: strength,
            ..self
        }
    }
}

/// A piece of liquid glass over `rect`: its shadow, then the bent view behind it. Paint it before
/// the content that sits on the glass.
pub fn glass(ctx: &egui::Context, rect: Rect, radius: f32, lens: &Lens) -> Vec<Shape> {
    let radius = radius.min(rect.height().min(rect.width()) / 2.0);
    let mut shapes = vec![glass::halo(
        rect.translate(Vec2::new(0.0, 2.0)),
        radius,
        Color32::from_black_alpha((lens.shadow * 255.0) as u8),
        14.0,
    )];
    let effect = Effect::of(ctx, &LIQUID);
    if !effect.available() {
        shapes.extend(plain_glass(rect, radius, lens));
        return shapes;
    }
    let light = light_towards(ctx, rect);
    let lens = *lens;
    shapes.push(effect.shape(rect, ctx.content_rect(), move |target| {
        let [left, bottom] = target.pixel(rect.left_bottom());
        let [right, top] = target.pixel(rect.right_top());
        target.set("u_pane", &[left, bottom, right, top]);
        target.set(
            "u_shape",
            &[
                target.pixels(radius),
                target.pixels(lens.blur),
                target.pixels(lens.bezel.min(radius.max(1.0))),
                target.pixels(lens.depth),
            ],
        );
        let tint = egui::Rgba::from(lens.tint);
        target.set(
            "u_tint",
            &[tint.r(), tint.g(), tint.b(), lens.tint_strength],
        );
        target.set(
            "u_optics",
            &[lens.vibrancy, lens.shine, lens.dispersion, lens.zoom],
        );
        // The frame's y axis points up.
        target.set("u_light", &[light.x, -light.y]);
        target.set("u_rim", &[target.pixels(RIM)]);
        let (point, strength) = lens.touch.unwrap_or((rect.center(), 0.0));
        let [x, y] = target.pixel(point);
        target.set(
            "u_touch",
            &[x, y, target.pixels(rect.height().max(24.0)), strength],
        );
    }));
    shapes
}

/// A capsule glass button: clear glass for plain buttons, tinted glass for prominent ones. It
/// swells and lights up under the pointer while `press` rises from 0 to 1.
pub fn button(painter: &egui::Painter, face: Rect, look: &FaceLook, press: f32) {
    let ctx = painter.ctx();
    let palette = Palette::of_ctx(ctx);
    let face = face.expand(press * (face.height() * SWELL).min(SWELL_LIMIT));
    let radius = face.height() / 2.0;
    let base = Lens::control(ctx);
    let mut lens = if !look.enabled {
        Lens {
            shine: base.shine * 0.5,
            ..base.tinted(palette.surface, 0.45)
        }
    } else if look.clear {
        base.tinted(palette.raised, base.tint_strength + 0.12 * look.glow)
    } else {
        // Prominent glass brightens gradually instead of switching to a lighter fill.
        let tint = look.fill.lerp_to_gamma(Color32::WHITE, 0.16 * look.glow);
        Lens {
            vibrancy: 1.2,
            ..base.tinted(tint, 0.8 + 0.06 * look.glow)
        }
    };
    if press > 0.0 {
        let point = ctx
            .pointer_latest_pos()
            .map_or(face.center(), |pointer| face.clamp(pointer));
        lens.touch = Some((point, press));
    }
    painter.extend(glass(ctx, face, radius, &lens));
    if look.focused {
        painter.rect_stroke(
            face.expand(3.0),
            corner(radius + 3.0),
            Stroke::new(2.0_f32, palette.accent.gamma_multiply(0.8)),
            StrokeKind::Outside,
        );
    }
}

/// An iOS switch in `rect`: a capsule track filling with green as `on` rises from 0 to 1 and a
/// white knob that turns into a glass loupe over the track while `press` rises.
pub fn switch(painter: &egui::Painter, rect: Rect, on: f32, press: f32, enabled: bool) {
    let ctx = painter.ctx();
    let dark = ctx.global_style().visuals.dark_mode;
    let off = Color32::from_rgba_unmultiplied(0x78, 0x78, 0x80, if dark { 130 } else { 84 });
    let fade = if enabled { 1.0 } else { 0.5 };
    let green = if dark {
        SWITCH_GREEN_DARK
    } else {
        SWITCH_GREEN
    };
    let track = off.lerp_to_gamma(green, on).gamma_multiply(fade);
    let radius = rect.height() / 2.0;
    painter.rect_filled(rect, corner(radius), track);

    // A round knob at rest, as on iOS; pressed, it stretches into a wider loupe.
    let rest = Vec2::splat(rect.height() - 4.0);
    let size = rest * Vec2::new(1.0 + 0.5 * press, 1.0 + 0.2 * press);
    let travel = rect.width() - rest.x - 4.0;
    let center = Pos2::new(
        rect.left() + 2.0 + rest.x / 2.0 + travel * on,
        rect.center().y,
    );
    let knob = Rect::from_center_size(center, size);
    let knob_radius = size.y / 2.0;
    if press > 0.01 {
        let lens = Lens {
            zoom: 1.0 + 0.25 * press,
            blur: 0.0,
            depth: 6.0,
            bezel: knob_radius,
            ..Lens::control(ctx).tinted(Color32::WHITE, 0.08)
        };
        painter.extend(glass(ctx, knob, knob_radius, &lens));
    }
    let solid = (1.0 - press) * fade.max(0.85);
    if solid > 0.0 {
        painter.add(glass::halo(
            knob.translate(Vec2::new(0.0, 1.5)),
            knob_radius,
            Color32::from_black_alpha((50.0 * solid) as u8),
            5.0,
        ));
        painter.rect_filled(
            knob,
            corner(knob_radius),
            Color32::WHITE.gamma_multiply(solid),
        );
    }
}

/// An indeterminate iOS progress capsule centered in `rect`, with a tinted bar gliding along it.
pub fn progress(painter: &egui::Painter, rect: Rect, color: Color32, time: f64, active: bool) {
    const HEIGHT: f32 = 6.0;
    let ctx = painter.ctx();
    let dark = ctx.global_style().visuals.dark_mode;
    let track = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), HEIGHT));
    let fill = Color32::from_rgba_unmultiplied(0x78, 0x78, 0x80, if dark { 92 } else { 52 });
    painter.rect_filled(track, corner(HEIGHT / 2.0), fill);
    if !active {
        return;
    }
    let phase = (time * 0.6).fract() as f32;
    // The bar eases in and out as it glides, like UIKit's indeterminate bars.
    let ease = phase * phase * (3.0 - 2.0 * phase);
    let width = track.width() * 0.35;
    let left = track.left() - width + (track.width() + width) * ease;
    let bar = Rect::from_min_max(
        Pos2::new(left.max(track.left()), track.top()),
        Pos2::new((left + width).min(track.right()), track.bottom()),
    );
    if bar.width() > 0.0 {
        painter.rect_filled(bar, corner(HEIGHT / 2.0), color);
        let shine = Rect::from_min_size(bar.min, Vec2::new(bar.width(), HEIGHT * 0.4));
        painter.rect_filled(shine, corner(HEIGHT / 2.0), Color32::from_white_alpha(70));
    }
    ctx.request_repaint_after(PROGRESS_FRAME);
}

/// Glass behind a card or row, lit by `tint`: clear at rest and tinted with the accent when
/// current or chosen; it brightens as `hover` rises from 0 to 1 under the pointer.
pub fn tile(ctx: &egui::Context, rect: Rect, tint: Tint, hover: f32) -> Vec<Shape> {
    let palette = Palette::of_ctx(ctx);
    let dark = ctx.global_style().visuals.dark_mode;
    let radius = style::metrics(ctx).radius as f32 * 0.75;
    let base = Lens {
        blur: 4.0,
        bezel: 14.0,
        depth: 10.0,
        ..Lens::control(ctx)
    };
    let lens = match tint {
        Tint::Rest | Tint::Hover => base.tinted(palette.raised, base.tint_strength + 0.14 * hover),
        Tint::Current => base.tinted(
            palette.accent,
            if dark { 0.38 } else { 0.24 } + 0.06 * hover,
        ),
        Tint::Accent => base.tinted(palette.accent, 0.82 + 0.06 * hover),
    };
    let mut shapes = glass(ctx, rect, radius, &lens);
    if tint == Tint::Current {
        shapes.push(Shape::rect_stroke(
            rect,
            corner(radius),
            Stroke::new(1.5_f32, palette.accent.gamma_multiply(0.7)),
            StrokeKind::Inside,
        ));
    }
    shapes
}

/// A plain section heading: an accent symbol beside a bold title, as in iOS grouped lists.
pub fn title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite) {
    let palette = Palette::of(ui);
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 18,
            right: 18,
            top: 14,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                sprites::show(ui, sprite, 2.0, palette.accent);
                ui.label(
                    egui::RichText::new(title)
                        .font(label_font(ui.ctx()))
                        .color(palette.ink),
                );
            });
        });
}

/// A capsule badge filled with `fill`.
pub fn pill(painter: &egui::Painter, rect: Rect, fill: Color32) {
    painter.rect_filled(rect, corner(rect.height() / 2.0), fill);
}

/// A small status dot; lit dots glow softly.
pub fn dot(painter: &egui::Painter, center: Pos2, color: Color32, lit: bool) {
    if lit {
        painter.add(glass::radial_glow(center, 10.0, color.gamma_multiply(0.45)));
    }
    painter.circle_filled(center, 4.5, color);
}

/// Text ink on a prominent glass button tinted with `fill`: white, as on iOS, except on the
/// palest tints.
pub fn ink_on(fill: Color32) -> Color32 {
    if modern::luminance(fill) > 0.6 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// How heavily a glass sheet is frosted.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Frost {
    /// Regular pane glass, for lists laid on the page.
    Pane,
    /// Heavily frosted glass that keeps text legible over anything: menus, tooltips, alerts.
    Menu,
}

/// `contents` on a sheet of glass reaching `margin` past them with `radius` corners. The sheet
/// is painted beneath the contents once their size is known.
pub fn sheet<R>(
    ui: &mut egui::Ui,
    margin: egui::Margin,
    radius: f32,
    frost: Frost,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let ctx = ui.ctx().clone();
    // Menus may scroll or clip their contents, so their glass goes on a painter reaching the
    // margin; lists stay clipped to the page they scroll in.
    let painter = match frost {
        Frost::Menu => ui.painter().with_clip_rect(ctx.content_rect()),
        Frost::Pane => ui.painter().clone(),
    };
    let backing = painter.add(Shape::Noop);
    let inner = contents(ui);
    let rect = ui.min_rect() + egui::epaint::MarginF32::from(margin);
    let pane = Lens::pane(&ctx);
    let lens = match frost {
        Frost::Pane => pane,
        Frost::Menu => Lens {
            blur: 14.0,
            tint_strength: pane.tint_strength + 0.3,
            ..pane
        },
    };
    painter.set(backing, glass(&ctx, rect, radius, &lens));
    inner
}

/// The iOS scroll edge effect over a scroll view's `viewport`: content blurs progressively as it
/// nears an edge it is cut off at, then fades into the backdrop instead of ending in a hard line.
pub fn scroll_edges(ui: &egui::Ui, viewport: Rect, cut_top: bool, cut_bottom: bool) {
    let ctx = ui.ctx();
    let effect = Effect::of(ctx, &SCROLL_EDGE);
    if !effect.available() {
        return;
    }
    let painter = ui
        .painter()
        .with_clip_rect(viewport.expand2(Vec2::new(0.0, EDGE_FADE)));
    for (cut, edge, side) in [
        (cut_top, viewport.top(), 1.0_f32),
        (cut_bottom, viewport.bottom(), -1.0),
    ] {
        if !cut {
            continue;
        }
        let (inner, outer) = (edge + side * EDGE_REACH, edge - side * EDGE_FADE);
        let band = Rect::from_x_y_ranges(viewport.x_range(), inner.min(outer)..=inner.max(outer));
        painter.add(effect.shape(band, ctx.content_rect(), move |target| {
            let [_, edge_px] = target.pixel(Pos2::new(viewport.left(), edge));
            target.set("u_edge", &[edge_px]);
            // Into the content is down the screen at the top edge, which is down the frame too.
            target.set("u_side", &[-side]);
            target.set(
                "u_reach",
                &[
                    target.pixels(EDGE_REACH),
                    target.pixels(EDGE_FADE),
                    target.pixels(EDGE_BLUR),
                ],
            );
        }));
    }
}

/// A unit vector from `rect` towards the light, leaning towards the pointer.
/// The pull grows with the pointer's distance, so the light glides as the pointer crosses the
/// glass instead of swinging round whenever it passes near the center.
fn light_towards(ctx: &egui::Context, rect: Rect) -> Vec2 {
    let reach = rect.size().length() / 2.0 + POINTER_REACH;
    let offset = ctx
        .pointer_latest_pos()
        .map_or(Vec2::ZERO, |pointer| pointer - rect.center());
    light_leaning(offset, reach)
}

/// The light leaning towards a pointer `offset` from the glass's center, fully at `reach`.
fn light_leaning(offset: Vec2, reach: f32) -> Vec2 {
    let pull = offset / offset.length().max(reach);
    (LIGHT + pull * POINTER_PULL).normalized()
}

/// Tinted glass with a bright rim, for GPUs that turn the shader down.
fn plain_glass(rect: Rect, radius: f32, lens: &Lens) -> [Shape; 2] {
    let strength = lens.tint_strength.max(0.65);
    [
        Shape::rect_filled(rect, corner(radius), lens.tint.gamma_multiply(strength)),
        Shape::rect_stroke(
            rect,
            corner(radius),
            Stroke::new(1.0_f32, Color32::from_white_alpha(90)),
            StrokeKind::Inside,
        ),
    ]
}

fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.clamp(0.0, 255.0).round() as u8)
}

/// Lens optics over a rounded, continuously curved outline: the view is pulled in from further
/// inside across the rim, with red and blue bent by different amounts, then blurred, saturated
/// and tinted. A thin specular line runs along the rim, brightest where the edge faces the light
/// and, more faintly, on the opposite side where light leaves the glass.
const FRAGMENT_SHADER: &str = r#"
// Pane corners in pixels: left, bottom, right, top.
uniform vec4 u_pane;
// Corner radius, blur reach, bezel width and bend depth, in pixels.
uniform vec4 u_shape;
// Tint color and strength.
uniform vec4 u_tint;
// Vibrancy, shine, dispersion and zoom.
uniform vec4 u_optics;
// Unit vector towards the light; y points up.
uniform vec2 u_light;
// Width of the specular line, in pixels.
uniform float u_rim;
// Touch point and glow radius in pixels, and its strength.
uniform vec4 u_touch;

// Signed distance to the outline, negative inside. Corners sit between a circle and a
// superellipse, like Apple's continuous corners; capsules and circles keep true round ends.
float outline(vec2 pixel, vec2 center, vec2 half_size) {
    float shortest = min(half_size.x, half_size.y);
    float radius = min(u_shape.x, shortest);
    vec2 q = abs(pixel - center) - (half_size - radius);
    vec2 m = max(q, 0.0);
    vec2 m2 = m * m;
    float squircle = sqrt(sqrt(m2.x * m2.x + m2.y * m2.y));
    float blend = 0.45 * smoothstep(1.0, 0.7, radius / max(shortest, 0.001));
    return mix(length(m), squircle, blend) + min(max(q.x, q.y), 0.0) - radius;
}

vec3 blurred(vec2 at) {
    float reach = u_shape.y;
    if (reach < 0.5) {
        return frame(at).rgb;
    }
    vec3 sum = vec3(0.0);
    const int TAPS = 24;
    for (int tap = 0; tap < TAPS; tap++) {
        float step_out = (float(tap) + 0.5) / float(TAPS);
        float angle = float(tap) * 2.39996;
        sum += frame(at + vec2(cos(angle), sin(angle)) * reach * sqrt(step_out)).rgb;
    }
    return sum / float(TAPS);
}

void main() {
    vec2 pixel = v_uv * u_size;
    vec2 center = (u_pane.xy + u_pane.zw) * 0.5;
    vec2 half_size = (u_pane.zw - u_pane.xy) * 0.5;
    float signed_distance = outline(pixel, center, half_size);
    float mask = clamp(0.5 - signed_distance, 0.0, 1.0);
    if (mask <= 0.0) {
        discard;
    }
    float inside = max(-signed_distance, 0.0);
    vec2 e = vec2(0.5, 0.0);
    vec2 normal = vec2(
        outline(pixel + e.xy, center, half_size) - outline(pixel - e.xy, center, half_size),
        outline(pixel + e.yx, center, half_size) - outline(pixel - e.yx, center, half_size));
    normal = length(normal) > 0.0001 ? normalize(normal) : vec2(0.0);

    // A convex rim: flat in the middle, curving ever steeper towards the edge.
    float rim = clamp(1.0 - inside / max(u_shape.z, 1.0), 0.0, 1.0);
    float bend = pow(rim, 2.4);
    vec2 look = center + (pixel - center) / max(u_optics.w, 0.01);
    vec2 shift = -normal * u_shape.w * bend;
    float spread = u_optics.z * bend;
    vec3 color;
    if (spread > 0.02) {
        color.r = blurred(look + shift * (1.0 - spread)).r;
        color.g = blurred(look + shift).g;
        color.b = blurred(look + shift * (1.0 + spread)).b;
    } else {
        color = blurred(look + shift);
    }

    float grey = dot(color, vec3(0.299, 0.587, 0.114));
    color = clamp(mix(vec3(grey), color, u_optics.x), 0.0, 1.0);
    color = mix(color, u_tint.rgb, u_tint.a);

    float facing = dot(normal, u_light);
    float lit = pow(abs(facing), 1.5) * (facing > 0.0 ? 1.0 : 0.6);
    float line = 1.0 - smoothstep(0.0, u_rim, inside);
    float glow = pow(rim, 3.0);
    float specular = u_optics.y * (line * (0.3 + 0.7 * lit) + glow * lit * 0.4);
    // The side away from the light sits a touch deeper in shade, giving the glass thickness.
    float shade = u_optics.y * 0.12 * rim * rim * max(-facing, 0.0);
    color = color * (1.0 - shade) + vec3(specular);

    if (u_touch.w > 0.0) {
        float reach = length(pixel - u_touch.xy) / max(u_touch.z, 1.0);
        color += vec3(u_touch.w * 0.3 * exp(-reach * reach * 2.0));
    }
    out_color = vec4(clamp(color, 0.0, 1.0) * mask, mask);
}
"#;

/// Blurs content more the closer it lies to the edge, then lets the blur fade past the edge so
/// the clipped content melts into whatever lies beyond.
const SCROLL_EDGE_SHADER: &str = r#"
// The edge's height in pixels, and which way the content lies from it: +1 up, -1 down.
uniform float u_edge;
uniform float u_side;
// Reach into the content, fade past the edge and the strongest blur, in pixels.
uniform vec3 u_reach;

void main() {
    vec2 pixel = v_uv * u_size;
    float into = (pixel.y - u_edge) * u_side;
    float strength = clamp(1.0 - into / u_reach.x, 0.0, 1.0);
    float alpha = into < 0.0 ? clamp(1.0 + into / u_reach.y, 0.0, 1.0) : 1.0;
    float reach = u_reach.z * strength * strength;
    vec3 sum = vec3(0.0);
    const int TAPS = 16;
    for (int tap = 0; tap < TAPS; tap++) {
        float step_out = (float(tap) + 0.5) / float(TAPS);
        float angle = float(tap) * 2.39996;
        sum += frame(pixel + vec2(cos(angle), sin(angle)) * reach * sqrt(step_out)).rgb;
    }
    out_color = vec4(sum / float(TAPS) * alpha, alpha);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::theme::skins;

    #[test]
    fn glass_is_a_shadow_and_a_lens_over_the_rect() {
        let ctx = egui::Context::default();
        style::install(&ctx, skins::find("liquid-glass"));
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let ctx = ui.ctx().clone();
            let rect = Rect::from_min_size(egui::pos2(20.0, 30.0), egui::vec2(200.0, 60.0));
            let shapes = glass(&ctx, rect, 30.0, &Lens::pane(&ctx));
            let egui::Shape::Callback(callback) = &shapes[1] else {
                panic!("the lens is a paint callback");
            };
            assert_eq!(callback.rect, rect);
        });
    }

    #[test]
    fn plain_glass_stays_translucent_enough_to_read_on() {
        let ctx = egui::Context::default();
        style::install(&ctx, skins::find("liquid-glass"));
        let lens = Lens::control(&ctx);
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(80.0, 30.0));
        let [Shape::Rect(body), _] = plain_glass(rect, 15.0, &lens) else {
            panic!("plain glass is a filled body and a rim");
        };
        assert!(body.fill.a() >= 160);
    }

    #[test]
    fn light_glides_instead_of_jumping_as_the_pointer_crosses_the_center() {
        let reach = 100.0;
        let left = light_leaning(Vec2::new(-2.0, 0.0), reach);
        let right = light_leaning(Vec2::new(2.0, 0.0), reach);
        assert!((left - right).length() < 0.05);
        let far = light_leaning(Vec2::new(reach * 3.0, 0.0), reach);
        assert!(far.x > left.x && (far.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn light_leans_towards_the_pointer_and_stays_a_unit_vector() {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 40.0));
        let light = light_towards(&ctx, rect);
        assert!((light.length() - 1.0).abs() < 1e-4);
        assert!(light.x < 0.0 && light.y < 0.0);
    }
}
