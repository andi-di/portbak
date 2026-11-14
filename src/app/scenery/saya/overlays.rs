//! Short, bounded portrait accents; no persistent particle simulation or textures.

use super::expressions::Expression;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::TAU;

pub(super) fn duration(expression: Expression) -> f64 {
    match expression {
        Expression::Neutral => 0.0,
        Expression::InLove => super::LOVE_HEART_SECONDS + super::LOVE_HEARTS[2].3,
        _ => 1.2,
    }
}

struct Accent<'a> {
    painter: &'a egui::Painter,
    opening: Rect,
    silver: Color32,
    rose: Color32,
    violet: Color32,
    unit: f32,
    t: f32,
}

impl Accent<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.opening.min + self.opening.size() * egui::vec2(x, y)
    }

    fn symbol(&self, center: Pos2, rows: &[&str], color: Color32) {
        let origin = center - egui::vec2(rows[0].len() as f32, rows.len() as f32) * self.unit * 0.5;
        for (y, row) in rows.iter().enumerate() {
            for (x, pixel) in row.bytes().enumerate() {
                if pixel == b'#' {
                    self.painter.rect_filled(
                        Rect::from_min_size(
                            origin + egui::vec2(x as f32, y as f32) * self.unit,
                            Vec2::splat(self.unit),
                        ),
                        0,
                        color,
                    );
                }
            }
        }
    }

    fn glint(&self, center: Pos2, radius: f32, color: Color32) {
        self.painter.add(egui::Shape::convex_polygon(
            vec![
                center - Vec2::Y * radius,
                center + Vec2::X * radius * 0.3,
                center + Vec2::Y * radius,
                center - Vec2::X * radius * 0.3,
            ],
            color,
            Stroke::NONE,
        ));
        self.painter.line_segment(
            [
                center - Vec2::X * radius * 0.7,
                center + Vec2::X * radius * 0.7,
            ],
            Stroke::new(self.unit * 0.6, color),
        );
    }

    fn ring(&self, center: Pos2, size: Vec2, color: Color32) {
        self.painter.add(egui::Shape::closed_line(
            (0..24)
                .map(|i| center + Vec2::angled(i as f32 / 24.0 * TAU) * size)
                .collect(),
            Stroke::new(self.unit * 0.65, color),
        ));
    }

    fn heart(&self, center: Pos2, color: Color32) {
        self.symbol(
            center,
            &[
                ".##.##.", "#######", "#######", ".#####.", "..###..", "...#...",
            ],
            color,
        );
    }
}

pub(super) fn paint(painter: &egui::Painter, opening: Rect, expression: Expression, elapsed: f64) {
    if !(0.0..duration(expression)).contains(&elapsed) {
        return;
    }
    let t = elapsed as f32 / duration(expression) as f32;
    let fade = (t * 8.0).min(1.0) * ((1.0 - t) * 4.0).min(1.0);
    let painter = painter.with_clip_rect(painter.clip_rect().intersect(opening));
    let a = Accent {
        painter: &painter,
        opening,
        t,
        silver: Color32::from_rgb(222, 218, 231).gamma_multiply(fade),
        rose: Color32::from_rgb(206, 76, 119).gamma_multiply(fade),
        violet: Color32::from_rgb(153, 160, 218).gamma_multiply(fade),
        unit: (opening.width() / 150.0).clamp(1.25, 3.0),
    };
    paint_accent(&a, expression);
}

