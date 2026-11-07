use super::ornaments;
use super::pixel::{self, PixelButton, Tone};
use super::skins;
use super::sprites::{self, Sprite};
use super::style::{self, Finish, Palette, Skin};
use super::widgets::alert;
use super::App;
use crate::model::{Job, Theme};
use chrono::Utc;
use eframe::egui;

const CARD_SHADOW: f32 = 4.0;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        let before = self.config.clone();
        self.show_header(ctx);
        self.show_footer(ctx);
        self.show_job_list(ctx);
        if self.settings_open {
            self.show_settings(ctx);
        } else {
            self.show_job_editor(ctx);
        }
        self.show_history(ctx);
        if self.config != before {
            self.dirty = true;
        }
        pixel::paint_overlays(ctx);
    }
}

impl App {
    /// Installs the configured skin whenever it differs from the one in use.
    fn ensure_skin(&self, ctx: &egui::Context) {
        let wanted = skins::find(&self.config.skin);
        if !style::installed(ctx).is_some_and(|skin| skin.id == wanted.id) {
            style::install(ctx, wanted);
        }
    }

    fn show_header(&mut self, ctx: &egui::Context) {
        let palette = palette_of(ctx);
        let frame = egui::Frame::new()
            .fill(palette.surface)
            .inner_margin(egui::Margin {
                left: 20,
                right: 20,
                top: 14,
                bottom: 0,
            });
        egui::TopBottomPanel::top("header")
            .frame(frame)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    logo(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let settings = PixelButton::new("Settings")
                            .sprite(Sprite::Gear)
                            .selected(self.settings_open);
                        if ui.add(settings).clicked() {
                            self.settings_open = !self.settings_open;
                        }
                    });
                });
                ui.add_space(12.0);
                let clip = ui.clip_rect();
                let top = ui.cursor().top();
                if !Skin::of(ui.ctx()).effects.stripes {
                    let rule = egui::Rect::from_min_max(
                        egui::pos2(clip.left(), top),
                        egui::pos2(clip.right(), top + 2.0),
                    );
                    ui.allocate_rect(rule, egui::Sense::hover());
                    ornaments::fading_rule(ui.painter(), rule, palette.accent);
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

    fn show_footer(&self, ctx: &egui::Context) {
        let palette = palette_of(ctx);
        let frame = egui::Frame::new()
            .fill(palette.sunken)
            .inner_margin(egui::Margin::symmetric(18, 8));
        egui::TopBottomPanel::bottom("footer")
            .frame(frame)
            .show_separator_line(false)
            .show(ctx, |ui| {
                let clip = ui.clip_rect();
                let rule =
                    egui::Rect::from_min_size(clip.left_top(), egui::vec2(clip.width(), 3.0));
                ui.painter().rect_filled(rule, 0.0, palette.outline);
                let wording = &Skin::of(ui.ctx()).wording;
                ui.horizontal(|ui| {
                    if self.active.is_some() {
                        pixel::led(ui, palette.gold, pixel::blink(ui, 0.25));
                        pixel::display_label(ui, wording.busy, 10.0, palette.gold);
                        pixel::marquee(ui, 180.0, palette.gold);
                        ui.label(&self.progress);
                    } else {
                        pixel::led(ui, palette.good, true);
                        let ready = blinking_cursor(ui, wording.ready, wording.cursor);
                        pixel::display_label(ui, &ready, 10.0, palette.good);
                        ui.label(
                            egui::RichText::new("Scheduling runs while the app is open")
                                .color(palette.dim),
                        );
                    }
                });
                if !self.error.is_empty() {
                    alert(ui, palette.bad, &self.error);
                }
                ui.label(
                    egui::RichText::new(format!("SETTINGS  {}", self.path.display()))
                        .small()
                        .color(palette.dim),
                );
            });
    }

    fn show_job_list(&mut self, ctx: &egui::Context) {
        let palette = palette_of(ctx);
        let frame = egui::Frame::new()
            .fill(palette.surface)
            .inner_margin(egui::Margin {
                left: 16,
                right: 14,
                top: 16,
                bottom: 8,
            });
        let panel = egui::SidePanel::left("jobs")
            .default_width(self.config.sidebar_width.clamp(240.0, 460.0))
            .width_range(240.0..=460.0)
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    pixel::display_label(ui, "JOBS", 16.0, palette.accent);
                    pixel::badge(ui, &self.config.jobs.len().to_string(), palette.gold);
                });
                ui.add_space(10.0);
                let new_job = PixelButton::new("New job")
                    .sprite(Sprite::Plus)
                    .tone(Tone::Accent)
                    .min_size(egui::vec2(ui.available_width() - 4.0, 0.0));
                if ui.add_enabled(!self.scheduling_paused(), new_job).clicked() {
                    self.add_job();
                }
                ui.add_space(12.0);
                let mut clicked = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for job in &self.config.jobs {
                        if self.job_card(ui, job) {
                            clicked = Some(job.id);
                        }
                    }
                });
                if clicked.is_some() {
                    self.selected = clicked;
                    self.settings_open = false;
                }
            });
        if ctx
            .read_response(egui::Id::new("jobs").with("__resize"))
            .is_some_and(|response| response.dragged())
        {
            self.config.sidebar_width = panel.response.rect.width().clamp(240.0, 460.0);
        }
    }

    /// Draws the job as a cartridge-like card; returns whether it was clicked.
    fn job_card(&self, ui: &mut egui::Ui, job: &Job) -> bool {
        let palette = Palette::of(ui);
        let selected = self.selected == Some(job.id);
        let running = self.active == Some(job.id);
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
        let width = ui.available_width() - CARD_SHADOW;
        let frame = egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(width - 24.0);
                ui.horizontal_top(|ui| {
                    let tint = if job.enabled {
                        palette.good
                    } else {
                        palette.metal
                    };
                    sprites::show_awake(ui, Sprite::Floppy, 3.0, tint, wake);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add(
                            egui::Label::new(egui::RichText::new(&job.name).color(palette.ink))
                                .truncate(),
                        );
                        ui.horizontal(|ui| {
                            let (color, lit) = match (running, job.enabled) {
                                (true, _) => (palette.gold, pixel::blink(ui, 0.25)),
                                (false, true) => (palette.good, true),
                                (false, false) => (palette.dim, false),
                            };
                            pixel::led(ui, color, lit).on_hover_text(self.activity(job));
                            ui.label(egui::RichText::new(self.activity(job)).small());
                            if self.queue.contains(&job.id) {
                                pixel::badge(ui, "Queued", palette.hot);
                            }
                        });
                        if running {
                            pixel::marquee(ui, ui.available_width().min(160.0), palette.gold);
                        }
                        if let Some(remaining) = job.countdown(Utc::now()) {
                            ui.horizontal(|ui| {
                                sprites::show(ui, Sprite::Clock, 1.0, palette.paper);
                                ui.label(
                                    egui::RichText::new(format!("Next check in: {remaining}"))
                                        .small()
                                        .color(palette.accent),
                                );
                            });
                        }
                        let status_color = if job.status.starts_with("Error") {
                            palette.bad
                        } else {
                            palette.dim
                        };
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(&job.status).small().color(status_color),
                            )
                            .wrap(),
                        );
                    });
                });
            });
        let rect = frame.response.rect;
        let response = ui
            .interact(rect, card_id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        ui.painter().set(
            background,
            card_shapes(ui.ctx(), rect, selected, response.hovered()),
        );
        ui.add_space(CARD_SHADOW + 4.0);
        response.clicked()
    }

    fn activity(&self, job: &Job) -> &'static str {
        match (self.active == Some(job.id), self.restoring, job.enabled) {
            (true, true, _) => "Restore running",
            (true, false, _) => "Backup running",
            (false, _, true) => "Schedule enabled",
            (false, _, false) => "Schedule disabled",
        }
    }
}

