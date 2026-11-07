//! Frutiger Aero fish: five glossy body shapes (clownfish, tang, angelfish, pufferfish and
//! goldfish), each in two color variants, drawn from vector shapes in the fish's own frame so
//! they tilt, flip and squash as they swim.

use super::glass;
use super::pixel::{mix, shaded_polygon};
use super::wildlife::edge_on;
use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2};
use std::f32::consts::TAU;

/// Distinct breeds: every shape in each of its color variants.
pub const BREEDS: u32 = 10;
const OVAL_STEPS: u32 = 20;
const INK: Color32 = Color32::from_rgb(0x14, 0x12, 0x20);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Form {
    Clown,
    Tang,
    Angel,
    Puffer,
    Goldfish,
}

/// One breed: a body shape and its colors.
struct Breed {
    form: Form,
    body: Color32,
    /// Bands, marks or spots.
    mark: Color32,
    fin: Color32,
}

fn breed(seed: u32) -> Breed {
    let rgb = Color32::from_rgb;
    let (form, body, mark, fin) = match seed % BREEDS {
        0 => (
            Form::Clown,
            rgb(0xff, 0x7a, 0x10),
            Color32::WHITE,
            rgb(0xff, 0x9a, 0x30),
        ),
        1 => (
            Form::Clown,
            rgb(0x4a, 0x26, 0x14),
            Color32::WHITE,
            rgb(0x6a, 0x36, 0x1c),
        ),
        2 => (
            Form::Tang,
            rgb(0x1c, 0x5c, 0xf0),
            INK,
            rgb(0xff, 0xd0, 0x20),
        ),
        3 => (
            Form::Tang,
            rgb(0xff, 0xdc, 0x18),
            rgb(0xff, 0xf0, 0x90),
            rgb(0xff, 0xe8, 0x50),
        ),
        4 => (
            Form::Angel,
            rgb(0xe8, 0xf0, 0xf8),
            INK,
            rgb(0xd0, 0xe0, 0xf0),
        ),
        5 => (
            Form::Angel,
            rgb(0xff, 0xb8, 0x30),
            rgb(0x6a, 0x3a, 0x10),
            rgb(0xff, 0xd0, 0x70),
        ),
        6 => (
            Form::Puffer,
            rgb(0xe8, 0xc8, 0x78),
            rgb(0x7a, 0x50, 0x20),
            rgb(0xf0, 0xd8, 0x98),
        ),
        7 => (
            Form::Puffer,
            rgb(0x40, 0xd8, 0xe8),
            rgb(0x0c, 0x30, 0x60),
            rgb(0x90, 0xf0, 0xf8),
        ),
        8 => (
            Form::Goldfish,
            rgb(0xff, 0x8c, 0x18),
            rgb(0xff, 0xc0, 0x40),
            rgb(0xff, 0xa8, 0x40),
        ),
        _ => (
            Form::Goldfish,
            rgb(0xfc, 0xf4, 0xec),
            rgb(0xf0, 0x40, 0x20),
            rgb(0xff, 0x70, 0x40),
        ),
    };
    Breed {
        form,
        body,
        mark,
        fin,
    }
}

/// Fish of one shape school together whatever their colors.
pub fn school(seed: u32) -> u32 {
    breed(seed).form as u32
}

/// How and where to draw one fish.
pub struct FishPose {
    pub center: Pos2,
    pub length: f32,
    /// 1 faces right, -1 left; values between squash the fish as it turns around.
    pub heading: f32,
    /// Nose up (negative) or down (positive), in radians.
    pub pitch: f32,
    /// Tail swing from -1 to 1.
    pub wag: f32,
    /// Which breed, any number; it wraps around the breeds.
    pub seed: u32,
    /// Fright from 0 to 1; a pufferfish blows itself up and bristles.
    pub alarm: f32,
}

/// A glossy tropical fish; by night it glows softly.
pub fn paint_fish(painter: &egui::Painter, pose: &FishPose, night: bool) {
    let breed = breed(pose.seed);
    if night {
        painter.add(glass::radial_glow(
            pose.center,
            pose.length * 1.6,
            breed.body.gamma_multiply(0.35),
        ));
    }
    let pen = Pen::new(painter, pose);
    match breed.form {
        Form::Clown => clownfish(&pen, &breed, pose.wag),
        Form::Tang => tang(&pen, &breed, pose.wag),
        Form::Angel => angelfish(&pen, &breed, pose.wag),
        Form::Puffer => pufferfish(&pen, &breed, pose.wag, pose.alarm),
        Form::Goldfish => goldfish(&pen, &breed, pose.wag),
    }
}

