use crate::app::rendering::glass::{self, Tint};
use crate::app::rendering::pixel::{self, PixelButton, Tone};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::skins;
use crate::app::theme::style::{self, metrics, Finish, Palette, Skin};
use crate::app::views::widgets::caption;
use crate::app::App;
use crate::model::Theme;
use eframe::egui;

const SWATCH: f32 = 14.0;

impl App {
    pub(in crate::app) fn show_settings(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let frame =
            egui::Frame::central_panel(&ctx.global_style()).inner_margin(egui::Margin::same(22));
        let mut close = false;
        egui::CentralPanel::default()
            .frame(frame)
            .show_inside(ui, |ui| {
                pixel::paint_scenery(ui);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    close = settings_page(ui, &mut self.config.skin, &mut self.config.theme);
                });
            });
        if close {
            self.settings_open = false;
        }
    }
}

/// Returns whether the user asked to go back to the jobs.
fn settings_page(ui: &mut egui::Ui, skin: &mut String, theme: &mut Theme) -> bool {
    let palette = Palette::of(ui);
    let close = ui
        .horizontal(|ui| {
            sprites::show(ui, Sprite::Gear, 4.0, palette.metal);
            ui.add_space(6.0);
            pixel::display_label(ui, "SETTINGS", 16.0, palette.ink);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let back = PixelButton::new("Back to jobs")
                    .sprite(Sprite::Rewind)
                    .tone(Tone::Accent);
                ui.add(back).clicked()
            })
            .inner
        })
        .inner;
    ui.add_space(14.0);
    pixel::section(ui, "Theme", Sprite::Monitor, palette.hot, |ui| {
        caption(ui, "Look");
        for candidate in skins::ALL {
            skin_card(ui, candidate, skin);
        }
    });
    pixel::section(ui, "Brightness", Sprite::Sun, palette.gold, |ui| {
        caption(ui, "Light or dark mode");
        ui.horizontal_wrapped(|ui| {
            for mode in Theme::ALL {
                pixel::choice(ui, theme, mode, mode.label(), Some(mode_sprite(mode)));
            }
        });
        ui.label(
            egui::RichText::new("System follows the operating system's light or dark setting.")
                .small()
                .color(palette.dim),
        );
    });
    close
}

/// A selectable card naming the skin and previewing both of its palettes.
fn skin_card(ui: &mut egui::Ui, candidate: &'static Skin, selected_id: &mut String) {
    let palette = Palette::of(ui);
    let selected = *selected_id == candidate.id;
    let background = ui.painter().add(egui::Shape::Noop);
    let frame = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    let ink = if selected {
                        palette.accent_ink
                    } else {
                        palette.ink
                    };
                    ui.label(egui::RichText::new(candidate.name).heading().color(ink));
                    ui.label(egui::RichText::new(candidate.description).color(ink));
                    swatches(ui, &candidate.dark);
                    swatches(ui, &candidate.light);
                });
                if selected {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pixel::badge(ui, "Active", palette.good);
                    });
                }
            });
        });
    let rect = frame.response.rect;
    let response = ui
        .interact(rect, ui.id().with(candidate.id), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() && !selected {
        *selected_id = candidate.id.to_owned();
    }
    if Skin::of(ui.ctx()).finish == Finish::Glass {
        let tint = match (selected, response.hovered()) {
            (true, _) => Tint::Accent,
            (false, true) => Tint::Hover,
            (false, false) => Tint::Rest,
        };
        ui.painter()
            .set(background, glass::tile(ui.ctx(), rect, tint));
        return;
    }
    let fill = match (selected, response.hovered()) {
        (true, _) => palette.accent,
        (false, true) => palette.raised,
        (false, false) => palette.sunken,
    };
    let radius = style::panel_radius(ui.ctx());
    ui.painter().set(
        background,
        vec![
            egui::Shape::rect_filled(rect, radius, style::glazed(ui.ctx(), fill)),
            egui::Shape::rect_stroke(
                rect,
                radius,
                egui::Stroke::new(metrics(ui.ctx()).border, palette.outline),
                egui::StrokeKind::Inside,
            ),
        ],
    );
}

/// A row of the palette's main colors.
fn swatches(ui: &mut egui::Ui, palette: &Palette) {
    let colors = [
        palette.background,
        palette.surface,
        palette.ink,
        palette.accent,
        palette.hot,
        palette.gold,
        palette.good,
        palette.bad,
    ];
    let outline = Palette::of(ui).outline;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for color in colors {
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(SWATCH), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                egui::CornerRadius::same(metrics(ui.ctx()).radius / 4),
                color,
            );
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(metrics(ui.ctx()).radius / 4),
                egui::Stroke::new(1.0_f32, outline),
                egui::StrokeKind::Inside,
            );
        }
    });
}

fn mode_sprite(theme: Theme) -> Sprite {
    match theme {
        Theme::System => Sprite::Monitor,
        Theme::Light => Sprite::Sun,
        Theme::Dark => Sprite::Moon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_page_renders_every_skin_and_keeps_the_choice() {
        for candidate in skins::ALL {
            for mode in Theme::ALL {
                let mut skin = candidate.id.to_owned();
                let mut theme = mode;
                let context = egui::Context::default();
                crate::app::theme::style::install(&context, candidate);
                context.set_theme(match mode {
                    Theme::System => egui::ThemePreference::System,
                    Theme::Light => egui::ThemePreference::Light,
                    Theme::Dark => egui::ThemePreference::Dark,
                });
                let _ = context.run_ui(egui::RawInput::default(), |ui| {
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        assert!(!settings_page(ui, &mut skin, &mut theme));
                    });
                });
                assert_eq!(skin, candidate.id);
                assert_eq!(theme, mode);
            }
        }
    }
}
