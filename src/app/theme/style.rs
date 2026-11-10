//! Skin machinery: everything visual comes from the active `Skin`, so new looks are data only.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::skins;
use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke,
    TextStyle,
};
use std::sync::Arc;

/// A complete look: colors for dark and light mode, fonts, sizes, geometry and decorations.
pub struct Skin {
    /// Stable key saved in the configuration.
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub dark: Palette,
    pub light: Palette,
    pub fonts: Fonts,
    pub typography: Typography,
    pub metrics: Metrics,
    pub effects: Effects,
    pub finish: Finish,
    pub wording: Wording,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    /// Pixel sprites, beveled buttons, square boxes and lamps.
    Pixel,
    /// Flat rounded controls and simple line icons.
    Modern,
    /// Glossy icons, gel buttons, glass tiles and orb lamps.
    Glass,
    /// Matte gothic panels, silver ornaments and crimson accents.
    Gothic,
    /// iOS-style liquid glass lenses, capsule controls, switches and line symbols.
    Liquid,
}

/// Flavor text in the display face; plain instructions stay the same in every skin.
pub struct Wording {
    /// Icon beside the wordmark in the header.
    pub mascot: Sprite,
    pub tagline: &'static str,
    pub welcome_title: &'static str,
    pub welcome_prompt: &'static str,
    pub ready: &'static str,
}

/// Colors by role, so widgets never hard-code a color.
pub struct Palette {
    pub background: Color32,
    pub grid: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub sunken: Color32,
    pub outline: Color32,
    pub highlight: Color32,
    pub shade: Color32,
    pub ink: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_ink: Color32,
    pub hot: Color32,
    pub gold: Color32,
    pub good: Color32,
    pub bad: Color32,
    pub metal: Color32,
    pub paper: Color32,
    pub scanline: Color32,
    /// Top of the wallpaper gradient, which fades down into `background`.
    pub sky: Color32,
}

/// Font files placed in front of egui's defaults, which stay as glyph fallbacks.
pub struct Fonts {
    /// Headings, readouts and titles; mapped to the monospace family.
    pub display: Option<&'static [u8]>,
    /// Labels, buttons and inputs; mapped to the proportional family.
    pub body: Option<&'static [u8]>,
}

pub struct Typography {
    pub heading: f32,
    pub body: f32,
    pub small: f32,
    /// Section titles, readouts and other small display-face text.
    pub label: f32,
    /// Multiplier for fixed display-face sizes, since faces differ in size at the same em.
    pub display_scale: f32,
    /// Drop shadow offset of display labels as a fraction of their size.
    pub display_drop: f32,
    /// Radius of the halo behind display text in pixels; 0 for none.
    pub display_glow: f32,
    /// Buttons, badges, captions and title bars are set in capitals.
    pub capitals: bool,
    /// Display labels written in capitals are set in title case instead.
    pub title_case: bool,
}

pub struct Metrics {
    /// Outline width of buttons, boxes and panels.
    pub border: f32,
    /// How far raised buttons stand off their shadow.
    pub lift: f32,
    /// Offset of the shadow behind panels and windows.
    pub shadow: i8,
    /// Blur of that shadow; 0 gives a hard diagonal pixel shadow.
    pub shadow_blur: u8,
    /// Corner rounding of panels, cards and windows; small controls use half of it.
    pub radius: u8,
}

pub struct Effects {
    pub scanlines: bool,
    pub grid: bool,
    pub stripes: bool,
    /// A sky, aurora and hills painted behind every panel.
    pub wallpaper: bool,
    /// Bubbles drifting up behind the panels; keeps the window repainting while visible.
    pub bubbles: bool,
    /// Poppable red hearts and a sparse sparkling pointer trail.
    pub hearts: bool,
    /// Opacity of panels, sections and cards; below 1 needs the wallpaper to show through.
    pub opacity: f32,
    pub title_bar: TitleBar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TitleBar {
    /// A solid colored band with stripes and three window lights.
    Striped,
    /// Plain heading on the card surface.
    Modern,
    /// A tinted glass band with a glossy reflection and a glowing title.
    Glass,
    /// Matte gothic panels, silver ornaments and crimson accents.
    Gothic,
    /// A bold heading beside an accent symbol, straight on the glass.
    Liquid,
}

impl Skin {
    /// The skin installed on `ctx`, or the default skin before installation (e.g. in tests).
    pub fn of(ctx: &egui::Context) -> &'static Self {
        installed(ctx).unwrap_or_else(skins::default)
    }

