//! Small faceted silhouettes with hotspots at the arrow tip and index fingertip.

use eframe::egui::{self, Color32, Pos2, Stroke};

const RIM: Color32 = Color32::from_rgb(40, 30, 34);
const SILVER: Color32 = Color32::from_rgb(181, 171, 181);
const LIGHT: Color32 = Color32::from_rgb(249, 246, 241);
const SHADE: Color32 = Color32::from_rgb(103, 92, 106);

pub(super) fn paint(painter: &egui::Painter, tip: Pos2, cursor: egui::CursorIcon) -> bool {
    match cursor {
        egui::CursorIcon::Default => arrow(painter, tip),
        egui::CursorIcon::PointingHand => hand(painter, tip),
        _ => return false,
    }
    true
}

fn facet(painter: &egui::Painter, tip: Pos2, points: &[(f32, f32)], color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        points
            .iter()
            .map(|&(x, y)| tip + egui::vec2(x, y))
            .collect(),
        color,
        Stroke::NONE,
    ));
}

fn outline(painter: &egui::Painter, tip: Pos2, points: &[(f32, f32)]) {
    painter.add(egui::Shape::closed_line(
        points
            .iter()
            .map(|&(x, y)| tip + egui::vec2(x, y))
            .collect(),
        Stroke::new(1.3_f32, RIM),
    ));
}

fn arrow(painter: &egui::Painter, tip: Pos2) {
    let at = |x, y| tip + egui::emath::Rot2::from_angle(-0.52) * egui::vec2(x, y);
    let face = |points: &[(f32, f32)], color| {
        painter.add(egui::Shape::convex_polygon(
            points.iter().map(|&(x, y)| at(x, y)).collect(),
            color,
            Stroke::NONE,
        ));
    };
    face(&[(0.0, 0.0), (-7.0, 28.0), (0.0, 21.0)], LIGHT);
    face(&[(0.0, 0.0), (0.0, 21.0), (7.0, 28.0)], SILVER);
    face(&[(0.0, 3.0), (0.0, 21.0), (1.0, 22.0)], SHADE);
    painter.add(egui::Shape::closed_line(
        vec![at(0.0, 0.0), at(-7.0, 28.0), at(0.0, 21.0), at(7.0, 28.0)],
        Stroke::new(1.3_f32, RIM),
    ));
    face(
        &[(0.0, 12.0), (1.8, 15.0), (0.0, 18.0), (-1.8, 15.0)],
        LIGHT,
    );
    face(
        &[(0.0, 13.0), (0.9, 15.0), (0.0, 17.0), (-0.9, 15.0)],
        Color32::from_rgb(107, 15, 31),
    );
}

fn hand(painter: &egui::Painter, tip: Pos2) {
    facet(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-1.8, 1.0),
            (-1.8, 12.0),
            (1.8, 12.0),
            (1.8, 1.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(0.6, 0.5), (1.8, 1.0), (1.8, 12.0), (0.6, 12.0)],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[
            (-1.8, 10.0),
            (7.0, 9.0),
            (10.5, 13.0),
            (10.0, 17.0),
            (7.0, 20.0),
            (-2.0, 20.0),
            (-5.0, 15.0),
        ],
        SILVER,
    );
    for (x, y) in [(1.8, 7.0), (4.7, 7.7), (7.6, 8.4)] {
        facet(
            painter,
            tip,
            &[
                (x, y + 1.0),
                (x + 1.0, y),
                (x + 2.8, y + 0.8),
                (x + 2.8, y + 6.0),
                (x, y + 6.0),
            ],
            LIGHT,
        );
        painter.line_segment(
            [
                tip + egui::vec2(x + 2.4, y + 2.0),
                tip + egui::vec2(x + 2.4, y + 5.0),
            ],
            Stroke::new(0.7_f32, SHADE),
        );
    }
    facet(
        painter,
        tip,
        &[
            (-4.5, 9.0),
            (-6.0, 9.0),
            (-7.0, 10.5),
            (-6.5, 12.0),
            (-2.0, 18.0),
            (-0.5, 16.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[
            (1.0, 16.0),
            (10.0, 15.0),
            (10.0, 17.0),
            (7.0, 20.0),
            (-2.0, 20.0),
        ],
        SHADE,
    );
    outline(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-1.8, 1.0),
            (-1.8, 11.0),
            (-4.5, 9.0),
            (-6.0, 9.0),
            (-7.0, 10.5),
            (-6.5, 12.0),
            (-3.0, 18.0),
            (-2.0, 20.0),
            (7.0, 20.0),
            (10.0, 17.0),
            (10.5, 14.4),
            (10.4, 9.2),
            (8.6, 8.4),
            (7.6, 9.4),
            (7.5, 8.5),
            (5.7, 7.7),
            (4.7, 8.7),
            (4.6, 7.8),
            (2.8, 7.0),
            (1.8, 8.0),
            (1.8, 1.0),
        ],
    );
    let cuff = [(-2.0, 20.0), (8.0, 20.0), (8.0, 24.0), (-2.0, 24.0)];
    facet(painter, tip, &cuff, Color32::from_rgb(103, 29, 54));
    outline(painter, tip, &cuff);
    painter.line_segment(
        [tip + egui::vec2(-1.0, 20.5), tip + egui::vec2(7.0, 20.5)],
        Stroke::new(0.8_f32, LIGHT),
    );
    facet(
        painter,
        tip,
        &[(3.0, 20.5), (5.0, 22.0), (3.0, 23.5), (1.0, 22.0)],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(3.0, 21.0), (4.0, 22.0), (3.0, 23.0), (2.0, 22.0)],
        Color32::from_rgb(107, 15, 31),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_arrow_and_hand_but_preserves_native_editing_cursors() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(paint(ui.painter(), Pos2::ZERO, egui::CursorIcon::Default));
            assert!(paint(
                ui.painter(),
                Pos2::ZERO,
                egui::CursorIcon::PointingHand
            ));
            for cursor in [
                egui::CursorIcon::Text,
                egui::CursorIcon::ResizeHorizontal,
                egui::CursorIcon::ResizeVertical,
                egui::CursorIcon::NotAllowed,
            ] {
                assert!(!paint(ui.painter(), Pos2::ZERO, cursor));
            }
        });
    }
}
