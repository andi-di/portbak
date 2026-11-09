//! Glossy Aero icons: bubbly, gradient-lit shapes with a sharp reflection, drawn in place of
//! pixel sprites.
//!
//! Coordinates are fractions of the icon's rectangle, as in `glyphs`; filled parts must be convex.

use crate::app::rendering::glass;
use crate::app::rendering::glyphs::{polar, Pen, Point};
use crate::app::rendering::pixel::{mix, readable_on, shaded_fan, shaded_polygon};
use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::time::Duration;

const WAKE_FRAME: Duration = Duration::from_millis(40);
/// Where a shape's reflection ends, as a fraction of its height.
const GLOSS_LINE: f32 = 0.5;
const ROUND_STEPS: u32 = 28;

/// Paints `sprite` into `rect`; `wake` from 0 to 1 animates icons that react to hover.
pub fn paint(painter: &egui::Painter, rect: Rect, sprite: Sprite, tint: Color32, wake: f32) {
    let palette = Palette::of_ctx(painter.ctx());
    let pen = Pen::new(painter, rect);
    match sprite {
        Sprite::Floppy => eco_bubble(&pen, palette, tint, wake),
        Sprite::Folder | Sprite::JobSettings | Sprite::FolderBackup => folder(&pen, palette),
        Sprite::Clock | Sprite::Weekdays => clock(&pen, palette, tint),
        Sprite::Shield => shield(&pen, palette, tint),
        Sprite::Trash => bin(&pen, palette, tint),
        Sprite::Rewind | Sprite::Interval => back_arrow(&pen, palette, tint),
        Sprite::Play => gel(
            &pen,
            palette,
            &[(0.16, 0.06), (0.94, 0.5), (0.16, 0.94)],
            tint,
        ),
        Sprite::Chest => panes(&pen, palette, tint, wake),
        Sprite::Package => cube(&pen, palette, tint),
        Sprite::Sun | Sprite::Daily => sun(&pen, palette),
        Sprite::Moon => moon(&pen, palette),
        Sprite::Monitor | Sprite::Theme => monitor(&pen, palette, tint),
        Sprite::Warning => warning(&pen, palette, tint),
        Sprite::Power => power(&pen, palette, tint),
        Sprite::Check => tube(
            &pen,
            palette,
            &[(0.08, 0.55), (0.38, 0.88), (0.95, 0.1)],
            tint,
        ),
        Sprite::Cursor => gel(
            &pen,
            palette,
            &[(0.1, 0.05), (0.95, 0.55), (0.1, 0.95)],
            tint,
        ),
        Sprite::Plus => plus(&pen, palette, tint),
        Sprite::Gear => gear(&pen, palette, tint),
    }
}

/// A clear soap bubble holding a fresh leaf; it wobbles, the leaf sways and little bubbles rise
/// on wake. Stands in for a backup job.
fn eco_bubble(pen: &Pen, palette: &Palette, tint: Color32, wake: f32) {
    let time = awake_time(pen, wake);
    let wobble = (time * 7.0).sin() * 0.05 * wake;
    let radii = Vec2::new(0.44 * (1.0 + wobble), 0.44 * (1.0 - wobble));
    let shell: Vec<Point> = (0..ROUND_STEPS)
        .map(|step| {
            let (sin, cos) = (step as f32 * TAU / ROUND_STEPS as f32).sin_cos();
            (0.5 + radii.x * cos, 0.5 + radii.y * sin)
        })
        .collect();
    let center = pen.at((0.5, 0.5));
    let reach = radii.x * pen.rect.width();
    let (clear, rim) = (tint.gamma_multiply(0.12), tint.gamma_multiply(0.45));
    pen.painter
        .add(shaded_fan(center, &pen.points(&shell), |point| {
            clear.lerp_to_gamma(rim, (point.distance(center) / reach).min(1.0))
        }));
    let sway = (time * 3.0).sin() * 0.25 * wake;
    leaf(pen, palette, (0.3, 0.74), 0.5, -0.8 + sway);
    let edge = mix(tint, palette.outline, 0.35);
    pen.painter.add(Shape::closed_line(
        pen.points(&shell),
        pen.stroke(edge, 0.8),
    ));
    reflection(pen, (0.34, 0.27), Vec2::new(0.17, 0.09), -0.6, 0.85);
    pen.circle((0.72, 0.72), 0.04, Color32::WHITE.gamma_multiply(0.6), None);
    if wake > 0.05 {
        for index in 0..2 {
            let rise = (time * 0.8 + index as f32 * 0.5).fract();
            let x = 0.9 + 0.03 * (time * 4.0 + index as f32).sin();
            pen.circle((x, 0.78 - rise * 0.6), 0.045 * wake, rim, Some(edge));
        }
    }
}

