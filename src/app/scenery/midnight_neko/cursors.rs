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
    facet(painter, tip, &[(0.0, 0.0), (1.0, 28.0), (7.0, 19.0)], LIGHT);
    facet(
        painter,
        tip,
        &[(0.0, 0.0), (7.0, 19.0), (17.0, 18.0)],
        SILVER,
    );
    facet(painter, tip, &[(0.0, 0.0), (7.0, 19.0), (8.5, 18.0)], SHADE);
    facet(
        painter,
        tip,
        &[(0.0, 0.0), (17.0, 18.0), (14.0, 17.0)],
        Color32::from_rgb(139, 128, 143),
    );
    outline(
        painter,
        tip,
        &[(0.0, 0.0), (1.0, 28.0), (7.0, 19.0), (17.0, 18.0)],
    );
    painter.line_segment(
        [tip + egui::vec2(2.0, 7.0), tip + egui::vec2(4.0, 17.0)],
        Stroke::new(0.8_f32, SILVER),
    );
    facet(
        painter,
        tip,
        &[(5.0, 12.0), (7.0, 14.0), (6.0, 17.0), (4.0, 15.0)],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(5.0, 13.0), (6.0, 14.0), (5.5, 16.0), (4.8, 15.0)],
        Color32::from_rgb(107, 15, 31),
    );
}

fn hand(painter: &egui::Painter, tip: Pos2) {
    // A slim raised index finger and three folded knuckles make the gesture readable.
    facet(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-2.0, 1.0),
            (-2.0, 15.0),
            (2.0, 15.0),
            (2.0, 1.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(0.5, 0.5), (2.0, 1.0), (2.0, 15.0), (0.5, 15.0)],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[
            (-2.0, 13.0),
            (7.0, 12.0),
            (11.0, 16.0),
            (10.0, 20.0),
            (7.0, 23.0),
            (-2.0, 23.0),
            (-6.0, 18.0),
        ],
        SILVER,
    );
    for (x, y) in [(2.0, 10.0), (5.0, 11.0), (8.0, 13.0)] {
        facet(
            painter,
            tip,
            &[
                (x, y + 1.0),
                (x + 1.0, y),
                (x + 3.0, y + 1.0),
                (x + 3.0, y + 6.0),
                (x, y + 6.0),
            ],
            LIGHT,
        );
        painter.line_segment(
            [
                tip + egui::vec2(x + 2.5, y + 2.0),
                tip + egui::vec2(x + 2.5, y + 5.0),
            ],
            Stroke::new(0.7_f32, SHADE),
        );
    }
    facet(
        painter,
        tip,
        &[
            (-5.0, 11.0),
            (-7.0, 11.0),
            (-9.0, 13.0),
            (-8.0, 16.0),
            (-3.0, 21.0),
            (0.0, 19.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[
            (1.0, 19.0),
            (10.0, 18.0),
            (10.0, 20.0),
            (7.0, 23.0),
            (-2.0, 23.0),
        ],
        SHADE,
    );
    painter.line_segment(
        [tip + egui::vec2(-1.5, 8.0), tip + egui::vec2(1.0, 8.0)],
        Stroke::new(0.7_f32, SILVER),
    );
    outline(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-2.0, 1.0),
            (-2.0, 13.0),
            (-5.0, 11.0),
            (-7.0, 11.0),
            (-9.0, 13.0),
            (-8.0, 16.0),
            (-3.0, 22.0),
            (-2.0, 23.0),
            (7.0, 23.0),
            (10.0, 20.0),
            (11.0, 18.0),
            (11.0, 14.0),
            (9.0, 13.0),
            (8.0, 13.0),
            (8.0, 12.0),
            (6.0, 11.0),
            (5.0, 11.0),
            (5.0, 10.0),
            (3.0, 10.0),
            (2.0, 11.0),
            (2.0, 1.0),
        ],
    );
    let cuff = [(-2.0, 23.0), (8.0, 23.0), (8.0, 27.0), (-2.0, 27.0)];
    facet(painter, tip, &cuff, Color32::from_rgb(103, 29, 54));
    outline(painter, tip, &cuff);
    painter.line_segment(
        [tip + egui::vec2(-1.0, 23.5), tip + egui::vec2(7.0, 23.5)],
        Stroke::new(0.8_f32, LIGHT),
    );
    facet(
        painter,
        tip,
        &[(3.0, 23.5), (5.0, 25.0), (3.0, 26.5), (1.0, 25.0)],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(3.0, 24.0), (4.0, 25.0), (3.0, 26.0), (2.0, 25.0)],
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