    pub fn palette(&self, dark: bool) -> &Palette {
        if dark {
            &self.dark
        } else {
            &self.light
        }
    }
}

impl Palette {
    pub fn of(ui: &egui::Ui) -> &'static Self {
        Self::of_ctx(ui.ctx())
    }

    pub fn of_ctx(ctx: &egui::Context) -> &'static Self {
        Skin::of(ctx).palette(ctx.global_style().visuals.dark_mode)
    }
}

/// The skin's display face at `size`.
pub fn display_font(size: f32) -> FontId {
    FontId::monospace(size)
}

/// The display face at the skin's label size, for titles and readouts.
pub fn label_font(ctx: &egui::Context) -> FontId {
    face_font(ctx, Skin::of(ctx).typography.label)
}

pub fn face_font(ctx: &egui::Context, size: f32) -> FontId {
    if Skin::of(ctx).finish == Finish::Modern {
        FontId::proportional(size)
    } else {
        display_font(size)
    }
}

/// Matte looks keep decorative details neutral; status colors retain their meaning.
pub fn decorative_color(ctx: &egui::Context, original: Color32) -> Color32 {
    if matches!(Skin::of(ctx).finish, Finish::Modern | Finish::Liquid) {
        Palette::of_ctx(ctx).ink
    } else if Skin::of(ctx).finish == Finish::Gothic {
        Palette::of_ctx(ctx).metal
    } else {
        original
    }
}

pub fn metrics(ctx: &egui::Context) -> &'static Metrics {
    &Skin::of(ctx).metrics
}

/// `color` at the skin's panel opacity, for panels, sections and cards.
pub fn glazed(ctx: &egui::Context, color: Color32) -> Color32 {
    color.gamma_multiply(Skin::of(ctx).effects.opacity)
}

