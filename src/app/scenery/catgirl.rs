//! Saya's static welcome figure and individually refined sidebar expressions.

mod dialogue;
mod expressions;
mod reactions;

use expressions::Expression;

use crate::app::rendering::pixel::pointing_hand;
use crate::app::theme::style::Palette;

use eframe::egui::{self, Color32, Pos2, Rect};

use std::time::Duration;

const REACTION_SECONDS: f64 = 3.0;
const LOVE_HEART_SECONDS: f64 = 0.6;
const LOVE_HEARTS: [(f32, f32, f32, f64); 3] = [
    (0.16, 0.43, 0.10, 0.0),
    (0.85, 0.48, 0.13, 0.07),
    (0.93, 0.32, 0.07, 0.14),
];

fn state_id() -> egui::Id {
    egui::Id::new("saya_sidebar_companion")
}

pub use reactions::{Action, Page};

pub fn trigger(ctx: &egui::Context, action: Action) {
    if !crate::app::theme::style::Skin::of(ctx).effects.hearts {
        return;
    }
    let time = ctx.input(|input| input.time);
    let random = uuid::Uuid::new_v4().as_u128();
    ctx.data_mut(|data| {
        let mut state: Companion = data.get_temp(state_id()).unwrap_or_default();
        state.react(action.pool(), time, random);
        state.action_time = Some(time);
        data.insert_temp(state_id(), state);
    });
    ctx.request_repaint();
}

pub fn enter_page(ctx: &egui::Context, page: Page) {
    if !crate::app::theme::style::Skin::of(ctx).effects.hearts {
        return;
    }
    let time = ctx.input(|input| input.time);
    let changed = ctx.data_mut(|data| {
        let mut state: Companion = data.get_temp(state_id()).unwrap_or_default();
        if state.page == Some(page) {
            return false;
        }
        state.page = Some(page);
        if state.action_time != Some(time) {
            state.react(page.pool(), time, uuid::Uuid::new_v4().as_u128());
        }
        data.insert_temp(state_id(), state);
        true
    });
    if changed {
        ctx.request_repaint();
    }
}

#[derive(Clone, Default)]

struct Companion {
    expression: Expression,
    love_started_at: Option<f64>,
    until: f64,
    paused_remaining: Option<f64>,
    hovered: bool,
    collapsed: bool,
    page: Option<Page>,
    action_time: Option<f64>,
    displayed: Expression,
    line_indices: [Option<usize>; 27],
}

impl Companion {
    fn react_to_click(&mut self, time: f64, random: u128) {
        self.set_expression(self.expression.next(random), time, random);
        self.action_time = Some(time);
    }

    fn react(&mut self, pool: &[Expression], time: f64, random: u128) {
        self.set_expression(
            Expression::choose(pool, self.expression, random),
            time,
            random,
        );
    }

    fn set_expression(&mut self, expression: Expression, time: f64, random: u128) {
        if expression != self.visible(time) {
            self.love_started_at = (expression == Expression::InLove).then_some(time);
        }
        self.expression = expression;
        self.until = time + REACTION_SECONDS;
        self.select_line(expression, random.rotate_left(64));
        self.paused_remaining = self.hovered.then_some(REACTION_SECONDS);
    }

    fn love_elapsed(&self, time: f64) -> Option<f64> {
        self.love_started_at
            .filter(|_| self.visible(time) == Expression::InLove)
            .map(|start| time - start)
            .filter(|elapsed| (0.0..LOVE_HEART_SECONDS + LOVE_HEARTS[2].3).contains(elapsed))
    }

    fn hold_while_hovered(&mut self, hovered: bool, time: f64) {
        if hovered != self.hovered {
            if hovered {
                self.paused_remaining = Some((self.until - time).max(0.0));
            } else if let Some(remaining) = self.paused_remaining.take() {
                self.until = time + remaining;
            }
            self.hovered = hovered;
        }
    }

