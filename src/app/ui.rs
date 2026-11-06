use super::pixel::{self, PixelButton, Tone};
use super::skins;
use super::sprites;
use super::style::{self, Palette, Skin};
use super::widgets::alert;
use super::App;
use crate::model::{Job, Theme};
use chrono::Utc;
use eframe::egui;

const CARD_SHADOW: f32 = 4.0;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_skin(ctx);
        self.tick(ctx);
        if let Some(id) = self.selected {
            self.ensure_summary(id, ctx);
        }
        ctx.set_theme(theme_preference(self.config.theme));
        if ctx.input(|input| input.viewport().close_requested()) && !self.permit_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = true;
        }
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
        if self.closing {
            self.show_quit_dialog(ctx);
        }
        if Skin::of(ctx).effects.scanlines {
            pixel::paint_scanlines(ctx, palette_of(ctx));
        }
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
                            .sprite(sprites::GEAR)
                            .selected(self.settings_open);
                        if ui.add(settings).clicked() {
                            self.settings_open = !self.settings_open;
                        }
                    });
                });
                ui.add_space(12.0);
                if !Skin::of(ui.ctx()).effects.stripes {
                    return;
                }
                let clip = ui.clip_rect();
                let top = ui.cursor().top();
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
                ui.horizontal(|ui| {
                    if self.active.is_some() {
                        pixel::led(ui, palette.gold, pixel::blink(ui, 0.25));
                        pixel::display_label(ui, "BUSY", 10.0, palette.gold);
                        pixel::marquee(ui, 180.0, palette.gold);
                        ui.label(&self.progress);
                    } else {
                        pixel::led(ui, palette.good, true);
                        let cursor = if pixel::blink(ui, 0.5) {
                            "READY_"
                        } else {
                            "READY "
                        };
                        pixel::display_label(ui, cursor, 10.0, palette.good);
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
        egui::SidePanel::left("jobs")
            .default_width(290.0)
            .width_range(240.0..=460.0)
            .frame(frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    pixel::display_label(ui, "JOBS", 16.0, palette.accent);
                    pixel::badge(ui, &self.config.jobs.len().to_string(), palette.gold);
                });
                ui.add_space(10.0);
                let new_job = PixelButton::new("New job")
                    .sprite(sprites::PLUS)
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
    }

    /// Draws the job as a cartridge-like card; returns whether it was clicked.
    fn job_card(&self, ui: &mut egui::Ui, job: &Job) -> bool {
        let palette = Palette::of(ui);
        let selected = self.selected == Some(job.id);
        let running = self.active == Some(job.id);
        let background = ui.painter().add(egui::Shape::Noop);
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
                    sprites::show(ui, sprites::FLOPPY, 3.0, tint);
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
                                sprites::show(ui, sprites::CLOCK, 1.0, palette.paper);
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
            .interact(rect, ui.id().with(job.id), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        ui.painter().set(
            background,
            card_shapes(rect, palette, selected, response.hovered()),
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

    fn show_quit_dialog(&mut self, ctx: &egui::Context) {
        let palette = palette_of(ctx);
        let window = egui::Window::new("Quit the app?")
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
        pixel::dialog(
            ctx,
            window,
            "Quit the app?",
            sprites::POWER,
            palette.hot,
            |ui| {
                ui.label("Quitting stops scheduling. The app keeps running when minimized.");
                if self.active.is_some() {
                    alert(
                        ui,
                        palette.gold,
                        "An operation is running. Please wait for it to finish before quitting.",
                    );
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let keep = PixelButton::new("Keep running")
                        .sprite(sprites::PLAY)
                        .tone(Tone::Accent);
                    if ui.add(keep).clicked() {
                        self.closing = false;
                    }
                    let quit = PixelButton::new("Quit")
                        .sprite(sprites::POWER)
                        .tone(Tone::Danger);
                    if ui.add_enabled(self.active.is_none(), quit).clicked() {
                        self.quit(ctx);
                    }
                });
            },
        );
    }
}

/// The floppy mascot with a twinkling glint, next to the wordmark.
fn logo(ui: &mut egui::Ui) {
    let palette = Palette::of(ui);
    let floppy = sprites::show(ui, sprites::FLOPPY, 4.0, palette.hot);
    if pixel::blink(ui, 0.8) {
        let glint = floppy.rect.right_top() + egui::vec2(-2.0, 2.0);
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
        ui.label(egui::RichText::new("PORTABLE BACKUP SYSTEM").color(palette.accent));
    });
}

fn card_shapes(
    rect: egui::Rect,
    palette: &Palette,
    selected: bool,
    hovered: bool,
) -> Vec<egui::Shape> {
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
        egui::Shape::rect_filled(
            rect.translate(egui::Vec2::splat(CARD_SHADOW)),
            0.0,
            palette.outline,
        ),
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