/// Display `text` as the skin sets it: words in capitals become title case where it asks.
pub fn titled(ctx: &egui::Context, text: &str) -> String {
    if !Skin::of(ctx).typography.title_case || text.chars().any(char::is_lowercase) {
        return text.to_owned();
    }
    text.split(' ')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or_else(String::new, |first| {
                first.to_string() + &letters.as_str().to_lowercase()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `text` in capitals when the skin sets its labels that way.
pub fn cased(ctx: &egui::Context, text: &str) -> String {
    if Skin::of(ctx).typography.capitals {
        text.to_uppercase()
    } else {
        text.to_owned()
    }
}

/// A sunken well around a readout or list; glass skins give it a thin glowing rim and Modern a
/// hairline.
pub fn well(ctx: &egui::Context) -> egui::Frame {
    let palette = Palette::of_ctx(ctx);
    let stroke = match Skin::of(ctx).finish {
        Finish::Glass => Stroke::new(1.0_f32, palette.hot.gamma_multiply(0.8)),
        Finish::Modern | Finish::Gothic => Stroke::new(1.0_f32, palette.outline),
        Finish::Liquid => Stroke::NONE,
        Finish::Pixel => Stroke::new(2.0_f32, palette.outline),
    };
    egui::Frame::new()
        .fill(ctx.global_style().visuals.extreme_bg_color)
        .stroke(stroke)
        .corner_radius(control_radius(ctx))
        .inner_margin(egui::Margin::symmetric(10, 6))
}

/// Corner rounding of panels and cards.
pub fn panel_radius(ctx: &egui::Context) -> CornerRadius {
    CornerRadius::same(metrics(ctx).radius)
}

/// Corner rounding of small controls such as fields and badges.
pub fn control_radius(ctx: &egui::Context) -> CornerRadius {
    CornerRadius::same(metrics(ctx).radius / 2)
}

pub fn installed(ctx: &egui::Context) -> Option<&'static Skin> {
    ctx.data(|data| data.get_temp::<&'static Skin>(skin_id()))
}

pub fn install(ctx: &egui::Context, skin: &'static Skin) {
    ctx.data_mut(|data| data.insert_temp(skin_id(), skin));
    ctx.set_fonts(font_definitions(&skin.fonts));
    ctx.style_mut_of(egui::Theme::Dark, |style| apply(style, skin, &skin.dark));
    ctx.style_mut_of(egui::Theme::Light, |style| apply(style, skin, &skin.light));
}

fn skin_id() -> egui::Id {
    egui::Id::new("portbak_skin")
}

fn font_definitions(fonts: &Fonts) -> FontDefinitions {
    let mut definitions = FontDefinitions::default();
    for (family, bytes) in [
        (FontFamily::Monospace, fonts.display),
        (FontFamily::Proportional, fonts.body),
    ] {
        let Some(bytes) = bytes else {
            continue;
        };
        let name = format!("skin_{family}");
        definitions
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_static(bytes)));
        definitions
            .families
            .entry(family)
            .or_default()
            .insert(0, name);
    }
    definitions
}

fn apply(style: &mut egui::Style, skin: &Skin, palette: &Palette) {
    let sizes = &skin.typography;
    let heading_font = |size| {
        if skin.finish == Finish::Modern {
            FontId::proportional(size)
        } else {
            display_font(size)
        }
    };
    for (text_style, font) in [
        (TextStyle::Heading, heading_font(sizes.heading)),
        (TextStyle::Monospace, heading_font(sizes.label)),
        (TextStyle::Body, FontId::proportional(sizes.body)),
        (TextStyle::Button, FontId::proportional(sizes.body)),
        (TextStyle::Small, FontId::proportional(sizes.small)),
    ] {
        style.text_styles.insert(text_style, font);
    }
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(10.0, 7.0);
    spacing.button_padding = egui::vec2(12.0, 5.0);
    spacing.interact_size = egui::vec2(44.0, 28.0);
    spacing.icon_width = 18.0;
    spacing.icon_width_inner = 10.0;
    spacing.icon_spacing = 8.0;
    spacing.combo_width = 150.0;
    spacing.window_margin = egui::Margin::same(16);
    spacing.menu_margin = egui::Margin::same(6);
    spacing.scroll = egui::style::ScrollStyle::solid();
    spacing.scroll.bar_width = 12.0;
    spacing.scroll.handle_min_length = 24.0;
    style.interaction.selectable_labels = false;
    apply_visuals(&mut style.visuals, skin, palette);
    if matches!(skin.finish, Finish::Modern | Finish::Gothic) {
        // Matte skins use hairline window borders and their accent for selection.
        style.visuals.window_stroke = Stroke::new(1.0_f32, palette.outline);
        style.visuals.selection.bg_fill = palette.accent;
    }
    if skin.finish == Finish::Liquid {
        style.spacing.scroll = egui::style::ScrollStyle::floating();
        style.spacing.scroll.handle_min_length = 24.0;
        liquid_visuals(&mut style.visuals, palette);
    }
}

/// iOS fills: borderless translucent grey fields, a hairline glass rim on popups and accent
/// selection; the glass itself is painted by the widgets.
fn liquid_visuals(visuals: &mut egui::Visuals, palette: &Palette) {
    let fill = |alpha| Color32::from_rgba_unmultiplied(0x78, 0x78, 0x80, alpha);
    let rim = if visuals.dark_mode {
        Color32::from_white_alpha(40)
    } else {
        Color32::from_white_alpha(150)
    };
    visuals.extreme_bg_color = fill(if visuals.dark_mode { 70 } else { 34 });
    visuals.code_bg_color = visuals.extreme_bg_color;
    visuals.faint_bg_color = fill(20);
    visuals.window_stroke = Stroke::new(1.0_f32, rim);
    visuals.window_shadow.color = Color32::from_black_alpha(40);
    visuals.popup_shadow.color = Color32::from_black_alpha(30);
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.45);
    visuals.selection.stroke = Stroke::new(1.0_f32, palette.accent);
    visuals.handle_shape = egui::style::HandleShape::Circle;
    let widgets = &mut visuals.widgets;
    for (state, alpha) in [
        (&mut widgets.inactive, 40),
        (&mut widgets.hovered, 64),
        (&mut widgets.active, 90),
        (&mut widgets.open, 64),
    ] {
        state.bg_fill = fill(alpha);
        state.weak_bg_fill = fill(alpha);
        state.bg_stroke = Stroke::NONE;
    }
    widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, fill(60));
    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, palette.accent.gamma_multiply(0.6));
    widgets.active.fg_stroke = Stroke::new(1.0_f32, palette.ink);
}