/// Draws in the fish's frame: x runs forward along the body and y down, both in body lengths.
struct Pen<'a> {
    painter: &'a egui::Painter,
    center: Pos2,
    length: f32,
    heading: f32,
    turn: Vec2,
}

impl<'a> Pen<'a> {
    fn new(painter: &'a egui::Painter, pose: &FishPose) -> Self {
        let heading = edge_on(pose.heading);
        Self {
            painter,
            center: pose.center,
            length: pose.length,
            heading,
            turn: Vec2::angled(pose.pitch * heading.signum()),
        }
    }

    fn at(&self, x: f32, y: f32) -> Pos2 {
        let local = Vec2::new(x * self.heading, y) * self.length;
        let turn = self.turn;
        self.center
            + Vec2::new(
                local.x * turn.x - local.y * turn.y,
                local.x * turn.y + local.y * turn.x,
            )
    }

    fn points(&self, local: &[(f32, f32)]) -> Vec<Pos2> {
        local.iter().map(|&(x, y)| self.at(x, y)).collect()
    }

    fn oval(&self, x: f32, y: f32, width: f32, height: f32) -> Vec<Pos2> {
        (0..OVAL_STEPS)
            .map(|step| {
                let around = Vec2::angled(step as f32 * TAU / OVAL_STEPS as f32);
                self.at(x + around.x * width, y + around.y * height)
            })
            .collect()
    }

    fn fill(&self, points: Vec<Pos2>, color: Color32) {
        self.painter
            .add(Shape::convex_polygon(points, color, Stroke::NONE));
    }

    /// Lit from the top of the screen: light along the back, darker towards the belly.
    fn body(&self, points: Vec<Pos2>, color: Color32) {
        let (top, bottom) = vertical_span(&points);
        let light = mix(color, Color32::WHITE, 0.3);
        let dark = mix(color, Color32::BLACK, 0.22);
        self.painter.add(shaded_polygon(&points, |point| {
            light.lerp_to_gamma(dark, (point.y - top) / (bottom - top).max(1.0))
        }));
        let edge = Stroke::new(1.0_f32, mix(color, Color32::BLACK, 0.5));
        self.painter.add(Shape::closed_line(points, edge));
    }

    /// A forked tail fan hinged at `x`, swinging with `wag`.
    fn tail(&self, x: f32, reach: f32, spread: f32, wag: f32, color: Color32) {
        let swing = wag * 0.08;
        let hinge = (x, 0.0);
        let notch = (x - reach * 0.75, swing * 0.5);
        self.fill(
            self.points(&[hinge, (x - reach, -spread + swing), notch]),
            color,
        );
        self.fill(
            self.points(&[hinge, notch, (x - reach, spread + swing)]),
            color,
        );
    }

    /// A white eye with a dark pupil and a glint.
    fn eye(&self, x: f32, y: f32, size: f32) {
        let center = self.at(x, y);
        let radius = size * self.length;
        self.painter.circle_filled(center, radius, Color32::WHITE);
        self.painter.circle_filled(center, radius * 0.6, INK);
        self.painter.circle_filled(
            center - Vec2::splat(radius * 0.25),
            radius * 0.22,
            Color32::WHITE,
        );
    }

    /// The glossy highlight along the back.
    fn shine(&self, x: f32, y: f32, width: f32, height: f32) {
        self.fill(
            self.oval(x, y, width, height),
            Color32::WHITE.gamma_multiply(0.5),
        );
    }
}