/// A lens-shaped leaf from `stem` reaching `length` along `angle`, with a midrib.
fn leaf(pen: &Pen, palette: &Palette, stem: Point, length: f32, angle: f32) {
    const STEPS: u32 = 10;
    let place = |along: f32, across: f32| -> Point {
        let (sin, cos) = angle.sin_cos();
        (
            stem.0 + along * cos - across * sin,
            stem.1 + along * sin + across * cos,
        )
    };
    let side = |sign: f32| -> Vec<Point> {
        (0..=STEPS)
            .map(|step| {
                let t = step as f32 / STEPS as f32;
                place(t * length, sign * (PI * t).sin() * length * 0.3)
            })
            .collect()
    };
    let mut outline = side(-1.0);
    outline.extend(side(1.0).into_iter().rev().skip(1));
    let green = mix(palette.good, palette.gold, 0.2);
    gel(pen, palette, &outline, green);
    let vein = mix(green, palette.outline, 0.45);
    pen.line(&[place(0.0, 0.0), place(length * 0.85, 0.0)], vein, 0.5);
    pen.line(&[place(-0.08, 0.02), place(0.0, 0.0)], vein, 0.8);
}

/// The yellow folder of a desktop, open with a sheet of paper peeking out.
fn folder(pen: &Pen, palette: &Palette) {
    let back = mix(palette.gold, palette.outline, 0.25);
    gel(
        pen,
        palette,
        &[(0.06, 0.12), (0.4, 0.12), (0.48, 0.24), (0.06, 0.24)],
        back,
    );
    gel(
        pen,
        palette,
        &[(0.06, 0.22), (0.94, 0.22), (0.94, 0.86), (0.06, 0.86)],
        back,
    );
    gel(
        pen,
        palette,
        &[(0.16, 0.3), (0.84, 0.3), (0.84, 0.7), (0.16, 0.7)],
        palette.paper,
    );
    gel(
        pen,
        palette,
        &[(0.12, 0.4), (0.98, 0.4), (0.9, 0.92), (0.02, 0.92)],
        palette.gold,
    );
}

fn clock(pen: &Pen, palette: &Palette, tint: Color32) {
    gel(pen, palette, &round((0.5, 0.5), 0.47), tint);
    let face = round((0.5, 0.5), 0.37);
    gel_part(pen, &face, vertical_span(&face), palette.paper);
    let ink = mix(palette.outline, palette.metal, 0.2);
    for index in 0..4 {
        let angle = index as f32 * FRAC_PI_2;
        let tick = [
            polar((0.5, 0.5), 0.27, angle),
            polar((0.5, 0.5), 0.33, angle),
        ];
        pen.line(&tick, ink, 0.7);
    }
    pen.line(&[(0.5, 0.5), (0.5, 0.24)], ink, 0.9);
    pen.line(&[(0.5, 0.5), (0.68, 0.58)], ink, 0.9);
    pen.circle((0.5, 0.5), 0.045, tint, Some(ink));
}

/// A security shield quartered in the tint and gold.
fn shield(pen: &Pen, palette: &Palette, tint: Color32) {
    const OUTLINE: [Point; 5] = [
        (0.12, 0.04),
        (0.88, 0.04),
        (0.88, 0.46),
        (0.5, 0.97),
        (0.12, 0.46),
    ];
    let span = vertical_span(&OUTLINE);
    let quarters: [(&[Point], Color32); 4] = [
        (
            &[(0.12, 0.04), (0.5, 0.04), (0.5, 0.42), (0.12, 0.42)],
            tint,
        ),
        (
            &[(0.5, 0.04), (0.88, 0.04), (0.88, 0.42), (0.5, 0.42)],
            palette.gold,
        ),
        (
            &[(0.12, 0.42), (0.5, 0.42), (0.5, 0.97), (0.12, 0.46)],
            palette.gold,
        ),
        (
            &[(0.5, 0.42), (0.88, 0.42), (0.88, 0.46), (0.5, 0.97)],
            tint,
        ),
    ];
    for (quarter, color) in quarters {
        gel_part(pen, quarter, span, color);
    }
    outline(pen, palette, &OUTLINE, tint);
}

