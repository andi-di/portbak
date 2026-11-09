//! Saya's static welcome figure and individually refined sidebar expressions.

mod dialogue;

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

#[derive(Clone, Copy)]
pub enum Action {
    NewJob,
    CheckNow,
    RemoveJob,
    Settings,
    SelectJob,
    Failed,
    AutomaticEnabled,
    AutomaticDisabled,
    RestoreConfirmation,
    RestoreCompleted,
    ProtectionEnabled,
    DarkMode,
    LightMode,
}

impl Action {
    fn priority(self) -> u8 {
        match self {
            Self::Failed | Self::RestoreCompleted | Self::ProtectionEnabled => 2,
            _ => 1,
        }
    }

    fn expression(self) -> Expression {
        match self {
            Self::NewJob => Expression::Happy,
            Self::CheckNow => Expression::Energetic,
            Self::RemoveJob => Expression::Pouting,
            Self::Settings => Expression::Curious,
            Self::SelectJob => Expression::Waiting,
            Self::Failed => Expression::Sad,
            Self::AutomaticEnabled => Expression::Energetic,
            Self::AutomaticDisabled => Expression::Pouting,
            Self::RestoreConfirmation => Expression::Question,
            Self::RestoreCompleted => Expression::Happy,
            Self::ProtectionEnabled => Expression::Dominant,
            Self::DarkMode => Expression::Happy,
            Self::LightMode => Expression::Sleepy,
        }
    }
}

pub fn trigger(ctx: &egui::Context, action: Action) {
    if !crate::app::theme::style::Skin::of(ctx).effects.hearts {
        return;
    }
    let time = ctx.input(|input| input.time);
    let random = uuid::Uuid::new_v4().as_u128();
    ctx.data_mut(|data| {
        let mut state: Companion = data.get_temp(state_id()).unwrap_or_default();
        state.request_priority(action.expression(), action.priority(), time, random);
        data.insert_temp(state_id(), state);
    });
    ctx.request_repaint();
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]

enum Expression {
    #[default]
    Neutral,

    Angry,

    Bored,

    BoredExpectant,

    Celebrating,

    Curious,

    Cute,

    Dominant,

    Embarrassed,

    Energetic,

    Excited,

    Flirty,

    Flustered,

    Happy,

    InLove,

    Killer,

    Meek,

    Pleading,

    Pouting,

    Praying,

    Question,

    Sad,

    Seductive,

    Shy,

    Sleepy,

    Smug,

    Waiting,
}

impl Expression {
    const REACTIONS: [Self; 26] = [
        Self::Angry,
        Self::Bored,
        Self::BoredExpectant,
        Self::Celebrating,
        Self::Curious,
        Self::Cute,
        Self::Dominant,
        Self::Embarrassed,
        Self::Energetic,
        Self::Excited,
        Self::Flirty,
        Self::Flustered,
        Self::Happy,
        Self::InLove,
        Self::Killer,
        Self::Meek,
        Self::Pleading,
        Self::Pouting,
        Self::Praying,
        Self::Question,
        Self::Sad,
        Self::Seductive,
        Self::Shy,
        Self::Sleepy,
        Self::Smug,
        Self::Waiting,
    ];

    fn next(self, random: u128) -> Self {
        let choices: Vec<_> = Self::REACTIONS
            .into_iter()
            .filter(|choice| *choice != self)
            .collect();

        choices[(random % choices.len() as u128) as usize]
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Neutral => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/neutral.png")
            }

            Self::Angry => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/angry.png")
            }

            Self::Bored => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/bored.png")
            }

            Self::BoredExpectant => include_bytes!(
                "../../../assets/themes/midnight-neko/expressions/bored-expectant.png"
            ),

            Self::Celebrating => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/celebrating.png")
            }

            Self::Curious => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/curious.png")
            }

            Self::Cute => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/cute.png")
            }

            Self::Dominant => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/dominant.png")
            }

            Self::Embarrassed => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/embarrassed.png")
            }

            Self::Energetic => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/energetic.png")
            }

            Self::Excited => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/excited.png")
            }

            Self::Flirty => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/flirty.png")
            }

            Self::Flustered => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/flustered.png")
            }

            Self::Happy => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/happy.png")
            }

            Self::InLove => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/in-love.png")
            }

            Self::Killer => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/killer.png")
            }

            Self::Meek => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/meek.png")
            }

            Self::Pleading => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/pleading.png")
            }

            Self::Pouting => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/pouting.png")
            }

            Self::Praying => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/praying.png")
            }

            Self::Question => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/question.png")
            }

            Self::Sad => include_bytes!("../../../assets/themes/midnight-neko/expressions/sad.png"),

            Self::Seductive => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/seductive.png")
            }

            Self::Shy => include_bytes!("../../../assets/themes/midnight-neko/expressions/shy.png"),

            Self::Sleepy => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/sleepy.png")
            }

            Self::Smug => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/smug.png")
            }

            Self::Waiting => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions/waiting.png")
            }
        }
    }
}

#[derive(Clone, Default)]

