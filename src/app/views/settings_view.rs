use crate::app::rendering::glass::{self, Tint};
use crate::app::rendering::liquid;
use crate::app::rendering::pixel::{self, PixelCheckbox};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::skins;
use crate::app::theme::style::{self, metrics, Finish, Palette, Skin};
use crate::app::views::widgets::{caption, centered_row, gib_field, page_frame, page_scroll_area};
use crate::app::App;
use crate::model::{Config, JobDefaults};
use eframe::egui;

const SWATCH: f32 = 14.0;

#[derive(Clone, Copy, Default, PartialEq)]
pub(in crate::app) enum SettingsTab {
    #[default]
    Jobs,
    Theme,
}

impl App {
    pub(in crate::app) fn show_settings(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(page_frame(ui))
            .show_inside(ui, |ui| {
                pixel::paint_scenery(ui);
                page_scroll_area(ui, |ui| {
                    settings_page(ui, &mut self.config, &mut self.settings_tab);
                });
            });
    }
}

fn settings_page(ui: &mut egui::Ui, config: &mut Config, tab: &mut SettingsTab) {
    let palette = Palette::of(ui);
    ui.horizontal(|ui| {
        sprites::show(ui, Sprite::Gear, 4.0, palette.metal);
        ui.add_space(6.0);
        pixel::display_label(ui, "SETTINGS", 16.0, palette.ink);
    });
    ui.add_space(14.0);
    ui.horizontal_wrapped(|ui| {
        pixel::choice(
            ui,
            tab,
            SettingsTab::Jobs,
            "Job settings",
            Some(Sprite::JobSettings),
        );
        pixel::choice(ui, tab, SettingsTab::Theme, "Theme", Some(Sprite::Theme));
    });
    ui.add_space(14.0);
    match tab {
        SettingsTab::Jobs => job_defaults(ui, &mut config.job_defaults),
        SettingsTab::Theme => {
            pixel::section(ui, "Theme", Sprite::Theme, palette.hot, |ui| {
                caption(ui, "Look");
                for candidate in skins::ALL {
                    skin_card(ui, candidate, &mut config.skin);
                }
            });
        }
    }
}

fn job_defaults(ui: &mut egui::Ui, defaults: &mut JobDefaults) {
    let palette = Palette::of(ui);
    ui.label("Defaults apply to new jobs. Each job can override these settings.");
    pixel::section(ui, "Folders", Sprite::Folder, palette.accent, |ui| {
        centered_row(ui, |ui| {
            ui.add(PixelCheckbox::new(
                &mut defaults.auto_destination,
                "Auto destination folder",
            ));
        });
        centered_row(ui, |ui| {
            ui.add(PixelCheckbox::new(&mut defaults.zip, "Compress as ZIP"));
        });
    });
    pixel::section(ui, "Scheduling", Sprite::Clock, palette.hot, |ui| {
        centered_row(ui, |ui| {
            ui.add(PixelCheckbox::new(
                &mut defaults.check_on_enable,
                "Check immediately when enabled",
            ));
        });
        let mut job = defaults.new_job();
        super::editor::schedule_controls(ui, &mut job);
        defaults.update_schedule(&job);
    });
    pixel::section(ui, "Retention", Sprite::Shield, palette.good, |ui| {
        centered_row(ui, |ui| {
            ui.add(PixelCheckbox::new(
                &mut defaults.rolling,
                "Enable rolling retention",
            ));
        });
        ui.add_enabled_ui(defaults.rolling, |ui| {
            centered_row(ui, |ui| {
                ui.add(PixelCheckbox::new(&mut defaults.count_enabled, "Max count"));
                ui.add_enabled(
                    defaults.count_enabled,
                    egui::DragValue::new(&mut defaults.max_count).range(1..=100_000),
                );
            });
            centered_row(ui, |ui| {
                ui.add(PixelCheckbox::new(
                    &mut defaults.size_enabled,
                    "Max total size",
                ));
                gib_field(ui, defaults.size_enabled, &mut defaults.max_bytes);
            });
            centered_row(ui, |ui| {
                ui.add(PixelCheckbox::new(
                    &mut defaults.count_protected,
                    "Count protected backups toward retention limits",
                ))
                .on_hover_text("Protected backups are never deleted by retention");
            });
        });
    });
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
            ui.spacing_mut().item_spacing.y = 4.0;
            let ink = if selected {
                palette.accent_ink
            } else {
                palette.ink
            };
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(candidate.name).heading().color(ink));
                if selected {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        pixel::badge(ui, "Active", palette.good);
                    });
                }
            });
            ui.label(egui::RichText::new(candidate.description).color(ink));
            swatches(ui, &candidate.dark);
            swatches(ui, &candidate.light);
        });
    let rect = frame.response.rect;
    let response =
        pixel::pointing_hand(ui.interact(rect, ui.id().with(candidate.id), egui::Sense::click()));
    if response.clicked() && !selected {
        *selected_id = candidate.id.to_owned();
    }
    let tint = match (selected, response.hovered()) {
        (true, _) => Tint::Accent,
        (false, true) => Tint::Hover,
        (false, false) => Tint::Rest,
    };
    match Skin::of(ui.ctx()).finish {
        Finish::Glass => {
            ui.painter()
                .set(background, glass::tile(ui.ctx(), rect, tint));
            return;
        }
        Finish::Liquid => {
            ui.painter()
                .set(background, liquid::tile(ui.ctx(), rect, tint));
            return;
        }
        _ => {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Theme;

    #[test]
    fn job_defaults_render_in_every_skin_without_changing_values() {
        for candidate in skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let context = egui::Context::default();
                style::install(&context, candidate);
                context.set_theme(theme);
                let mut config = Config::default();
                let before = config.clone();
                let mut tab = SettingsTab::Jobs;
                let _ = context.run_ui(egui::RawInput::default(), |ui| {
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        settings_page(ui, &mut config, &mut tab);
                    });
                });
                assert!(config == before);
            }
        }
    }

    #[test]
    fn theme_cards_fit_the_page_with_or_without_a_vertical_scrollbar() {
        for skin in skins::ALL {
            let ctx = egui::Context::default();
            style::install(&ctx, skin);
            let mut config = Config {
                skin: skin.id.into(),
                ..Config::default()
            };
            for width in (480..=810).step_by(5) {
                let _ = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width as f32, 700.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        let scroll = page_scroll_area(ui, |ui| {
                            settings_page(ui, &mut config, &mut SettingsTab::Theme);
                        });
                        assert!(
                            scroll.content_size.x <= scroll.inner_rect.width(),
                            "{} at {width}: {} > {}",
                            skin.id,
                            scroll.content_size.x,
                            scroll.inner_rect.width()
                        );
                    },
                );
            }
        }
    }

    #[test]
    fn settings_page_renders_every_skin_and_keeps_the_choice() {
        for candidate in skins::ALL {
            for mode in Theme::ALL {
                let mut config = Config {
                    skin: candidate.id.to_owned(),
                    ..Config::default()
                };
                let mut tab = SettingsTab::Theme;
                let context = egui::Context::default();
                crate::app::theme::style::install(&context, candidate);
                context.set_theme(match mode {
                    Theme::System => egui::ThemePreference::System,
                    Theme::Light => egui::ThemePreference::Light,
                    Theme::Dark => egui::ThemePreference::Dark,
                });
                let _ = context.run_ui(egui::RawInput::default(), |ui| {
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        settings_page(ui, &mut config, &mut tab);
                    });
                });
                assert_eq!(config.skin, candidate.id);
            }
        }
    }
}