    fn select_line(&mut self, expression: Expression, random: u128) {
        let index = expression as usize;
        let count = expression.lines().len();
        let previous = self.line_indices[index];
        let mut choice = (random % (count - usize::from(previous.is_some())) as u128) as usize;
        if previous.is_some_and(|old| choice >= old) {
            choice += 1;
        }
        self.line_indices[index] = Some(choice);
        self.displayed = expression;
    }

    fn line(&self, time: f64) -> &'static str {
        let expression = self.visible(time);
        expression.lines()[self.line_indices[expression as usize].unwrap_or(0)]
    }

    fn visible(&self, time: f64) -> Expression {
        if self
            .paused_remaining
            .is_some_and(|remaining| remaining > 0.0)
            || time < self.until
        {
            self.expression
        } else {
            Expression::Neutral
        }
    }
}

fn texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("saya_artwork", name));

    let cached: Option<Result<egui::TextureHandle, String>> =
        ctx.data_mut(|data| data.get_temp(id));

    let result = cached.unwrap_or_else(|| {
        let result = image::load_from_memory(bytes)
            .map(|image| {
                let rgba = image.to_rgba8();

                ctx.load_texture(
                    name,
                    egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    ),
                    egui::TextureOptions::NEAREST,
                )
            })
            .map_err(|error| format!("Could not load Saya artwork {name}: {error}"));

        if let Err(error) = &result {
            eprintln!("{error}");
        }

        ctx.data_mut(|data| data.insert_temp(id, result.clone()));

        result
    });

    result.ok()
}

fn paint(ui: &egui::Ui, rect: Rect, name: &str, bytes: &[u8]) {
    if let Some(texture) = texture(ui.ctx(), name, bytes) {
        let size = texture.size_vec2();

        let fitted = size * (rect.width() / size.x).min(rect.height() / size.y);

        ui.painter().image(
            texture.id(),
            Rect::from_center_size(rect.center(), fitted),
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

pub fn show_header(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(56.0), egui::Sense::hover());

    paint_emblem(ui, rect);
    response.on_hover_text("Kurobane family emblem")
}

pub fn paint_emblem(ui: &egui::Ui, rect: egui::Rect) {
    paint(
        ui,
        rect,
        "family-emblem",
        include_bytes!("../../../assets/themes/midnight-neko/saya-emblem.png"),
    );
}

// Leave room for the longest wrapped line when Saya changes expressions.
const DIALOGUE_HEIGHT: f32 = 56.0;
const MAX_DIALOGUE_CHARACTERS: usize = 120;
const MIN_DIALOGUE_FONT_SIZE: f32 = 8.0;
// Tuck the trim's transparent lower edge beneath the strip so they read as one panel.
const PORTRAIT_TOGGLE_OVERLAP: f32 = 4.0;
const TOGGLE_HEIGHT: f32 = 28.0;
const LETTER_TRIM_OVERHANG: f32 = 7.0;
const LETTER_TRIM_HEIGHT: f32 = 10.0;
const LETTER_TOP_PADDING: f32 = 2.0;
const LETTER_BOTTOM_PADDING: f32 = 3.0;
const LETTER_TEXT_INSET: f32 = 12.0;
// The straight rails' outer edges sit 4.8% in from either texture edge.
const FRAME_RAIL_SPAN: f32 = 1.0 - 2.0 * 0.048;
const FRAME_ART: &[u8] = include_bytes!("../../../assets/themes/midnight-neko/saya-frame.png");

fn letter_text(
    ui: &egui::Ui,
    width: f32,
    line: &str,
    font: &egui::FontId,
) -> std::sync::Arc<egui::Galley> {
    debug_assert!(line.chars().count() <= MAX_DIALOGUE_CHARACTERS);
    let minimum_width = (300.0 - 4.0) * FRAME_RAIL_SPAN;
    let scale = (width / minimum_width).sqrt().clamp(1.0, 1.25);
    let mut font = font.clone();
    font.size = (11.0 * scale).min(13.0);
    let text_width = (width - 2.0 * LETTER_TEXT_INSET).max(1.0);
    let text_height =
        DIALOGUE_HEIGHT - LETTER_TOP_PADDING - LETTER_BOTTOM_PADDING - LETTER_TRIM_HEIGHT;
    loop {
        let mut job = egui::text::LayoutJob::simple(
            line.to_owned(),
            font.clone(),
            Palette::of(ui).ink,
            text_width,
        );
        job.halign = egui::Align::Center;
        let mut galley = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()));
        if galley.size().x > text_width {
            job.wrap.break_anywhere = true;
            galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        }
        if galley.size().y <= text_height || font.size <= MIN_DIALOGUE_FONT_SIZE {
            return galley;
        }
        font.size = (font.size - 0.5).max(MIN_DIALOGUE_FONT_SIZE);
    }
}