/// The skin's mascot next to the wordmark; pixel mascots get a twinkling glint.
fn logo(ui: &mut egui::Ui) {
    let palette = Palette::of(ui);
    let skin = Skin::of(ui.ctx());
    let mascot = sprites::show(ui, skin.wording.mascot, 4.0, palette.hot);
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
        pixel::display_label(ui, "PORTBAK", 24.0, palette.gold);
        let tagline = Skin::of(ui.ctx()).wording.tagline;
        ui.label(egui::RichText::new(tagline).color(palette.accent));
    });
}

/// `text` followed by a cursor that blinks, padded while hidden so the width stays put.
fn blinking_cursor(ui: &egui::Ui, text: &str, cursor: &str) -> String {
    if cursor.is_empty() || pixel::blink(ui, 0.5) {
        format!("{text}{cursor}")
    } else {
        format!("{text}{}", " ".repeat(cursor.chars().count()))
    }
}

fn card_shapes(
    ctx: &egui::Context,
    rect: egui::Rect,
    selected: bool,
    hovered: bool,
) -> Vec<egui::Shape> {
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
    let mut shapes = vec![
        shadow.as_shape(rect, 0.0).into(),
        egui::Shape::rect_filled(rect, 0.0, fill),
        egui::Shape::rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(if selected { 3.0_f32 } else { 2.0 }, border),
            egui::StrokeKind::Inside,
        ),
    ];
    if selected {
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
    fn sidebar_width_stays_fixed_across_frames() {
        for skin in ["retro", "dark-fantasy"] {
            for with_job in [false, true] {
                let mut config = Config {
                    skin: skin.into(),
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
                    let _ = context.run(input, |ctx| app.show_job_list(ctx));
                    let width = egui::containers::panel::PanelState::load(&context, id)
                        .unwrap()
                        .rect
                        .width();
                    assert!(
                        (width - 350.0).abs() < 1.0,
                        "{skin} with_job={with_job}: {width}"
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
            let _ = ctx.run(input, |ctx| app.show_job_list(ctx));
        };
        frame(&mut app, &ctx, vec![]);
        let start = egui::pos2(290.0, 300.0);
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
    fn selected_job_renders_at_minimum_window_width_with_wide_sidebar() {
        for skin in ["retro", "dark-fantasy"] {
            for enabled in [false, true] {
                let config = Config {
                    skin: skin.into(),
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
                for width in [1100.0, 800.0, 750.0] {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 600.0),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run(input, |ctx| {
                        app.show_header(ctx);
                        app.show_footer(ctx);
                        app.show_job_list(ctx);
                        app.show_job_editor(ctx);
                    });
                }
            }
        }
    }
}
