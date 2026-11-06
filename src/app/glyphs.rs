//! Engraved vector icons: smooth line art with dark bodies, drawn in place of pixel sprites.
//!
//! Coordinates are fractions of the icon's rectangle; shapes passed to `fill` must be convex.

use super::pixel::mix;
use super::sprites::Sprite;
use super::style::Palette;
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::{PI, TAU};
use std::time::Duration;

const FLICKER_FRAME: Duration = Duration::from_millis(50);

type Point = (f32, f32);

/// Grid size in sprite cells, so callers' scales give comparable sizes to the pixel set.
pub fn grid(sprite: Sprite) -> Vec2 {
    match sprite {
        Sprite::Check => egui::vec2(7.0, 6.0),
        Sprite::Cursor => egui::vec2(4.0, 7.0),
        _ => egui::vec2(12.0, 12.0),
    }
}

/// Paints `sprite` into `rect`; `wake` from 0 to 1 animates icons that react to hover.
pub fn paint(painter: &egui::Painter, rect: Rect, sprite: Sprite, tint: Color32, wake: f32) {
    let palette = Palette::of_ctx(painter.ctx());
    let pen = Pen::new(painter, rect);
    let dark = mix(tint, palette.outline, 0.62);
    match sprite {
        Sprite::Floppy => tome(&pen, palette, tint, dark),
        Sprite::Folder => scroll(&pen, palette, tint),
        Sprite::Clock => hourglass(&pen, palette, tint, dark),
        Sprite::Shield => shield(&pen, palette, tint, dark),
        Sprite::Trash => skull(&pen, palette),
        Sprite::Rewind => chevrons(&pen, tint),
        Sprite::Play => sword(&pen, palette, tint),
        Sprite::Chest => mimic(&pen, palette, tint, wake),
        Sprite::Package => urn(&pen, palette, tint, dark),
        Sprite::Sun => sun(&pen, palette),
        Sprite::Moon => moon(&pen, palette),
        Sprite::Monitor => eye(&pen, palette, tint),
        Sprite::Warning => warning(&pen, palette, tint),
        Sprite::Power => bonfire(&pen, palette),
        Sprite::Check => pen.line(&[(0.08, 0.55), (0.38, 0.88), (0.95, 0.1)], tint, 1.6),
        Sprite::Cursor => pen.fill(&[(0.1, 0.1), (0.95, 0.5), (0.1, 0.9)], tint),
        Sprite::Plus => cross(&pen, tint),
        Sprite::Gear => wheel(&pen, tint, dark),
    }
}

/// Maps fractional coordinates into the icon rectangle with a line width matching its size.
struct Pen<'a> {
    painter: &'a egui::Painter,
    rect: Rect,
    width: f32,
}

impl<'a> Pen<'a> {
    fn new(painter: &'a egui::Painter, rect: Rect) -> Self {
        let width = (rect.width().min(rect.height()) / 12.0).clamp(1.0, 5.0);
        Self {
            painter,
            rect,
            width,
        }
    }

    fn at(&self, (x, y): Point) -> Pos2 {
        self.rect.min + egui::vec2(x * self.rect.width(), y * self.rect.height())
    }

    fn points(&self, points: &[Point]) -> Vec<Pos2> {
        points.iter().map(|point| self.at(*point)).collect()
    }

    fn stroke(&self, color: Color32, weight: f32) -> Stroke {
        Stroke::new(self.width * weight, color)
    }

    fn fill(&self, points: &[Point], color: Color32) {
        let shape = Shape::convex_polygon(self.points(points), color, Stroke::NONE);
        self.painter.add(shape);
    }

    fn shape(&self, points: &[Point], fill: Color32, edge: Color32) {
        let shape = Shape::convex_polygon(self.points(points), fill, self.stroke(edge, 1.0));
        self.painter.add(shape);
    }

    fn line(&self, points: &[Point], color: Color32, weight: f32) {
        let stroke = self.stroke(color, weight);
        self.painter.add(Shape::line(self.points(points), stroke));
    }

    /// A rectangle with corners rounded in proportion to its short side.
    fn rect(&self, min: Point, max: Point, fill: Color32, edge: Option<Color32>) {
        let rect = Rect::from_two_pos(self.at(min), self.at(max));
        let radius = rect.width().min(rect.height()) * 0.22;
        self.painter.rect_filled(rect, radius, fill);
        if let Some(edge) = edge {
            let stroke = self.stroke(edge, 1.0);
            self.painter
                .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
        }
    }

