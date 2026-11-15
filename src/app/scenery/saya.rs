//! Saya's static welcome figure and individually refined sidebar expressions.

mod dialogue;
mod expressions;
mod gallery;
mod overlays;
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

pub use gallery::{
    close as close_gallery, is_open as gallery_open, show as show_gallery, suppress_home_click,
};
pub use reactions::{Action, Page};

pub fn trigger(ctx: &egui::Context, action: Action) {
    if !crate::app::theme::style::Skin::of(ctx).effects.hearts {
        return;
    }
    let time = ctx.input(|input| input.time);
    let random = uuid::Uuid::new_v4().as_u128();
    ctx.data_mut(|data| {
        let mut state: SayaCompanion = data.get_temp(state_id()).unwrap_or_default();
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
        let mut state: SayaCompanion = data.get_temp(state_id()).unwrap_or_default();
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

struct SayaCompanion {
    expression: Expression,
    overlay_started_at: Option<f64>,
    until: f64,
    paused_remaining: Option<f64>,
    hovered: bool,
    collapsed: bool,
    page: Option<Page>,
    action_time: Option<f64>,
    displayed: Expression,
    line_indices: std::collections::HashMap<Expression, usize>,
    preview: bool,
}

impl SayaCompanion {
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
            self.overlay_started_at = overlays::duration(expression).map(|_| time);
        }
        self.preview = false;
        self.expression = expression;
        self.until = time + REACTION_SECONDS;
        self.select_line(expression, random.rotate_left(64));
        self.paused_remaining = self.hovered.then_some(REACTION_SECONDS);
    }

    fn overlay_elapsed(&self, time: f64) -> Option<f64> {
        self.overlay_started_at
            .filter(|_| overlays::duration(self.visible(time)).is_some())
            .map(|start| time - start)
            .filter(|elapsed| {
                (0.0..overlays::duration(self.visible(time)).unwrap_or(0.0)).contains(elapsed)
            })
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
        let count = expression.lines().len();
        let previous = self.line_indices.get(&expression).copied();
        let mut choice = (random % (count - usize::from(previous.is_some())) as u128) as usize;
        if previous.is_some_and(|old| choice >= old) {
            choice += 1;
        }
        self.line_indices.insert(expression, choice);
        self.displayed = expression;
    }

    fn line(&self, time: f64) -> &'static str {
        let expression = self.visible(time);
        expression.lines()[self.line_indices.get(&expression).copied().unwrap_or(0)]
    }

    fn visible(&self, time: f64) -> Expression {
        if self.preview
            || self
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

fn decode_artwork(bytes: &[u8]) -> Result<egui::ColorImage, String> {
    let image = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let image = if image.width() > 1024 || image.height() > 1024 {
        image.thumbnail(1024, 1024)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

fn artwork_id(name: &str) -> egui::Id {
    egui::Id::new(("saya_artwork", name))
}

type DecodedArtwork = (String, Result<egui::ColorImage, String>);

#[derive(Clone)]
struct Preloader(std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<DecodedArtwork>>>);

/// Decode off-thread with one pending image; upload at most one image per frame.
pub fn preload(ctx: &egui::Context) {
    if !super::moonveil::decorations_visible(ctx) {
        return;
    }
    let id = egui::Id::new("saya_artwork_preloader");
    let preloader = ctx
        .data_mut(|data| data.get_temp::<Preloader>(id))
        .unwrap_or_else(|| {
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            std::thread::spawn(move || {
                for expression in Expression::ALL.iter().copied() {
                    let image = decode_artwork(expression.bytes());
                    if sender.send((format!("{expression:?}"), image)).is_err() {
                        break;
                    }
                }
            });
            let preloader = Preloader(std::sync::Arc::new(std::sync::Mutex::new(receiver)));
            ctx.data_mut(|data| data.insert_temp(id, preloader.clone()));
            preloader
        });
    let frame_id = id.with("uploaded_frame");
    let frame = ctx.cumulative_frame_nr();
    if ctx.data_mut(|data| data.get_temp::<u64>(frame_id)) == Some(frame) {
        return;
    }
    ctx.data_mut(|data| data.insert_temp(frame_id, frame));
    let Ok(receiver) = preloader.0.lock() else {
        return;
    };
    let Ok((name, decoded)) = receiver.try_recv() else {
        return;
    };
    let id = artwork_id(&name);
    if ctx
        .data_mut(|data| data.get_temp::<Result<egui::TextureHandle, String>>(id))
        .is_some()
    {
        return;
    }
    let result = decoded.map(|image| ctx.load_texture(&name, image, egui::TextureOptions::NEAREST));
    if let Err(error) = &result {
        eprintln!("Could not preload Saya artwork {name}: {error}");
    }
    ctx.data_mut(|data| data.insert_temp(id, result));
}

fn texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let id = artwork_id(name);
    let cached: Option<Result<egui::TextureHandle, String>> =
        ctx.data_mut(|data| data.get_temp(id));
    cached
        .unwrap_or_else(|| {
            let result = decode_artwork(bytes)
                .map(|image| ctx.load_texture(name, image, egui::TextureOptions::NEAREST));
            if let Err(error) = &result {
                eprintln!("Could not load Saya artwork {name}: {error}");
            }
            ctx.data_mut(|data| data.insert_temp(id, result.clone()));
            result
        })
        .ok()
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
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(48.0), egui::Sense::hover());

    paint_emblem(ui, rect);
    gallery::emblem_hold(ui, rect);
    response.on_hover_text("Kurobane family emblem")
}

pub fn paint_emblem(ui: &egui::Ui, rect: egui::Rect) {
    paint(
        ui,
        rect,
        "family-emblem",
        include_bytes!("../../../assets/themes/moonveil/saya-emblem.png"),
    );
}

const DIALOGUE_HEIGHT: f32 = 60.0;
const MAX_DIALOGUE_CHARACTERS: usize = 120;
const MIN_DIALOGUE_FONT_SIZE: f32 = 9.0;
const TOGGLE_HEIGHT: f32 = 26.0;
const LETTER_TOP_PADDING: f32 = 7.0;
const LETTER_BOTTOM_PADDING: f32 = 7.0;
const LETTER_TEXT_INSET: f32 = 14.0;
const PANEL_INSET: f32 = 5.0;
const PANEL_BORDER: f32 = 1.0;
const COLLAPSE_SECONDS: f32 = 0.24;

struct SayaLayout {
    outer: Rect,
    portrait: Rect,
    letter: Rect,
    toggle: Rect,
}

impl SayaLayout {
    fn new(bounds: Rect, expansion: f32) -> Self {
        let width = (bounds.width() - PANEL_BORDER * 2.0).max(0.0);
        let height = TOGGLE_HEIGHT + PANEL_BORDER * 2.0 + (width + DIALOGUE_HEIGHT) * expansion;
        let outer = Rect::from_min_max(
            egui::pos2(bounds.left(), bounds.bottom() - height),
            bounds.max,
        );
        let inner = outer.shrink(PANEL_BORDER);
        let portrait = Rect::from_min_size(inner.min, egui::Vec2::splat(width));
        let letter =
            Rect::from_min_size(portrait.left_bottom(), egui::vec2(width, DIALOGUE_HEIGHT));
        let toggle = Rect::from_min_max(
            egui::pos2(inner.left(), inner.bottom() - TOGGLE_HEIGHT),
            inner.max,
        );
        Self {
            outer,
            portrait,
            letter,
            toggle,
        }
    }
}

fn expansion(ui: &egui::Ui) -> f32 {
    let collapsed = ui
        .ctx()
        .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
        .is_some_and(|state| state.collapsed);
    let amount = ui.ctx().animate_bool_with_time(
        egui::Id::new("saya_expansion"),
        !collapsed,
        COLLAPSE_SECONDS,
    );
    amount * amount * (3.0 - 2.0 * amount)
}

fn letter_text(
    ui: &egui::Ui,
    width: f32,
    line: &str,
    font: &egui::FontId,
) -> std::sync::Arc<egui::Galley> {
    debug_assert!(line.chars().count() <= MAX_DIALOGUE_CHARACTERS);
    let minimum_width = 300.0 - (PANEL_INSET + PANEL_BORDER) * 2.0;
    let scale = (width / minimum_width).sqrt().clamp(1.0, 1.25);
    let mut font = font.clone();
    font.size = (12.0 * scale).min(13.5);
    let text_width = (width - 2.0 * LETTER_TEXT_INSET).max(1.0);
    let text_height = DIALOGUE_HEIGHT - LETTER_TOP_PADDING - LETTER_BOTTOM_PADDING;
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

fn show_letter(ui: &mut egui::Ui, rect: Rect, galley: std::sync::Arc<egui::Galley>) {
    let palette = Palette::of(ui);
    ui.painter().rect_filled(rect, 0, palette.surface);
    ui.painter().hline(
        rect.x_range(),
        rect.top(),
        egui::Stroke::new(1.0_f32, palette.metal.gamma_multiply(0.35)),
    );
    let position = egui::pos2(
        rect.center().x - galley.size().x / 2.0,
        rect.center().y - galley.mesh_bounds.center().y,
    );
    let mut text_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("saya_letter")
            .max_rect(Rect::from_min_size(position, galley.size()))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    text_ui.label(galley);
}

fn paint_love_hearts(ui: &egui::Ui, opening: Rect, elapsed: f64) {
    let painter = ui
        .painter()
        .with_clip_rect(ui.clip_rect().intersect(opening));
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

pub fn reserved_height(ui: &egui::Ui) -> f32 {
    SayaLayout::new(ui.clip_rect().shrink(PANEL_INSET), expansion(ui))
        .outer
        .height()
        + 6.0
}

pub fn show_sidebar(ui: &mut egui::Ui) {
    let amount = expansion(ui);
    let layout = SayaLayout::new(ui.clip_rect().shrink(PANEL_INSET), amount);
    let palette = Palette::of(ui);
    ui.painter().rect_filled(layout.outer, 3, palette.surface);
    if amount > 0.0 {
        let clip = Rect::from_min_max(
            layout.outer.min + egui::Vec2::splat(PANEL_BORDER),
            egui::pos2(layout.outer.right() - PANEL_BORDER, layout.toggle.top()),
        );
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .id(egui::Id::new("saya_content"))
                .max_rect(layout.outer),
        );
        content_ui.set_clip_rect(ui.clip_rect().intersect(clip));
        content_ui.set_opacity(amount);
        if amount < 1.0 {
            content_ui.disable();
        }
        show_portrait(&mut content_ui, layout.portrait, layout.letter);
    }
    let collapsed = ui
        .ctx()
        .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
        .is_some_and(|state| state.collapsed);
    let mut strip_ui = ui.new_child(
        egui::UiBuilder::new()
            .id(egui::Id::new("saya_strip"))
            .max_rect(layout.toggle)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    show_toggle(&mut strip_ui, collapsed);
    ui.painter().rect_stroke(
        layout.outer,
        3,
        egui::Stroke::new(1.0_f32, palette.metal.gamma_multiply(0.55)),
        egui::StrokeKind::Inside,
    );
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
    painter.rect_filled(rect, 0, palette.surface);
    painter.hline(
        rect.x_range(),
        rect.top(),
        egui::Stroke::new(1.0_f32, palette.metal.gamma_multiply(0.35)),
    );
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
            let mut state = data
                .get_temp::<SayaCompanion>(state_id())
                .unwrap_or_default();
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

fn show_portrait(ui: &mut egui::Ui, rect: Rect, letter: Rect) {
    let time = ui.input(|input| input.time);

    let id = state_id();

    let mut state: SayaCompanion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());

    let portrait_id = ui.make_persistent_id("saya_portrait");
    let hover_rect = rect.intersect(ui.clip_rect().shrink2(egui::vec2(5.0, 0.0)));
    let response = pointing_hand(ui.interact(hover_rect, portrait_id, egui::Sense::click()));

    if !state.preview {
        state.hold_while_hovered(response.hovered(), time);
    }
    if response.clicked() && !state.preview {
        let random = uuid::Uuid::new_v4().as_u128();
        state.react_to_click(time, random);
    }

    let expression = state.visible(time);
    if state.displayed != expression || !state.line_indices.contains_key(&expression) {
        state.select_line(expression, uuid::Uuid::new_v4().as_u128());
    }

    let font = egui::TextStyle::Small.resolve(ui.style());
    let galley = letter_text(ui, letter.width(), state.line(time), &font);
    paint(ui, rect, &format!("{expression:?}"), expression.bytes());
    if let Some(elapsed) = state.overlay_elapsed(time) {
        overlays::paint(ui, rect, expression, elapsed);
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
    show_letter(ui, letter, galley);
    crate::app::rendering::pixel::subtle_outline(ui, &response, rect, Palette::of(ui).metal);

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
        include_bytes!("../../../assets/themes/moonveil/saya-full-body.png"),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn preloader_uploads_only_one_artwork_per_frame() {
        let ctx = eframe::egui::Context::default();
        let (sender, receiver) = std::sync::mpsc::sync_channel(2);
        for name in ["preload-a", "preload-b"] {
            sender
                .send((
                    name.to_owned(),
                    Ok(eframe::egui::ColorImage::filled(
                        [2, 2],
                        eframe::egui::Color32::WHITE,
                    )),
                ))
                .unwrap();
        }
        ctx.data_mut(|data| {
            data.insert_temp(
                eframe::egui::Id::new("saya_artwork_preloader"),
                super::Preloader(std::sync::Arc::new(std::sync::Mutex::new(receiver))),
            )
        });
        let _ = ctx.run_ui(eframe::egui::RawInput::default(), |_| {
            super::preload(&ctx);
            super::preload(&ctx);
            assert!(ctx
                .data_mut(
                    |data| data.get_temp::<Result<eframe::egui::TextureHandle, String>>(
                        super::artwork_id("preload-a")
                    )
                )
                .is_some());
            assert!(ctx
                .data_mut(
                    |data| data.get_temp::<Result<eframe::egui::TextureHandle, String>>(
                        super::artwork_id("preload-b")
                    )
                )
                .is_none());
        });
        let _ = ctx.run_ui(eframe::egui::RawInput::default(), |_| super::preload(&ctx));
        assert!(ctx
            .data_mut(
                |data| data.get_temp::<Result<eframe::egui::TextureHandle, String>>(
                    super::artwork_id("preload-b")
                )
            )
            .is_some());
    }

    #[test]
    fn dialogue_budget_fits_the_fixed_box_at_every_sidebar_width() {
        let ctx = egui::Context::default();
        crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("moonveil"));
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
                            let width = width - (PANEL_INSET + PANEL_BORDER) * 2.0;
                            let galley = letter_text(ui, width, line, &font);
                            assert!(
                                galley.size().y
                                    <= DIALOGUE_HEIGHT - LETTER_TOP_PADDING - LETTER_BOTTOM_PADDING,
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
                let narrow = letter_text(ui, 288.0, "Hello, Human.", &font);
                let wide = letter_text(ui, 448.0, "Hello, Human.", &font);
                assert!(
                    wide.job.sections[0].format.font_id.size
                        > narrow.job.sections[0].format.font_id.size
                );
            });
        }
    }

    #[test]
    fn companion_sections_share_one_frame_and_animation_keeps_the_caret_anchored() {
        for width in [300.0, 460.0] {
            let bounds = Rect::from_min_size(Pos2::ZERO, egui::vec2(width, 800.0));
            let expanded = SayaLayout::new(bounds, 1.0);
            assert_eq!(expanded.portrait.width(), expanded.portrait.height());
            assert_eq!(expanded.portrait.left_bottom(), expanded.letter.left_top());
            assert_eq!(expanded.letter.left_bottom(), expanded.toggle.left_top());
            assert_eq!(expanded.portrait.width(), expanded.letter.width());
            assert_eq!(expanded.letter.width(), expanded.toggle.width());
            let mut previous = 0.0;
            for amount in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let layout = SayaLayout::new(bounds, amount);
                assert_eq!(layout.toggle, expanded.toggle);
                assert!(layout.outer.height() > previous);
                previous = layout.outer.height();
            }
        }
    }

    #[test]
    fn collapse_animation_has_intermediate_states_and_can_reverse() {
        let ctx = egui::Context::default();
        for (time, collapsed) in [
            (0.0, false),
            (0.1, true),
            (0.22, true),
            (0.5, true),
            (0.6, false),
            (0.72, false),
            (1.0, false),
        ] {
            let _ = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    ctx.data_mut(|data| {
                        let mut state = data
                            .get_temp::<SayaCompanion>(state_id())
                            .unwrap_or_default();
                        state.collapsed = collapsed;
                        data.insert_temp(state_id(), state);
                    });
                    let value = expansion(ui);
                    if time == 0.22 || time == 0.72 {
                        assert!(value > 0.0 && value < 1.0);
                    }
                    if time == 0.5 {
                        assert_eq!(value, 0.0);
                    }
                    if time == 0.0 || time == 1.0 {
                        assert_eq!(value, 1.0);
                    }
                },
            );
        }
    }

    #[test]
    fn actions_replace_hovered_reactions_immediately() {
        let mut state = SayaCompanion::default();
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
        let mut state = SayaCompanion::default();
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
        crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("moonveil"));
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
                        .data_mut(|data| data.get_temp::<SayaCompanion>(state_id()))
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
    fn love_hearts_play_once_per_entry_and_stop_when_interrupted() {
        let mut state = SayaCompanion::default();
        state.set_expression(Expression::Love, 1.0, 0);
        assert_eq!(state.overlay_elapsed(1.25), Some(0.25));

        state.set_expression(Expression::Love, 1.5, 1);
        state.hold_while_hovered(true, 2.0);
        assert_eq!(state.visible(2.0), Expression::Love);
        assert_eq!(state.overlay_elapsed(2.0), None);

        state.set_expression(Expression::Happy, 2.25, 2);
        state.set_expression(Expression::Love, 2.5, 3);
        assert_eq!(state.overlay_elapsed(2.75), Some(0.25));
        state.set_expression(Expression::Sad, 2.75, 4);
        assert_eq!(state.overlay_elapsed(2.75), None);
    }

    #[test]
    fn authored_overlays_finish_even_while_hovered() {
        for expression in Expression::REACTIONS {
            let mut state = SayaCompanion::default();
            state.set_expression(expression, 10.0, 0);
            state.hold_while_hovered(true, 10.1);
            assert_eq!(
                state.overlay_elapsed(10.2),
                matches!(expression, Expression::Love | Expression::Excited).then_some(10.2 - 10.0)
            );
            assert_eq!(state.overlay_elapsed(12.0), None);
            assert_eq!(state.visible(12.0), expression);
            state.set_expression(Expression::Neutral, 12.0, 0);
            assert_eq!(state.overlay_elapsed(12.0), None);
        }
    }

    #[test]
    fn dialogue_varies_without_repeating_each_expressions_previous_line() {
        let mut state = super::SayaCompanion::default();
        for expression in super::Expression::REACTIONS
            .into_iter()
            .chain([super::Expression::Neutral])
        {
            let lines = expression.lines();
            assert_eq!(lines.len(), 4);
            let unique: std::collections::HashSet<_> = lines.iter().collect();
            assert_eq!(unique.len(), lines.len());
            for previous in 0..lines.len() {
                let mut reached = std::collections::HashSet::new();
                for seed in 0..lines.len() - 1 {
                    state.line_indices.insert(expression, previous);
                    state.select_line(expression, seed as u128);
                    let selected = state.line_indices[&expression];
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
            let next: Vec<_> = (0..Expression::REACTIONS.len() as u128)
                .map(|seed| current.next(seed))
                .collect();

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
        let mut state = SayaCompanion::default();
        assert!(!state.collapsed);
        state.react_to_click(10.0, 4);
        assert_ne!(state.visible(10.1), Expression::Neutral);
        assert_eq!(state.visible(14.0), Expression::Neutral);
        assert_eq!(state.visible(1000.0), Expression::Neutral);
    }
}
