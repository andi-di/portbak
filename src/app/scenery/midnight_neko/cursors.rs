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
    facet(painter, tip, &[(0.0, 0.0), (2.0, 26.0), (9.0, 17.0)], LIGHT);
    facet(
        painter,
        tip,
        &[(0.0, 0.0), (9.0, 17.0), (23.0, 16.0)],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[(0.0, 0.0), (9.0, 17.0), (11.0, 16.0)],
        SHADE,
    );
    outline(
        painter,
        tip,
        &[(0.0, 0.0), (2.0, 26.0), (9.0, 17.0), (23.0, 16.0)],
    );
    facet(
        painter,
        tip,
        &[(6.0, 11.0), (8.0, 12.0), (8.0, 15.0), (6.0, 14.0)],
        Color32::from_rgb(107, 15, 31),
    );
}

fn hand(painter: &egui::Painter, tip: Pos2) {
    // Each finger is its own convex piece; the final outline unifies the gauntlet.
    facet(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-3.0, 2.0),
            (-3.0, 18.0),
            (3.0, 18.0),
            (3.0, 2.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(0.0, 0.0), (3.0, 2.0), (3.0, 18.0), (0.0, 18.0)],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[
            (-3.0, 15.0),
            (11.0, 12.0),
            (16.0, 17.0),
            (15.0, 23.0),
            (10.0, 28.0),
            (-2.0, 28.0),
            (-8.0, 20.0),
        ],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[
            (4.0, 11.0),
            (7.0, 9.0),
            (10.0, 11.0),
            (10.0, 20.0),
            (4.0, 20.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[
            (10.0, 12.0),
            (13.0, 10.0),
            (16.0, 12.0),
            (16.0, 20.0),
            (10.0, 21.0),
        ],
        SILVER,
    );
    facet(
        painter,
        tip,
        &[
            (-7.0, 12.0),
            (-10.0, 13.0),
            (-11.0, 16.0),
            (-5.0, 25.0),
            (0.0, 23.0),
        ],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[
            (9.0, 20.0),
            (16.0, 17.0),
            (15.0, 23.0),
            (10.0, 28.0),
            (-2.0, 28.0),
        ],
        SHADE,
    );
    for x in [3.0, 9.0] {
        painter.line_segment(
            [tip + egui::vec2(x, 13.0), tip + egui::vec2(x, 20.0)],
            Stroke::new(0.8_f32, SHADE),
        );
    }
    outline(
        painter,
        tip,
        &[
            (0.0, 0.0),
            (-3.0, 2.0),
            (-3.0, 15.0),
            (-7.0, 12.0),
            (-10.0, 13.0),
            (-11.0, 16.0),
            (-5.0, 25.0),
            (-2.0, 28.0),
            (10.0, 28.0),
            (15.0, 23.0),
            (16.0, 20.0),
            (16.0, 12.0),
            (13.0, 10.0),
            (10.0, 12.0),
            (10.0, 11.0),
            (7.0, 9.0),
            (4.0, 11.0),
            (3.0, 11.0),
            (3.0, 2.0),
        ],
    );
    let cuff = [(-3.0, 28.0), (13.0, 28.0), (13.0, 35.0), (-3.0, 35.0)];
    facet(painter, tip, &cuff, Color32::from_rgb(103, 29, 54));
    outline(painter, tip, &cuff);
    painter.line_segment(
        [tip + egui::vec2(-2.0, 29.0), tip + egui::vec2(12.0, 29.0)],
        Stroke::new(1.0_f32, LIGHT),
    );
    facet(
        painter,
        tip,
        &[(5.0, 29.0), (8.0, 32.0), (5.0, 35.0), (2.0, 32.0)],
        LIGHT,
    );
    facet(
        painter,
        tip,
        &[(5.0, 30.0), (7.0, 32.0), (5.0, 34.0), (3.0, 32.0)],
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
