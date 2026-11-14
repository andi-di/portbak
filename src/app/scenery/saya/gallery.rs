use super::{expressions::Expression, state_id, SayaCompanion};
use crate::app::rendering::pixel::{self, PixelButton};
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Rect, Vec2};
use std::time::Duration;

const HOLD_SECONDS: f64 = 2.0;

#[derive(Clone, Default)]
struct Gallery {
    open: bool,
    hold_started: Option<f64>,
    hold_fired: bool,
    selected: Expression,
    search: String,
}

fn id() -> egui::Id {
    egui::Id::new("saya_looking_glass")
}

fn state(ctx: &egui::Context) -> Gallery {
    ctx.data_mut(|data| data.get_temp(id()).unwrap_or_default())
}

pub fn is_open(ctx: &egui::Context) -> bool {
    state(ctx).open
}

pub fn suppress_home_click(ctx: &egui::Context) -> bool {
    state(ctx).hold_fired
}

impl Gallery {
    fn hold(&mut self, time: f64, pressed: bool, held_inside: bool) -> bool {
        if pressed {
            self.hold_fired = false;
            self.hold_started = held_inside.then_some(time);
        }
        if !held_inside {
            self.hold_started = None;
        }
        if self.open || self.hold_fired {
            return false;
        }
        if self
            .hold_started
            .is_some_and(|start| time - start >= HOLD_SECONDS)
        {
            self.hold_fired = true;
            self.hold_started = None;
            self.open = true;
            return true;
        }
        false
    }
}

pub(super) fn emblem_hold(ui: &egui::Ui, rect: Rect) {
    let ctx = ui.ctx();
    let mut gallery = state(ctx);
    let hovered = ui.rect_contains_pointer(rect);
    let (time, pressed, held_inside) = ui.input(|input| {
        (
            input.time,
            input.pointer.primary_pressed(),
            input.focused
                && input.pointer.primary_down()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|pos| rect.contains(pos))
                && hovered,
        )
    });
    if gallery.hold(time, pressed, held_inside) {
        let expression = ctx
            .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
            .unwrap_or_default()
            .visible(time);
        gallery.selected = expression;
        select(ctx, expression, time);
        ctx.memory_mut(|memory| memory.request_focus(id().with("search")));
    }
    if let Some(start) = gallery.hold_started {
        let progress = ((time - start) / HOLD_SECONDS).clamp(0.0, 1.0) as f32;
        let color = Palette::of(ui).accent.gamma_multiply(0.3 + progress * 0.7);
        let center = rect.center();
        let radius = rect.width() * 0.46;
        let points = (0..=48)
            .map(|index| {
                center
                    + Vec2::angled(
                        -std::f32::consts::FRAC_PI_2
                            + index as f32 / 48.0 * std::f32::consts::TAU * progress,
                    ) * radius
            })
            .collect();
        ui.painter()
            .add(egui::Shape::line(points, egui::Stroke::new(2.0_f32, color)));
        ctx.request_repaint_after(Duration::from_millis(25));
    }
    ctx.data_mut(|data| data.insert_temp(id(), gallery));
}

fn select(ctx: &egui::Context, expression: Expression, time: f64) {
    ctx.data_mut(|data| {
        let mut companion = data
            .get_temp::<SayaCompanion>(state_id())
            .unwrap_or_default();
        companion.set_expression(expression, time, uuid::Uuid::new_v4().as_u128());
        companion.overlay_started_at = (expression != Expression::Neutral).then_some(time);
        companion.preview = true;
        companion.collapsed = false;
        data.insert_temp(state_id(), companion);
    });
    ctx.request_repaint();
}

pub fn close(ctx: &egui::Context) {
    let mut gallery = state(ctx);
    if !gallery.open {
        return;
    }
    gallery.open = false;
    ctx.data_mut(|data| data.insert_temp(id(), gallery));
    ctx.data_mut(|data| {
        let mut companion = data
            .get_temp::<SayaCompanion>(state_id())
            .unwrap_or_default();
        companion.preview = false;
        companion.until = 0.0;
        companion.paused_remaining = None;
        companion.overlay_started_at = None;
        companion.hovered = false;
        data.insert_temp(state_id(), companion);
    });
}

pub fn show(ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let mut gallery = state(&ctx);
    if !gallery.open {
        return;
    }
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        close(&ctx);
        return;
    }
    let mut dismiss = false;
    egui::CentralPanel::default()
        .frame(
            crate::app::views::widgets::page_frame(ui)
                .inner_margin(egui::Margin::same(crate::app::views::widgets::PAGE_PADDING)),
        )
        .show_inside(ui, |ui| {
            pixel::paint_scenery(ui);
            ui.horizontal(|ui| {
                ui.heading("Saya’s Looking Glass");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    dismiss = close_button(ui).clicked();
                });
            });
            ui.weak(format!("{} expressions", Expression::ALL.len()));
            ui.add_space(12.0);
            let width = ui.available_width();
            let height = ui.available_height();
            if width >= 520.0 {
                let size = (width * 0.48).min(460.0).min((height - 120.0).max(160.0));
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(size);
                        preview(ui, &gallery, size, (height - size - 60.0).max(80.0));
                    });
                    ui.add_space(16.0);
                    ui.vertical(|ui| grid(ui, &mut gallery, (height - 38.0).max(80.0)));
                });
            } else {
                let size = width.min((height * 0.42).clamp(120.0, 360.0));
                ui.vertical_centered(|ui| preview(ui, &gallery, size, 100.0));
                grid(ui, &mut gallery, (ui.available_height() - 38.0).max(80.0));
            }
        });
    if dismiss {
        gallery.open = false;
        close(&ctx);
    }
    ctx.data_mut(|data| data.insert_temp(id(), gallery));
}

