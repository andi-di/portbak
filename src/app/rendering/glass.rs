//! Aero glass faces: clear panes with screen-fixed reflections and sweeping shimmers, glossy gel
//! buttons, glass tiles, orb lamps and a shining progress bar.

use crate::app::rendering::frost;
use crate::app::rendering::pixel::{mix, shaded_fan, shaded_polygon, FaceLook};
use crate::app::theme::style::{self, Palette, Skin};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::time::Duration;

const ARC_STEPS: u32 = 8;
/// egui shortens repaint delays by the predicted frame time, so this animates at about 30 fps.
const ANIMATION_FRAME: Duration = Duration::from_millis(50);
/// How far down a face its glossy reflection reaches, as a fraction of its height.
const GLOSS_DEPTH: f32 = 0.5;
/// Big panes keep their gloss to a band at the top instead of washing out their upper half.
const PANE_GLOSS_LIMIT: f32 = 30.0;
/// Reflection streaks lean right: a streak is the set of points where `x + y * LEAN` is constant.
const LEAN: f32 = 0.55;
/// Screen-space distance between repeating reflection streaks.
const STREAK_PERIOD: f32 = 520.0;
/// Seconds between shimmers sweeping over a pane, and the share of that time a sweep takes.
const SHIMMER_PERIOD: f32 = 7.0;
const SHIMMER_SWEEP: f32 = 0.2;
/// Colored gels stay translucent but dense enough for light text on them.
const GEL_OPACITY: f32 = 0.78;
/// Clear glass buttons let most of the backdrop through.
const CLEAR_OPACITY: f32 = 0.16;

/// Corner radii clockwise from the top left: north-west, north-east, south-east, south-west.
pub type Corners = [f32; 4];

/// How a glass tile is lit.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    /// Clear glass in the panel color.
    Rest,
    /// Lightly tinted with the accent while pointed at.
    Hover,
    /// Accent-tinted with an accent rim and a breathing glow; text on it keeps the normal ink.
    Current,
    /// A glossy accent gel; text on it uses the accent ink.
    Accent,
}

/// How a glass pane is colored.
#[derive(Clone, Copy)]
pub struct Glaze {
    /// Body color, brightened towards the top.
    pub base: Color32,
    /// Opacity of the body; reflections and rims are painted on top at their own strength.
    pub opacity: f32,
    /// Color of the outer edge.
    pub rim: Color32,
    /// A colored halo around the pane instead of a drop shadow.
    pub glow: Option<Color32>,
    /// Light pooling in the lower half, as in a glossy gel.
    pub pool: bool,
}

impl Glaze {
    /// Clear glass in the skin's panel color at the skin's panel opacity.
    pub fn clear(ctx: &egui::Context) -> Self {
        let palette = Palette::of_ctx(ctx);
        Self {
            base: palette.surface,
            opacity: Skin::of(ctx).effects.opacity,
            rim: palette.outline.gamma_multiply(0.45),
            glow: None,
            pool: false,
        }
    }
}

/// A sheet of glass: the frosted view behind it, a tinted body and rim, light pooling at the
/// bottom and the `sheen` on top.
pub fn pane(ctx: &egui::Context, rect: Rect, radius: f32, glaze: &Glaze) -> Vec<Shape> {
    let palette = Palette::of_ctx(ctx);
    let corners = [radius; 4];
    let mut shapes = vec![match glaze.glow {
        Some(color) => halo(rect, radius, color, 16.0),
        None => soft_shadow(rect, radius, palette, 12.0),
    }];
    shapes.push(frost::backdrop(ctx, rect, radius));
    let top = mix(glaze.base, Color32::WHITE, 0.5).gamma_multiply((glaze.opacity * 1.5).min(1.0));
    shapes.push(gradient(
        rect,
        corners,
        top,
        glaze.base.gamma_multiply(glaze.opacity),
    ));
    shapes.push(bottom_light(rect, radius, palette.hot, 0.3));
    if glaze.pool {
        shapes.push(light_pool(rect.shrink(1.5), radius - 1.5, glaze.base));
    }
    shapes.push(Shape::rect_stroke(
        rect,
        corner(radius),
        Stroke::new(1.0_f32, glaze.rim),
        StrokeKind::Inside,
    ));
    shapes.extend(sheen(ctx, rect, radius));
    shapes
}

