use crate::app::rendering::glass::{self, Tint};
use crate::app::rendering::liquid;
use crate::app::rendering::ornaments;
use crate::app::rendering::pixel::{self, Hint, PixelButton, Tone};
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::theme::skins;
use crate::app::theme::style::{self, Finish, Palette, Skin};
use crate::app::views::widgets::alert;
use crate::app::App;
use crate::model::{Job, Theme};
use chrono::Utc;
use eframe::egui;

const CARD_SHADOW: f32 = 4.0;
const CARD_HORIZONTAL_PADDING: i8 = 8;
const SIDEBAR_WIDTH: std::ops::RangeInclusive<f32> = 300.0..=460.0;

fn clamp_sidebar_width(width: f32) -> f32 {
    width.clamp(*SIDEBAR_WIDTH.start(), *SIDEBAR_WIDTH.end())
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.initialize_theme(ctx);
        if ctx.input(|input| input.viewport().close_requested()) {
            if !self.persist() {
                eprintln!("Could not save settings on exit: {}", self.error);
            }
            return;
        }
        self.ensure_skin(ctx);
        self.tick(ctx);
        if let Some(id) = self.selected {
            self.ensure_summary(id, ctx);
        }
        ctx.set_theme(theme_preference(self.config.theme));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        pixel::paint_underlay(&ctx);
        let before = self.config.clone();
        self.show_header(ui);
        self.show_footer(ui);
        self.show_job_list(ui);
        self.close_inactive_history();
        if self.settings_open {
            self.show_settings(ui);
        } else {
            self.show_job_editor(ui);
        }
        self.close_inactive_history();
        if self.config != before {
            self.dirty = true;
        }
        pixel::paint_overlays(&ctx);
    }
}

impl App {
    fn initialize_theme(&mut self, ctx: &egui::Context) {
        if self.config.theme == Theme::System {
            self.config.theme =
                if ctx.system_theme().unwrap_or(egui::Theme::Dark) == egui::Theme::Dark {
                    Theme::Dark
                } else {
                    Theme::Light
                };
            self.persist();
        }
    }

    /// Installs the configured skin whenever it differs from the one in use.
    fn ensure_skin(&self, ctx: &egui::Context) {
        let wanted = skins::find(&self.config.skin);
        if !style::installed(ctx).is_some_and(|skin| skin.id == wanted.id) {
            style::install(ctx, wanted);
        }
    }

    fn toggle_settings(&mut self) {
        self.settings_open = !self.settings_open;
        self.selected = None;
        self.backups_tab = false;
    }