fn apply_visuals(visuals: &mut egui::Visuals, skin: &Skin, palette: &Palette) {
    let metrics = &skin.metrics;
    let outline = Stroke::new(metrics.border, palette.outline);
    let control_radius = CornerRadius::same(metrics.radius / 2);
    visuals.override_text_color = Some(palette.ink);
    // The open background is clearer than the panels on it, so the scenery stays the brightest layer.
    visuals.panel_fill = palette
        .background
        .gamma_multiply(skin.effects.opacity * skin.effects.opacity);
    // Inputs and popups stay denser than panels so their text reads over the scenery.
    let field = (skin.effects.opacity + 1.0) / 2.0;
    visuals.window_fill = palette.surface.gamma_multiply(field);
    visuals.window_stroke = if matches!(skin.finish, Finish::Glass | Finish::Gothic) {
        Stroke::new(
            metrics.border,
            if skin.finish == Finish::Gothic {
                palette.outline
            } else {
                palette.hot
            },
        )
    } else {
        Stroke::new(metrics.border + 1.0, palette.outline)
    };
    visuals.window_corner_radius = CornerRadius::same(metrics.radius);
    visuals.menu_corner_radius = control_radius;
    visuals.window_shadow = drop_shadow(metrics, palette, metrics.shadow + 3);
    visuals.popup_shadow = drop_shadow(metrics, palette, metrics.shadow - 1);
    visuals.faint_bg_color = palette.raised.gamma_multiply(field);
    visuals.extreme_bg_color = palette.sunken.gamma_multiply(field);
    visuals.code_bg_color = palette.sunken.gamma_multiply(field);
    visuals.warn_fg_color = palette.gold;
    visuals.error_fg_color = palette.bad;
    visuals.hyperlink_color = if skin.finish == Finish::Gothic {
        palette.hot
    } else {
        palette.accent
    };
    visuals.selection.bg_fill = palette.hot;
    visuals.selection.stroke = Stroke::new(metrics.border, palette.paper);
    visuals.text_cursor.stroke = Stroke::new(metrics.border + 1.0, palette.accent);
    visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.6 };
    visuals.striped = false;
    visuals.slider_trailing_fill = true;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    let widgets = &mut visuals.widgets;
    let states = [
        (&mut widgets.noninteractive, palette.surface, palette.ink),
        (&mut widgets.inactive, palette.raised, palette.ink),
        (&mut widgets.hovered, palette.highlight, palette.ink),
        (&mut widgets.active, palette.accent, palette.accent_ink),
        (&mut widgets.open, palette.highlight, palette.ink),
    ];
    for (state, fill, ink) in states {
        let fill = fill.gamma_multiply(field);
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = outline;
        state.fg_stroke = Stroke::new(metrics.border, ink);
        state.corner_radius = control_radius;
        state.expansion = 0.0;
    }
    widgets.noninteractive.bg_stroke = Stroke::new(metrics.border, palette.shade);
    widgets.hovered.bg_stroke = Stroke::new(metrics.border, palette.accent);
    widgets.open.bg_stroke = Stroke::new(metrics.border, palette.accent);
}

/// The skin's drop shadow: hard and diagonal without blur, soft and falling straight down with it.
pub fn drop_shadow(metrics: &Metrics, palette: &Palette, offset: i8) -> Shadow {
    if metrics.shadow_blur == 0 {
        return Shadow {
            offset: [offset, offset],
            blur: 0,
            spread: 0,
            color: palette.outline,
        };
    }
    Shadow {
        offset: [0, offset],
        blur: metrics.shadow_blur,
        spread: 0,
        color: if metrics.lift == 0.0 {
            Color32::from_black_alpha(22)
        } else {
            palette.outline.gamma_multiply(0.7)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_skin_is_found_and_default_applies_before_install() {
        let context = egui::Context::default();
        assert!(installed(&context).is_none());
        assert!(std::ptr::eq(Skin::of(&context), skins::default()));
        for skin in skins::ALL.iter().chain(skins::ALL.iter().rev()) {
            install(&context, skin);
            assert!(std::ptr::eq(Skin::of(&context), *skin));
            for theme in [egui::Theme::Light, egui::Theme::Dark] {
                let style = context.style_of(theme);
                let modern = skin.finish == Finish::Modern;
                assert_eq!(
                    style.visuals.widgets.inactive.corner_radius,
                    CornerRadius::same(skin.metrics.radius / 2)
                );
                assert_eq!(
                    style.text_styles[&TextStyle::Heading].family,
                    if modern {
                        FontFamily::Proportional
                    } else {
                        FontFamily::Monospace
                    }
                );
            }
        }
    }
}