/// The light on a glass surface: sky reflections that stay fixed to the screen as the surface
/// moves, an occasional shimmer, a gloss band, bright edges and twinkling corner glints.
pub fn sheen(ctx: &egui::Context, rect: Rect, radius: f32) -> Vec<Shape> {
    let time = ctx.input(|input| input.time) as f32;
    let outline = rounded(rect, [radius; 4]);
    let mut shapes = reflections(&outline, rect);
    shapes.push(shimmer(&outline, rect, time));
    let depth = (rect.height() * GLOSS_DEPTH).min(PANE_GLOSS_LIMIT);
    shapes.push(gloss_band(rect.shrink(1.0), radius - 1.0, depth, 0.75));
    shapes.push(inner_highlight(rect, radius));
    shapes.push(top_edge_light(rect, radius));
    shapes.extend(glints(rect, time));
    shapes
}

/// A pill-shaped gel button with a breathing hover glow, pressed in by darkening its top.
pub fn gel_button(painter: &egui::Painter, face: Rect, look: &FaceLook) {
    let ctx = painter.ctx();
    let palette = Palette::of_ctx(ctx);
    let time = ctx.input(|input| input.time) as f32;
    let radius = face.height() / 2.0;
    painter.add(soft_shadow(face, radius, palette, 6.0));
    if look.glow > 0.0 {
        let breath = 0.75 + 0.25 * (time * 4.0).sin();
        let color = if look.clear { palette.hot } else { look.fill };
        painter.add(halo(
            face,
            radius,
            color.gamma_multiply(0.9 * look.glow * breath),
            12.0,
        ));
        ctx.request_repaint_after(ANIMATION_FRAME);
    }
    if !look.enabled {
        clear_gel(painter, face, radius, palette.surface, false, 0.6);
    } else if look.clear {
        clear_gel(painter, face, radius, look.fill, look.pressed, 1.0);
    } else {
        gel(painter, face, radius, look.fill, look.pressed);
    }
    if look.glow > 0.0 {
        let outline = rounded(face, [radius; 4]);
        let sweep = (time * 0.8).fract();
        painter.add(sweep_band(&outline, face, sweep, 0.55 * look.glow));
    }
    if look.focused {
        let ring = Stroke::new(2.0_f32, palette.hot);
        painter.rect_stroke(
            face.expand(2.5),
            corner(radius + 2.5),
            ring,
            StrokeKind::Outside,
        );
    }
}

/// A small glossy pill, e.g. behind a badge's text.
pub fn pill(painter: &egui::Painter, rect: Rect, fill: Color32) {
    gel(painter, rect, rect.height() / 2.0, fill, false);
}

/// A rounded, softly sunken glass check box well.
pub fn check_socket(painter: &egui::Painter, rect: Rect, border: Color32) {
    const RADIUS: f32 = 5.0;
    let palette = Palette::of_ctx(painter.ctx());
    let top = mix(palette.sunken, palette.shade, 0.45).gamma_multiply(0.7);
    painter.add(gradient(
        rect,
        [RADIUS; 4],
        top,
        palette.raised.gamma_multiply(0.5),
    ));
    painter.add(bottom_light(rect, RADIUS, palette.hot, 0.5));
    painter.add(gloss(rect.shrink(1.0), RADIUS - 1.0, 0.7));
    let edge = Stroke::new(1.0_f32, border);
    painter.rect_stroke(rect, corner(RADIUS), edge, StrokeKind::Inside);
    painter.add(inner_highlight(rect, RADIUS));
}

/// A glossy glass orb lamp; lit orbs glow around and brighten at the core.
pub fn orb(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, lit: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    if lit {
        painter.add(radial_glow(
            center,
            radius * 2.4,
            color.gamma_multiply(0.45),
        ));
    }
    sphere(painter, center, radius, color, if lit { 0.45 } else { 0.1 });
    let edge = Stroke::new(1.0_f32, mix(color, palette.outline, 0.6));
    painter.circle_stroke(center, radius, edge);
}

