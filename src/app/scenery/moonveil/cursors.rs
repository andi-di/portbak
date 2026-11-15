//! Small faceted silhouettes with hotspots at the arrow tip and index fingertip.

use eframe::egui::{self, Color32, Pos2, Stroke};

const RIM: Color32 = Color32::from_rgb(40, 30, 34);
const SILVER: Color32 = Color32::from_rgb(181, 171, 181);
const LIGHT: Color32 = Color32::from_rgb(249, 246, 241);
const SHADE: Color32 = Color32::from_rgb(103, 92, 106);

pub(super) fn paint(painter: &egui::Painter, tip: Option<Pos2>, cursor: egui::CursorIcon) -> bool {
    let down = painter.ctx().input(|input| input.pointer.primary_down());
    let id = egui::Id::new("saya_held_hand_cursor");
    let held = painter.ctx().data_mut(|data| {
        let held = down
            && ((tip.is_some() && cursor == egui::CursorIcon::PointingHand)
                || data.get_temp::<bool>(id).unwrap_or(false));
        data.insert_temp(id, held);
        held
    });
    let Some(tip) = tip else { return false };
    match cursor {
        egui::CursorIcon::Default if held => hand(painter, tip, true),
        egui::CursorIcon::Default => arrow(painter, tip),
        egui::CursorIcon::PointingHand => hand(painter, tip, held),
        _ => return false,
    }
    true
}

fn hand_point(tip: Pos2, x: f32, y: f32, held: bool) -> Pos2 {
    if held {
        tip + egui::vec2(x * 0.96 + y * 0.025, y * 0.86)
    } else {
        tip + egui::vec2(x, y)
    }
}

fn facet(painter: &egui::Painter, tip: Pos2, points: &[(f32, f32)], color: Color32, held: bool) {
    let color = if held && color == Color32::from_rgb(107, 15, 31) {
        Color32::from_rgb(183, 39, 66)
    } else {
        color
    };
    painter.add(egui::Shape::convex_polygon(
        points
            .iter()
            .map(|&(x, y)| hand_point(tip, x, y, held))
            .collect(),
        color,
        Stroke::NONE,
    ));
}

fn outline(painter: &egui::Painter, tip: Pos2, points: &[(f32, f32)], held: bool) {
    painter.add(egui::Shape::closed_line(
        points
            .iter()
            .map(|&(x, y)| hand_point(tip, x, y, held))
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
    face(&[(0.0, 0.0), (-9.0, 28.0), (0.0, 21.0)], LIGHT);
    face(&[(0.0, 0.0), (0.0, 21.0), (9.0, 28.0)], SILVER);
    face(&[(0.0, 3.0), (0.0, 21.0), (1.0, 22.0)], SHADE);
    painter.add(egui::Shape::closed_line(
        vec![at(0.0, 0.0), at(-9.0, 28.0), at(0.0, 21.0), at(9.0, 28.0)],
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

fn hand(painter: &egui::Painter, tip: Pos2, held: bool) {
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
        held,
    );
    facet(
        painter,
        tip,
        &[(0.6, 0.5), (1.8, 1.0), (1.8, 12.0), (0.6, 12.0)],
        SILVER,
        held,
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
        held,
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
            held,
        );
        painter.line_segment(
            [
                hand_point(tip, x + 2.4, y + 2.0, held),
                hand_point(tip, x + 2.4, y + 5.0, held),
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
        held,
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
        held,
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
        held,
    );
    let cuff = [(-2.0, 20.0), (8.0, 20.0), (8.0, 24.0), (-2.0, 24.0)];
    facet(painter, tip, &cuff, Color32::from_rgb(103, 29, 54), held);
    outline(painter, tip, &cuff, held);
    painter.line_segment(
        [
            hand_point(tip, -1.0, 20.5, held),
            hand_point(tip, 7.0, 20.5, held),
        ],
        Stroke::new(0.8_f32, LIGHT),
    );
    facet(
        painter,
        tip,
        &[(3.0, 20.5), (5.0, 22.0), (3.0, 23.5), (1.0, 22.0)],
        LIGHT,
        held,
    );
    facet(
        painter,
        tip,
        &[(3.0, 21.0), (4.0, 22.0), (3.0, 23.0), (2.0, 22.0)],
        Color32::from_rgb(107, 15, 31),
        held,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_hand_keeps_hotspot_and_persists_outside_the_button_until_release() {
        let tip = egui::pos2(50.0, 50.0);
        assert_eq!(hand_point(tip, 0.0, 0.0, true), tip);
        assert!(hand_point(tip, 0.0, 24.0, true).y < hand_point(tip, 0.0, 24.0, false).y);
        let context = egui::Context::default();
        let frame = |events, pointer, cursor| {
            context
                .run_ui(
                    egui::RawInput {
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        assert_eq!(paint(ui.painter(), pointer, cursor), pointer.is_some());
                    },
                )
                .shapes
                .len()
        };
        let press = egui::Event::PointerButton {
            pos: tip,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        };
        let pressed_shapes = frame(
            vec![egui::Event::PointerMoved(tip), press],
            Some(tip),
            egui::CursorIcon::PointingHand,
        );
        let dragged_shapes = frame(
            vec![egui::Event::PointerMoved(tip + egui::vec2(100.0, 0.0))],
            Some(tip),
            egui::CursorIcon::Default,
        );
        assert_eq!(pressed_shapes, dragged_shapes);
        let release = egui::Event::PointerButton {
            pos: tip,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            vec![release, egui::Event::PointerGone],
            None,
            egui::CursorIcon::Default,
        );
        let background_press = egui::Event::PointerButton {
            pos: tip,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        };
        let background_shapes = frame(
            vec![egui::Event::PointerMoved(tip), background_press],
            Some(tip),
            egui::CursorIcon::Default,
        );
        assert!(background_shapes < pressed_shapes);
    }

    #[test]
    fn styles_arrow_and_hand_but_preserves_native_editing_cursors() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(paint(
                ui.painter(),
                Some(Pos2::ZERO),
                egui::CursorIcon::Default
            ));
            assert!(paint(
                ui.painter(),
                Some(Pos2::ZERO),
                egui::CursorIcon::PointingHand
            ));
            for cursor in [
                egui::CursorIcon::Text,
                egui::CursorIcon::ResizeHorizontal,
                egui::CursorIcon::ResizeVertical,
                egui::CursorIcon::NotAllowed,
            ] {
                assert!(!paint(ui.painter(), Some(Pos2::ZERO), cursor));
            }
        });
    }
}