fn show_letter(ui: &mut egui::Ui, portrait: Rect, galley: std::sync::Arc<egui::Galley>) {
    let width = portrait.width() * FRAME_RAIL_SPAN;
    let rect = Rect::from_min_max(
        egui::pos2(
            portrait.center().x - width / 2.0,
            portrait.bottom() - portrait.height() * 0.04,
        ),
        egui::pos2(
            portrait.center().x + width / 2.0,
            portrait.bottom() + DIALOGUE_HEIGHT - LETTER_TRIM_OVERHANG,
        ),
    );
    ui.painter().rect_filled(rect, 0.0, Palette::of(ui).surface);

    // Center the glyphs themselves between the rails, independent of the font's line metrics.
    let text_top_edge = portrait.bottom() + LETTER_TOP_PADDING;
    let text_bottom_edge =
        rect.bottom() + LETTER_TRIM_OVERHANG - LETTER_TRIM_HEIGHT - LETTER_BOTTOM_PADDING;
    let text_center = (text_top_edge + text_bottom_edge) / 2.0;
    let text_top = text_center - galley.mesh_bounds.center().y;
    let text_rect = Rect::from_min_size(
        egui::pos2(rect.center().x - galley.size().x / 2.0, text_top),
        galley.size(),
    );
    let mut text_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("saya_letter")
            .max_rect(text_rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    text_ui.label(galley);
    // The letter can use the panel margins, just like the portrait.
    ui.add_space((rect.bottom() + LETTER_TRIM_OVERHANG - ui.cursor().top()).max(0.0));
    paint_letter_trim(ui, rect);
}

fn paint_letter_trim(ui: &egui::Ui, rect: Rect) {
    let painter = ui.painter();
    let bottom = rect.bottom() + LETTER_TRIM_OVERHANG;
    // Keep the full five-point stroke inside the measured outer edges.
    for x in [rect.left() + 2.5, rect.right() - 2.5] {
        for (width, color) in [
            (5.0_f32, Color32::from_rgb(57, 34, 44)),
            (3.0, Color32::from_rgb(182, 163, 156)),
            (1.0, Color32::from_rgb(104, 47, 66)),
        ] {
            painter.vline(
                x,
                rect.top()..=rect.bottom(),
                egui::Stroke::new(width, color),
            );
        }
    }

    let Some(frame) = texture(ui.ctx(), "portrait-frame", FRAME_ART) else {
        return;
    };
    // Reuse the frame's corners and ruby at fixed proportions. Stretch only the plain rails.
    let cap = 22.0_f32.min(rect.width() / 5.0);
    let left = rect.left() - 4.0;
    let right = rect.right() + 4.0;
    let center = rect.center().x;
    let edges = [
        left,
        left + cap,
        center - cap,
        center + cap,
        right - cap,
        right,
    ];
    let strips = [
        (0.0, 0.13),
        (0.13, 0.16),
        (0.37, 0.63),
        (0.84, 0.87),
        (0.87, 1.0),
    ];
    for (edge, (u0, u1)) in edges.windows(2).zip(strips) {
        painter.image(
            frame.id(),
            Rect::from_min_max(
                egui::pos2(edge[0], bottom - LETTER_TRIM_HEIGHT),
                egui::pos2(edge[1], bottom),
            ),
            Rect::from_min_max(egui::pos2(u0, 0.88), egui::pos2(u1, 1.0)),
            Color32::WHITE,
        );
    }
}

fn paint_love_hearts(ui: &egui::Ui, opening: Rect, elapsed: f64) {
    let painter = ui.painter().with_clip_rect(opening);
    // The top row has two lobes. The following rows taper to the point.
    let rows = [
        (1, 3, 0),
        (5, 7, 0),
        (0, 8, 1),
        (0, 8, 2),
        (1, 7, 3),
        (2, 6, 4),
        (3, 5, 5),
        (4, 4, 6),
    ];
    for (x, y, size, delay) in LOVE_HEARTS {
        let age = ((elapsed - delay) / LOVE_HEART_SECONDS) as f32;
        if !(0.0..1.0).contains(&age) {
            continue;
        }
        let opacity = (age * 12.0).min(1.0) * (1.0 - age).min(0.65) / 0.65;
        let cell = (opening.width() * size / 9.0).max(0.75);
        let center = opening.min + opening.size() * egui::vec2(x, y - 0.14 * age);
        let origin = (center - egui::vec2(4.5, 3.5) * cell).round();
        let ruby = Color32::from_rgb(223, 54, 100).gamma_multiply(opacity);
        for (left, right, row) in rows {
            painter.rect_filled(
                Rect::from_min_size(
                    origin + egui::vec2(left as f32, row as f32) * cell,
                    egui::vec2((right - left + 1) as f32, 1.0) * cell,
                ),
                0.0,
                ruby,
            );
        }
        painter.rect_filled(
            Rect::from_min_size(
                origin + egui::vec2(1.0, 1.0) * cell,
                egui::vec2(2.0, 1.0) * cell,
            ),
            0.0,
            Color32::from_rgb(255, 180, 199).gamma_multiply(opacity),
        );
    }
}

fn frame_width(ui: &egui::Ui) -> f32 {
    (ui.clip_rect().width() - 4.0).max(0.0)
}

pub fn reserved_height(ui: &egui::Ui) -> f32 {
    let collapsed = ui
        .ctx()
        .data_mut(|data| data.get_temp::<Companion>(state_id()))
        .is_some_and(|state| state.collapsed);
    if collapsed {
        TOGGLE_HEIGHT
    } else {
        frame_width(ui) + DIALOGUE_HEIGHT + TOGGLE_HEIGHT - PORTRAIT_TOGGLE_OVERLAP
    }
}

pub fn show_sidebar(ui: &mut egui::Ui) {
    let toggle_rect = Rect::from_min_size(
        egui::pos2(
            ui.clip_rect().left() + 5.0,
            ui.clip_rect().bottom() - 5.0 - TOGGLE_HEIGHT,
        ),
        egui::vec2(ui.clip_rect().width() - 10.0, TOGGLE_HEIGHT),
    );
    let state = ui
        .ctx()
        .data_mut(|data| data.get_temp::<Companion>(state_id()))
        .unwrap_or_default();
    if !state.collapsed {
        let top = toggle_rect.top() + PORTRAIT_TOGGLE_OVERLAP - DIALOGUE_HEIGHT - frame_width(ui);
        ui.add_space((top - ui.cursor().top()).max(0.0));
        show(ui);
    }
    // Painting in the panel margins must not enlarge the sidebar's layout footprint.
    let mut strip_ui = ui.new_child(
        egui::UiBuilder::new()
            .id(egui::Id::new("saya_strip"))
            .max_rect(toggle_rect)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    show_toggle(&mut strip_ui, state.collapsed);
}

fn show_toggle(ui: &mut egui::Ui, collapsed: bool) {
    let palette = Palette::of(ui);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), TOGGLE_HEIGHT),
        egui::Sense::hover(),
    );
    let hit_rect = rect.intersect(ui.clip_rect().shrink2(egui::vec2(5.0, 0.0)));
    let response = pointing_hand(ui.interact(
        hit_rect,
        ui.make_persistent_id("saya_toggle"),
        egui::Sense::click(),
    ));
    let label = if collapsed {
        "Show Saya"
    } else {
        "Minimize Saya"
    };
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), collapsed, label)
    });
    let center = rect.center();
    let painter = ui.painter();
    painter.rect_filled(rect, 0, palette.sunken);
    painter.rect_stroke(
        rect,
        0,
        egui::Stroke::new(1.0_f32, palette.metal.gamma_multiply(0.55)),
        egui::StrokeKind::Inside,
    );
    for side in [-1.0, 1.0] {
        let start = center + egui::vec2(side * 18.0, 0.0);
        let end = egui::pos2(
            if side < 0.0 {
                rect.left() + 16.0
            } else {
                rect.right() - 16.0
            },
            center.y,
        );
        painter.line_segment(
            [start, end],
            egui::Stroke::new(0.8_f32, palette.metal.gamma_multiply(0.5)),
        );
        crate::app::rendering::gothic::lamp(
            painter,
            Rect::from_center_size(end, egui::Vec2::splat(6.0)),
            palette.metal,
        );
    }
    let direction = if collapsed { -1.0 } else { 1.0 };
    painter.add(egui::Shape::line(
        vec![
            center + egui::vec2(-5.0, -direction * 2.5),
            center + egui::vec2(0.0, direction * 2.5),
            center + egui::vec2(5.0, -direction * 2.5),
        ],
        egui::Stroke::new(1.6_f32, palette.metal),
    ));
    crate::app::rendering::pixel::subtle_highlight(ui, &response, rect, palette.metal);
    if response.clone().on_hover_text(label).clicked() {
        let time = ui.input(|input| input.time);
        ui.ctx().data_mut(|data| {
            let mut state = data.get_temp::<Companion>(state_id()).unwrap_or_default();
            state.hold_while_hovered(false, time);
            state.collapsed = !collapsed;
            data.insert_temp(state_id(), state);
        });
        if collapsed {
            trigger(ui.ctx(), Action::Maximized);
        }
        ui.ctx().request_repaint();
    }
}