/// A shaded ball lit from below with a reflection on top; `glow` brightens its core.
pub fn sphere(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, glow: f32) {
    let core = mix(color, Color32::WHITE, glow);
    let rim = mix(color, Color32::BLACK, 0.35);
    let light = center + Vec2::new(0.0, radius * 0.45);
    let reach = radius * 1.45;
    painter.add(shaded_fan(
        light,
        &ellipse(center, Vec2::splat(radius), 0.0),
        |point| core.lerp_to_gamma(rim, (point.distance(light) / reach).min(1.0)),
    ));
    let shine = ellipse(
        center - Vec2::new(0.0, radius * 0.42),
        Vec2::new(radius * 0.62, radius * 0.4),
        0.0,
    );
    let (top, bottom) = vertical_span(&shine);
    painter.add(shaded_polygon(&shine, |point| {
        let t = (point.y - top) / (bottom - top).max(1.0);
        Color32::WHITE.gamma_multiply(0.85 - 0.8 * t)
    }));
}

/// An indeterminate progress bar: a glossy gel filling a glass track, with light running through.
pub fn progress(painter: &egui::Painter, track: Rect, color: Color32, time: f64, active: bool) {
    const RADIUS: f32 = 6.0;
    let ctx = painter.ctx();
    let palette = Palette::of_ctx(ctx);
    let time = time as f32;
    painter.add(gradient(
        track,
        [RADIUS; 4],
        style::glazed(ctx, mix(palette.sunken, palette.shade, 0.35)),
        style::glazed(ctx, palette.raised),
    ));
    let inner = track.shrink(2.0);
    if active {
        let fill = mix(color, Color32::WHITE, 0.15).gamma_multiply(GEL_OPACITY);
        painter.add(gradient(
            inner,
            [RADIUS - 2.0; 4],
            fill,
            mix(color, Color32::BLACK, 0.15).gamma_multiply(GEL_OPACITY),
        ));
        // Diagonal stripes marching along the gel, the classic busy bar.
        let outline = rounded(inner, [RADIUS - 2.0; 4]);
        let stripe = 14.0;
        let offset = (time * 30.0) % (stripe * 2.0);
        let mut x = inner.left() - inner.height() - stripe * 2.0 + offset;
        while x < inner.right() + stripe {
            let band = [
                egui::pos2(x, inner.bottom()),
                egui::pos2(x + inner.height(), inner.top()),
                egui::pos2(x + inner.height() + stripe, inner.top()),
                egui::pos2(x + stripe, inner.bottom()),
            ];
            let clipped = clip(&band, &outline);
            if clipped.len() >= 3 {
                painter.add(Shape::convex_polygon(
                    clipped,
                    Color32::WHITE.gamma_multiply(0.16),
                    Stroke::NONE,
                ));
            }
            x += stripe * 2.0;
        }
        painter.add(sweep_band(&outline, inner, (time * 0.6).fract(), 0.8));
        painter.add(gloss(inner, RADIUS - 2.0, 0.9));
    }
    let edge = Stroke::new(1.0_f32, palette.outline.gamma_multiply(0.5));
    painter.rect_stroke(track, corner(RADIUS), edge, StrokeKind::Inside);
    painter.add(inner_highlight(track, RADIUS));
}

/// The shapes of a glass tile behind a card or row, lit by `tint`.
pub fn tile(ctx: &egui::Context, rect: Rect, tint: Tint) -> Vec<Shape> {
    let palette = Palette::of_ctx(ctx);
    let radius = style::metrics(ctx).radius as f32;
    let time = ctx.input(|input| input.time) as f32;
    let clear = Glaze::clear(ctx);
    let breath = 0.7 + 0.3 * (time * 2.2).sin();
    let glaze = match tint {
        Tint::Rest => clear,
        Tint::Hover => Glaze {
            base: mix(palette.surface, palette.hot, 0.35),
            rim: palette.hot,
            glow: Some(palette.hot.gamma_multiply(0.55)),
            ..clear
        },
        Tint::Current => Glaze {
            base: mix(palette.surface, palette.accent, 0.2),
            opacity: (clear.opacity + 0.15).min(1.0),
            rim: palette.accent,
            glow: Some(palette.hot.gamma_multiply(0.8 * breath)),
            pool: false,
        },
        Tint::Accent => Glaze {
            base: palette.accent,
            opacity: GEL_OPACITY,
            rim: mix(palette.accent, palette.outline, 0.5),
            glow: Some(palette.accent.gamma_multiply(0.6)),
            pool: true,
        },
    };
    if tint != Tint::Rest {
        ctx.request_repaint_after(ANIMATION_FRAME);
    }
    pane(ctx, rect, radius, &glaze)
}