fn paint_accent(a: &Accent<'_>, expression: Expression) {
    use Expression::*;
    let t = a.t;
    let pulse = 0.75 + (t * TAU * 2.0).sin() * 0.25;
    match expression {
        Angry => a.symbol(
            a.at(0.80, 0.25),
            &["#...#", "##.##", ".....", "##.##", "#...#"],
            a.rose.gamma_multiply(pulse),
        ),
        Bored => {
            let p = a.at(0.10 + t * 0.22, 0.37 + (t * TAU).sin() * 0.015);
            a.painter.rect_filled(
                Rect::from_center_size(p, Vec2::splat(a.unit)),
                0,
                a.silver.gamma_multiply(0.65),
            );
        }
        BoredExpectant => {
            for i in 0..3 {
                if t * 5.0 > i as f32 {
                    a.painter.rect_filled(
                        Rect::from_center_size(
                            a.at(0.76 + i as f32 * 0.025, 0.32),
                            Vec2::splat(a.unit * 1.5),
                        ),
                        0,
                        a.silver,
                    );
                }
            }
        }
        Celebrating => {
            for i in 0..12 {
                let x = 0.12 + i as f32 / 12.0 * 0.76;
                let y = 0.07 + t * (0.20 + (i % 3) as f32 * 0.035);
                let center = a.at(x + (t * 9.0 + i as f32).sin() * 0.015, y);
                a.painter.rect_filled(
                    Rect::from_center_size(center, egui::vec2(a.unit, a.unit * 2.5)),
                    0,
                    a.rose,
                );
            }
        }
        Curious => a.glint(
            a.at(
                0.80 + (t * TAU).cos() * 0.035,
                0.29 + (t * TAU).sin() * 0.045,
            ),
            a.unit * 2.0,
            a.silver,
        ),
        Cute => {
            let center = a.at(0.76, 0.16);
            for i in 0..5 {
                let p =
                    center + Vec2::angled(i as f32 / 5.0 * TAU) * a.unit * 3.0 * t.min(0.3) / 0.3;
                a.painter.circle_filled(p, a.unit * 2.0, a.rose);
            }
            a.painter.circle_filled(center, a.unit, a.silver);
        }
        Dominant => a.symbol(
            a.at(0.50, 0.08 - t * 0.015),
            &[
                "#...#...#",
                "##.###.##",
                "#########",
                ".#######.",
                ".#.#.#.#.",
                ".#######.",
            ],
            a.silver,
        ),
        Embarrassed => {
            for x in [0.39, 0.61] {
                for layer in (1..=4).rev() {
                    let center = a.at(x, 0.36);
                    let size = egui::vec2(0.026, 0.011) * a.opening.width() * layer as f32 / 4.0;
                    a.painter.rect_filled(
                        Rect::from_center_size(center, size),
                        3,
                        a.rose.gamma_multiply(0.07),
                    );
                }
            }
        }
        Energetic => a.symbol(
            a.at(0.80, 0.27),
            &[
                "...##", "..##.", ".##..", "####.", "..##.", ".##..", "##...",
            ],
            a.silver.gamma_multiply(pulse),
        ),
        Excited => a.glint(a.at(0.79, 0.24), a.unit * (2.0 + t * 9.0), a.silver),
        Flirty => a.heart(a.at(0.79, 0.29), a.rose.gamma_multiply(pulse)),
        Flustered => {
            let center = a.at(0.79, 0.25);
            let points = (0..32)
                .map(|i| {
                    let fraction = i as f32 / 31.0;
                    center
                        + Vec2::angled(fraction * TAU * 2.0 + (t * TAU).sin() * 0.3)
                            * fraction
                            * a.unit
                            * 5.0
                })
                .collect();
            a.painter.add(egui::Shape::line(
                points,
                Stroke::new(a.unit * 0.7, a.silver),
            ));
        }
        Happy => {
            for side in [-1.0, 1.0] {
                for i in 0..3 {
                    let center = a.at(0.50 + side * 0.29, 0.24);
                    let direction = Vec2::angled((i as f32 - 1.0) * 0.65) * egui::vec2(side, 1.0);
                    a.painter.line_segment(
                        [
                            center + direction * a.unit * 3.0,
                            center + direction * a.unit * (6.0 + t * 3.0),
                        ],
                        Stroke::new(a.unit, a.silver),
                    );
                }
            }
        }
        InLove | Neutral => {}
        Killer => {
            let center = a.at(0.40 + t * 0.20, 0.295);
            a.glint(center, a.unit * 4.0, a.rose);
        }
        Meek => {
            let center = a.at(0.80 + (t * TAU).sin() * 0.025, 0.25 + t * 0.18);
            a.painter.add(egui::Shape::convex_polygon(
                vec![
                    center - Vec2::Y * a.unit * 6.0,
                    center + egui::vec2(3.0, -1.0) * a.unit,
                    center + Vec2::Y * a.unit * 5.0,
                    center + egui::vec2(-2.0, 1.0) * a.unit,
                ],
                a.silver,
                Stroke::NONE,
            ));
            a.painter.line_segment(
                [
                    center - Vec2::Y * a.unit * 4.0,
                    center + Vec2::Y * a.unit * 7.0,
                ],
                Stroke::new(a.unit * 0.5, a.violet),
            );
        }
        Pleading => {
            for x in [0.40, 0.60] {
                a.glint(a.at(x, 0.33), a.unit * (1.5 + pulse), a.silver);
            }
        }
        Pouting => {
            let center = a.at(0.75 + t * 0.06, 0.40 - t * 0.06);
            for i in 0..3 {
                a.painter.circle_filled(
                    center + egui::vec2(i as f32 * 2.5, -(i % 2) as f32) * a.unit,
                    a.unit * (2.0 + t),
                    a.silver.gamma_multiply(0.4),
                );
            }
        }
        Praying => a.ring(a.at(0.50, 0.09), egui::vec2(8.0, 2.5) * a.unit, a.silver),
        Question => a.symbol(
            a.at(0.80, 0.25 - (t * 9.0).sin().max(0.0) * 0.008),
            &[
                ".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#..",
            ],
            a.silver,
        ),
        Sad => {
            let center = a.at(0.61, 0.33 + t * 0.14);
            a.painter.add(egui::Shape::convex_polygon(
                vec![
                    center - Vec2::Y * a.unit * 3.5,
                    center + egui::vec2(1.5, 1.0) * a.unit,
                    center + Vec2::Y * a.unit * 2.0,
                    center + egui::vec2(-1.5, 1.0) * a.unit,
                ],
                a.violet,
                Stroke::NONE,
            ));
        }
        Seductive => {
            let center = a.at(0.80, 0.40);
            for i in (0..7).rev() {
                let angle = i as f32 * 2.4;
                let radius = a.unit * (1.0 + i as f32 * 0.35) * (t * 4.0).min(1.0);
                a.painter.circle_filled(
                    center + Vec2::angled(angle) * radius,
                    a.unit * 2.0,
                    a.rose.gamma_multiply(0.6 + (i % 2) as f32 * 0.4),
                );
            }
        }
        Shy => {
            for side in [false, true] {
                let x = if side { 1.0 } else { 0.0 };
                let inset = if side { -1.0 } else { 1.0 } * 0.07 * (t * std::f32::consts::PI).sin();
                a.painter.add(egui::Shape::convex_polygon(
                    vec![a.at(x, 0.0), a.at(x + inset, 0.15), a.at(x, 0.34)],
                    a.rose.gamma_multiply(0.25),
                    Stroke::NONE,
                ));
            }
        }
        Sleepy => a.symbol(
            a.at(0.79 + t * 0.025, 0.30 - t * 0.10),
            &["#####", "...#.", "..#..", ".#...", "#####"],
            a.silver,
        ),
        Smug => a.glint(a.at(0.64, 0.42), a.unit * 4.0 * pulse, a.silver),
        Waiting => {
            let center = a.at(0.80, 0.28);
            let points: Vec<_> = (0..18)
                .map(|i| center + Vec2::angled(i as f32 / 17.0 * 4.5 + t * TAU) * a.unit * 4.0)
                .collect();
            a.painter
                .add(egui::Shape::line(points, Stroke::new(a.unit, a.silver)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_accent_is_bounded_clipped_and_finishes_without_textures() {
        let ctx = egui::Context::default();
        let opening = Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::splat(300.0));
        let clip = opening.shrink(25.0);
        for expression in Expression::REACTIONS {
            if expression == Expression::InLove {
                continue;
            }
            for elapsed in [0.15, 0.6, 1.0] {
                let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    paint(
                        &ui.painter().with_clip_rect(clip),
                        opening,
                        expression,
                        elapsed,
                    );
                });
                assert!(!output.shapes.is_empty(), "{expression:?}");
                assert!(output.shapes.len() < 100, "{expression:?}");
                for shape in &output.shapes {
                    assert!(clip.contains_rect(shape.clip_rect));
                    assert!(
                        opening.contains_rect(shape.shape.visual_bounding_rect()),
                        "{expression:?}"
                    );
                }
                assert!(ctx.tessellate(output.shapes, output.pixels_per_point).len() < 10);
            }
            let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                paint(ui.painter(), opening, expression, duration(expression));
            });
            assert!(output.shapes.is_empty(), "{expression:?}");
            assert!(output.textures_delta.set.is_empty());
        }
    }
}