/// A ribbed glass recycling bin; remove.
fn bin(pen: &Pen, palette: &Palette, tint: Color32) {
    let glass = mix(tint, palette.metal, 0.5);
    gel(
        pen,
        palette,
        &[(0.2, 0.26), (0.8, 0.26), (0.72, 0.95), (0.28, 0.95)],
        glass,
    );
    let (rib, shine) = (edge_of(palette, glass), Color32::WHITE.gamma_multiply(0.45));
    for x in [0.4, 0.5, 0.6] {
        pen.line(&[(x, 0.34), (x, 0.88)], rib, 0.5);
        pen.line(&[(x + 0.03, 0.34), (x + 0.03, 0.88)], shine, 0.4);
    }
    gel(
        pen,
        palette,
        &[(0.12, 0.13), (0.88, 0.13), (0.88, 0.26), (0.12, 0.26)],
        glass,
    );
    gel(
        pen,
        palette,
        &[(0.38, 0.04), (0.62, 0.04), (0.62, 0.13), (0.38, 0.13)],
        glass,
    );
}

/// A curved arrow sweeping back to the left; back and restore.
fn back_arrow(pen: &Pen, palette: &Palette, tint: Color32) {
    const STEPS: u32 = 16;
    let center = (0.52, 0.56);
    let mut path = vec![(0.3, 0.86)];
    path.extend(
        (0..=STEPS).map(|step| polar(center, 0.3, FRAC_PI_2 - step as f32 * PI / STEPS as f32)),
    );
    path.push((0.32, 0.26));
    tube(pen, palette, &path, tint);
    gel(
        pen,
        palette,
        &[(0.04, 0.26), (0.36, 0.04), (0.36, 0.48)],
        tint,
    );
}

/// Three stacked glass panes whose top pane lifts on wake; manage backups.
fn panes(pen: &Pen, palette: &Palette, tint: Color32, wake: f32) {
    let shades = [
        mix(tint, palette.outline, 0.3),
        tint,
        mix(tint, Color32::WHITE, 0.2),
    ];
    for (index, color) in shades.into_iter().enumerate() {
        let lift = if index == 2 { wake * 0.12 } else { 0.0 };
        let (dx, top) = (lift * 0.6, 0.62 - index as f32 * 0.22 - lift);
        let pane = [
            (0.16 + dx, top),
            (0.96 + dx, top),
            (0.84 + dx, top + 0.24),
            (0.04 + dx, top + 0.24),
        ];
        gel(pen, palette, &pane, color);
    }
}

/// A glossy box with a zipper down its front edge; ZIP archives.
fn cube(pen: &Pen, palette: &Palette, tint: Color32) {
    let faces: [(&[Point], Color32); 3] = [
        (
            &[(0.5, 0.06), (0.92, 0.26), (0.5, 0.46), (0.08, 0.26)],
            mix(tint, Color32::WHITE, 0.35),
        ),
        (
            &[(0.08, 0.26), (0.5, 0.46), (0.5, 0.96), (0.08, 0.76)],
            tint,
        ),
        (
            &[(0.5, 0.46), (0.92, 0.26), (0.92, 0.76), (0.5, 0.96)],
            mix(tint, Color32::BLACK, 0.2),
        ),
    ];
    for (face, color) in faces {
        gel(pen, palette, face, color);
    }
    for index in 0..6 {
        let y = 0.52 + index as f32 * 0.07;
        pen.line(&[(0.45, y), (0.55, y)], palette.paper, 0.6);
    }
    pen.circle((0.5, 0.48), 0.05, palette.metal, Some(palette.outline));
}