fn close_button(ui: &mut egui::Ui) -> egui::Response {
    let response = ui
        .add(PixelButton::new("").compact().min_size(Vec2::splat(36.0)))
        .on_hover_text("Close gallery (Esc)");
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close gallery"));
    let center = response.rect.center();
    let stroke = egui::Stroke::new(1.8_f32, Palette::of(ui).ink);
    for direction in [-1.0, 1.0] {
        let offset = egui::vec2(6.0, 6.0 * direction);
        ui.painter()
            .line_segment([center - offset, center + offset], stroke);
    }
    response
}

fn preview(ui: &mut egui::Ui, gallery: &Gallery, size: f32, dialogue_height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let expression = gallery.selected;
    super::paint(ui, rect, &format!("{expression:?}"), expression.bytes());
    let companion = ui
        .ctx()
        .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
        .unwrap_or_default();
    let time = ui.input(|input| input.time);
    if companion.expression == expression {
        if let Some(elapsed) = companion.overlay_elapsed(time) {
            if expression == Expression::InLove {
                super::paint_love_hearts(ui, rect, elapsed);
            } else {
                super::overlays::paint(ui.painter(), rect, expression, elapsed);
            }
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }
    ui.label(
        egui::RichText::new(expression.label())
            .strong()
            .color(Palette::of(ui).ink),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("Dialogue")
            .strong()
            .color(Palette::of(ui).ink),
    );
    egui::ScrollArea::vertical()
        .id_salt("gallery_dialogue")
        .max_height(dialogue_height)
        .show(ui, |ui| {
            for (index, line) in expression.lines().iter().enumerate() {
                ui.horizontal_top(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}.", index + 1)).color(Palette::of(ui).dim),
                    );
                    ui.add(
                        egui::Label::new(egui::RichText::new(*line).color(Palette::of(ui).ink))
                            .wrap(),
                    );
                });
                ui.add_space(6.0);
            }
        });
}

fn columns(width: f32, spacing: f32) -> usize {
    ((width + spacing) / (112.0 + spacing)).floor().max(1.0) as usize
}

