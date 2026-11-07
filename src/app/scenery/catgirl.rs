//! A single adult nautical catgirl with live gaze and brief, non-repeating click reactions.

use crate::app::rendering::pixel::mix;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::TAU;

const ART: &[u8] = include_bytes!("../../../assets/themes/kawaii/catgirl.png");
const CROP_HEIGHT: f32 = 1044.0;
const REACTION_SECONDS: f64 = 3.0;

pub fn reserved_height(width: f32) -> f32 {
    width.min(340.0) * CROP_HEIGHT / 1024.0 + 48.0
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Expression {
    #[default]
    Pout,
    Excited,
    Surprised,
    Smirk,
    Smug,
    Shy,
}

impl Expression {
    const ALL: [Self; 6] = [
        Self::Pout,
        Self::Excited,
        Self::Surprised,
        Self::Smirk,
        Self::Smug,
        Self::Shy,
    ];

    fn next(self, random: u128) -> Self {
        let index = Self::ALL
            .iter()
            .position(|value| *value == self)
            .unwrap_or(0);
        Self::ALL[(index + 1 + random as usize % (Self::ALL.len() - 1)) % Self::ALL.len()]
    }

    fn line(self) -> &'static str {
        match self {
            Self::Pout => "Hmph! I was keeping an eye on your files.",
            Self::Excited => "You came to see me! I mean… your backups!",
            Self::Surprised => "Nya?! You startled me!",
            Self::Smirk => "Good thing you have me looking after things.",
            Self::Smug => "Of course I noticed. I notice everything.",
            Self::Shy => "It's nice when you stop by…",
        }
    }
}

#[derive(Clone, Default)]
struct Companion {
    expression: Expression,
    until: f64,
    gaze: Vec2,
}

fn texture(ctx: &egui::Context) -> Result<egui::TextureHandle, String> {
    let id = egui::Id::new("kawaii_catgirl_art");
    if let Some(cached) = ctx.data_mut(|data| data.get_temp(id)) {
        return cached;
    }
    let loaded = image::load_from_memory(ART)
        .map(|image| {
            let rgba = image.to_rgba8();
            ctx.load_texture(
                "Kawaii catgirl",
                egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                ),
                egui::TextureOptions::LINEAR,
            )
        })
        .map_err(|error| format!("Could not load Kawaii catgirl: {error}"));
    if let Err(error) = &loaded {
        eprintln!("{error}");
    }
    ctx.data_mut(|data| data.insert_temp(id, loaded.clone()));
    loaded
}

pub fn show(ui: &mut egui::Ui) {
    let Ok(texture) = texture(ui.ctx()) else {
        return;
    };
    let width = ui.available_width().min(340.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, width * CROP_HEIGHT / 1024.0),
        egui::Sense::click(),
    );
    let (time, pointer, dt) =
        ui.input(|input| (input.time, input.pointer.hover_pos(), input.stable_dt));
    let id = egui::Id::new("kawaii_companion_state");
    let mut state: Companion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());
    if response.clicked() {
        state.expression = state.expression.next(uuid::Uuid::new_v4().as_u128());
        state.until = time + REACTION_SECONDS;
    }
    let face_center = rect.min + egui::vec2(514.0, 480.0) * (width / 1024.0);
    let wanted = gaze(pointer, face_center, width);
    state.gaze += (wanted - state.gaze) * (1.0 - (-dt.min(0.05) * 14.0).exp());
    let expression = (time < state.until).then_some(state.expression);
    let bounce = if expression.is_some() {
        let age = (time - (state.until - REACTION_SECONDS)) as f32;
        (age * 14.0).sin() * (-age * 3.5).exp() * 3.0
    } else {
        0.0
    };
    let art_rect = rect.translate(egui::vec2(0.0, bounce));
    ui.painter().image(
        texture.id(),
        art_rect,
        Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, CROP_HEIGHT / 1536.0)),
        Color32::WHITE,
    );
    face(ui.painter(), art_rect, state.gaze, expression, time);
    response.on_hover_text("Click for a reaction");
    if let Some(expression) = expression {
        ui.label(egui::RichText::new(expression.line()).small());
    } else {
        ui.label(egui::RichText::new("I'm watching… your files, obviously.").small());
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(33));
}

