//! Vector pen shared by the glossy and modern icon sets.
//!
//! Coordinates are fractions of the icon's rectangle.

use crate::app::rendering::sprites::Sprite;
use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, Vec2};

pub(super) type Point = (f32, f32);

/// Grid size in sprite cells, so callers' scales give comparable sizes to the pixel set.
pub fn grid(sprite: Sprite) -> Vec2 {
    match sprite {
        Sprite::Check => egui::vec2(7.0, 6.0),
        Sprite::Cursor => egui::vec2(4.0, 7.0),
        _ => egui::vec2(12.0, 12.0),
    }
}

/// Maps fractional coordinates into the icon rectangle with a line width matching its size.
pub(super) struct Pen<'a> {
    pub(super) painter: &'a egui::Painter,
    pub(super) rect: Rect,
    width: f32,
}

impl<'a> Pen<'a> {
    pub(super) fn new(painter: &'a egui::Painter, rect: Rect) -> Self {
        let width = (rect.width().min(rect.height()) / 12.0).clamp(1.0, 5.0);
        Self {
            painter,
            rect,
            width,
        }
    }

    pub(super) fn at(&self, (x, y): Point) -> Pos2 {
        self.rect.min + egui::vec2(x * self.rect.width(), y * self.rect.height())
    }

    pub(super) fn points(&self, points: &[Point]) -> Vec<Pos2> {
        points.iter().map(|point| self.at(*point)).collect()
    }

    pub(super) fn stroke(&self, color: Color32, weight: f32) -> Stroke {
        Stroke::new(self.width * weight, color)
    }

    pub(super) fn line(&self, points: &[Point], color: Color32, weight: f32) {
        let stroke = self.stroke(color, weight);
        self.painter.add(Shape::line(self.points(points), stroke));
    }

    /// A circle whose radius is a fraction of the icon width.
    pub(super) fn circle(&self, center: Point, radius: f32, fill: Color32, edge: Option<Color32>) {
        let stroke = edge.map_or(Stroke::NONE, |edge| self.stroke(edge, 1.0));
        let radius = radius * self.rect.width();
        self.painter
            .add(Shape::circle_filled(self.at(center), radius, fill));
        if edge.is_some() {
            self.painter.circle_stroke(self.at(center), radius, stroke);
        }
    }
}

pub(super) fn polar(center: Point, radius: f32, angle: f32) -> Point {
    (
        center.0 + radius * angle.cos(),
        center.1 + radius * angle.sin(),
    )
}