    fn toggle_job(&mut self, id: uuid::Uuid) {
        self.selected = (self.selected != Some(id)).then_some(id);
        self.settings_open = false;
        self.backups_tab = false;
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let palette = palette_of(&ctx);
        let frame = pixel::panel_frame(&ctx, palette.surface).inner_margin(egui::Margin {
            left: 20,
            right: 20,
            top: 14,
            bottom: 0,
        });
        egui::Panel::top("header")
            .frame(frame)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                // Liquid glass keeps the top bar clear: only its buttons are glass.
                let liquid = Skin::of(ui.ctx()).finish == Finish::Liquid;
                if !liquid {
                    pixel::panel_decoration(ui, 1.9);
                }
                ui.horizontal(|ui| {
                    logo(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let settings = PixelButton::new("Settings")
                            .icon_only()
                            .sprite(Sprite::Gear)
                            .selected(self.settings_open);
                        if ui.add(settings).hint("Settings").clicked() {
                            self.toggle_settings();
                            if self.settings_open {
                                crate::app::scenery::catgirl::trigger(
                                    ui.ctx(),
                                    crate::app::scenery::catgirl::Action::Settings,
                                );
                            }
                        }
                        let dark = self.config.theme == Theme::Dark;
                        let (label, sprite) = if dark {
                            ("Light", Sprite::Sun)
                        } else {
                            ("Dark", Sprite::Moon)
                        };
                        if ui
                            .add(PixelButton::new(label).sprite(sprite).icon_only())
                            .hint(format!("Switch to {label} mode"))
                            .clicked()
                        {
                            self.config.theme = if dark { Theme::Light } else { Theme::Dark };
                            ctx.set_theme(theme_preference(self.config.theme));
                            crate::app::scenery::catgirl::trigger(
                                &ctx,
                                if dark {
                                    crate::app::scenery::catgirl::Action::LightMode
                                } else {
                                    crate::app::scenery::catgirl::Action::DarkMode
                                },
                            );
                        }
                    });
                });
                ui.add_space(12.0);
                let clip = ui.clip_rect();
                let top = ui.cursor().top();
                if liquid || Skin::of(ui.ctx()).finish == Finish::Glass {
                    ui.add_space(8.0);
                    return;
                }
                if !Skin::of(ui.ctx()).effects.stripes {
                    let rule = egui::Rect::from_min_max(
                        egui::pos2(clip.left(), top),
                        egui::pos2(clip.right(), top + 2.0),
                    );
                    ui.allocate_rect(rule, egui::Sense::hover());
                    if Skin::of(ui.ctx()).finish == Finish::Modern {
                        ui.painter().rect_filled(rule, 0, palette.outline);
                    } else {
                        ornaments::fading_rule(ui.painter(), rule, palette.accent);
                    }
                    return;
                }
                let band = egui::Rect::from_min_max(
                    egui::pos2(clip.left(), top),
                    egui::pos2(clip.right(), top + 16.0),
                );
                ui.allocate_rect(band, egui::Sense::hover());
                pixel::stripes(ui.painter(), band, palette);
                let rule = egui::Rect::from_min_max(
                    egui::pos2(clip.left(), band.bottom()),
                    egui::pos2(clip.right(), band.bottom() + 3.0),
                );
                ui.painter().rect_filled(rule, 0.0, palette.outline);
            });
    }

    fn show_footer(&self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let palette = palette_of(&ctx);
        // The liquid glass bar is a capsule, so its content keeps clear of the round ends.
        let side = if Skin::of(&ctx).finish == Finish::Liquid {
            34
        } else {
            18
        };
        let frame = pixel::panel_frame(&ctx, palette.sunken)
            .inner_margin(egui::Margin::symmetric(side, 10));
        egui::Panel::bottom("footer")
            .frame(frame)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                pixel::panel_decoration(ui, 1.0);
                if !matches!(Skin::of(ui.ctx()).finish, Finish::Glass | Finish::Liquid) {
                    let clip = ui.clip_rect();
                    let rule =
                        egui::Rect::from_min_size(clip.left_top(), egui::vec2(clip.width(), 3.0));
                    ui.painter().rect_filled(rule, 0.0, palette.outline);
                }
                ui.horizontal(|ui| {
                    pixel::display_label(ui, Skin::of(ui.ctx()).wording.ready, 10.0, palette.good);
                    ui.label(
                        egui::RichText::new("Scheduling runs while the app is open")
                            .color(palette.dim),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(112.0, 24.0), egui::Sense::hover());
                        let recent = self
                            .activity_until
                            .is_some_and(|until| until > std::time::Instant::now());
                        let active = self.active.is_some() || recent;
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .id_salt("activity")
                                .max_rect(rect)
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                            |ui| {
                                if Skin::of(ui.ctx()).finish == Finish::Gothic {
                                    crate::app::rendering::gothic::activity(
                                        ui.painter(),
                                        rect,
                                        active,
                                    );
                                    return;
                                }
                                pixel::led(
                                    ui,
                                    if active { palette.gold } else { palette.dim },
                                    active,
                                );
                                pixel::activity_track(ui, 80.0, palette.gold, active);
                            },
                        );
                        if active {
                            response.hint(if self.active.is_some() {
                                self.progress.as_str()
                            } else {
                                "Recent backup activity"
                            });
                        } else {
                            response.hint("No backup or restore in progress");
                        }
                    });
                });
                if !self.error.is_empty() {
                    alert(ui, palette.bad, &self.error);
                }
            });
    }

    fn show_job_list(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let palette = palette_of(&ctx);
        // The liquid glass sidebar floats inset in its panel; content keeps iOS's margin inside it.
        let inset = if Skin::of(&ctx).finish == Finish::Liquid {
            8
        } else {
            0
        };
        let frame = pixel::panel_frame(&ctx, palette.surface).inner_margin(egui::Margin {
            left: 16 + inset,
            right: 14 + inset,
            top: 16 + inset / 2,
            bottom: 8 + inset,
        });
        let panel = egui::Panel::left("jobs")
            .show_separator_line(Skin::of(&ctx).finish != Finish::Liquid)
            .default_size(clamp_sidebar_width(self.config.sidebar_width))
            .size_range(SIDEBAR_WIDTH)
            .frame(frame)
            .show_inside(ui, |ui| {
                pixel::panel_decoration(ui, 1.0);
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    pixel::display_label(
                        ui,
                        "JOBS",
                        16.0,
                        style::decorative_color(ui.ctx(), palette.accent),
                    );
                    let count = self.config.jobs.len().to_string();
                    // iOS lists counts as secondary text rather than in a badge.
                    if Skin::of(ui.ctx()).finish == Finish::Liquid {
                        ui.label(egui::RichText::new(count).color(palette.dim));
                    } else {
                        let color = style::decorative_color(ui.ctx(), palette.gold);
                        pixel::badge(ui, &count, color);
                    }
                });
                ui.add_space(10.0);
                let new_job = PixelButton::new("New job")
                    .sprite(Sprite::Plus)
                    .tone(Tone::Accent)
                    .min_size(egui::vec2(ui.available_width() - 4.0, 0.0));
                if ui.add_enabled(self.can_add_job(), new_job).clicked() {
                    self.add_job();
                    crate::app::scenery::catgirl::trigger(
                        ui.ctx(),
                        crate::app::scenery::catgirl::Action::NewJob,
                    );
                }
                ui.add_space(12.0);
                let show_mascot = Skin::of(ui.ctx()).effects.hearts
                    && (!self.config.jobs.is_empty() || self.settings_open);
                let list_height = if show_mascot {
                    (ui.available_height()
                        - ui.spacing().item_spacing.y
                        - crate::app::scenery::catgirl::reserved_height(ui))
                    .max(0.0)
                } else {
                    ui.available_height()
                };
                let mut clicked = None;
                egui::ScrollArea::vertical()
                    .max_height(list_height)
                    .show(ui, |ui| {
                        for job in &self.config.jobs {
                            if self.job_card(ui, job) {
                                clicked = Some(job.id);
                            }
                        }
                    });
                if let Some(id) = clicked {
                    self.toggle_job(id);
                    crate::app::scenery::catgirl::trigger(
                        ui.ctx(),
                        crate::app::scenery::catgirl::Action::SelectJob,
                    );
                }
                if show_mascot {
                    ui.vertical_centered(crate::app::scenery::catgirl::show);
                }
            });
        if ctx
            .read_response(egui::Id::new("jobs").with("__resize"))
            .is_some_and(|response| response.dragged())
        {
            self.config.sidebar_width = clamp_sidebar_width(panel.response.rect.width());
        }
    }

    /// Draws the job as a cartridge-like card; returns whether it was clicked.
    fn job_card(&self, ui: &mut egui::Ui, job: &Job) -> bool {
        ui.push_id(job.id, |ui| self.job_card_contents(ui, job))
            .inner
    }

    fn job_card_contents(&self, ui: &mut egui::Ui, job: &Job) -> bool {
        let palette = Palette::of(ui);
        let selected = self.selected == Some(job.id);
        let background = ui.painter().add(egui::Shape::Noop);
        let card_id = ui.id().with(job.id);
        // The card's hover is only known after its contents, so wake the icon from last frame's.
        let hovered = ui
            .ctx()
            .read_response(card_id)
            .is_some_and(|response| response.hovered());
        let wake = ui
            .ctx()
            .animate_bool_with_time(card_id.with("wake"), hovered, 0.2);
        // Running and queued work replaces the countdown in the card's readout.
        let badge = match self.activity_label(job.id) {
            Some(label) => Some((label.to_uppercase(), palette.gold)),
            None => job
                .countdown(Utc::now())
                .map(|remaining| (remaining.to_uppercase(), palette.good)),
        };
        let width = ui.available_width() - CARD_SHADOW;
        let gothic = Skin::of(ui.ctx()).finish == Finish::Gothic;
        let (top, bottom) = if gothic { (2, 0) } else { (4, 4) };
        let frame = egui::Frame::new()
            .inner_margin(egui::Margin {
                left: CARD_HORIZONTAL_PADDING,
                right: CARD_HORIZONTAL_PADDING,
                top,
                bottom,
            })
            .show(ui, |ui| {
                ui.set_width(width - 2.0 * f32::from(CARD_HORIZONTAL_PADDING));
                if gothic {
                    saya_job_identity(ui, job, badge);
                    return;
                }
                let well = style::well(ui.ctx());
                let countdown_size = badge.as_ref().map_or(egui::Vec2::ZERO, |(text, color)| {
                    ui.painter()
                        .layout_no_wrap(text.clone(), style::label_font(ui.ctx()), *color)
                        .size()
                        + well.total_margin().sum()
                });
                let icon_size = sprites::size(ui.ctx(), Sprite::Floppy, 3.0);
                let row_height = icon_size
                    .y
                    .max(countdown_size.y)
                    .max(ui.text_style_height(&egui::TextStyle::Body));
                let (row, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_height),
                    egui::Sense::hover(),
                );
                let icon_rect = egui::Rect::from_center_size(
                    egui::pos2(row.left() + icon_size.x / 2.0, row.center().y),
                    icon_size,
                );
                sprites::paint_awake(
                    ui.painter(),
                    icon_rect.min,
                    Sprite::Floppy,
                    3.0,
                    style::job_symbol_color(ui.ctx(), job.enabled),
                    wake,
                );
                let gap = ui.spacing().item_spacing.x;
                let name_right = if let Some((remaining, color)) = badge {
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(row.right() - countdown_size.x / 2.0, row.center().y),
                        countdown_size,
                    );
                    ui.scope_builder(
                        egui::UiBuilder::new()
                            .id_salt((job.id, "countdown"))
                            .max_rect(rect),
                        |ui| {
                            well.show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(remaining)
                                        .font(style::label_font(ui.ctx()))
                                        .color(color),
                                );
                            });
                        },
                    );
                    rect.left() - gap
                } else {
                    row.right()
                };
                let name_rect = egui::Rect::from_min_max(
                    egui::pos2(icon_rect.right() + gap, row.top()),
                    egui::pos2(name_right.max(icon_rect.right() + gap), row.bottom()),
                );
                let mut text = egui::text::LayoutJob::simple(
                    job.display_name().to_owned(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    palette.ink,
                    name_rect.width(),
                );
                text.wrap.max_rows = 1;
                text.wrap.break_anywhere = true;
                let galley = ui.painter().layout_job(text);
                let origin = egui::pos2(
                    name_rect.left(),
                    name_rect.center().y - galley.size().y / 2.0,
                );
                ui.painter()
                    .with_clip_rect(name_rect.intersect(ui.clip_rect()))
                    .galley(origin, galley, palette.ink);
            });
        let rect = frame.response.rect;
        let response = pixel::pointing_hand(ui.interact(rect, card_id, egui::Sense::click()));
        let glass_hover = ui.ctx().animate_bool_with_time(
            card_id.with("glass_hover"),
            response.hovered(),
            liquid::HOVER_SECONDS,
        );
        ui.painter().set(
            background,
            card_shapes(ui.ctx(), rect, selected, response.hovered(), glass_hover),
        );
        ui.add_space(CARD_SHADOW);
        response.clicked()
    }
}