fn gaze(pointer: Option<Pos2>, center: Pos2, width: f32) -> Vec2 {
    pointer.map_or(Vec2::ZERO, |point| {
        let offset = (point - center) / width.max(1.0);
        egui::vec2(offset.x.clamp(-1.0, 1.0), offset.y.clamp(-1.0, 1.0))
    })
}

fn face(
    painter: &egui::Painter,
    rect: Rect,
    gaze: Vec2,
    expression: Option<Expression>,
    time: f64,
) {
    let scale = rect.width() / 1024.0;
    let at = |x, y| rect.min + egui::vec2(x, y) * scale;
    let ink = Color32::from_rgb(68, 27, 48);
    let blink = time.rem_euclid(4.7) > 4.56 && expression != Some(Expression::Surprised);
    for (x, side) in [(419.0, -1.0), (609.0, 1.0)] {
        let center = at(x, 478.0);
        let narrow = matches!(expression, Some(Expression::Smug | Expression::Smirk));
        let height = if blink {
            2.0
        } else if narrow {
            15.0
        } else {
            29.0
        };
        eye(painter, center, scale, height, gaze, expression);
        let brow_y = if expression == Some(Expression::Surprised) {
            420.0
        } else {
            433.0
        };
        let tilt = match expression {
            Some(Expression::Pout) => -10.0,
            Some(Expression::Shy | Expression::Surprised) => 6.0,
            _ => -4.0,
        } * side;
        painter.line_segment(
            [at(x - 48.0, brow_y - tilt), at(x + 48.0, brow_y + tilt)],
            Stroke::new(3.2 * scale, ink),
        );
    }
    let center = at(514.0, 587.0);
    match expression {
        Some(Expression::Surprised) => {
            painter.add(egui::Shape::ellipse_filled(
                center,
                egui::vec2(15.0, 23.0) * scale,
                ink,
            ));
            painter.add(egui::Shape::ellipse_filled(
                center + Vec2::Y * 9.0 * scale,
                egui::vec2(10.0, 7.0) * scale,
                Color32::from_rgb(224, 119, 145),
            ));
        }
        Some(Expression::Excited) => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    at(482.0, 578.0),
                    at(546.0, 578.0),
                    at(538.0, 604.0),
                    at(514.0, 614.0),
                    at(490.0, 604.0),
                ],
                Color32::from_rgb(166, 57, 89),
                Stroke::new(2.0 * scale, ink),
            ));
            painter.line_segment(
                [at(489.0, 582.0), at(539.0, 582.0)],
                Stroke::new(5.0 * scale, Color32::WHITE),
            );
        }
        Some(Expression::Pout) => {
            painter.add(egui::Shape::line(
                vec![
                    at(488.0, 591.0),
                    at(504.0, 585.0),
                    at(514.0, 590.0),
                    at(524.0, 585.0),
                    at(540.0, 591.0),
                ],
                Stroke::new(3.0 * scale, ink),
            ));
        }
        _ => {
            let lift = if expression == Some(Expression::Smirk) {
                12.0
            } else {
                0.0
            };
            painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [
                    at(486.0, 581.0),
                    at(502.0, 601.0),
                    at(531.0, 597.0),
                    at(546.0, 580.0 - lift),
                ],
                false,
                Color32::TRANSPARENT,
                Stroke::new(3.0 * scale, ink),
            ));
        }
    }
    if matches!(expression, Some(Expression::Shy | Expression::Pout)) {
        for x in [365.0, 663.0] {
            painter.add(egui::Shape::ellipse_filled(
                at(x, 524.0),
                egui::vec2(32.0, 12.0) * scale,
                Color32::from_rgba_unmultiplied(244, 106, 134, 48),
            ));
        }
    }
}