/// A faint light line just inside a glass edge, where real glass catches the light.
pub fn inner_highlight(rect: Rect, radius: f32) -> Shape {
    Shape::rect_stroke(
        rect.shrink(1.0),
        corner(radius - 1.0),
        Stroke::new(1.0_f32, Color32::WHITE.gamma_multiply(0.45)),
        StrokeKind::Inside,
    )
}

/// The glossy reflection over the top of a face whose top corners have `radius`.
pub fn gloss(rect: Rect, radius: f32, strength: f32) -> Shape {
    gloss_band(rect, radius, rect.height() * GLOSS_DEPTH, strength)
}

/// A rounded rectangle filled with a vertical gradient from `top` to `bottom`.
pub fn gradient(rect: Rect, corners: Corners, top: Color32, bottom: Color32) -> Shape {
    let height = rect.height().max(1.0);
    shaded_polygon(&rounded(rect, corners), |point| {
        top.lerp_to_gamma(bottom, ((point.y - rect.top()) / height).clamp(0.0, 1.0))
    })
}

/// Points of an ellipse with `radii`, turned by `angle` radians.
pub fn ellipse(center: Pos2, radii: Vec2, angle: f32) -> Vec<Pos2> {
    const STEPS: u32 = 24;
    let turn = Vec2::angled(angle);
    (0..STEPS)
        .map(|step| {
            let local = Vec2::angled(step as f32 * TAU / STEPS as f32) * radii;
            center
                + Vec2::new(
                    local.x * turn.x - local.y * turn.y,
                    local.x * turn.y + local.y * turn.x,
                )
        })
        .collect()
}

/// A disc of `color` at its center fading smoothly to nothing at `radius`.
pub fn radial_glow(center: Pos2, radius: f32, color: Color32) -> Shape {
    const RINGS: u32 = 6;
    const SEGMENTS: u32 = 40;
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, color);
    for ring in 1..=RINGS {
        let t = ring as f32 / RINGS as f32;
        let fade = color.gamma_multiply((1.0 - t) * (1.0 - t));
        for segment in 0..SEGMENTS {
            let angle = segment as f32 * TAU / SEGMENTS as f32;
            mesh.colored_vertex(center + Vec2::angled(angle) * radius * t, fade);
        }
    }
    let at = |ring: u32, segment: u32| 1 + (ring - 1) * SEGMENTS + segment % SEGMENTS;
    for segment in 0..SEGMENTS {
        mesh.add_triangle(0, at(1, segment), at(1, segment + 1));
        for ring in 2..=RINGS {
            let (inner, outer) = (at(ring - 1, segment), at(ring, segment));
            let (inner_next, outer_next) = (at(ring - 1, segment + 1), at(ring, segment + 1));
            mesh.add_triangle(inner, outer, outer_next);
            mesh.add_triangle(inner, outer_next, inner_next);
        }
    }
    mesh.into()
}

/// A four-pointed twinkle of light with a soft glow around it.
pub fn sparkle(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    if size < 0.5 {
        return;
    }
    painter.add(radial_glow(center, size * 0.9, color.gamma_multiply(0.5)));
    painter.add(star(center, size, Color32::WHITE));
}