/// Saya's head, the name, and the right-aligned readout share one vertically centered row.
fn saya_job_identity(ui: &mut egui::Ui, job: &Job, badge: Option<(String, egui::Color32)>) {
    const ICON_SIZE: f32 = 72.0;
    // The head artwork has about six points of transparent space above and below.
    const TRANSPARENT_INSET: f32 = 6.0;
    let palette = Palette::of(ui);
    let (row, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ICON_SIZE - 2.0 * TRANSPARENT_INSET),
        egui::Sense::hover(),
    );
    let icon = egui::Rect::from_min_size(
        row.min - egui::Vec2::Y * TRANSPARENT_INSET,
        egui::Vec2::splat(ICON_SIZE),
    );
    crate::app::rendering::gothic::job_icon(ui.painter(), icon, job.enabled);
    let gap = ui.spacing().item_spacing.x;
    let left = icon.right() + gap;
    let painter = ui.painter().with_clip_rect(row.intersect(ui.clip_rect()));
    let mut name_right = row.right();
    if let Some((text, color)) = badge {
        let text =
            ui.painter()
                .layout_no_wrap(text, egui::TextStyle::Small.resolve(ui.style()), color);
        let size = text.size() + egui::vec2(10.0, 6.0);
        let rect = egui::Rect::from_center_size(
            egui::pos2(row.right() - size.x / 2.0, row.center().y),
            size,
        );
        painter.rect_filled(rect, 3, palette.sunken);
        painter.galley(rect.min + egui::vec2(5.0, 3.0), text, color);
        name_right = rect.left() - gap;
    }
    let mut name = egui::text::LayoutJob::simple(
        job.display_name().to_owned(),
        egui::TextStyle::Body.resolve(ui.style()),
        palette.ink,
        (name_right - left).max(0.0),
    );
    name.wrap.max_rows = 2;
    name.wrap.break_anywhere = true;
    let name = ui.painter().layout_job(name);
    let origin = egui::pos2(left, row.center().y - name.size().y / 2.0);
    painter.galley(origin, name, palette.ink);
}