/// A rounded orange fish with three bold bands edged in black and paddle-round fins.
fn clownfish(pen: &Pen, breed: &Breed, wag: f32) {
    pen.tail(-0.4, 0.3, 0.26, wag, breed.fin);
    pen.fill(pen.oval(-0.05, -0.24, 0.22, 0.1), breed.fin);
    pen.fill(pen.oval(0.0, 0.24, 0.12, 0.07), breed.fin);
    pen.body(pen.oval(0.0, 0.0, 0.5, 0.27), breed.body);
    for x in [0.27_f32, -0.02, -0.33] {
        let height = 0.27 * (1.0 - (x / 0.5).powi(2)).sqrt() * 0.95;
        pen.fill(pen.oval(x, 0.0, 0.065, height), INK);
        pen.fill(pen.oval(x, 0.0, 0.045, height * 0.96), breed.mark);
    }
    pen.fill(pen.oval(0.12, 0.08, 0.08, 0.045), breed.fin);
    pen.shine(0.05, -0.13, 0.28, 0.06);
    pen.eye(0.36, -0.05, 0.065);
}

/// An oval surgeonfish with a long dorsal ribbon, a deep forked tail and a crescent mark.
fn tang(pen: &Pen, breed: &Breed, wag: f32) {
    pen.tail(-0.44, 0.34, 0.32, wag, breed.fin);
    let ribbon = [(-0.38, -0.16), (-0.3, -0.34), (0.18, -0.34), (0.32, -0.2)];
    pen.fill(pen.points(&ribbon), breed.fin);
    let keel = [(-0.36, 0.14), (-0.25, 0.3), (0.1, 0.3), (0.2, 0.2)];
    pen.fill(pen.points(&keel), breed.fin);
    pen.body(pen.oval(0.0, 0.0, 0.5, 0.29), breed.body);
    // A crescent: a dark oval with the body color laid over most of it.
    pen.fill(pen.oval(-0.05, -0.04, 0.32, 0.13), breed.mark);
    pen.fill(pen.oval(0.02, 0.02, 0.26, 0.09), breed.body);
    pen.shine(0.05, -0.15, 0.3, 0.05);
    pen.eye(0.33, -0.08, 0.06);
}

/// A tall disc with a pointed snout, long trailing sail fins and thin dark stripes.
fn angelfish(pen: &Pen, breed: &Breed, wag: f32) {
    let trail = wag * 0.04;
    // Sail fins sweeping back from the top and bottom of the disc.
    for side in [-1.0_f32, 1.0] {
        let sail = [
            (0.1, side * 0.26),
            (-0.12, side * 0.5),
            (-0.42, side * 0.66 + trail),
            (-0.3, side * 0.18),
        ];
        pen.fill(pen.points(&sail), breed.fin);
    }
    pen.tail(-0.3, 0.24, 0.22, wag, breed.fin);
    let mut disc = pen.oval(-0.02, 0.0, 0.33, 0.34);
    disc.push(pen.at(0.42, -0.02));
    pen.body(convex_hull(disc), breed.body);
    for x in [0.16_f32, -0.04, -0.22] {
        pen.fill(pen.oval(x, 0.0, 0.025, 0.3), breed.mark);
    }
    pen.shine(0.0, -0.18, 0.2, 0.06);
    pen.eye(0.24, -0.08, 0.06);
}

/// A round spotted fish with a pale belly that blows itself up and bristles when frightened.
fn pufferfish(pen: &Pen, breed: &Breed, wag: f32, alarm: f32) {
    let swell = 1.0 + 0.4 * alarm.clamp(0.0, 1.0);
    pen.tail(-0.32 * swell, 0.2, 0.16, wag, breed.fin);
    if alarm > 0.1 {
        for spike in 0..16 {
            let around = Vec2::angled(spike as f32 * TAU / 16.0);
            let root = (around.x * 0.34 * swell, around.y * 0.31 * swell);
            let tip = 1.0 + 0.3 * alarm;
            pen.painter.line_segment(
                [pen.at(root.0, root.1), pen.at(root.0 * tip, root.1 * tip)],
                Stroke::new(1.2_f32, mix(breed.mark, breed.body, 0.4)),
            );
        }
    }
    pen.body(pen.oval(0.0, 0.0, 0.36 * swell, 0.33 * swell), breed.body);
    let belly = mix(breed.body, Color32::WHITE, 0.65);
    pen.fill(
        pen.oval(0.02, 0.13 * swell, 0.27 * swell, 0.14 * swell),
        belly,
    );
    for (x, y) in [
        (-0.18, -0.12),
        (-0.05, -0.2),
        (0.08, -0.14),
        (-0.2, 0.02),
        (-0.06, -0.04),
        (0.12, -0.03),
        (-0.24, -0.06),
    ] {
        pen.fill(pen.oval(x * swell, y * swell, 0.03, 0.03), breed.mark);
    }
    pen.fill(pen.oval(0.06, 0.06, 0.07, 0.045), breed.fin);
    pen.shine(0.0, -0.2 * swell, 0.2 * swell, 0.05);
    pen.eye(0.2 * swell, -0.08 * swell, 0.08);
}

