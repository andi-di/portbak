//! Saya's static welcome figure and individually refined sidebar expressions.

use crate::app::theme::style::Palette;

use eframe::egui::{self, Color32, Pos2, Rect};

use std::time::Duration;

const REACTION_SECONDS: f64 = 4.0;

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

    fn line(self) -> &'static str {
        match self {
            Self::Neutral => "Hmph. I suppose you can stay.",

            Self::Angry => "I wasn't waiting for you!",

            Self::Bored => "Still here. Still keeping watch.",

            Self::BoredExpectant => "Go on. I'm listening.",

            Self::Celebrating => "Naturally, everything went according to my plan.",

            Self::Curious => "What have you brought me, Human?",

            Self::Cute => "Don't call me cute.",

            Self::Dominant => "Your files are under my protection, Mortal.",

            Self::Embarrassed => "You needn't stare, Human.",

            Self::Energetic => "Shall we get to work?",

            Self::Excited => "You came back! I mean… good timing!",

            Self::Flirty => "You do enjoy my company, Human.",

            Self::Flustered => "Nya?! You startled me!",

            Self::Happy => "It's nice having you around.",

            Self::InLove => "Maybe I missed you. Just a little.",

            Self::Killer => "Nothing gets past me. Nothing.",

            Self::Meek => "Perhaps you could stay a little longer.",

            Self::Pleading => "You will return, will you not?",

            Self::Pouting => "A little attention would not hurt…",

            Self::Praying => "May your files remain safe.",

            Self::Question => "Might I ask you something?",

            Self::Sad => "Do take care of yourself, Human.",

            Self::Seductive => "I seem to have your attention.",

            Self::Shy => "You noticed me…?",

            Self::Sleepy => "Five more minutes… nya…",

            Self::Smug => "Of course I noticed. I notice everything.",

            Self::Waiting => "I was merely passing the time.",
        }
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
}

impl Companion {
    fn visible(&self, time: f64) -> Expression {
        if time < self.until {
            self.expression
        } else {
            Expression::Neutral
        }
    }

    fn react(&mut self, time: f64, random: u128) {
        self.expression = self.expression.next(random);

        self.until = time + REACTION_SECONDS;
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

    paint(
        ui,
        rect,
        "head",
        include_bytes!("../../../assets/themes/midnight-neko/saya-head.png"),
    );

    response.on_hover_text("Kurobane Saya")
}

pub fn reserved_height(width: f32) -> f32 {
    width.min(340.0) + 48.0
}

pub fn show(ui: &mut egui::Ui) {
    let time = ui.input(|input| input.time);

    let id = egui::Id::new("saya_sidebar_companion");

    let mut state: Companion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());

    let width = ui.available_width().min(340.0);

    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, width), egui::Sense::click());

    if response.clicked() {
        state.react(time, uuid::Uuid::new_v4().as_u128());
    }

    let expression = state.visible(time);

    paint(ui, rect, &format!("{expression:?}"), expression.bytes());

    ui.add_space(4.0);

    ui.label(
        egui::RichText::new(expression.line())
            .small()
            .color(Palette::of(ui).dim),
    );

    if time < state.until {
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
}