/// The skin's mascot next to the wordmark; pixel mascots get a twinkling glint.
fn logo(ui: &mut egui::Ui) {
    let palette = Palette::of(ui);
    let skin = Skin::of(ui.ctx());
    let mascot = match skin.finish {
        Finish::Gothic => crate::app::scenery::catgirl::show_header(ui),
        Finish::Liquid => sprites::show(ui, skin.wording.mascot, 4.0, palette.accent),
        _ => sprites::show(ui, skin.wording.mascot, 4.0, palette.hot),
    };
    if skin.finish == Finish::Pixel && pixel::blink(ui, 0.8) {
        let glint = mascot.rect.right_top() + egui::vec2(-2.0, 2.0);
        for (offset, size) in [
            (egui::vec2(-2.0, -6.0), egui::vec2(4.0, 12.0)),
            (egui::vec2(-6.0, -2.0), egui::vec2(12.0, 4.0)),
        ] {
            let rect = egui::Rect::from_min_size(glint + offset, size);
            ui.painter().rect_filled(rect, 0.0, palette.paper);
        }
    }
    ui.add_space(6.0);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        pixel::display_label(
            ui,
            "PORTBAK",
            if skin.finish == Finish::Modern {
                20.0
            } else {
                24.0
            },
            if matches!(skin.finish, Finish::Modern | Finish::Liquid) {
                palette.ink
            } else {
                palette.gold
            },
        );
        let tagline = Skin::of(ui.ctx()).wording.tagline;
        if !tagline.is_empty() {
            ui.label(egui::RichText::new(tagline).color(palette.accent));
        }
    });
}

