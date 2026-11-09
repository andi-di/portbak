//! Saya's static welcome figure and individually refined sidebar expressions.

mod dialogue;

use crate::app::theme::style::Palette;

use eframe::egui::{self, Color32, Pos2, Rect};

use std::time::Duration;

const REACTION_SECONDS: f64 = 3.0;

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
        self.reaction_priority = 0;
        self.expression = expression;
        self.until = time + REACTION_SECONDS;
        self.select_line(expression, random.rotate_left(64));
        self.schedule(time, random);
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

    paint_head(ui, rect);
    response.on_hover_text("Kurobane Saya")
}

pub fn paint_head(ui: &egui::Ui, rect: egui::Rect) {
    paint(
        ui,
        rect,
        "head",
        include_bytes!("../../../assets/themes/midnight-neko/saya-head.png"),
    );
}

pub fn reserved_height(width: f32) -> f32 {
    width.min(340.0) + 48.0
}

pub fn show(ui: &mut egui::Ui) {
    let time = ui.input(|input| input.time);

    let id = state_id();

    let mut state: Companion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());

    let width = ui
        .available_width()
        .min(340.0)
        .min((ui.available_height() - 48.0).max(24.0));

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, width), egui::Sense::click());

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

    paint(ui, rect, &format!("{expression:?}"), expression.bytes());

    ui.add_space(4.0);

    ui.label(
        egui::RichText::new(state.line(time))
            .small()
            .color(Palette::of(ui).dim),
    );

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