    /// A circle whose radius is a fraction of the icon width.
    fn circle(&self, center: Point, radius: f32, fill: Color32, edge: Option<Color32>) {
        let stroke = edge.map_or(Stroke::NONE, |edge| self.stroke(edge, 1.0));
        let radius = radius * self.rect.width();
        self.painter
            .add(Shape::circle_filled(self.at(center), radius, fill));
        if edge.is_some() {
            self.painter.circle_stroke(self.at(center), radius, stroke);
        }
    }
}

fn polar(center: Point, radius: f32, angle: f32) -> Point {
    (
        center.0 + radius * angle.cos(),
        center.1 + radius * angle.sin(),
    )
}

/// A closed book with a gem on the cover; stands in for stored data.
fn tome(pen: &Pen, palette: &Palette, tint: Color32, dark: Color32) {
    let pages = mix(palette.paper, palette.gold, 0.3);
    pen.rect((0.2, 0.1), (0.92, 0.9), pages, Some(palette.outline));
    pen.rect((0.1, 0.04), (0.84, 0.96), dark, Some(tint));
    pen.line(&[(0.24, 0.04), (0.24, 0.96)], tint, 1.0);
    for corner in [
        [(0.84, 0.04), (0.84, 0.2), (0.68, 0.04)],
        [(0.84, 0.96), (0.68, 0.96), (0.84, 0.8)],
    ] {
        pen.fill(&corner, palette.gold);
    }
    pen.shape(
        &[(0.54, 0.28), (0.7, 0.5), (0.54, 0.72), (0.38, 0.5)],
        palette.gold,
        palette.outline,
    );
    pen.circle((0.54, 0.5), 0.05, palette.bad, None);
}

/// A rolled parchment scroll.
fn scroll(pen: &Pen, palette: &Palette, tint: Color32) {
    let parchment = mix(palette.paper, palette.gold, 0.35);
    pen.rect((0.18, 0.16), (0.82, 0.84), parchment, Some(palette.outline));
    let ink = mix(parchment, palette.outline, 0.55);
    for (y, right) in [(0.36, 0.7), (0.5, 0.7), (0.64, 0.56)] {
        pen.line(&[(0.3, y), (right, y)], ink, 0.7);
    }
    for (top, bottom) in [(0.06, 0.22), (0.78, 0.94)] {
        pen.rect((0.08, top), (0.92, bottom), tint, Some(palette.outline));
    }
}

fn hourglass(pen: &Pen, palette: &Palette, tint: Color32, dark: Color32) {
    pen.shape(&[(0.24, 0.12), (0.76, 0.12), (0.5, 0.5)], dark, tint);
    pen.shape(&[(0.5, 0.5), (0.76, 0.88), (0.24, 0.88)], dark, tint);
    pen.fill(&[(0.36, 0.22), (0.64, 0.22), (0.5, 0.42)], palette.gold);
    pen.fill(&[(0.5, 0.6), (0.7, 0.86), (0.3, 0.86)], palette.gold);
    for x in [0.16, 0.84] {
        pen.line(&[(x, 0.1), (x, 0.9)], tint, 1.0);
    }
    for (top, bottom) in [(0.02, 0.12), (0.88, 0.98)] {
        pen.rect((0.08, top), (0.92, bottom), tint, None);
    }
}

fn shield(pen: &Pen, palette: &Palette, tint: Color32, dark: Color32) {
    pen.shape(
        &[
            (0.1, 0.04),
            (0.9, 0.04),
            (0.9, 0.46),
            (0.5, 0.97),
            (0.1, 0.46),
        ],
        dark,
        tint,
    );
    pen.line(&[(0.5, 0.14), (0.5, 0.8)], palette.gold, 1.4);
    pen.line(&[(0.24, 0.34), (0.76, 0.34)], palette.gold, 1.4);
}

/// A skull with ember eyes; the delete icon.
fn skull(pen: &Pen, palette: &Palette) {
    let bone = mix(palette.paper, palette.gold, 0.15);
    pen.circle((0.5, 0.42), 0.38, bone, Some(palette.outline));
    pen.rect((0.3, 0.62), (0.7, 0.94), bone, Some(palette.outline));
    for x in [0.34, 0.66] {
        pen.circle((x, 0.45), 0.12, palette.outline, None);
        pen.circle((x, 0.46), 0.04, palette.hot, None);
    }
    pen.fill(&[(0.5, 0.58), (0.56, 0.7), (0.44, 0.7)], palette.outline);
    for x in [0.41, 0.5, 0.59] {
        pen.line(&[(x, 0.76), (x, 0.92)], palette.outline, 0.7);
    }
}