struct Companion {
    expression: Expression,
    love_started_at: Option<f64>,
    until: f64,
    next_random: Option<f64>,
    hovered: bool,
    pending: Option<Expression>,
    pending_priority: u8,
    reaction_priority: u8,
    cooldown_until: f64,
    displayed: Expression,
    line_indices: [Option<usize>; 27],
}

impl Companion {
    fn schedule(&mut self, time: f64, random: u128) {
        self.next_random = Some(time + 18.0 + (random as u64 % 37_001) as f64 / 1000.0);
    }

    fn tick(&mut self, time: f64, hovered: bool, random: u128) {
        if !hovered && time >= self.cooldown_until {
            if let Some(expression) = self.pending.take() {
                self.request_priority(expression, self.pending_priority, time, random);
                return;
            }
        }
        match self.next_random {
            None => self.schedule(time, random),
            Some(deadline)
                if time >= deadline && !hovered && self.visible(time) == Expression::Neutral =>
            {
                self.react(time, random)
            }
            _ => {}
        }
    }

    fn react_to_click(&mut self, time: f64, random: u128) {
        self.pending = None;
        self.pending_priority = 0;
        self.set_expression(self.expression.next(random), time, random);
        self.cooldown_until = time + 2.0;
    }

    fn request_priority(&mut self, expression: Expression, priority: u8, time: f64, random: u128) {
        if priority < self.reaction_priority && time < self.until {
            return;
        }
        if self.hovered || time < self.cooldown_until {
            if self.pending.is_none() || priority >= self.pending_priority {
                self.pending = Some(expression);
                self.pending_priority = priority;
            }
        } else {
            self.set_expression(expression, time, random);
            self.reaction_priority = priority;
            self.cooldown_until = time + 2.0;
        }
    }

    fn set_expression(&mut self, expression: Expression, time: f64, random: u128) {
        if expression != self.visible(time) {
            self.love_started_at = (expression == Expression::InLove).then_some(time);
        }
        self.reaction_priority = 0;
        self.expression = expression;
        self.until = time + REACTION_SECONDS;
        self.select_line(expression, random.rotate_left(64));
        self.schedule(time, random);
    }

    fn love_elapsed(&self, time: f64) -> Option<f64> {
        self.love_started_at
            .filter(|_| self.visible(time) == Expression::InLove)
            .map(|start| time - start)
            .filter(|elapsed| (0.0..LOVE_HEART_SECONDS + LOVE_HEARTS[2].3).contains(elapsed))
    }