/// A short golden body with a flowing double veil tail and a sail on its back.
fn goldfish(pen: &Pen, breed: &Breed, wag: f32) {
    let veil = breed.fin.gamma_multiply(0.85);
    let swing = wag * 0.12;
    for side in [-1.0_f32, 1.0] {
        let lobe = [
            (-0.28, 0.0),
            (-0.55, side * 0.3 + swing),
            (-0.85, side * 0.22 + swing * 1.6),
            (-0.72, side * 0.04 + swing),
        ];
        pen.painter
            .add(shaded_polygon(&pen.points(&lobe), |_| veil));
    }
    pen.fill(
        pen.points(&[(-0.15, -0.2), (0.02, -0.46), (0.18, -0.2)]),
        veil,
    );
    pen.fill(pen.oval(-0.05, 0.22, 0.1, 0.07), veil);
    pen.body(pen.oval(0.05, 0.0, 0.38, 0.26), breed.body);
    pen.fill(
        pen.oval(0.0, -0.06, 0.16, 0.1),
        breed.mark.gamma_multiply(0.7),
    );
    pen.shine(0.08, -0.13, 0.22, 0.05);
    pen.eye(0.3, -0.06, 0.07);
}

fn vertical_span(points: &[Pos2]) -> (f32, f32) {
    points
        .iter()
        .fold((f32::MAX, f32::MIN), |(top, bottom), point| {
            (top.min(point.y), bottom.max(point.y))
        })
}

/// The convex outline around `points` (monotone chain), so a snout added to an oval stays a
/// fillable convex shape.
fn convex_hull(mut points: Vec<Pos2>) -> Vec<Pos2> {
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let cross = |o: Pos2, a: Pos2, b: Pos2| (a - o).x * (b - o).y - (a - o).y * (b - o).x;
    let mut hull: Vec<Pos2> = Vec::with_capacity(points.len() * 2);
    for pass in [false, true] {
        let start = hull.len();
        let ordered: Box<dyn Iterator<Item = &Pos2>> = if pass {
            Box::new(points.iter().rev())
        } else {
            Box::new(points.iter())
        };
        for &point in ordered {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
            {
                hull.pop();
            }
            hull.push(point);
        }
        hull.pop();
    }
    hull
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_five_shapes_in_two_colors_each() {
        let forms: Vec<Form> = (0..BREEDS).map(|seed| breed(seed).form).collect();
        for form in [
            Form::Clown,
            Form::Tang,
            Form::Angel,
            Form::Puffer,
            Form::Goldfish,
        ] {
            assert_eq!(forms.iter().filter(|other| **other == form).count(), 2);
        }
        assert_eq!(school(0), school(1));
        assert_ne!(school(0), school(2));
    }

    #[test]
    fn every_breed_paints_in_any_pose() {
        let context = egui::Context::default();
        let _ = context.run_ui(egui::RawInput::default(), |ui| {
            let ctx = &ui.ctx().clone();
            let painter = ctx.layer_painter(egui::LayerId::background());
            for seed in 0..BREEDS {
                for (heading, alarm) in [(1.0, 0.0), (-0.4, 1.0), (0.0, 0.5)] {
                    let pose = FishPose {
                        center: egui::pos2(100.0, 100.0),
                        length: 40.0,
                        heading,
                        pitch: 0.3,
                        wag: -0.7,
                        seed,
                        alarm,
                    };
                    paint_fish(&painter, &pose, seed % 2 == 0);
                }
            }
        });
    }

    #[test]
    fn the_hull_wraps_all_points_convexly() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(4.0, 0.0),
            egui::pos2(4.0, 4.0),
            egui::pos2(0.0, 4.0),
            egui::pos2(2.0, 2.0),
            egui::pos2(6.0, 2.0),
        ];
        let hull = convex_hull(points);
        assert_eq!(hull.len(), 5);
        assert!(!hull.contains(&egui::pos2(2.0, 2.0)));
    }
}