/// A glossy gel face: lit gradient, light pooling at the bottom and a sharp reflection on top.
fn gel(painter: &egui::Painter, face: Rect, radius: f32, fill: Color32, pressed: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    let (top, bottom) = if pressed {
        (
            mix(fill, Color32::BLACK, 0.25),
            mix(fill, Color32::WHITE, 0.1),
        )
    } else {
        (
            mix(fill, Color32::WHITE, 0.25),
            mix(fill, Color32::BLACK, 0.15),
        )
    };
    painter.add(gradient(
        face,
        [radius; 4],
        top.gamma_multiply(GEL_OPACITY),
        bottom.gamma_multiply(GEL_OPACITY),
    ));
    let inner = face.shrink(1.5);
    painter.add(light_pool(inner, radius - 1.5, fill));
    if !pressed {
        painter.add(gloss(inner, radius - 1.5, 1.0));
    }
    let edge = Stroke::new(1.0_f32, mix(fill, palette.outline, 0.55));
    painter.rect_stroke(face, corner(radius), edge, StrokeKind::Inside);
    painter.add(inner_highlight(face, radius));
}

/// A clear glass bubble face: mostly see-through, with an aqua glow below and a bright gloss.
fn clear_gel(
    painter: &egui::Painter,
    face: Rect,
    radius: f32,
    fill: Color32,
    pressed: bool,
    strength: f32,
) {
    let palette = Palette::of_ctx(painter.ctx());
    let opacity = CLEAR_OPACITY * strength;
    let (top, bottom) = if pressed {
        (mix(fill, palette.accent, 0.4), fill)
    } else {
        (mix(fill, Color32::WHITE, 0.6), fill)
    };
    painter.add(gradient(
        face,
        [radius; 4],
        top.gamma_multiply(opacity * 2.2),
        bottom.gamma_multiply(opacity),
    ));
    let inner = face.shrink(1.5);
    painter.add(bottom_light(
        inner,
        radius - 1.5,
        palette.hot,
        0.8 * strength,
    ));
    if !pressed {
        painter.add(gloss(inner, radius - 1.5, strength));
    }
    let edge = Stroke::new(1.0_f32, palette.outline.gamma_multiply(0.5 * strength));
    painter.rect_stroke(face, corner(radius), edge, StrokeKind::Inside);
    painter.add(inner_highlight(face, radius));
    painter.add(top_edge_light(face, radius));
}

/// Light gathering in the lower half of a gel, as if lit from inside.
fn light_pool(rect: Rect, radius: f32, fill: Color32) -> Shape {
    let lower = Rect::from_min_max(egui::pos2(rect.left(), rect.center().y), rect.max);
    let glow = mix(fill, Color32::WHITE, 0.55).gamma_multiply(0.6);
    gradient(
        lower,
        [0.0, 0.0, radius, radius],
        Color32::TRANSPARENT,
        glow,
    )
}

/// Colored light seeping up from the bottom edge of a pane.
fn bottom_light(rect: Rect, radius: f32, color: Color32, strength: f32) -> Shape {
    let depth = (rect.height() * 0.45).min(40.0);
    let lower = Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - depth), rect.max);
    gradient(
        lower,
        [0.0, 0.0, radius, radius],
        Color32::TRANSPARENT,
        color.gamma_multiply(0.45 * strength),
    )
}

fn gloss_band(rect: Rect, radius: f32, depth: f32, strength: f32) -> Shape {
    let upper = Rect::from_min_max(rect.min, egui::pos2(rect.right(), rect.top() + depth));
    let top = Color32::WHITE.gamma_multiply(0.75 * strength);
    let fade = Color32::WHITE.gamma_multiply(0.1 * strength);
    gradient(upper, [radius, radius, 0.0, 0.0], top, fade)
}

/// A crisp white line along the top edge where light hits the glass hardest.
fn top_edge_light(rect: Rect, radius: f32) -> Shape {
    let inset = radius.max(4.0);
    let y = rect.top() + 1.5;
    Shape::line_segment(
        [
            egui::pos2(rect.left() + inset, y),
            egui::pos2(rect.right() - inset, y),
        ],
        Stroke::new(1.0_f32, Color32::WHITE.gamma_multiply(0.9)),
    )
}