/// Two barbed chevrons pointing left; back and restore.
fn chevrons(pen: &Pen, tint: Color32) {
    for offset in [0.0, 0.44] {
        let tip = (0.06 + offset, 0.5);
        pen.line(
            &[(0.44 + offset, 0.1), tip, (0.44 + offset, 0.9)],
            tint,
            2.2,
        );
        pen.fill(
            &[
                (0.0 + offset, 0.5),
                (0.16 + offset, 0.36),
                (0.16 + offset, 0.64),
            ],
            tint,
        );
    }
}

/// A sword raised to the upper right; run now.
fn sword(pen: &Pen, palette: &Palette, tint: Color32) {
    let turn = |points: &[Point]| -> Vec<Point> {
        let (sin, cos) = (-PI / 4.0).sin_cos();
        points
            .iter()
            .map(|(x, y)| {
                let (dx, dy) = (x - 0.5, y - 0.5);
                (0.5 + dx * cos - dy * sin, 0.5 + dx * sin + dy * cos)
            })
            .collect()
    };
    let blade = [
        (0.32, 0.43),
        (0.86, 0.43),
        (1.0, 0.5),
        (0.86, 0.57),
        (0.32, 0.57),
    ];
    pen.shape(&turn(&blade), tint, palette.outline);
    pen.line(&turn(&[(0.36, 0.5), (0.8, 0.5)]), palette.outline, 0.6);
    let guard = [(0.24, 0.26), (0.32, 0.26), (0.32, 0.74), (0.24, 0.74)];
    pen.shape(&turn(&guard), palette.gold, palette.outline);
    let grip = [(0.08, 0.45), (0.24, 0.45), (0.24, 0.55), (0.08, 0.55)];
    pen.fill(&turn(&grip), mix(palette.gold, palette.outline, 0.55));
    pen.circle(turn(&[(0.05, 0.5)])[0], 0.06, palette.gold, None);
}

/// A treasure chest that wakes into a mimic: the lid lifts, fangs and tongue come out, eyes
/// open, and once fully awake it chomps. Manage backups.
fn mimic(pen: &Pen, palette: &Palette, tint: Color32, wake: f32) {
    let ctx = pen.painter.ctx();
    let chomp = if wake > 0.0 {
        ctx.request_repaint_after(FLICKER_FRAME);
        let time = ctx.input(|input| input.time) as f32;
        0.75 + 0.25 * (time * 11.0).sin() * wake
    } else {
        0.0
    };
    let gap = 0.2 * wake * chomp;
    let rim = 0.52;
    let lid_bottom = rim - gap;
    let wood = mix(palette.gold, palette.outline, 0.62);
    if gap > 0.01 {
        pen.rect(
            (0.1, lid_bottom),
            (0.9, rim + 0.04),
            mix(palette.accent, palette.outline, 0.6),
            None,
        );
    }
    pen.rect((0.06, rim), (0.94, 0.95), wood, Some(tint));
    let fang = 0.11 * wake;
    if fang > 0.005 {
        for index in 0..4 {
            let x = 0.24 + index as f32 * 0.17;
            pen.fill(
                &[
                    (x - 0.05, rim + 0.01),
                    (x + 0.05, rim + 0.01),
                    (x, rim - fang),
                ],
                palette.paper,
            );
        }
    }
    let mut lid = vec![(0.94, lid_bottom), (0.06, lid_bottom)];
    lid.extend((0..=10).map(|step| {
        let angle = PI + step as f32 * PI / 10.0;
        (
            0.5 + 0.44 * angle.cos(),
            lid_bottom - 0.14 + 0.22 * angle.sin(),
        )
    }));
    pen.shape(&lid, wood, tint);
    if fang > 0.005 {
        for index in 0..5 {
            let x = 0.16 + index as f32 * 0.17;
            pen.fill(
                &[
                    (x - 0.06, lid_bottom - 0.01),
                    (x + 0.06, lid_bottom - 0.01),
                    (x, lid_bottom + fang * 1.2),
                ],
                palette.paper,
            );
        }
    }
    for (left, right) in [(0.18, 0.26), (0.74, 0.82)] {
        pen.rect(
            (left, lid_bottom - 0.3),
            (right, lid_bottom),
            palette.gold,
            None,
        );
        pen.rect((left, rim), (right, 0.95), palette.gold, None);
    }
    pen.rect(
        (0.4, rim + 0.08),
        (0.6, rim + 0.28),
        palette.gold,
        Some(palette.outline),
    );
    pen.circle((0.5, rim + 0.16), 0.03, palette.outline, None);
    if wake > 0.05 {
        let tip = rim + 0.1 + 0.32 * wake;
        pen.shape(
            &[
                (0.54, rim - 0.02),
                (0.72, rim - 0.02),
                (0.74, tip - 0.08),
                (0.65, tip),
                (0.56, tip - 0.08),
            ],
            mix(palette.bad, palette.paper, 0.25),
            palette.outline,
        );
        for x in [0.36, 0.64] {
            let center = (x, lid_bottom - 0.17);
            pen.circle(center, 0.075 * wake, palette.gold, Some(palette.outline));
            pen.rect(
                (x - 0.012, center.1 - 0.06 * wake),
                (x + 0.012, center.1 + 0.06 * wake),
                palette.outline,
                None,
            );
        }
    }
}