pub fn show(ui: &mut egui::Ui) {
    let time = ui.input(|input| input.time);

    let id = state_id();

    let mut state: Companion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());

    let width = frame_width(ui);
    let size = egui::Vec2::splat(width);
    let portrait_id = ui.next_auto_id();
    let hover_rect = Rect::from_min_size(
        egui::pos2(ui.clip_rect().center().x - width / 2.0, ui.cursor().top()),
        size,
    );
    // The artwork reaches the panel edge, but the divider owns its resize gutter.
    let hover_rect = hover_rect.intersect(ui.clip_rect().shrink2(egui::vec2(10.0, 0.0)));
    let response = pointing_hand(ui.interact(hover_rect, portrait_id, egui::Sense::click()));

    state.hold_while_hovered(response.hovered(), time);
    if response.clicked() {
        let random = uuid::Uuid::new_v4().as_u128();
        state.react_to_click(time, random);
    }

    let expression = state.visible(time);
    if state.displayed != expression || state.line_indices[expression as usize].is_none() {
        state.select_line(expression, uuid::Uuid::new_v4().as_u128());
    }

    let font = egui::TextStyle::Small.resolve(ui.style());
    let galley = letter_text(ui, width * FRAME_RAIL_SPAN, state.line(time), &font);
    let (_, slot) = ui.allocate_space(egui::vec2(ui.available_width(), width));
    // The frame uses the panel's side margins without widening the job list.
    let rect = Rect::from_center_size(egui::pos2(ui.clip_rect().center().x, slot.center().y), size);
    // Tuck the portrait's crop edges beneath the frame rails.
    let opening = Rect::from_min_max(
        rect.min + rect.size() * egui::vec2(0.065, 0.066),
        rect.min + rect.size() * egui::vec2(0.935, 0.936),
    );
    paint(ui, opening, &format!("{expression:?}"), expression.bytes());
    if let Some(elapsed) = state.love_elapsed(time) {
        paint_love_hearts(ui, opening, elapsed);
        ui.ctx().request_repaint();
    }
    show_letter(ui, rect, galley);
    paint(ui, rect, "portrait-frame", FRAME_ART);
    crate::app::rendering::pixel::subtle_outline(ui, &response, opening, Palette::of(ui).metal);

    if !state.hovered && time < state.until {
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(state.until - time));
    }

    ui.ctx().data_mut(|data| data.insert_temp(id, state));
}