fn sun(pen: &Pen, palette: &Palette) {
    let center = (0.5, 0.5);
    pen.circle(center, 0.48, palette.gold.gamma_multiply(0.15), None);
    for index in 0..12 {
        let angle = index as f32 * TAU / 12.0;
        let reach = if index % 2 == 0 { 0.5 } else { 0.42 };
        let ray = [
            polar(center, 0.26, angle - 0.14),
            polar(center, reach, angle),
            polar(center, 0.26, angle + 0.14),
        ];
        let ray = pen.points(&ray);
        let color = mix(palette.gold, Color32::WHITE, 0.2).gamma_multiply(0.85);
        pen.painter
            .add(Shape::convex_polygon(ray, color, egui::Stroke::NONE));
    }
    let radius = 0.26 * pen.rect.width();
    glass::sphere(pen.painter, pen.at(center), radius, palette.gold, 0.45);
}

/// A crescent lit from above, with two small stars.
fn moon(pen: &Pen, palette: &Palette) {
    const STEPS: u32 = 24;
    let center = (0.46, 0.54);
    let (top, bottom) = (pen.at((0.0, 0.08)).y, pen.at((0.0, 0.98)).y);
    let light = mix(palette.gold, Color32::WHITE, 0.55);
    let shade = |point: Pos2| {
        light.lerp_to_gamma(
            palette.gold,
            ((point.y - top) / (bottom - top)).clamp(0.0, 1.0),
        )
    };
    let mut mesh = egui::Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let angle = -0.05 * PI + t * 1.6 * PI;
        let thickness = 0.26 * (PI * t).sin();
        for radius in [0.42, 0.42 - thickness] {
            let point = pen.at(polar(center, radius, angle));
            mesh.colored_vertex(point, shade(point));
        }
        if step > 0 {
            let last = step * 2;
            mesh.add_triangle(last - 2, last - 1, last);
            mesh.add_triangle(last - 1, last, last + 1);
        }
    }
    pen.painter.add(mesh);
    for (center, size) in [((0.78, 0.2), 0.09), ((0.88, 0.46), 0.06)] {
        twinkle(pen, center, size, palette.paper);
    }
}

/// A flat screen showing an aurora sky, on a small stand; status and system.
fn monitor(pen: &Pen, palette: &Palette, tint: Color32) {
    let frame = Rect::from_two_pos(pen.at((0.04, 0.06)), pen.at((0.96, 0.72)));
    let corner = frame.height() * 0.12;
    let dark = mix(palette.outline, palette.metal, 0.3);
    let bezel = glass::gradient(frame, [corner; 4], mix(dark, Color32::WHITE, 0.2), dark);
    pen.painter.add(bezel);
    let screen = Rect::from_two_pos(pen.at((0.1, 0.13)), pen.at((0.9, 0.64)));
    let deep = mix(tint, palette.outline, 0.35);
    let sky = glass::gradient(
        screen,
        [corner * 0.4; 4],
        mix(tint, Color32::WHITE, 0.35),
        deep,
    );
    pen.painter.add(sky);
    let swoosh = [(0.12, 0.5), (0.35, 0.38), (0.6, 0.44), (0.88, 0.28)];
    pen.line(&swoosh, mix(palette.hot, Color32::WHITE, 0.4), 0.8);
    let glare = pen.points(&[(0.1, 0.13), (0.9, 0.13), (0.9, 0.28), (0.1, 0.44)]);
    let glare_color = Color32::WHITE.gamma_multiply(0.3);
    pen.painter.add(Shape::convex_polygon(
        glare,
        glare_color,
        egui::Stroke::NONE,
    ));
    gel(
        pen,
        palette,
        &[(0.44, 0.72), (0.56, 0.72), (0.6, 0.84), (0.4, 0.84)],
        palette.metal,
    );
    gel(
        pen,
        palette,
        &[(0.22, 0.84), (0.78, 0.84), (0.84, 0.95), (0.16, 0.95)],
        palette.metal,
    );
}

fn warning(pen: &Pen, palette: &Palette, tint: Color32) {
    gel(
        pen,
        palette,
        &[(0.5, 0.04), (0.97, 0.92), (0.03, 0.92)],
        tint,
    );
    let ink = readable_on(tint, palette);
    pen.line(&[(0.5, 0.36), (0.5, 0.62)], ink, 1.5);
    pen.circle((0.5, 0.77), 0.06, ink, None);
}