/// Broad and narrow diagonal streaks reflecting the sky, placed in screen space so they stay put
/// while the pane scrolls under them, like light on a real window.
fn reflections(outline: &[Pos2], rect: Rect) -> Vec<Shape> {
    const STREAKS: [(f32, f32, f32); 3] =
        [(0.0, 90.0, 0.2), (120.0, 16.0, 0.28), (155.0, 6.0, 0.35)];
    let low = rect.left() + rect.top() * LEAN;
    let high = rect.right() + rect.bottom() * LEAN;
    let mut shapes = Vec::new();
    let first = (low / STREAK_PERIOD).floor() - 1.0;
    let last = (high / STREAK_PERIOD).ceil();
    let mut period = first;
    while period <= last {
        for (offset, width, peak) in STREAKS {
            let center = period * STREAK_PERIOD + offset;
            if let Some(shape) = streak(outline, rect, center, width, peak) {
                shapes.push(shape);
            }
        }
        period += 1.0;
    }
    shapes
}

/// One bright streak sweeping across the pane every few seconds.
fn shimmer(outline: &[Pos2], rect: Rect, time: f32) -> Shape {
    let phase = (time / SHIMMER_PERIOD + seed(rect)).fract();
    if phase > SHIMMER_SWEEP {
        return Shape::Noop;
    }
    sweep_band(outline, rect, phase / SHIMMER_SWEEP, 0.4)
}

/// A soft band of light crossing `rect` from left (`progress` 0) to right (`progress` 1).
fn sweep_band(outline: &[Pos2], rect: Rect, progress: f32, peak: f32) -> Shape {
    let width = (rect.width() * 0.18).clamp(24.0, 120.0);
    let low = rect.left() + rect.top() * LEAN - width;
    let high = rect.right() + rect.bottom() * LEAN + width;
    let center = low + (high - low) * progress;
    streak(outline, rect, center, width, peak).unwrap_or(Shape::Noop)
}

/// A diagonal band of light centered on `x + y * LEAN == center`, clipped to the pane outline,
/// brightest along its middle and towards the top.
fn streak(outline: &[Pos2], rect: Rect, center: f32, width: f32, peak: f32) -> Option<Shape> {
    let half = width / 2.0;
    let height = rect.height().max(1.0);
    let at = |u: f32, y: f32| egui::pos2(u - y * LEAN, y);
    // Each half fades linearly from the core to its edge, which a triangle fan renders exactly.
    let halves: Vec<Shape> = [(center - half, center), (center, center + half)]
        .into_iter()
        .filter_map(|(from, to)| {
            let band = [
                at(from, rect.top()),
                at(to, rect.top()),
                at(to, rect.bottom()),
                at(from, rect.bottom()),
            ];
            let clipped = clip(&band, outline);
            (clipped.len() >= 3).then(|| {
                shaded_polygon(&clipped, |point| {
                    let across = 1.0 - ((point.x + point.y * LEAN - center).abs() / half).min(1.0);
                    let down = 1.0 - 0.7 * ((point.y - rect.top()) / height).clamp(0.0, 1.0);
                    Color32::WHITE.gamma_multiply(peak * across * down)
                })
            })
        })
        .collect();
    (!halves.is_empty()).then(|| Shape::Vec(halves))
}

/// Tiny twinkles that flare up now and then on the pane's top corners.
fn glints(rect: Rect, time: f32) -> Vec<Shape> {
    let seed = seed(rect);
    [
        (rect.left_top() + Vec2::new(14.0, 3.0), 0.0),
        (rect.right_top() + Vec2::new(-22.0, 3.0), 0.5),
    ]
    .into_iter()
    .filter_map(|(center, shift)| {
        let phase = (time / 4.0 + seed * 3.0 + shift).fract();
        let flare = (1.0 - (phase * 6.0 - 1.0).abs()).max(0.0);
        (flare > 0.0).then(|| star(center, 9.0 * flare, Color32::WHITE.gamma_multiply(flare)))
    })
    .collect()
}

/// A four-pointed star of `color` fading towards its tips.
fn star(center: Pos2, size: f32, color: Color32) -> Shape {
    let points: Vec<Pos2> = (0..8)
        .map(|step| {
            let reach = if step % 2 == 0 { size } else { size * 0.18 };
            center + Vec2::angled(step as f32 * PI / 4.0 - FRAC_PI_2) * reach
        })
        .collect();
    shaded_fan(center, &points, |point| {
        color.gamma_multiply(1.0 - (point.distance(center) / size.max(0.1)).min(1.0))
    })
}

/// A stable per-pane offset so panes shimmer and glint at different moments.
fn seed(rect: Rect) -> f32 {
    ((rect.left() * 0.013 + rect.width() * 0.007).sin() * 0.5 + 0.5).fract()
}