fn grid(ui: &mut egui::Ui, gallery: &mut Gallery, height: f32) {
    ui.add(
        egui::TextEdit::singleline(&mut gallery.search)
            .id(id().with("search"))
            .hint_text("Find an expression…")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);
    let search = gallery.search.to_lowercase();
    let expressions: Vec<_> = Expression::ALL
        .iter()
        .copied()
        .filter(|expression| expression.label().to_lowercase().contains(&search))
        .collect();
    if expressions.is_empty() {
        ui.weak("No matching expressions.");
    }
    egui::ScrollArea::vertical()
        .id_salt("expression_grid")
        .max_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let spacing = ui.spacing().item_spacing;
            let count = columns(ui.available_width(), spacing.x);
            let cell =
                ((ui.available_width() - spacing.x * (count - 1) as f32) / count as f32).max(1.0);
            for row in expressions.chunks(count) {
                ui.horizontal(|ui| {
                    for &expression in row {
                        ui.push_id(expression, |ui| {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(cell, cell + 34.0),
                                egui::Sense::click(),
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::Button,
                                    true,
                                    gallery.selected == expression,
                                    expression.label(),
                                )
                            });
                            if ui.is_rect_visible(rect) {
                                let palette = Palette::of(ui);
                                ui.painter().rect_filled(rect, 3, palette.surface);
                                let image = Rect::from_min_size(
                                    rect.min + Vec2::splat(4.0),
                                    Vec2::splat((cell - 8.0).max(1.0)),
                                );
                                super::paint(
                                    ui,
                                    image,
                                    &format!("{expression:?}"),
                                    expression.bytes(),
                                );
                                let text = ui.painter().layout(
                                    expression.label(),
                                    egui::TextStyle::Small.resolve(ui.style()),
                                    palette.ink,
                                    (cell - 8.0).max(1.0),
                                );
                                ui.painter().galley(
                                    egui::pos2(
                                        rect.center().x - text.size().x * 0.5,
                                        image.bottom() + 6.0,
                                    ),
                                    text,
                                    Color32::WHITE,
                                );
                                if gallery.selected == expression {
                                    ui.painter().rect_stroke(
                                        rect.shrink(1.0),
                                        3,
                                        egui::Stroke::new(1.5_f32, palette.accent),
                                        egui::StrokeKind::Inside,
                                    );
                                }
                                pixel::subtle_highlight(ui, &response, rect, palette.metal);
                            }
                            if response.gained_focus() {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.clicked() {
                                gallery.selected = expression;
                                select(ui.ctx(), expression, ui.input(|input| input.time));
                            }
                        });
                    }
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_requires_two_uninterrupted_seconds_and_fires_once() {
        let mut gallery = Gallery::default();
        assert!(!gallery.hold(1.0, true, true));
        assert!(!gallery.hold(2.99, false, true));
        assert!(gallery.hold(3.0, false, true));
        assert!(gallery.open && gallery.hold_fired);
        assert!(!gallery.hold(4.0, false, true));
        assert!(!gallery.hold(4.1, false, false));
        assert!(gallery.hold_fired);
        gallery.open = false;
        assert!(!gallery.hold(5.0, true, true));
        assert!(!gallery.hold_fired);
        assert!(!gallery.hold(5.5, false, false));
        assert!(!gallery.hold(8.0, false, true));
        assert!(!gallery.open);
    }

    #[test]
    fn pointer_hold_on_emblem_opens_gallery_and_suppresses_home_click() {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::splat(56.0));
        for (frame, time) in [1.0, 2.9, 3.0].into_iter().enumerate() {
            let mut input = egui::RawInput {
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1024.0, 940.0),
                )),
                ..Default::default()
            };
            input.events.push(egui::Event::PointerMoved(rect.center()));
            if frame == 0 {
                input.events.push(egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            let _ = ctx.run_ui(input, |ui| emblem_hold(ui, rect));
            assert_eq!(is_open(&ctx), frame == 2);
        }
        assert!(suppress_home_click(&ctx));
    }

    #[test]
    fn gallery_selection_holds_expression_and_close_restores_neutral() {
        let ctx = egui::Context::default();
        ctx.data_mut(|data| {
            data.insert_temp(
                id(),
                Gallery {
                    open: true,
                    ..Default::default()
                },
            )
        });
        select(&ctx, Expression::Sad, 1.0);
        let companion = ctx
            .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
            .unwrap();
        assert_eq!(companion.visible(100.0), Expression::Sad);
        assert_eq!(companion.overlay_elapsed(1.2), Some(1.2 - 1.0));
        assert_eq!(companion.overlay_elapsed(100.0), None);
        select(&ctx, Expression::Sad, 100.0);
        let companion = ctx
            .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
            .unwrap();
        assert_eq!(companion.overlay_elapsed(100.1), Some(100.1 - 100.0));
        close(&ctx);
        let companion = ctx
            .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
            .unwrap();
        assert_eq!(companion.visible(100.1), Expression::Neutral);
    }

    #[test]
    fn preview_lists_every_authored_line_for_the_selected_expression() {
        let ctx = egui::Context::default();
        for expression in Expression::ALL {
            let gallery = Gallery {
                selected: *expression,
                ..Default::default()
            };
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1024.0, 4096.0),
                    )),
                    ..Default::default()
                },
                |ui| preview(ui, &gallery, 300.0, 3000.0),
            );
            for line in expression.lines() {
                assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == *line)), "{expression:?}: {line}");
            }
        }
    }

    #[test]
    fn catalog_contains_every_reaction_and_neutral_once_in_alphabetical_order() {
        let names: Vec<_> = Expression::ALL
            .iter()
            .map(|expression| expression.label())
            .collect();
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(Expression::ALL.len(), Expression::REACTIONS.len() + 1);
        assert_eq!(
            Expression::ALL
                .iter()
                .filter(|expression| **expression == Expression::Neutral)
                .count(),
            1
        );
        assert!(Expression::REACTIONS
            .iter()
            .all(|expression| Expression::ALL.contains(expression)));
        assert_eq!(Expression::BoredExpectant.label(), "Bored Expectant");
        assert_eq!(Expression::InLove.label(), "In Love");
        assert!(columns(900.0, 10.0) > columns(260.0, 10.0));
    }

    #[test]
    fn gallery_page_fits_small_and_large_viewports_and_escape_closes_it() {
        for size in [
            egui::vec2(600.0, 700.0),
            egui::vec2(1024.0, 940.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let ctx = egui::Context::default();
            crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("moonveil"));
            ctx.data_mut(|data| {
                data.insert_temp(
                    id(),
                    Gallery {
                        open: true,
                        ..Default::default()
                    },
                )
            });
            for frame in 0..3 {
                let mut input = egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                };
                if frame == 2 {
                    input.events.push(egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                let output = ctx.run_ui(input, show);
                if frame == 1 {
                    let title = output.shapes.iter().find(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("Looking Glass"))).expect("gallery heading");
                    assert!(
                        Rect::from_min_size(egui::Pos2::ZERO, size)
                            .contains_rect(title.shape.visual_bounding_rect()),
                        "heading outside viewport: {size:?}"
                    );
                }
            }
            assert!(!is_open(&ctx));
        }
    }
}