/// The power symbol as a glossy tube; automatic backups on and off.
fn power(pen: &Pen, palette: &Palette, tint: Color32) {
    const STEPS: u32 = 24;
    let gap = 0.65;
    let ring: Vec<Point> = (0..=STEPS)
        .map(|step| {
            let angle = -FRAC_PI_2 + gap + step as f32 * (TAU - 2.0 * gap) / STEPS as f32;
            polar((0.5, 0.56), 0.36, angle)
        })
        .collect();
    tube(pen, palette, &ring, tint);
    tube(pen, palette, &[(0.5, 0.06), (0.5, 0.5)], tint);
}

/// A plus with three seamless glossy parts; new job.
fn plus(pen: &Pen, palette: &Palette, tint: Color32) {
    let (near, far) = (0.36, 0.64);
    let parts: [&[Point]; 3] = [
        &[(near, 0.06), (far, 0.06), (far, 0.94), (near, 0.94)],
        &[(0.06, near), (near, near), (near, far), (0.06, far)],
        &[(far, near), (0.94, near), (0.94, far), (far, far)],
    ];
    for part in parts {
        gel_part(pen, part, (0.06, 0.94), tint);
    }
    let cross = [
        (near, 0.06),
        (far, 0.06),
        (far, near),
        (0.94, near),
        (0.94, far),
        (far, far),
        (far, 0.94),
        (near, 0.94),
        (near, far),
        (0.06, far),
        (0.06, near),
        (near, near),
    ];
    outline(pen, palette, &cross, tint);
}

/// A polished cog wheel; settings.
fn gear(pen: &Pen, palette: &Palette, tint: Color32) {
    let center = (0.5, 0.5);
    let steel = mix(tint, palette.metal, 0.5);
    for index in 0..8 {
        let angle = index as f32 * TAU / 8.0;
        let tooth = [
            polar(center, 0.3, angle - 0.3),
            polar(center, 0.49, angle - 0.17),
            polar(center, 0.49, angle + 0.17),
            polar(center, 0.3, angle + 0.3),
        ];
        gel(pen, palette, &tooth, steel);
    }
    gel(pen, palette, &round(center, 0.35), steel);
    let hole = mix(palette.outline, steel, 0.3);
    pen.circle(center, 0.12, hole, Some(edge_of(palette, steel)));
}

/// One convex part with its lit gradient, reflection and outline.
fn gel(pen: &Pen, palette: &Palette, part: &[Point], tint: Color32) {
    gel_part(pen, part, vertical_span(part), tint);
    outline(pen, palette, part, tint);
}

/// Fills a convex part with a lit gradient and a reflection keyed to `span`, the vertical extent
/// of the whole shape, so parts of one shape join without seams.
fn gel_part(pen: &Pen, part: &[Point], span: (f32, f32), tint: Color32) {
    let points = pen.points(part);
    let (top, bottom) = (pen.at((0.0, span.0)).y, pen.at((0.0, span.1)).y);
    let height = (bottom - top).max(1.0);
    let light = mix(tint, Color32::WHITE, 0.35);
    let deep = mix(tint, Color32::BLACK, 0.18);
    let along = |point: Pos2| ((point.y - top) / height).clamp(0.0, 1.0);
    pen.painter.add(shaded_polygon(&points, |point| {
        light.lerp_to_gamma(deep, along(point))
    }));
    let upper = clip_below(&points, top + height * GLOSS_LINE);
    pen.painter.add(shaded_polygon(&upper, |point| {
        Color32::WHITE.gamma_multiply(0.6 - 0.9 * along(point))
    }));
}

/// A glossy tube along `path`: a dark rim, the tinted body and a thin highlight above.
fn tube(pen: &Pen, palette: &Palette, path: &[Point], tint: Color32) {
    pen.line(path, edge_of(palette, tint), 2.6);
    pen.line(path, tint, 1.7);
    let raised: Vec<Point> = path.iter().map(|(x, y)| (*x, y - 0.035)).collect();
    pen.line(&raised, Color32::WHITE.gamma_multiply(0.55), 0.5);
}