/// A shadow falling mostly below a face, heavier towards its bottom edge.
fn soft_shadow(rect: Rect, radius: f32, palette: &Palette, reach: f32) -> Shape {
    let color = palette.outline.gamma_multiply(0.4);
    let height = rect.height().max(1.0);
    aura(rect, radius, reach, |point| {
        color.gamma_multiply(0.35 + 0.65 * ((point.y - rect.top()) / height).clamp(0.0, 1.0))
    })
}

/// `color` spreading `reach` points out from a rounded edge, leaving the inside clear.
pub fn halo(rect: Rect, radius: f32, color: Color32, reach: f32) -> Shape {
    aura(rect, radius, reach, |_| color)
}

/// Light or shade spreading `reach` pixels out from a rounded edge and fading to nothing. Unlike
/// an egui shadow it leaves the inside empty, so it never tints the glass laid over it.
fn aura(rect: Rect, radius: f32, reach: f32, shade: impl Fn(Pos2) -> Color32) -> Shape {
    let radius = radius.max(1.0);
    let inner = rounded(rect, [radius; 4]);
    let outer = rounded(rect.expand(reach), [radius + reach; 4]);
    if inner.len() != outer.len() {
        return Shape::Noop;
    }
    let mut mesh = egui::Mesh::default();
    for (near, far) in inner.iter().zip(&outer) {
        mesh.colored_vertex(*near, shade(*near));
        mesh.colored_vertex(*far, Color32::TRANSPARENT);
    }
    let count = inner.len() as u32;
    for index in 0..count {
        let (a, b) = (index * 2, (index + 1) % count * 2);
        mesh.add_triangle(a, a + 1, b + 1);
        mesh.add_triangle(a, b + 1, b);
    }
    mesh.into()
}

/// Clips `subject` to the convex, clockwise polygon `window` (Sutherland–Hodgman).
fn clip(subject: &[Pos2], window: &[Pos2]) -> Vec<Pos2> {
    let mut output = subject.to_vec();
    for (index, &from) in window.iter().enumerate() {
        let to = window[(index + 1) % window.len()];
        let edge = to - from;
        let inside = |point: Pos2| edge.x * (point.y - from.y) - edge.y * (point.x - from.x) >= 0.0;
        let input = std::mem::take(&mut output);
        for (point_index, &current) in input.iter().enumerate() {
            let previous = input[(point_index + input.len() - 1) % input.len()];
            let crossing = || {
                let side = |point: Pos2| edge.x * (point.y - from.y) - edge.y * (point.x - from.x);
                let (a, b) = (side(previous), side(current));
                previous + (current - previous) * (a / (a - b))
            };
            match (inside(previous), inside(current)) {
                (true, true) => output.push(current),
                (true, false) => output.push(crossing()),
                (false, true) => {
                    output.push(crossing());
                    output.push(current);
                }
                (false, false) => {}
            }
        }
        if output.is_empty() {
            break;
        }
    }
    output
}

/// The outline of `rect` with rounded corners, clockwise from the top left.
fn rounded(rect: Rect, corners: Corners) -> Vec<Pos2> {
    let corners = fit(rect, corners);
    let centers = [
        egui::pos2(rect.left() + corners[0], rect.top() + corners[0]),
        egui::pos2(rect.right() - corners[1], rect.top() + corners[1]),
        egui::pos2(rect.right() - corners[2], rect.bottom() - corners[2]),
        egui::pos2(rect.left() + corners[3], rect.bottom() - corners[3]),
    ];
    let mut points = Vec::new();
    for (index, (center, radius)) in centers.into_iter().zip(corners).enumerate() {
        let start = PI + index as f32 * FRAC_PI_2;
        if radius < 0.5 {
            points.push(center);
            continue;
        }
        points.extend((0..=ARC_STEPS).map(|step| {
            center + Vec2::angled(start + step as f32 * FRAC_PI_2 / ARC_STEPS as f32) * radius
        }));
    }
    points
}

