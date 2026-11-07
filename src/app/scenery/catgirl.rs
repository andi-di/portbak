//! Approved illustrated portraits; expression changes never paint over the character's eyes.

use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect};
use std::{sync::Arc, time::Duration};

const REACTION_SECONDS: f64 = 4.0;
const PORTRAIT_SIZE: [u32; 2] = [384, 448];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Expression {
    #[default]
    Neutral,
    Angry,
    Pout,
    CatSmug,
    Excited,
    Happy,
    Surprised,
    Superior,
    InLove,
    ColdStare,
    Sleepy,
    Bored,
    Leaning,
    Yandere,
    Shy,
}

impl Expression {
    const REACTIONS: [Self; 14] = [
        Self::Angry,
        Self::Pout,
        Self::CatSmug,
        Self::Excited,
        Self::Happy,
        Self::Surprised,
        Self::Superior,
        Self::InLove,
        Self::ColdStare,
        Self::Sleepy,
        Self::Bored,
        Self::Leaning,
        Self::Yandere,
        Self::Shy,
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
            Self::CatSmug => "You know you like having me around.",
            Self::Neutral => "Hmph. I suppose you can stay.",
            Self::Angry => "I wasn't waiting for you!",
            Self::Pout => "A little attention wouldn't hurt…",
            Self::Excited => "You came back! I mean… good timing!",
            Self::Happy => "It's nice having you around.",
            Self::Surprised => "Nya?! You startled me!",
            Self::Superior => "Of course I noticed. I notice everything.",
            Self::InLove => "Maybe I missed you. Just a little.",
            Self::ColdStare => "I take keeping watch very seriously.",
            Self::Sleepy => "Five more minutes… nya…",
            Self::Bored => "Still here. Still keeping watch.",
            Self::Leaning => "Go on. I'm listening.",
            Self::Yandere => "Nothing gets past me. Nothing.",
            Self::Shy => "You noticed me…?",
        }
    }

    fn cell(self) -> u32 {
        match self {
            Self::Neutral => 0,
            Self::Angry => 1,
            Self::Pout => 14,
            Self::CatSmug => 2,
            Self::Excited => 3,
            Self::Happy => 4,
            Self::Surprised => 5,
            Self::Superior => 6,
            Self::InLove => 7,
            Self::ColdStare => 8,
            Self::Sleepy => 9,
            Self::Bored => 10,
            Self::Leaning => 11,
            Self::Yandere => 12,
            Self::Shy => 13,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Source {
    Expressions,
    Portraits,
    FullBody,
}

impl Source {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Expressions => {
                include_bytes!("../../../assets/themes/midnight-neko/pixel-face.png")
            }
            Self::Portraits => {
                include_bytes!("../../../assets/themes/midnight-neko/portraits.png")
            }
            Self::FullBody => {
                include_bytes!("../../../assets/themes/midnight-neko/upper-body.png")
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

fn source_image(ctx: &egui::Context, source: Source) -> Result<Arc<image::DynamicImage>, String> {
    let id = egui::Id::new(format!("midnight_neko_source_{source:?}"));
    if let Some(cached) = ctx.data_mut(|data| data.get_temp(id)) {
        return cached;
    }
    let loaded = image::load_from_memory(source.bytes())
        .map(Arc::new)
        .map_err(|error| format!("Could not load Midnight Neko artwork: {error}"));
    if let Err(error) = &loaded {
        eprintln!("{error}");
    }
    ctx.data_mut(|data| data.insert_temp(id, loaded.clone()));
    loaded
}

fn cached_texture(
    ctx: &egui::Context,
    id: egui::Id,
    options: egui::TextureOptions,
    pixels: impl FnOnce() -> Result<image::RgbaImage, String>,
) -> Result<egui::TextureHandle, String> {
    if let Some(cached) = ctx.data_mut(|data| data.get_temp(id)) {
        return cached;
    }
    let result = pixels().map(|rgba| {
        ctx.load_texture(
            format!("Midnight Neko {id:?}"),
            egui::ColorImage::from_rgba_unmultiplied(
                [rgba.width() as usize, rgba.height() as usize],
                rgba.as_raw(),
            ),
            options,
        )
    });
    ctx.data_mut(|data| data.insert_temp(id, result.clone()));
    result
}

fn texture(ctx: &egui::Context, source: Source, icon: bool) -> Result<egui::TextureHandle, String> {
    cached_texture(
        ctx,
        egui::Id::new(format!("midnight_neko_{source:?}_{icon}")),
        if icon {
            egui::TextureOptions::NEAREST
        } else {
            egui::TextureOptions::LINEAR
        },
        || {
            let image = source_image(ctx, source)?;
            Ok(image.to_rgba8())
        },
    )
}

fn portrait_texture(
    ctx: &egui::Context,
    expression: Expression,
    full_body: bool,
) -> Result<egui::TextureHandle, String> {
    cached_texture(
        ctx,
        egui::Id::new(format!("midnight_neko_v2_{expression:?}_{full_body}")),
        egui::TextureOptions::LINEAR,
        || {
            let source = if full_body {
                Source::FullBody
            } else {
                Source::Portraits
            };
            let image = source_image(ctx, source)?;
            let cell = expression.cell();
            let width = image.width() / 4;
            let height = image.height() / 4;
            let crop = image
                .crop_imm(cell % 4 * width, cell / 4 * height, width, height)
                .to_rgba8();
            Ok(crop)
        },
    )
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect) {
    if let Ok(texture) = texture(painter.ctx(), Source::Expressions, true) {
        let size = texture.size_vec2();
        let fitted = size * (rect.width() / size.x).min(rect.height() / size.y);
        painter.image(
            texture.id(),
            Rect::from_center_size(rect.center(), fitted),
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

pub fn reserved_height(width: f32) -> f32 {
    width.min(340.0) * PORTRAIT_SIZE[1] as f32 / PORTRAIT_SIZE[0] as f32 + 48.0
}

pub fn show(ui: &mut egui::Ui) {
    show_companion(ui, false);
}

pub fn show_welcome(ui: &mut egui::Ui) {
    show_companion(ui, true);
}

fn show_companion(ui: &mut egui::Ui, full_figure: bool) {
    let time = ui.input(|input| input.time);
    let id = egui::Id::new("midnight_neko_companion");
    let mut state: Companion = ui
        .ctx()
        .data_mut(|data| data.get_temp(id).unwrap_or_default());
    let width = ui.available_width().min(340.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, reserved_height(width) - 48.0),
        egui::Sense::click(),
    );
    if response.clicked() {
        state.react(time, uuid::Uuid::new_v4().as_u128());
    }
    let expression = state.visible(time);
    let palette = Palette::of(ui);
    if let Ok(texture) = portrait_texture(ui.ctx(), expression, full_figure) {
        let size = texture.size_vec2();
        let fitted = size * (rect.width() / size.x).min(rect.height() / size.y);
        ui.painter().image(
            texture.id(),
            Rect::from_center_size(rect.center(), fitted),
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(expression.line())
            .small()
            .color(palette.dim),
    );
    if time < state.until {
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f64(state.until - time));
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reaction_is_reachable_without_immediate_repeats() {
        for current in Expression::REACTIONS
            .into_iter()
            .chain([Expression::Neutral])
        {
            let next: Vec<_> = (0..14).map(|seed| current.next(seed)).collect();
            assert!(!next.contains(&current));
            assert!(!next.contains(&Expression::Neutral));
            assert!(Expression::REACTIONS
                .iter()
                .all(|value| *value == current || next.contains(value)));
        }
    }

    #[test]
    fn starts_neutral_and_returns_to_neutral_after_a_reaction() {
        let mut state = Companion::default();
        assert_eq!(state.visible(0.0), Expression::Neutral);
        state.react(10.0, 4);
        assert_ne!(state.visible(10.1), Expression::Neutral);
        assert_eq!(state.visible(14.0), Expression::Neutral);
    }

    #[test]
    fn every_jobs_portrait_has_the_same_pixel_resolution() {
        let ctx = egui::Context::default();
        for expression in Expression::REACTIONS
            .into_iter()
            .chain([Expression::Neutral])
        {
            let texture = portrait_texture(&ctx, expression, false).unwrap();
            assert!(texture.size()[0] > 0 && texture.size()[1] > 0);
        }
    }
}
