//! Flat controls and semantic line icons for the Modern skin.
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::style::{label_font, Palette};
use eframe::egui::{self, Color32, Rect, Stroke, StrokeKind};

pub fn control(painter: &egui::Painter, rect: Rect, fill: Color32, focused: bool, enabled: bool) {
    let palette = Palette::of_ctx(painter.ctx());
    painter.rect_filled(rect, 6, fill);
    painter.rect_stroke(
        rect,
        6,
        Stroke::new(1.0_f32, palette.outline),
        StrokeKind::Inside,
    );
    if focused && enabled {
        painter.rect_stroke(
            rect.expand(2.0),
            8,
            Stroke::new(2.0_f32, palette.accent),
            StrokeKind::Outside,
        );
    }
}

pub fn title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite) {
    let palette = Palette::of(ui);
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                sprites::show(ui, sprite, 2.0, palette.hot);
                ui.label(
                    egui::RichText::new(title)
                        .font(label_font(ui.ctx()))
                        .color(palette.ink),
                );
            });
        });
}

pub fn progress(ui: &mut egui::Ui, width: f32, color: Color32, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
    let palette = Palette::of(ui);
    ui.painter().rect_filled(rect, 4, palette.raised);
    if active {
        let phase = (ui.input(|input| input.time) * 0.7).fract() as f32;
        let segment = Rect::from_min_max(
            egui::pos2(rect.left() + rect.width() * phase, rect.top()),
            egui::pos2(
                (rect.left() + rect.width() * phase + rect.width() * 0.3).min(rect.right()),
                rect.bottom(),
            ),
        );
        ui.painter().rect_filled(segment, 4, color);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(30));
    }
    response
}

pub fn icon(painter: &egui::Painter, rect: Rect, sprite: Sprite, tint: Color32) {
    let point = |x: f32, y: f32| {
        egui::pos2(
            rect.left() + rect.width() * x,
            rect.top() + rect.height() * y,
        )
    };
    let stroke = Stroke::new(
        (rect.width().min(rect.height()) / 12.0).clamp(1.0, 2.5),
        tint,
    );
    let line = |points: &[(f32, f32)]| {
        painter.add(egui::Shape::line(
            points.iter().map(|&(x, y)| point(x, y)).collect(),
            stroke,
        ));
    };
    let box_at = |x, y, w, h| {
        painter.rect_stroke(
            Rect::from_min_max(point(x, y), point(x + w, y + h)),
            2,
            stroke,
            StrokeKind::Inside,
        );
    };
    match sprite {
        Sprite::Floppy => {
            box_at(0.15, 0.1, 0.7, 0.8);
            box_at(0.3, 0.1, 0.4, 0.3);
            box_at(0.3, 0.6, 0.4, 0.3);
        }
        Sprite::Folder | Sprite::JobSettings | Sprite::FolderBackup => line(&[
            (0.1, 0.3),
            (0.1, 0.85),
            (0.9, 0.85),
            (0.9, 0.3),
            (0.5, 0.3),
            (0.4, 0.15),
            (0.1, 0.15),
            (0.1, 0.3),
        ]),
        Sprite::Clock | Sprite::Weekdays => {
            painter.circle_stroke(point(0.5, 0.5), rect.width() * 0.38, stroke);
            line(&[(0.5, 0.25), (0.5, 0.5), (0.7, 0.6)]);
        }
        Sprite::Shield => line(&[
            (0.5, 0.1),
            (0.85, 0.25),
            (0.8, 0.65),
            (0.5, 0.9),
            (0.2, 0.65),
            (0.15, 0.25),
            (0.5, 0.1),
        ]),
        Sprite::Trash => {
            line(&[(0.15, 0.25), (0.85, 0.25)]);
            line(&[(0.25, 0.3), (0.3, 0.9), (0.7, 0.9), (0.75, 0.3)]);
            line(&[(0.35, 0.2), (0.35, 0.1), (0.65, 0.1), (0.65, 0.2)]);
        }
        Sprite::Rewind | Sprite::Back | Sprite::Interval => {
            line(&[(0.5, 0.2), (0.2, 0.5), (0.5, 0.8)]);
            line(&[(0.2, 0.5), (0.85, 0.5)]);
        }
        Sprite::Play | Sprite::Cursor => {
            line(&[(0.25, 0.15), (0.85, 0.5), (0.25, 0.85), (0.25, 0.15)])
        }
        Sprite::Chest => {
            box_at(0.1, 0.3, 0.8, 0.6);
            line(&[(0.1, 0.5), (0.9, 0.5)]);
            line(&[(0.5, 0.4), (0.5, 0.65)]);
        }
        Sprite::Package => {
            box_at(0.15, 0.15, 0.7, 0.7);
            line(&[(0.15, 0.35), (0.85, 0.35)]);
            line(&[(0.5, 0.15), (0.5, 0.5)]);
        }
        Sprite::Sun | Sprite::Daily => {
            painter.circle_stroke(point(0.5, 0.5), rect.width() * 0.2, stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                line(&[
                    (0.5 + a.cos() * 0.32, 0.5 + a.sin() * 0.32),
                    (0.5 + a.cos() * 0.44, 0.5 + a.sin() * 0.44),
                ]);
            }
        }
        Sprite::Moon => line(&[
            (0.6, 0.1),
            (0.35, 0.15),
            (0.15, 0.4),
            (0.2, 0.7),
            (0.45, 0.85),
            (0.75, 0.8),
            (0.9, 0.6),
            (0.65, 0.65),
            (0.45, 0.45),
            (0.45, 0.25),
            (0.6, 0.1),
        ]),
        Sprite::Monitor | Sprite::Theme => {
            box_at(0.1, 0.15, 0.8, 0.55);
            line(&[(0.5, 0.7), (0.5, 0.9)]);
            line(&[(0.3, 0.9), (0.7, 0.9)]);
        }
        Sprite::Warning => {
            line(&[(0.5, 0.1), (0.95, 0.85), (0.05, 0.85), (0.5, 0.1)]);
            line(&[(0.5, 0.35), (0.5, 0.6)]);
            painter.circle_filled(point(0.5, 0.73), stroke.width * 0.6, tint);
        }
        Sprite::Power => {
            let points: Vec<_> = (0..25)
                .map(|i| {
                    let a = -1.0 + i as f32 * (std::f32::consts::TAU - 1.14) / 24.0;
                    (0.5 + a.cos() * 0.35, 0.5 + a.sin() * 0.35)
                })
                .collect();
            line(&points);
            line(&[(0.5, 0.05), (0.5, 0.5)]);
        }
        Sprite::Check => line(&[(0.1, 0.5), (0.4, 0.8), (0.9, 0.15)]),
        Sprite::Plus => {
            line(&[(0.5, 0.15), (0.5, 0.85)]);
            line(&[(0.15, 0.5), (0.85, 0.5)]);
        }
        Sprite::Gear => {
            painter.circle_stroke(point(0.5, 0.5), rect.width() * 0.28, stroke);
            painter.circle_stroke(point(0.5, 0.5), rect.width() * 0.1, stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                line(&[
                    (0.5 + a.cos() * 0.28, 0.5 + a.sin() * 0.28),
                    (0.5 + a.cos() * 0.43, 0.5 + a.sin() * 0.43),
                ]);
            }
        }
    }
}

fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

/// Selects the higher-contrast foreground for neutral and semantic filled controls.
pub fn foreground(fill: Color32) -> Color32 {
    let dark = Color32::from_rgb(0x18, 0x18, 0x1b);
    let background = luminance(fill);
    let contrast_dark = (background + 0.05) / (luminance(dark) + 0.05);
    let contrast_white = 1.05 / (background + 0.05);
    if contrast_dark > contrast_white {
        dark
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::rendering::pixel;
    use crate::app::theme::skins;
    use crate::app::theme::style;

    #[test]
    fn modern_text_and_controls_have_readable_contrast() {
        let skin = skins::find("modern");
        let contrast = |a: Color32, b: Color32| {
            let a = luminance(a);
            let b = luminance(b);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        for palette in [&skin.dark, &skin.light] {
            for surface in [palette.background, palette.surface, palette.raised] {
                for ink in [
                    palette.ink,
                    palette.dim,
                    palette.hot,
                    palette.good,
                    palette.gold,
                    palette.bad,
                ] {
                    assert!(contrast(surface, ink) >= 4.5, "{surface:?} / {ink:?}");
                }
            }
            for base in [palette.accent, palette.good, palette.gold, palette.bad] {
                for fill in [
                    base,
                    pixel::mix(base, Color32::WHITE, 0.18),
                    pixel::mix(base, Color32::BLACK, 0.12),
                ] {
                    assert!(contrast(fill, foreground(fill)) >= 4.5, "{fill:?}");
                }
            }
        }
    }

    #[test]
    fn every_modern_icon_paints_in_both_modes() {
        let ctx = egui::Context::default();
        style::install(&ctx, skins::find("modern"));
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.set_theme(theme);
            let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let tint = Palette::of(ui).ink;
                    for sprite in Sprite::ALL {
                        sprites::show(ui, sprite, 2.0, tint);
                    }
                });
            });
            assert!(!ctx
                .tessellate(output.shapes, output.pixels_per_point)
                .is_empty());
        }
    }
}