/// Shrinks the radii so neighbors never overlap along an edge, like CSS border-radius.
fn fit(rect: Rect, corners: Corners) -> Corners {
    let corners = corners.map(|radius| radius.max(0.0));
    let [nw, ne, se, sw] = corners;
    let scale = [
        (rect.width(), nw + ne),
        (rect.height(), ne + se),
        (rect.width(), se + sw),
        (rect.height(), sw + nw),
    ]
    .into_iter()
    .filter(|(_, sum)| *sum > 0.0)
    .map(|(length, sum)| length.max(0.0) / sum)
    .fold(1.0_f32, f32::min);
    corners.map(|radius| radius * scale)
}

fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.clamp(0.0, 255.0).round() as u8)
}

fn vertical_span(points: &[Pos2]) -> (f32, f32) {
    points
        .iter()
        .fold((f32::MAX, f32::MIN), |(top, bottom), point| {
            (top.min(point.y), bottom.max(point.y))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_corners_stay_inside_and_never_overlap() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(40.0, 10.0));
        for corners in [[0.0; 4], [5.0; 4], [30.0; 4], [0.0, 0.0, 20.0, 20.0]] {
            let fitted = fit(rect, corners);
            assert!(fitted[0] + fitted[1] <= rect.width() + 1e-3);
            assert!(fitted[1] + fitted[2] <= rect.height() + 1e-3);
            for point in rounded(rect, corners) {
                assert!(
                    rect.expand(1e-3).contains(point),
                    "{point:?} for {corners:?}"
                );
            }
        }
        assert_eq!(rounded(rect, [0.0; 4]).len(), 4);
    }

    #[test]
    fn ellipses_are_turned_around_their_center() {
        let center = egui::pos2(10.0, 20.0);
        let points = ellipse(center, egui::vec2(4.0, 2.0), FRAC_PI_2);
        assert!((points[0] - (center + egui::vec2(0.0, 4.0))).length() < 1e-4);
        let sum = points
            .iter()
            .fold(Vec2::ZERO, |sum, point| sum + point.to_vec2());
        assert!(((sum / points.len() as f32).to_pos2() - center).length() < 1e-3);
    }

    #[test]
    fn clipping_keeps_the_part_inside_the_window() {
        let window = rounded(
            Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 50.0)),
            [10.0; 4],
        );
        let band = [
            egui::pos2(40.0, -20.0),
            egui::pos2(60.0, -20.0),
            egui::pos2(60.0, 80.0),
            egui::pos2(40.0, 80.0),
        ];
        let clipped = clip(&band, &window);
        assert!(clipped.len() >= 4);
        for point in &clipped {
            assert!((40.0 - 1e-3..=60.0 + 1e-3).contains(&point.x), "{point:?}");
            assert!((-1e-3..=50.0 + 1e-3).contains(&point.y), "{point:?}");
        }
        let outside = band.map(|point| point + Vec2::new(300.0, 0.0));
        assert!(clip(&outside, &window).is_empty());
    }

    #[test]
    fn auras_leave_the_inside_of_their_edge_clear() {
        let rect = Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(80.0, 30.0));
        let Shape::Mesh(mesh) = halo(rect, 15.0, Color32::WHITE, 12.0) else {
            panic!("an aura is a mesh");
        };
        // A point strictly inside the edge survives clipping a tiny square around it.
        let interior = rounded(rect.shrink(1.0), [14.0; 4]);
        for vertex in &mesh.vertices {
            let probe = Rect::from_center_size(vertex.pos, Vec2::splat(0.1));
            let corners = [
                probe.left_top(),
                probe.right_top(),
                probe.right_bottom(),
                probe.left_bottom(),
            ];
            assert!(clip(&corners, &interior).is_empty(), "{:?}", vertex.pos);
            assert!(rect.expand(12.0 + 1e-3).contains(vertex.pos));
        }
    }

    #[test]
    fn panes_paint_with_every_tint() {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(egui::pos2(30.0, 40.0), egui::vec2(300.0, 120.0));
        assert!(pane(&ctx, rect, 10.0, &Glaze::clear(&ctx)).len() > 6);
        for tint in [Tint::Rest, Tint::Hover, Tint::Current, Tint::Accent] {
            assert!(!tile(&ctx, rect, tint).is_empty());
        }
    }
}