/// `glass_hover` eases `hovered` in and out for liquid glass.
fn card_shapes(
    ctx: &egui::Context,
    rect: egui::Rect,
    selected: bool,
    hovered: bool,
    glass_hover: f32,
) -> Vec<egui::Shape> {
    let tint = match (selected, hovered) {
        (true, _) => Tint::Current,
        (false, true) => Tint::Hover,
        (false, false) => Tint::Rest,
    };
    match Skin::of(ctx).finish {
        Finish::Glass => return glass::tile(ctx, rect, tint),
        Finish::Liquid => return liquid::tile(ctx, rect, tint, glass_hover),
        _ => {}
    }
    let palette = palette_of(ctx);
    let shadow = style::drop_shadow(style::metrics(ctx), palette, CARD_SHADOW as i8);
    let fill = match (selected, hovered) {
        (true, _) => palette.raised,
        (false, true) => pixel::mix(palette.surface, palette.raised, 0.6),
        (false, false) => palette.surface,
    };
    let border = if selected {
        palette.accent
    } else {
        palette.outline
    };
    let modern = matches!(Skin::of(ctx).finish, Finish::Modern | Finish::Gothic);
    let radius = style::panel_radius(ctx);
    let mut shapes = vec![
        shadow.as_shape(rect, radius).into(),
        egui::Shape::rect_filled(rect, radius, style::glazed(ctx, fill)),
        egui::Shape::rect_stroke(
            rect,
            radius,
            egui::Stroke::new(
                if modern {
                    1.0
                } else if selected {
                    3.0_f32
                } else {
                    2.0
                },
                border,
            ),
            egui::StrokeKind::Inside,
        ),
    ];
    if selected && !modern {
        let notch = egui::Rect::from_min_size(rect.min, egui::vec2(6.0, rect.height()));
        shapes.push(egui::Shape::rect_filled(notch, 0.0, palette.accent));
    }
    shapes
}

fn palette_of(ctx: &egui::Context) -> &'static Palette {
    Palette::of_ctx(ctx)
}