fn eye(
    painter: &egui::Painter,
    center: Pos2,
    scale: f32,
    height: f32,
    gaze: Vec2,
    expression: Option<Expression>,
) {
    let ink = Color32::from_rgb(62, 27, 49);
    let points: Vec<_> = (0..32)
        .map(|step| {
            let angle = step as f32 / 32.0 * TAU;
            let sine = angle.sin();
            let y = sine.signum() * sine.abs().powf(1.7) * height;
            center + egui::vec2(angle.cos() * 58.0, y) * scale
        })
        .collect();
    painter.add(egui::Shape::convex_polygon(
        points,
        Color32::from_rgb(255, 250, 253),
        Stroke::new(2.0 * scale, ink),
    ));
    if height > 3.0 {
        let clip = Rect::from_center_size(center, egui::vec2(112.0, height * 1.7) * scale);
        let eye_painter = painter.with_clip_rect(painter.clip_rect().intersect(clip));
        let iris = center + egui::vec2(gaze.x * 13.0, gaze.y * 6.0) * scale;
        eye_painter.add(egui::Shape::ellipse_filled(
            iris,
            egui::vec2(28.0, 30.0) * scale,
            Color32::from_rgb(69, 32, 99),
        ));
        eye_painter.add(egui::Shape::ellipse_filled(
            iris + Vec2::Y * 3.0 * scale,
            egui::vec2(23.0, 25.0) * scale,
            Color32::from_rgb(164, 91, 190),
        ));
        eye_painter.add(egui::Shape::ellipse_filled(
            iris,
            egui::vec2(8.0, 19.0) * scale,
            ink,
        ));
        if expression == Some(Expression::Excited) {
            star(&eye_painter, iris, 23.0 * scale);
        } else {
            eye_painter.circle_filled(
                iris + egui::vec2(-7.0, -10.0) * scale,
                6.0 * scale,
                Color32::WHITE,
            );
            eye_painter.circle_filled(
                iris + egui::vec2(8.0, 11.0) * scale,
                3.0 * scale,
                Color32::from_rgb(255, 217, 246),
            );
        }
    }
    let upper: Vec<_> = (0..=16)
        .map(|step| {
            let angle = std::f32::consts::PI + step as f32 / 16.0 * std::f32::consts::PI;
            center + egui::vec2(angle.cos() * 58.0, -angle.sin().abs().powf(1.7) * height) * scale
        })
        .collect();
    painter.add(egui::Shape::line(upper, Stroke::new(5.0 * scale, ink)));
    for side in [-1.0, 1.0] {
        painter.line_segment(
            [
                center + egui::vec2(side * 51.0, -height * 0.5) * scale,
                center + egui::vec2(side * 65.0, -height * 0.9 - 5.0) * scale,
            ],
            Stroke::new(3.0 * scale, ink),
        );
    }
}

fn star(painter: &egui::Painter, center: Pos2, radius: f32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, Color32::WHITE);
    for index in 0..=8 {
        let reach = if index % 2 == 0 { radius } else { radius * 0.3 };
        let point = center + Vec2::angled(index as f32 / 8.0 * TAU) * reach;
        mesh.colored_vertex(
            point,
            mix(Color32::WHITE, Color32::from_rgb(255, 200, 83), 0.6),
        );
        if index > 0 {
            mesh.add_triangle(0, index, index + 1);
        }
    }
    painter.add(mesh);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactions_do_not_repeat_and_every_expression_can_be_reached() {
        for current in Expression::ALL {
            let next: Vec<_> = (0..5).map(|seed| current.next(seed)).collect();
            assert!(!next.contains(&current));
            assert!(Expression::ALL
                .iter()
                .all(|value| *value == current || next.contains(value)));
        }
    }

    #[test]
    fn gaze_is_bounded_and_returns_to_center_without_pointer() {
        assert_eq!(gaze(None, Pos2::ZERO, 300.0), Vec2::ZERO);
        assert_eq!(
            gaze(Some(egui::pos2(10000.0, -10000.0)), Pos2::ZERO, 300.0),
            egui::vec2(1.0, -1.0)
        );
    }

    #[test]
    fn artwork_and_every_expression_render_in_both_modes() {
        let image = image::load_from_memory(ART).unwrap().to_rgba8();
        assert_eq!(image.dimensions(), (1024, 1536));
        assert!(image.pixels().any(|pixel| pixel[3] == 0));
        let ctx = egui::Context::default();
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.set_theme(theme);
            for expression in Expression::ALL {
                let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    show(ui);
                    face(
                        ui.painter(),
                        Rect::from_min_size(Pos2::ZERO, egui::vec2(340.0, 346.0)),
                        Vec2::new(-1.0, 1.0),
                        Some(expression),
                        1.0,
                    );
                });
                assert!(!output.shapes.is_empty());
            }
        }
    }
}