fn outline(pen: &Pen, palette: &Palette, points: &[Point], tint: Color32) {
    pen.painter.add(Shape::closed_line(
        pen.points(points),
        pen.stroke(edge_of(palette, tint), 0.7),
    ));
}

fn edge_of(palette: &Palette, tint: Color32) -> Color32 {
    mix(tint, palette.outline, 0.55)
}

/// A curved white reflection, as on a bubble or glass ball.
fn reflection(pen: &Pen, center: Point, radii: Vec2, angle: f32, strength: f32) {
    let scale = Vec2::new(pen.rect.width(), pen.rect.height());
    let points = glass::ellipse(pen.at(center), radii * scale, angle);
    let color = Color32::WHITE.gamma_multiply(strength);
    pen.painter
        .add(Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// A thin four-pointed sparkle.
fn twinkle(pen: &Pen, center: Point, size: f32, color: Color32) {
    let (x, y) = center;
    for (reach, thin) in [
        ((size, 0.0), (0.0, size * 0.25)),
        ((0.0, size), (size * 0.25, 0.0)),
    ] {
        let spike = [
            (x - reach.0, y - reach.1),
            (x + thin.0, y - thin.1),
            (x + reach.0, y + reach.1),
            (x - thin.0, y + thin.1),
        ];
        let spike = pen.points(&spike);
        pen.painter
            .add(Shape::convex_polygon(spike, color, egui::Stroke::NONE));
    }
}

/// A circle of `radius` (a fraction of the width) as polygon points.
fn round(center: Point, radius: f32) -> Vec<Point> {
    (0..ROUND_STEPS)
        .map(|step| polar(center, radius, step as f32 * TAU / ROUND_STEPS as f32))
        .collect()
}

/// The clock for wake animations, keeping the window repainting while awake.
fn awake_time(pen: &Pen, wake: f32) -> f32 {
    if wake <= 0.0 {
        return 0.0;
    }
    let ctx = pen.painter.ctx();
    ctx.request_repaint_after(WAKE_FRAME);
    ctx.input(|input| input.time) as f32
}

/// The part of a convex polygon above the horizontal line at `y`.
fn clip_below(points: &[Pos2], y: f32) -> Vec<Pos2> {
    let mut kept = Vec::new();
    for (index, &from) in points.iter().enumerate() {
        let to = points[(index + 1) % points.len()];
        if from.y <= y {
            kept.push(from);
        }
        if (from.y <= y) != (to.y <= y) {
            kept.push(from + (to - from) * ((y - from.y) / (to.y - from.y)));
        }
    }
    kept
}

fn vertical_span(points: &[Point]) -> (f32, f32) {
    points
        .iter()
        .fold((f32::MAX, f32::MIN), |(top, bottom), (_, y)| {
            (top.min(*y), bottom.max(*y))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::rendering::glyphs;
    use crate::app::theme::skins;
    use crate::app::theme::style;

    #[test]
    fn every_icon_paints_in_both_palettes() {
        let context = egui::Context::default();
        style::install(&context, skins::find("frutiger-aero"));
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            context.set_theme(theme);
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                let ctx = &ui.ctx().clone();
                let painter = ctx.layer_painter(egui::LayerId::background());
                let palette = Palette::of_ctx(ctx);
                for sprite in Sprite::ALL {
                    let rect = Rect::from_min_size(Pos2::ZERO, glyphs::grid(sprite) * 2.0);
                    for (tint, wake) in [
                        (palette.accent, 0.0),
                        (palette.paper, 0.5),
                        (palette.good, 1.0),
                    ] {
                        paint(&painter, rect, sprite, tint, wake);
                    }
                }
            });
        }
    }

    #[test]
    fn clipping_keeps_only_the_part_above_the_line() {
        let square = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(0.0, 10.0),
        ];
        let upper = clip_below(&square, 4.0);
        assert_eq!(upper.len(), 4);
        assert!(upper.iter().all(|point| point.y <= 4.0));
        assert!(upper.contains(&Pos2::new(10.0, 4.0)) && upper.contains(&Pos2::new(0.0, 4.0)));
        assert!(clip_below(&square, -1.0).is_empty());
        assert_eq!(clip_below(&square, 20.0), square.to_vec());
    }
}