fn theme_preference(theme: Theme) -> egui::ThemePreference {
    match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Light => egui::ThemePreference::Light,
        Theme::Dark => egui::ThemePreference::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Config;

    #[test]
    fn settings_leave_the_job_and_close_to_the_main_screen() {
        let mut app = App::new(
            Config {
                jobs: vec![Job::default()],
                ..Config::default()
            },
            std::path::PathBuf::new(),
            tempfile::tempfile().unwrap(),
        );
        app.selected = Some(app.config.jobs[0].id);
        app.backups_tab = true;
        app.toggle_settings();
        assert!(app.settings_open);
        assert!(app.selected.is_none());
        assert!(!app.backups_tab);
        app.toggle_settings();
        assert!(!app.settings_open);
        assert!(app.selected.is_none());
        app.toggle_settings();
        let id = app.config.jobs[0].id;
        app.toggle_job(id);
        assert_eq!(app.selected, Some(id));
        assert!(!app.settings_open);
        app.toggle_job(id);
        assert!(app.selected.is_none());
    }

    #[test]
    fn check_transitions_do_not_emit_widget_id_warnings() {
        for skin in skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let ctx = egui::Context::default();
                style::install(&ctx, skin);
                ctx.set_theme(theme);
                let mut app = App::new(
                    Config {
                        jobs: vec![Job {
                            name: "My job".into(),
                            ..Job::default()
                        }],
                        ..Config::default()
                    },
                    std::path::PathBuf::new(),
                    tempfile::tempfile().unwrap(),
                );
                let mut footer_label = None;
                for frame in 0..8 {
                    app.active = (frame % 2 == 1).then_some(app.config.jobs[0].id);
                    app.progress = "Checking source files ...".into();
                    app.config.jobs[0].status = if frame < 2 {
                        "Not checked yet"
                    } else {
                        "No changes detected"
                    }
                    .into();
                    let output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(1100.0, 960.0),
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            app.show_header(ui);
                            app.show_footer(ui);
                            app.show_job_list(ui);
                            app.show_job_editor(ui);
                        },
                    );
                    for shape in output.shapes {
                        if let egui::Shape::Text(text) = shape.shape {
                            if text.galley.text() == "Scheduling runs while the app is open" {
                                let position = (text.pos, text.galley.size());
                                if frame == 2 {
                                    footer_label = Some(position);
                                }
                                if frame > 2 {
                                    assert_eq!(
                                        Some(position),
                                        footer_label,
                                        "{} {theme:?}: footer moved",
                                        skin.id
                                    );
                                }
                            }
                            assert!(
                                !text.galley.text().contains("ID ")
                                    && !text.galley.text().contains("Double use"),
                                "{} {theme:?} frame={frame}: {}",
                                skin.id,
                                text.galley.text()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn toggling_job_keeps_name_layout_stable_without_id_clashes() {
        for skin in skins::ALL {
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let ctx = egui::Context::default();
                style::install(&ctx, skin);
                ctx.set_theme(theme);
                let app = App::new(
                    Config::default(),
                    std::path::PathBuf::new(),
                    tempfile::tempfile().unwrap(),
                );
                let mut job = Job {
                    name: "Artifact test job with a long name".into(),
                    enabled: true,
                    next_run: Some(Utc::now() + chrono::Duration::minutes(10)),
                    ..Job::default()
                };
                let mut first_disabled = None;
                let mut enabled_name = None;
                for frame in 0..9 {
                    if frame == 3 {
                        job.enabled = false;
                    }
                    if frame == 6 {
                        job.enabled = true;
                    }
                    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                        ui.set_width(300.0);
                        app.job_card(ui, &job);
                    });
                    let name = output
                        .shapes
                        .iter()
                        .find_map(|shape| match &shape.shape {
                            egui::Shape::Text(text) if text.galley.text() == job.name => {
                                Some((text.pos, text.galley.size(), shape.clip_rect))
                            }
                            _ => None,
                        })
                        .expect("job name must be rendered");
                    if frame == 3 {
                        first_disabled = Some(name);
                    }
                    assert!(
                        !output.shapes.iter().any(|shape| match &shape.shape {
                            egui::Shape::Text(text) =>
                                text.galley.text().contains("ID ")
                                    || text.galley.text().contains("Double use"),
                            _ => false,
                        }),
                        "{} {theme:?}: widget-ID collision",
                        skin.id
                    );
                    if frame == 2 {
                        enabled_name = Some(name);
                    }
                    if frame > 3 && frame < 6 {
                        assert_eq!(Some(name), first_disabled, "{} {theme:?}", skin.id);
                    }
                    if frame >= 6 {
                        assert_eq!(Some(name), enabled_name, "{} {theme:?}", skin.id);
                    }
                }
            }
        }
    }

    #[test]
    fn initial_system_appearance_is_saved_and_ignores_later_os_changes() {
        for system in [egui::Theme::Light, egui::Theme::Dark] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("portbak.json");
            let mut app = App::new(
                Config::default(),
                path.clone(),
                tempfile::tempfile().unwrap(),
            );
            let context = egui::Context::default();
            let _ = context.run_ui(
                egui::RawInput {
                    system_theme: Some(system),
                    ..Default::default()
                },
                |_| app.initialize_theme(&context),
            );
            let expected = if system == egui::Theme::Dark {
                Theme::Dark
            } else {
                Theme::Light
            };
            let saved: Config = crate::engine::load(&path).unwrap();
            assert_eq!(saved.theme, expected);
            let mut restarted = App::new(saved, path.clone(), tempfile::tempfile().unwrap());
            let _ = context.run_ui(
                egui::RawInput {
                    system_theme: Some(if system == egui::Theme::Dark {
                        egui::Theme::Light
                    } else {
                        egui::Theme::Dark
                    }),
                    ..Default::default()
                },
                |_| restarted.initialize_theme(&context),
            );
            assert_eq!(restarted.config.theme, expected);
        }
    }

    #[test]
    fn sidebar_width_stays_fixed_across_frames() {
        for (skin, dark) in skins::ALL
            .iter()
            .flat_map(|skin| [(*skin, false), (*skin, true)])
        {
            for with_job in [false, true] {
                let mut config = Config {
                    skin: skin.id.into(),
                    ..Config::default()
                };
                if with_job {
                    config.jobs.push(Job {
                        name: "A long job name that should not stretch the sidebar".into(),
                        status: "A long status that should wrap inside the sidebar".into(),
                        ..Job::default()
                    });
                }
                let mut app = App::new(
                    config,
                    std::path::PathBuf::new(),
                    tempfile::tempfile().unwrap(),
                );
                let context = egui::Context::default();
                app.ensure_skin(&context);
                context.set_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                });
                let id = egui::Id::new("jobs");
                context.data_mut(|data| {
                    data.insert_persisted(
                        id,
                        egui::containers::panel::PanelState {
                            rect: egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(350.0, 600.0),
                            ),
                        },
                    )
                });
                for _ in 0..4 {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1100.0, 960.0),
                        )),
                        ..Default::default()
                    };
                    let _ = context.run_ui(input, |ui| app.show_job_list(ui));
                    let width = egui::containers::panel::PanelState::load(&context, id)
                        .unwrap()
                        .rect
                        .width();
                    assert!(
                        (width - 350.0).abs() < 1.0,
                        "{} with_job={with_job}: {width}",
                        skin.id
                    );
                }
            }
        }
    }

    #[test]
    fn dragged_sidebar_width_survives_frames_and_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut app = App::new(
            Config::default(),
            path.clone(),
            tempfile::tempfile().unwrap(),
        );
        let ctx = egui::Context::default();
        app.ensure_skin(&ctx);
        let frame = |app: &mut App, ctx: &egui::Context, events| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1100.0, 960.0),
                )),
                events,
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| app.show_job_list(ui));
        };
        frame(&mut app, &ctx, vec![]);
        let start = egui::pos2(300.0, 300.0);
        let end = egui::pos2(390.0, 300.0);
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start)]);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(
            (app.config.sidebar_width - 390.0).abs() < 1.0,
            "width: {}",
            app.config.sidebar_width
        );
        for _ in 0..5 {
            frame(&mut app, &ctx, vec![]);
            let width = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("jobs"))
                .unwrap()
                .rect
                .width();
            assert!((width - 390.0).abs() < 1.0, "width: {width}");
        }
        assert!(app.persist());
        let config: Config = crate::engine::load(&path).unwrap();
        let mut restarted = App::new(config, path, tempfile::tempfile().unwrap());
        let new_ctx = egui::Context::default();
        restarted.ensure_skin(&new_ctx);
        frame(&mut restarted, &new_ctx, vec![]);
        let width = egui::containers::panel::PanelState::load(&new_ctx, egui::Id::new("jobs"))
            .unwrap()
            .rect
            .width();
        assert!((width - 390.0).abs() < 1.0, "width after restart: {width}");
    }

    #[test]
    fn midnight_neko_portrait_fits_at_minimum_window_size_with_wide_sidebar() {
        let ctx = egui::Context::default();
        let mut app = App::new(
            Config {
                skin: "midnight-neko".into(),
                sidebar_width: *SIDEBAR_WIDTH.end(),
                jobs: (0..8).map(|_| Job::default()).collect(),
                ..Config::default()
            },
            std::path::PathBuf::new(),
            tempfile::tempfile().unwrap(),
        );
        app.ensure_skin(&ctx);
        for _ in 0..2 {
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        crate::MIN_WINDOW_SIZE,
                    )),
                    ..Default::default()
                },
                |ui| {
                    app.show_header(ui);
                    app.show_footer(ui);
                    app.show_job_list(ui);
                },
            );
            let portrait = output.shapes.iter().find_map(|clipped| {
                let egui::Shape::Mesh(mesh) = &clipped.shape else {
                    return None;
                };
                let rect = mesh.calc_bounds();
                (rect.left() < 10.0 && rect.width() > 450.0 && rect.width() == rect.height())
                    .then_some((rect, clipped.clip_rect))
            });
            let (rect, clip) = portrait.expect("full-width square portrait");
            assert!(clip.contains_rect(rect), "{rect:?} outside {clip:?}");
            assert!(
                rect.bottom() + 100.0 <= clip.bottom(),
                "{rect:?} in {clip:?}"
            );
        }
    }

    #[test]
    fn selected_job_renders_at_minimum_window_width_with_wide_sidebar() {
        for (skin, dark) in skins::ALL
            .iter()
            .flat_map(|skin| [(*skin, false), (*skin, true)])
        {
            for enabled in [false, true] {
                let config = Config {
                    skin: skin.id.into(),
                    sidebar_width: 460.0,
                    jobs: vec![Job {
                        enabled,
                        ..Job::default()
                    }],
                    ..Config::default()
                };
                let mut app = App::new(
                    config,
                    std::path::PathBuf::new(),
                    tempfile::tempfile().unwrap(),
                );
                let ctx = egui::Context::default();
                app.ensure_skin(&ctx);
                ctx.set_theme(if dark {
                    egui::Theme::Dark
                } else {
                    egui::Theme::Light
                });
                for width in [1100.0, 800.0, 750.0] {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 600.0),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run_ui(input, |ui| {
                        app.show_header(ui);
                        app.show_footer(ui);
                        app.show_job_list(ui);
                        app.show_job_editor(ui);
                    });
                }
            }
        }
    }

    #[test]
    fn narrow_windows_keep_every_view_inside_the_window() {
        use crate::app::views::settings_view::SettingsTab;
        const SCREEN: egui::Vec2 = crate::MIN_WINDOW_SIZE;
        for sidebar_width in [*SIDEBAR_WIDTH.start(), *SIDEBAR_WIDTH.end()] {
            for skin in skins::ALL {
                for view in ["job settings", "backups", "settings", "theme"] {
                    let ctx = egui::Context::default();
                    style::install(&ctx, skin);
                    let mut app = App::new(
                        Config {
                            sidebar_width,
                            jobs: vec![Job {
                                name: "A job with a fairly long name".into(),
                                source: "C:/Users/someone/Documents/Projects".into(),
                                target: "C:/Backups/target".into(),
                                schedule: crate::model::Schedule::Weekdays,
                                enabled: true,
                                next_run: Some(Utc::now()),
                                ..Job::default()
                            }],
                            ..Config::default()
                        },
                        std::path::PathBuf::new(),
                        tempfile::tempfile().unwrap(),
                    );
                    if view == "settings" || view == "theme" {
                        app.toggle_settings();
                        if view == "theme" {
                            app.settings_tab = SettingsTab::Theme;
                        }
                    } else {
                        app.selected = Some(app.config.jobs[0].id);
                        app.backups_tab = view == "backups";
                    }
                    let mut overflow = Vec::new();
                    // Later frames settle sizes measured in earlier ones.
                    for _ in 0..4 {
                        let input = egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, SCREEN)),
                            ..Default::default()
                        };
                        let output = ctx.run_ui(input, |ui| {
                            app.show_header(ui);
                            app.show_footer(ui);
                            app.show_job_list(ui);
                            if app.settings_open {
                                app.show_settings(ui);
                            } else {
                                app.show_job_editor(ui);
                            }
                        });
                        // Scenery meshes and full-width bands may bleed off the edge; controls
                        // and panels may not.
                        overflow = output
                            .shapes
                            .iter()
                            .filter(|clipped| !matches!(clipped.shape, egui::Shape::Mesh(_)))
                            .map(|clipped| clipped.shape.visual_bounding_rect())
                            .filter(|rect| {
                                rect.is_finite()
                                    && rect.width() < SCREEN.x
                                    && rect.right() > SCREEN.x + 0.5
                            })
                            .collect();
                    }
                    assert!(
                        overflow.is_empty(),
                        "{} {view} with a {sidebar_width} px job list: {overflow:?}",
                        skin.id
                    );
                }
            }
        }
    }
}