    fn hold_while_hovered(&mut self, hovered: bool, time: f64) {
        self.hovered = hovered;
        if hovered && self.visible(time) != Expression::Neutral {
            self.until = time + REACTION_SECONDS;
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
        if time < self.until {
            self.expression
        } else {
            Expression::Neutral
        }
    }

    fn react(&mut self, time: f64, random: u128) {
        self.set_expression(self.expression.next(random), time, random);
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
const DIALOGUE_HEIGHT: f32 = 100.0;
const LETTER_TRIM_OVERHANG: f32 = 7.0;
const LETTER_TRIM_HEIGHT: f32 = 20.0;
const LETTER_PADDING: f32 = 8.0;
const LETTER_TEXT_INSET: f32 = 16.0;
// The straight rails' outer edges sit 4.8% in from either texture edge.
const FRAME_RAIL_SPAN: f32 = 1.0 - 2.0 * 0.048;
const FRAME_ART: &[u8] = include_bytes!("../../../assets/themes/midnight-neko/saya-frame.png");

fn letter_text(
    ui: &egui::Ui,
    width: f32,
    line: &str,
    font: &egui::FontId,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(
        line.to_owned(),
        font.clone(),
        Palette::of(ui).ink,
        (width - 2.0 * LETTER_TEXT_INSET).max(1.0),
    );
    job.halign = egui::Align::Center;
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

fn letter_height(galley: &egui::Galley) -> f32 {
    galley.size().y.max(24.0) + 2.0 * LETTER_PADDING + LETTER_TRIM_HEIGHT
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
            portrait.bottom() + letter_height(&galley) - LETTER_TRIM_OVERHANG,
        ),
    );
    ui.painter().rect_filled(rect, 0.0, Palette::of(ui).surface);

    // Center the glyphs themselves between the rails, independent of the font's line metrics.
    let text_center =
        (portrait.bottom() + rect.bottom() + LETTER_TRIM_OVERHANG - LETTER_TRIM_HEIGHT) / 2.0;
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
    frame_width(ui) + DIALOGUE_HEIGHT
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
    let response = pointing_hand(ui.interact(hover_rect, portrait_id, egui::Sense::click()));

    state.hold_while_hovered(response.hovered(), time);
    if response.clicked() {
        let random = uuid::Uuid::new_v4().as_u128();
        state.react_to_click(time, random);
    }
    state.tick(time, response.hovered(), uuid::Uuid::new_v4().as_u128());

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

    if time < state.until {
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(state.until - time));
    }

    if let Some(deadline) = state.next_random {
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64((deadline - time).max(0.25)));
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
    fn every_dialogue_line_fits_the_reserved_height_at_minimum_sidebar_width() {
        use super::{
            frame_width, letter_height, letter_text, reserved_height, Expression, FRAME_RAIL_SPAN,
        };
        use eframe::egui;

        let ctx = egui::Context::default();
        crate::app::theme::style::install(&ctx, crate::app::theme::skins::find("midnight-neko"));
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(300.0, crate::MIN_WINDOW_SIZE.y),
                )),
                ..Default::default()
            },
            |ui| {
                let width = frame_width(ui);
                let font = egui::TextStyle::Small.resolve(ui.style());
                for expression in [Expression::Neutral]
                    .into_iter()
                    .chain(Expression::REACTIONS)
                {
                    for line in expression.lines() {
                        let galley = letter_text(ui, width * FRAME_RAIL_SPAN, line, &font);
                        assert!(
                            width + letter_height(&galley) <= reserved_height(ui),
                            "{line}"
                        );
                    }
                }
            },
        );
    }

    #[test]
    fn direct_click_reacts_immediately_even_during_hover_and_cooldown() {
        use super::{Companion, Expression};
        let mut state = Companion::default();
        state.request_priority(Expression::Happy, 2, 0.0, 1);
        state.hold_while_hovered(true, 0.1);
        state.request_priority(Expression::Sad, 2, 0.1, 2);
        state.react_to_click(0.2, 3);
        let reaction = state.visible(0.2);
        assert_ne!(reaction, Expression::Happy);
        assert!(state.pending.is_none());
        state.react_to_click(0.3, 4);
        assert_ne!(state.visible(0.3), reaction);
        let reaction = state.visible(0.3);
        state.hold_while_hovered(true, 1.0);
        state.tick(1.0, true, 5);
        assert_eq!(state.visible(1.0), reaction);
    }

    #[test]
    fn outcomes_win_cooldowns_and_hover_defers_changes() {
        use super::{Companion, Expression};
        let mut state = Companion::default();
        state.request_priority(Expression::Waiting, 1, 0.0, 1);
        state.request_priority(Expression::Sad, 2, 0.5, 2);
        state.request_priority(Expression::Cute, 1, 1.0, 3);
        assert_eq!(state.visible(1.0), Expression::Waiting);
        state.tick(2.0, false, 4);
        assert_eq!(state.visible(2.0), Expression::Sad);
        state.request_priority(Expression::Curious, 1, 2.5, 5);
        state.next_random = Some(2.5);
        state.tick(3.0, false, 6);
        assert_eq!(state.visible(3.0), Expression::Sad);
        state.hold_while_hovered(true, 4.0);
        let line = state.line(4.0);
        state.request_priority(Expression::Happy, 2, 4.0, 7);
        state.tick(4.0, true, 8);
        assert_eq!(state.visible(4.0), Expression::Sad);
        assert_eq!(state.line(4.0), line);
        state.hold_while_hovered(false, 4.1);
        state.tick(4.1, false, 9);
        assert_eq!(state.visible(4.1), Expression::Happy);
    }

    #[test]
    fn random_deadlines_vary_and_do_not_interrupt_hovered_reactions() {
        let mut state = super::Companion::default();
        state.tick(0.0, false, 1000);
        assert_eq!(state.next_random, Some(19.0));
        state.tick(18.9, false, 2000);
        assert_eq!(state.visible(18.9), super::Expression::Neutral);
        state.tick(19.0, false, 2000);
        let expression = state.visible(19.0);
        assert_ne!(expression, super::Expression::Neutral);
        assert_eq!(state.next_random, Some(39.0));
        state.set_expression(expression, 40.0, 2000);
        state.hold_while_hovered(true, 41.0);
        state.next_random = Some(41.0);
        state.request_priority(super::Expression::Curious, 1, 41.0, 3000);
        let line = state.line(41.0);
        state.tick(41.0, true, 4000);
        assert_eq!(state.visible(41.0), expression);
        assert_eq!(state.line(41.0), line);
        state.hold_while_hovered(false, 41.1);
        state.tick(41.1, false, 5000);
        assert_eq!(state.visible(41.1), super::Expression::Curious);
        assert!(state.next_random.unwrap() > 41.1 + 18.0);
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

    fn sidebar_returns_to_neutral_after_reaction() {
        let mut state = Companion::default();

        state.react(10.0, 4);

        assert_ne!(state.visible(10.1), Expression::Neutral);

        assert_eq!(state.visible(14.0), Expression::Neutral);
    }

    #[test]
    fn hovering_holds_expression_and_line_until_three_seconds_after_leaving() {
        let mut state = Companion::default();
        state.react(10.0, 4);
        let expression = state.visible(10.0);
        let line = state.line(10.0);
        state.hold_while_hovered(true, 12.5);
        state.hold_while_hovered(true, 15.0);
        assert_eq!(state.visible(17.9), expression);
        assert_eq!(state.line(17.9), line);
        state.hold_while_hovered(false, 17.9);
        assert_eq!(state.visible(18.0), Expression::Neutral);
        state.hold_while_hovered(true, 19.0);
        assert_eq!(state.visible(19.0), Expression::Neutral);
    }
}