/// A sealed urn; compressed archives.
fn urn(pen: &Pen, palette: &Palette, tint: Color32, dark: Color32) {
    pen.shape(
        &[
            (0.3, 0.26),
            (0.7, 0.26),
            (0.88, 0.5),
            (0.74, 0.84),
            (0.26, 0.84),
            (0.12, 0.5),
        ],
        dark,
        tint,
    );
    pen.rect((0.36, 0.12), (0.64, 0.26), dark, Some(tint));
    pen.rect((0.3, 0.84), (0.7, 0.96), dark, Some(tint));
    pen.circle((0.5, 0.07), 0.06, palette.gold, None);
    pen.line(&[(0.15, 0.5), (0.85, 0.5)], palette.gold, 1.2);
}

fn sun(pen: &Pen, palette: &Palette) {
    let center = (0.5, 0.5);
    for index in 0..12 {
        let angle = index as f32 * TAU / 12.0;
        let reach = if index % 2 == 0 { 0.5 } else { 0.4 };
        pen.fill(
            &[
                polar(center, 0.22, angle - 0.2),
                polar(center, reach, angle),
                polar(center, 0.22, angle + 0.2),
            ],
            palette.gold,
        );
    }
    pen.circle(
        center,
        0.22,
        palette.gold,
        Some(mix(palette.gold, palette.outline, 0.45)),
    );
}

/// A crescent whose horns taper to points, built as a strip between two arcs.
fn moon(pen: &Pen, palette: &Palette) {
    const STEPS: u32 = 24;
    let center = (0.5, 0.5);
    let mut mesh = Mesh::default();
    for step in 0..=STEPS {
        let t = step as f32 / STEPS as f32;
        let angle = -0.05 * PI + t * 1.6 * PI;
        let thickness = 0.28 * (PI * t).sin();
        mesh.colored_vertex(pen.at(polar(center, 0.44, angle)), palette.gold);
        mesh.colored_vertex(pen.at(polar(center, 0.44 - thickness, angle)), palette.gold);
        if step > 0 {
            let last = step * 2;
            mesh.add_triangle(last - 2, last - 1, last);
            mesh.add_triangle(last - 1, last, last + 1);
        }
    }
    pen.painter.add(mesh);
}

/// A watching eye with a slit pupil; status and system.
fn eye(pen: &Pen, palette: &Palette, tint: Color32) {
    const STEPS: u32 = 16;
    let lid = |sign: f32| -> Vec<Point> {
        (0..=STEPS)
            .map(|step| {
                let t = step as f32 / STEPS as f32;
                (0.04 + 0.92 * t, 0.54 + sign * 0.3 * (PI * t).sin())
            })
            .collect()
    };
    let mut almond = lid(-1.0);
    almond.extend(lid(1.0).into_iter().rev().skip(1));
    pen.shape(&almond, mix(palette.paper, palette.outline, 0.75), tint);
    pen.circle((0.5, 0.54), 0.17, tint, Some(palette.outline));
    pen.rect((0.47, 0.4), (0.53, 0.68), palette.outline, None);
    pen.circle((0.57, 0.47), 0.03, palette.paper, None);
    for (from, to) in [
        ((0.28, 0.32), (0.22, 0.14)),
        ((0.5, 0.24), (0.5, 0.04)),
        ((0.72, 0.32), (0.78, 0.14)),
    ] {
        pen.line(&[from, to], tint, 1.0);
    }
}

fn warning(pen: &Pen, palette: &Palette, tint: Color32) {
    pen.shape(
        &[(0.5, 0.04), (0.96, 0.92), (0.04, 0.92)],
        mix(tint, palette.outline, 0.7),
        tint,
    );
    pen.line(&[(0.5, 0.34), (0.5, 0.64)], tint, 1.6);
    pen.circle((0.5, 0.78), 0.06, tint, None);
}