pub fn show_welcome(ui: &mut egui::Ui) {
    let height = (ui.available_height() - 210.0).clamp(180.0, 420.0);

    let width = ui.available_width().min(340.0);

    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    paint(
        ui,
        rect,
        "full_body",
        include_bytes!("../../../assets/themes/midnight-neko/saya-full-body.png"),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn dialogue_budget_fits_the_fixed_box_at_every_sidebar_width() {
        let ctx = egui::Context::default();
        crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("midnight-neko"));
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.set_theme(theme);
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                let font = egui::TextStyle::Small.resolve(ui.style());
                let wide_text = "W".repeat(MAX_DIALOGUE_CHARACTERS);
                for width in [300.0, 350.0, 460.0] {
                    for expression in Expression::REACTIONS
                        .into_iter()
                        .chain([Expression::Neutral])
                    {
                        for line in expression
                            .lines()
                            .iter()
                            .copied()
                            .chain([wide_text.as_str()])
                        {
                            assert!(line.chars().count() <= MAX_DIALOGUE_CHARACTERS, "{line}");
                            let width = (width - 4.0) * FRAME_RAIL_SPAN;
                            let galley = letter_text(ui, width, line, &font);
                            assert!(
                                galley.size().y
                                    <= DIALOGUE_HEIGHT
                                        - LETTER_TOP_PADDING
                                        - LETTER_BOTTOM_PADDING
                                        - LETTER_TRIM_HEIGHT,
                                "{line}"
                            );
                            assert!(
                                galley.size().x <= width - LETTER_TEXT_INSET * 2.0 + 1.0,
                                "{line}: {:?}, width={width}",
                                galley.size()
                            );
                        }
                    }
                }
                let narrow = letter_text(ui, 296.0 * FRAME_RAIL_SPAN, "Hello, Human.", &font);
                let wide = letter_text(ui, 456.0 * FRAME_RAIL_SPAN, "Hello, Human.", &font);
                assert!(
                    wide.job.sections[0].format.font_id.size
                        > narrow.job.sections[0].format.font_id.size
                );
            });
        }
    }

    #[test]
    fn actions_replace_hovered_reactions_immediately() {
        let mut state = Companion::default();
        state.react(Action::NewJob.pool(), 0.0, 1);
        state.hold_while_hovered(true, 1.0);
        state.react(Action::Failed.pool(), 1.5, 2);
        assert_eq!(state.visible(100.0), Expression::Sad);
        state.react_to_click(101.0, 3);
        assert_ne!(state.visible(101.0), Expression::Sad);
        state.react(Action::Failed.pool(), 102.0, 4);
        assert_eq!(state.visible(102.0), Expression::Sad);
        state.hold_while_hovered(false, 103.0);
        assert_eq!(state.visible(105.9), Expression::Sad);
        assert_eq!(state.visible(106.0), Expression::Neutral);
    }

    #[test]
    fn timer_pauses_and_resumes_its_remaining_duration() {
        let mut state = Companion::default();
        state.react(Action::Failed.pool(), 10.0, 0);
        state.hold_while_hovered(true, 12.5);
        let line = state.line(12.5);
        assert_eq!(state.visible(100.0), Expression::Sad);
        assert_eq!(state.line(100.0), line);
        state.hold_while_hovered(false, 100.0);
        assert_eq!(state.visible(100.49), Expression::Sad);
        assert_eq!(state.visible(100.5), Expression::Neutral);
        assert_eq!(state.visible(10000.0), Expression::Neutral);
    }

    #[test]
    fn event_pools_do_not_repeat_and_single_expression_pools_still_work() {
        for pool in [
            Action::NewJob.pool(),
            Action::Maximized.pool(),
            Page::StartScreen.pool(),
            Page::JobSettings(uuid::Uuid::nil()).pool(),
            Page::Backups(uuid::Uuid::nil()).pool(),
            Page::SettingsJobs.pool(),
            Page::SettingsTheme.pool(),
        ] {
            for current in pool {
                for random in 0..20 {
                    let next = Expression::choose(pool, *current, random);
                    assert!(pool.contains(&next));
                    assert_ne!(*current, next);
                }
            }
        }
        assert_eq!(
            Expression::choose(Action::Failed.pool(), Expression::Sad, 0),
            Expression::Sad
        );
    }

    #[test]
    fn page_changes_do_not_overwrite_actions_or_repeat_on_redraw() {
        let ctx = egui::Context::default();
        crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("midnight-neko"));
        let mut until = 0.0;
        for (time, page) in [
            (0.0, Page::StartScreen),
            (0.5, Page::SettingsJobs),
            (1.0, Page::SettingsJobs),
            (1.5, Page::SettingsTheme),
        ] {
            let _ = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |_| {
                    if time == 0.5 {
                        trigger(&ctx, Action::Failed);
                    }
                    enter_page(&ctx, page);
                    let state = ctx
                        .data_mut(|data| data.get_temp::<Companion>(state_id()))
                        .unwrap();
                    if time == 0.5 || time == 1.0 {
                        assert_eq!(state.visible(time), Expression::Sad);
                    }
                    if time == 0.5 {
                        until = state.until;
                    }
                    if time == 1.0 {
                        assert_eq!(state.until, until);
                    }
                    if time == 1.5 {
                        assert!(Page::SettingsTheme.pool().contains(&state.visible(time)));
                    }
                },
            );
        }
    }

    #[test]
    fn love_hearts_play_once_per_entry_and_end_if_interrupted() {
        let mut state = Companion::default();
        state.set_expression(Expression::InLove, 1.0, 0);
        assert_eq!(state.love_elapsed(1.25), Some(0.25));

        state.set_expression(Expression::InLove, 1.5, 1);
        state.hold_while_hovered(true, 2.0);
        assert_eq!(state.visible(2.0), Expression::InLove);
        assert_eq!(state.love_elapsed(2.0), None);

        state.set_expression(Expression::Happy, 2.25, 2);
        state.set_expression(Expression::InLove, 2.5, 3);
        assert_eq!(state.love_elapsed(2.75), Some(0.25));
        state.set_expression(Expression::Sad, 2.75, 4);
        assert_eq!(state.love_elapsed(2.75), None);
    }

    #[test]
    fn dialogue_varies_without_repeating_each_expressions_previous_line() {
        let mut state = super::Companion::default();
        for expression in super::Expression::REACTIONS
            .into_iter()
            .chain([super::Expression::Neutral])
        {
            let lines = expression.lines();
            assert!(lines.len() >= 4);
            let unique: std::collections::HashSet<_> = lines.iter().collect();
            assert_eq!(unique.len(), lines.len());
            for previous in 0..lines.len() {
                let mut reached = std::collections::HashSet::new();
                for seed in 0..lines.len() - 1 {
                    state.line_indices[expression as usize] = Some(previous);
                    state.select_line(expression, seed as u128);
                    let selected = state.line_indices[expression as usize].unwrap();
                    assert_ne!(selected, previous);
                    reached.insert(selected);
                }
                assert_eq!(reached.len(), lines.len() - 1);
            }
        }
    }

    use super::*;

    #[test]

    fn all_refined_expressions_are_reachable_and_have_consistent_resolution() {
        for current in Expression::REACTIONS
            .into_iter()
            .chain([Expression::Neutral])
        {
            let next: Vec<_> = (0..26).map(|seed| current.next(seed)).collect();

            assert!(!next.contains(&current));

            assert!(Expression::REACTIONS
                .iter()
                .all(|value| *value == current || next.contains(value)));

            let image = image::load_from_memory(current.bytes()).unwrap();

            assert_eq!((image.width(), image.height()), (512, 512));
        }
    }

    #[test]
    fn sidebar_starts_expanded_and_returns_to_neutral_without_idle_reactions() {
        let mut state = Companion::default();
        assert!(!state.collapsed);
        state.react_to_click(10.0, 4);
        assert_ne!(state.visible(10.1), Expression::Neutral);
        assert_eq!(state.visible(14.0), Expression::Neutral);
        assert_eq!(state.visible(1000.0), Expression::Neutral);
    }
}
