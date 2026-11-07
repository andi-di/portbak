//! Skin machinery: everything visual comes from the active `Skin`, so new looks are data only.

use super::skins;
use super::sprites::Sprite;
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
    /// Vector glyphs, forged plates, spiked sockets and diamond lamps.
    Engraved,
}

/// Flavor text in the display face; plain instructions stay the same in every skin.
pub struct Wording {
    /// Icon beside the wordmark in the header.
    pub mascot: Sprite,
    pub tagline: &'static str,
    pub welcome_title: &'static str,
    pub welcome_prompt: &'static str,
    pub ready: &'static str,
    pub busy: &'static str,
    /// Blinks after the ready text; empty for none.
    pub cursor: &'static str,
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
    /// Color the window edges sink into when the vignette effect is on.
    pub vignette: Color32,
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
}

pub struct Effects {
    pub scanlines: bool,
    pub grid: bool,
    pub stripes: bool,
    /// Sparks drifting up behind the content; keeps the window repainting while visible.
    pub embers: bool,
    pub vignette: bool,
    pub title_bar: TitleBar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TitleBar {
    /// A solid colored band with stripes and three window lights.
    Striped,
    /// Plain heading on the card surface.
    Modern,
    /// A dark band with a tinted icon, a fading colored rule and diamond ornaments.
    Engraved,
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
        Skin::of(ctx).palette(ctx.style().visuals.dark_mode)
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

pub fn radius(ctx: &egui::Context, modern: u8) -> CornerRadius {
    CornerRadius::same(if Skin::of(ctx).finish == Finish::Modern {
        modern
    } else {
        0
    })
}

pub fn metrics(ctx: &egui::Context) -> &'static Metrics {
    &Skin::of(ctx).metrics
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
    apply_visuals(&mut style.visuals, &skin.metrics, palette);
    if skin.finish == Finish::Modern {
        style.visuals.window_corner_radius = CornerRadius::same(10);
        style.visuals.menu_corner_radius = CornerRadius::same(6);
        style.visuals.window_stroke = Stroke::new(1.0_f32, palette.outline);
        style.visuals.selection.bg_fill = palette.accent;
        for state in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            state.corner_radius = CornerRadius::same(6);
        }
    }
}

fn apply_visuals(visuals: &mut egui::Visuals, metrics: &Metrics, palette: &Palette) {
    let outline = Stroke::new(metrics.border, palette.outline);
    visuals.override_text_color = Some(palette.ink);
    visuals.panel_fill = palette.background;
    visuals.window_fill = palette.surface;
    visuals.window_stroke = Stroke::new(metrics.border + 1.0, palette.outline);
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = drop_shadow(metrics, palette, metrics.shadow + 3);
    visuals.popup_shadow = drop_shadow(metrics, palette, metrics.shadow - 1);
    visuals.faint_bg_color = palette.raised;
    visuals.extreme_bg_color = palette.sunken;
    visuals.code_bg_color = palette.sunken;
    visuals.warn_fg_color = palette.gold;
    visuals.error_fg_color = palette.bad;
    visuals.hyperlink_color = palette.accent;
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
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = outline;
        state.fg_stroke = Stroke::new(metrics.border, ink);
        state.corner_radius = CornerRadius::ZERO;
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
                    CornerRadius::same(if modern { 6 } else { 0 })
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