/// A flickering fire on crossed logs; automatic backups on and off.
fn bonfire(pen: &Pen, palette: &Palette) {
    let ctx = pen.painter.ctx();
    let time = ctx.input(|input| input.time) as f32;
    ctx.request_repaint_after(FLICKER_FRAME);
    let flicker = (time * 9.0).sin() * 0.05 + (time * 13.7).sin() * 0.03;
    let lean = (time * 5.3).sin() * 0.04;
    let stone = mix(palette.metal, palette.outline, 0.45);
    for x in [0.12, 0.3, 0.5, 0.7, 0.88] {
        pen.circle((x, 0.92), 0.08, stone, Some(palette.outline));
    }
    let wood = mix(palette.gold, palette.outline, 0.55);
    pen.line(&[(0.12, 0.9), (0.88, 0.68)], palette.outline, 3.4);
    pen.line(&[(0.12, 0.68), (0.88, 0.9)], palette.outline, 3.4);
    pen.line(&[(0.14, 0.89), (0.86, 0.69)], wood, 2.0);
    pen.line(&[(0.14, 0.69), (0.86, 0.89)], wood, 2.0);
    for (x, height) in [(0.3, 0.34), (0.7, 0.3)] {
        pen.fill(&flame((x, 0.74), height - flicker, 0.1, -lean), palette.hot);
    }
    let layers = [
        (0.7, 0.26, palette.hot, Some(palette.outline)),
        (0.46, 0.16, palette.gold, None),
        (0.22, 0.07, mix(palette.paper, palette.gold, 0.3), None),
    ];
    for (height, half, color, edge) in layers {
        let points = flame((0.5, 0.76), height + flicker, half, lean);
        match edge {
            Some(edge) => pen.shape(&points, color, edge),
            None => pen.fill(&points, color),
        }
    }
}

/// A convex teardrop rising from `base`; `lean` shifts the tip sideways.
fn flame(base: Point, height: f32, half: f32, lean: f32) -> [Point; 8] {
    let (x, y) = base;
    [
        (x + lean, y - height),
        (x + half * 0.75 + lean * 0.5, y - height * 0.62),
        (x + half, y - height * 0.32),
        (x + half * 0.8, y - height * 0.08),
        (x, y),
        (x - half * 0.8, y - height * 0.08),
        (x - half, y - height * 0.32),
        (x - half * 0.75 + lean * 0.5, y - height * 0.62),
    ]
}

/// A cross with flared arms; new job.
fn cross(pen: &Pen, tint: Color32) {
    for arm in [
        [(0.3, 0.02), (0.7, 0.02), (0.56, 0.44), (0.44, 0.44)],
        [(0.44, 0.56), (0.56, 0.56), (0.7, 0.98), (0.3, 0.98)],
        [(0.02, 0.3), (0.44, 0.44), (0.44, 0.56), (0.02, 0.7)],
        [(0.56, 0.44), (0.98, 0.3), (0.98, 0.7), (0.56, 0.56)],
    ] {
        pen.fill(&arm, tint);
    }
    pen.rect((0.43, 0.43), (0.57, 0.57), tint, None);
}

/// A spiked wheel; settings.
fn wheel(pen: &Pen, tint: Color32, dark: Color32) {
    let center = (0.5, 0.5);
    for index in 0..8 {
        let angle = index as f32 * TAU / 8.0;
        pen.fill(
            &[
                polar(center, 0.22, angle - 0.36),
                polar(center, 0.5, angle),
                polar(center, 0.22, angle + 0.36),
            ],
            tint,
        );
    }
    pen.circle(center, 0.25, dark, Some(tint));
    pen.circle(center, 0.1, tint, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{skins, style};

    #[test]
    fn every_glyph_paints_in_both_palettes() {
        let context = egui::Context::default();
        style::install(&context, skins::find("dark-fantasy"));
        for dark in [true, false] {
            context.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            let _ = context.run(egui::RawInput::default(), |ctx| {
                let painter = ctx.layer_painter(egui::LayerId::background());
                for sprite in Sprite::ALL {
                    let rect = Rect::from_min_size(Pos2::ZERO, grid(sprite) * 2.0);
                    for wake in [0.0, 0.5, 1.0] {
                        paint(&painter, rect, sprite, Palette::of_ctx(ctx).accent, wake);
                    }
                }
            });
        }
    }

    #[test]
    fn flames_stay_convex() {
        let points = flame((0.5, 0.76), 0.7, 0.26, 0.04);
        let turns: Vec<f32> = (0..points.len())
            .map(|index| {
                let (a, b, c) = (
                    points[index],
                    points[(index + 1) % points.len()],
                    points[(index + 2) % points.len()],
                );
                (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)
            })
            .collect();
        assert!(turns.iter().all(|turn| *turn >= 0.0) || turns.iter().all(|turn| *turn <= 0.0));
    }
}
