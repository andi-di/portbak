//! Approved illustrated portraits; expression changes never paint over the character's eyes.

use crate::app::rendering::gothic;
use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect};
use std::{sync::Arc, time::Duration};

const REACTION_SECONDS: f64 = 4.0;
const PORTRAIT_SIZE: [u32; 2] = [384, 448];
const EYE_LINE: f32 = 244.0;
const EYE_DISTANCE: f32 = 96.0;
const BACKDROP: image::Rgba<u8> = image::Rgba([0x21, 0x19, 0x1d, 255]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Expression {
    #[default]
    Neutral,
    Angry,
    Pout,
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
    const REACTIONS: [Self; 13] = [
        Self::Angry,
        Self::Pout,
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

    fn portrait(self) -> (Source, Rect) {
        let (source, bounds) = match self {
            Self::Neutral => (Source::Expressions, [270.0, 28.0, 616.0, 420.0]),
            Self::Angry => (Source::Expressions, [636.0, 78.0, 898.0, 317.0]),
            Self::Pout => (Source::Expressions, [936.0, 78.0, 1200.0, 317.0]),
            Self::Excited => (Source::Expressions, [1230.0, 78.0, 1488.0, 317.0]),
            Self::Happy => (Source::Expressions, [636.0, 366.0, 898.0, 607.0]),
            Self::Surprised => (Source::Expressions, [936.0, 366.0, 1200.0, 607.0]),
            Self::Superior => (Source::Expressions, [1230.0, 366.0, 1488.0, 607.0]),
            Self::InLove => (Source::Expressions, [636.0, 660.0, 898.0, 931.0]),
            Self::ColdStare => (Source::Expressions, [936.0, 660.0, 1200.0, 931.0]),
            Self::Sleepy => (Source::Expressions, [1230.0, 660.0, 1488.0, 931.0]),
            Self::Bored => (Source::Poses, [170.0, 90.0, 740.0, 735.0]),
            Self::Leaning => (Source::Poses, [803.0, 92.0, 1440.0, 927.0]),
            Self::Yandere => (Source::Yandere, [35.0, 60.0, 1162.0, 1286.0]),
            Self::Shy => (Source::Shy, [35.0, 65.0, 1162.0, 1286.0]),
        };
        (
            source,
            Rect::from_min_max(
                egui::pos2(bounds[0], bounds[1]),
                egui::pos2(bounds[2], bounds[3]),
            ),
        )
    }

    /// Iris centers measured on the approved source illustrations.
    fn eyes(self) -> [Pos2; 2] {
        let points = match self {
            Self::Neutral => [369.0, 213.0, 444.0, 196.0],
            Self::Angry => [755.0, 212.9, 823.6, 207.5],
            Self::Pout => [1052.6, 206.4, 1117.9, 194.1],
            Self::Excited => [1348.1, 197.2, 1412.4, 183.9],
            Self::Happy => [763.3, 487.6, 830.6, 471.2],
            Self::Surprised => [1055.5, 484.9, 1120.2, 474.5],
            Self::Superior => [1363.5, 473.9, 1431.4, 462.0],
            Self::InLove => [755.8, 780.1, 823.4, 758.9],
            Self::ColdStare => [1045.5, 797.9, 1117.9, 791.9],
            Self::Sleepy => [1349.4, 790.7, 1425.0, 773.5],
            Self::Bored => [398.9, 344.4, 513.7, 320.3],
            Self::Leaning => [1183.4, 376.1, 1288.3, 341.3],
            Self::Yandere => [528.9, 499.0, 726.5, 431.5],
            Self::Shy => [545.1, 561.9, 741.7, 523.6],
        };
        [
            egui::pos2(points[0], points[1]),
            egui::pos2(points[2], points[3]),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
enum Source {
    Expressions,
    Poses,
    Yandere,
    Shy,
}

impl Source {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Expressions => {
                include_bytes!("../../../assets/themes/midnight-neko/expressions.png")
            }
            Self::Poses => include_bytes!("../../../assets/themes/midnight-neko/poses.png"),
            Self::Yandere => include_bytes!("../../../assets/themes/midnight-neko/yandere.png"),
            Self::Shy => include_bytes!("../../../assets/themes/midnight-neko/shy.png"),
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
            Ok(if icon {
                image.crop_imm(31, 800, 158, 163).to_rgba8()
            } else {
                image.to_rgba8()
            })
        },
    )
}

fn normalized_portrait(image: &image::DynamicImage, expression: Expression) -> image::RgbaImage {
    let (_, bounds) = expression.portrait();
    let [left_eye, right_eye] = expression.eyes();
    let center = left_eye.lerp(right_eye, 0.5);
    let scale = EYE_DISTANCE / left_eye.distance(right_eye);
    let crop = image
        .crop_imm(
            bounds.left() as u32,
            bounds.top() as u32,
            bounds.width() as u32,
            bounds.height() as u32,
        )
        .to_rgba8();
    let resized = image::imageops::resize(
        &crop,
        (crop.width() as f32 * scale).round() as u32,
        (crop.height() as f32 * scale).round() as u32,
        image::imageops::FilterType::Lanczos3,
    );
    let mut portrait = image::RgbaImage::from_pixel(PORTRAIT_SIZE[0], PORTRAIT_SIZE[1], BACKDROP);
    let x = (PORTRAIT_SIZE[0] as f32 / 2.0 - (center.x - bounds.left()) * scale).round() as i64;
    let y = (EYE_LINE - (center.y - bounds.top()) * scale).round() as i64;
    image::imageops::overlay(&mut portrait, &resized, x, y);
    portrait
}

fn portrait_texture(
    ctx: &egui::Context,
    expression: Expression,
) -> Result<egui::TextureHandle, String> {
    cached_texture(
        ctx,
        egui::Id::new(format!("midnight_neko_portrait_{expression:?}")),
        egui::TextureOptions::LINEAR,
        || {
            let image = source_image(ctx, expression.portrait().0)?;
            Ok(normalized_portrait(&image, expression))
        },
    )
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect) {
    if let Ok(texture) = texture(painter.ctx(), Source::Expressions, true) {
        painter.image(
            texture.id(),
            rect,
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
    ui.painter()
        .rect_filled(rect, 2, Color32::from_rgb(0x21, 0x19, 0x1d));
    if full_figure && expression == Expression::Neutral {
        paint_full_figure(ui.painter(), rect);
    } else {
        if let Ok(texture) = portrait_texture(ui.ctx(), expression) {
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
    gothic::corners(ui.painter(), rect.shrink(1.0), palette.metal);
    response.on_hover_text("Click Midnight Neko for another expression");
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

fn paint_full_figure(painter: &egui::Painter, rect: Rect) {
    if let Ok(texture) = texture(painter.ctx(), Source::Expressions, false) {
        let pixels = Rect::from_min_max(egui::pos2(2.0, 2.0), egui::pos2(622.0, 786.0));
        let size = texture.size_vec2();
        let uv = Rect::from_min_max(
            egui::pos2(pixels.min.x / size.x, pixels.min.y / size.y),
            egui::pos2(pixels.max.x / size.x, pixels.max.y / size.y),
        );
        let art_size =
            pixels.size() * (rect.width() / pixels.width()).min(rect.height() / pixels.height());
        painter.image(
            texture.id(),
            Rect::from_center_size(rect.center(), art_size),
            uv,
            Color32::WHITE,
        );
    }
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
            let next: Vec<_> = (0..13).map(|seed| current.next(seed)).collect();
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
    fn every_portrait_is_inside_its_embedded_atlas() {
        for expression in Expression::REACTIONS
            .into_iter()
            .chain([Expression::Neutral])
        {
            let (source, pixels) = expression.portrait();
            let image = image::load_from_memory(source.bytes()).unwrap();
            assert!(pixels.is_positive());
            assert!(pixels.min.x >= 0.0 && pixels.min.y >= 0.0);
            assert!(pixels.max.x <= image.width() as f32 && pixels.max.y <= image.height() as f32);
        }
    }

    #[test]
    fn every_jobs_portrait_has_the_same_pixel_resolution() {
        let ctx = egui::Context::default();
        for expression in Expression::REACTIONS
            .into_iter()
            .chain([Expression::Neutral])
        {
            let texture = portrait_texture(&ctx, expression).unwrap();
            assert_eq!(texture.size(), [384, 448]);
        }
    }
}
