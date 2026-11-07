//! Retro widgets and decorations painted from plain rectangles: bevels, hard shadows, LEDs.

use crate::app::rendering::glass::{self, Glaze};
use crate::app::rendering::ornaments;
use crate::app::rendering::sprites::{self, Sprite};
use crate::app::rendering::{forge, gothic};
use crate::app::scenery::{aurora, midnight_neko};
use crate::app::theme::style::{self, label_font, Finish, Palette, Skin, TitleBar};
use eframe::egui::{
    self, Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextStyle, Vec2, Widget,
    WidgetInfo, WidgetType,
};
use std::time::Duration;

const ICON_SCALE: f32 = 2.0;
const MARQUEE_FRAME: Duration = Duration::from_millis(60);

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Plain,
    Accent,
    Good,
    Danger,
}

/// How a button face should look in its current interaction state.
pub struct FaceLook {
    pub fill: Color32,
    pub enabled: bool,
    pub pressed: bool,
    /// Hover glow from 0 (none) to 1 (full).
    pub glow: f32,
    pub focused: bool,
    /// A plain face, which the glass finish renders as clear glass instead of a colored gel.
    pub clear: bool,
}

/// A chunky beveled push button; `selected` renders it latched down like a toggle key.
pub struct PixelButton {
    label: String,
    sprite: Option<Sprite>,
    icon_only: bool,
    tone: Tone,
    selected: bool,
    min_size: Vec2,
}

impl PixelButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            sprite: None,
            icon_only: false,
            tone: Tone::Plain,
            selected: false,
            min_size: Vec2::ZERO,
        }
    }

    pub fn sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
        self
    }

    pub fn icon_only(mut self) -> Self {
        self.icon_only = true;
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = size;
        self
    }

    fn base_fill(&self, palette: &Palette) -> Color32 {
        if self.selected {
            return palette.accent;
        }
        match self.tone {
            Tone::Plain => palette.raised,
            Tone::Accent => palette.accent,
            Tone::Good => palette.good,
            Tone::Danger => palette.bad,
        }
    }
}

impl Widget for PixelButton {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let m = style::metrics(ui.ctx());
        let palette = Palette::of(ui);
        let modern = matches!(Skin::of(ui.ctx()).finish, Finish::Modern | Finish::Gothic);
        let text = if self.icon_only {
            String::new()
        } else {
            style::cased(ui.ctx(), &self.label)
        };
        let font = TextStyle::Button.resolve(ui.style());
        let galley = ui
            .painter()
            .layout_no_wrap(text.clone(), font, Color32::PLACEHOLDER);
        let icon = self.sprite.map_or(Vec2::ZERO, |sprite| {
            sprites::size(ui.ctx(), sprite, ICON_SCALE)
        });
        let gap = if icon.x > 0.0 && !text.is_empty() {
            8.0
        } else {
            0.0
        };
        let content = egui::vec2(icon.x + gap + galley.size().x, icon.y.max(galley.size().y));
        let padding = ui.spacing().button_padding + Vec2::splat(m.border * 2.0);
        let finish = Skin::of(ui.ctx()).finish;
        // Plates and gels press straight down, so they only need room below.
        let margin = if finish == Finish::Pixel {
            Vec2::splat(m.lift)
        } else {
            Vec2::new(0.0, m.lift)
        };
        // `min_size` bounds the whole footprint, so a full-width button cannot widen its panel.
        let face_size = (content + padding * 2.0).max(self.min_size - margin);
        let (rect, response) = ui.allocate_exact_size(face_size + margin, Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, enabled, self.selected, &self.label)
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let sunk = enabled && (self.selected || response.is_pointer_button_down_on());
        let hovered = enabled && response.hovered();
        let glow = ui
            .ctx()
            .animate_bool_with_time(response.id.with("glow"), hovered, 0.15);
        let base = self.base_fill(palette);
        let fill = if modern && self.tone == Tone::Plain && !self.selected {
            if !enabled {
                palette.surface
            } else if hovered {
                palette.highlight
            } else {
                palette.raised
            }
        } else if modern && sunk && enabled {
            mix(base, Color32::BLACK, 0.12)
        } else {
            match (enabled, hovered) {
                (false, _) => palette.surface,
                (true, true) => mix(base, Color32::WHITE, 0.18),
                (true, false) => base,
            }
        };
        let painter = ui.painter();
        let face = if modern {
            let face = Rect::from_min_size(rect.min, face_size);
            if finish == Finish::Gothic {
                gothic::control(painter, face, fill, response.has_focus(), enabled);
            } else {
                crate::app::rendering::modern::control(
                    painter,
                    face,
                    fill,
                    response.has_focus(),
                    enabled,
                );
            }
            face
        } else if finish == Finish::Pixel {
            let offset = if sunk {
                Vec2::splat(m.lift)
            } else {
                Vec2::ZERO
            };
            let face = Rect::from_min_size(rect.min + offset, face_size);
            pixel_face(painter, face, fill, sunk, enabled, response.has_focus());
            face
        } else {
            let press = Vec2::new(0.0, if sunk { 1.0 } else { 0.0 });
            let face = Rect::from_min_size(rect.min + press, face_size);
            let look = FaceLook {
                fill,
                enabled,
                pressed: sunk,
                glow,
                focused: response.has_focus(),
                clear: self.tone == Tone::Plain && !self.selected,
            };
            if finish == Finish::Glass {
                glass::gel_button(painter, face, &look);
            } else {
                forge::plate(painter, face, &look);
            }
            face
        };
        let ink = if modern && self.tone == Tone::Plain && !self.selected {
            if enabled {
                palette.ink
            } else {
                palette.dim
            }
        } else if modern && enabled {
            crate::app::rendering::modern::foreground(fill)
        } else if enabled {
            readable_on(fill, palette)
        } else {
            palette.dim
        };
        let mut cursor = face.center() - content / 2.0;
        if let Some(sprite) = self.sprite {
            let tint = match (enabled, self.tone, self.selected) {
                (false, ..) => palette.dim,
                (true, Tone::Plain, false) if finish == Finish::Gothic => palette.metal,
                (true, Tone::Plain, false) => palette.accent,
                _ => ink,
            };
            let top = cursor.y + (content.y - icon.y) / 2.0;
            let origin = egui::pos2(cursor.x, top);
            sprites::paint_awake(painter, origin, sprite, ICON_SCALE, tint, glow);
            cursor.x += icon.x + gap;
        }
        let top = cursor.y + (content.y - galley.size().y) / 2.0;
        painter.galley(egui::pos2(cursor.x, top).round(), galley, ink);
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

/// A beveled face over a hard shadow, the retro button look.
fn pixel_face(
    painter: &egui::Painter,
    face: Rect,
    fill: Color32,
    sunk: bool,
    enabled: bool,
    focused: bool,
) {
    let m = style::metrics(painter.ctx());
    let palette = Palette::of_ctx(painter.ctx());
    let edge = if enabled {
        palette.outline
    } else {
        palette.shade
    };
    if !sunk {
        painter.rect_filled(face.translate(Vec2::splat(m.lift)), 0.0, edge);
    }
    painter.rect_filled(face, 0.0, fill);
    bevel(painter, face.shrink(m.border), fill, sunk, m.border);
    painter.rect_stroke(face, 0.0, Stroke::new(m.border, edge), StrokeKind::Inside);
    if focused {
        painter.rect_stroke(
            face.expand(2.0),
            0.0,
            Stroke::new(2.0_f32, palette.hot),
            StrokeKind::Outside,
        );
    }
}

/// A check box, square or spiked by skin; the label wraps beside it.
pub struct PixelCheckbox<'a> {
    checked: &'a mut bool,
    label: String,
}

impl<'a> PixelCheckbox<'a> {
    pub fn new(checked: &'a mut bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
        }
    }
}

impl Widget for PixelCheckbox<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let m = style::metrics(ui.ctx());
        let palette = Palette::of(ui);
        let box_side = ui.spacing().icon_width + 4.0;
        let spacing = ui.spacing().icon_spacing;
        let wrap = (ui.available_width() - box_side - spacing).max(40.0);
        let font = TextStyle::Body.resolve(ui.style());
        let galley = ui
            .painter()
            .layout(self.label.clone(), font, Color32::PLACEHOLDER, wrap);
        let size = egui::vec2(
            box_side + spacing + galley.size().x,
            box_side.max(galley.size().y),
        );
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let enabled = ui.is_enabled();
        let checked = *self.checked;
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Checkbox, enabled, checked, &self.label)
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let first_line = galley
            .rows
            .first()
            .map_or(box_side, |row| row.height())
            .max(box_side);
        let square = Rect::from_center_size(
            egui::pos2(rect.left() + box_side / 2.0, rect.top() + first_line / 2.0),
            Vec2::splat(box_side),
        );
        let border = match (enabled, response.hovered()) {
            (false, _) => palette.shade,
            (true, true) => palette.accent,
            (true, false) => palette.outline,
        };
        match Skin::of(ui.ctx()).finish {
            Finish::Pixel => {
                painter.rect_filled(square, 0.0, palette.sunken);
                bevel(
                    painter,
                    square.shrink(m.border),
                    palette.sunken,
                    true,
                    m.border,
                );
                painter.rect_stroke(
                    square,
                    0.0,
                    Stroke::new(m.border, border),
                    StrokeKind::Inside,
                );
            }
            Finish::Modern | Finish::Gothic => {
                let fill = if checked {
                    palette.accent
                } else {
                    palette.sunken
                };
                if Skin::of(ui.ctx()).finish == Finish::Gothic {
                    gothic::control(painter, square, fill, response.has_focus(), enabled);
                } else {
                    crate::app::rendering::modern::control(
                        painter,
                        square,
                        fill,
                        response.has_focus(),
                        enabled,
                    );
                }
            }
            Finish::Engraved => forge::spiked_socket(painter, square, border),
            Finish::Glass => glass::check_socket(painter, square, border),
        }
        if checked {
            let tint = if matches!(Skin::of(ui.ctx()).finish, Finish::Modern | Finish::Gothic)
                && enabled
            {
                palette.accent_ink
            } else if enabled {
                palette.good
            } else {
                palette.dim
            };
            sprites::paint_centered(painter, square.center(), Sprite::Check, ICON_SCALE, tint);
        }
        let ink = if enabled { palette.ink } else { palette.dim };
        let text_top =
            rect.top() + (first_line - galley.rows.first().map_or(0.0, |r| r.height())) / 2.0;
        painter.galley(
            egui::pos2(square.right() + spacing, text_top).round(),
            galley,
            ink,
        );
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

/// Toggles `current` to `value` on click, like `selectable_value`, drawn as a latching key.
pub fn choice<T: PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    value: T,
    label: &str,
    sprite: Option<Sprite>,
) -> Response {
    let mut button = PixelButton::new(label).selected(*current == value);
    if let Some(sprite) = sprite {
        button = button.sprite(sprite);
    }
    let mut response = ui.add(button);
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}

/// Total height of a `PixelButton` with a single text line, including its shadow.
pub fn button_height(ui: &egui::Ui) -> f32 {
    let m = style::metrics(ui.ctx());
    ui.text_style_height(&TextStyle::Button)
        + (ui.spacing().button_padding.y + m.border * 2.0) * 2.0
        + m.lift
}

/// A framed panel with a striped title bar, like a window on an old desktop.
pub fn section<R>(
    ui: &mut egui::Ui,
    title: &str,
    sprite: Sprite,
    band: Color32,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let m = style::metrics(ui.ctx());
    let palette = Palette::of(ui);
    let glassy = Skin::of(ui.ctx()).finish == Finish::Glass;
    let margin = egui::Margin {
        left: 0,
        top: 0,
        right: m.shadow,
        bottom: m.shadow + 6,
    };
    let backing = ui.painter().add(egui::Shape::Noop);
    let mut frame = egui::Frame::new()
        .corner_radius(style::panel_radius(ui.ctx()))
        .outer_margin(margin);
    if !glassy {
        frame = frame
            .fill(style::glazed(ui.ctx(), palette.surface))
            .stroke(Stroke::new(m.border, palette.outline))
            .shadow(style::drop_shadow(m, palette, m.shadow));
    }
    let framed = frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        title_bar(ui, title, sprite, band);
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 7.0;
                ui.set_min_width(ui.available_width());
                contents(ui)
            })
            .inner
    });
    if Skin::of(ui.ctx()).finish == Finish::Gothic {
        gothic::corners(
            ui.painter(),
            (framed.response.rect - egui::epaint::MarginF32::from(margin)).shrink(1.0),
            palette.metal,
        );
    }
    if glassy {
        let rect = framed.response.rect - egui::epaint::MarginF32::from(margin);
        let pane = glass::pane(ui.ctx(), rect, m.radius as f32, &Glaze::clear(ui.ctx()));
        ui.painter().set(backing, pane);
    }
    framed.inner
}

pub fn title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite, band: Color32) {
    match Skin::of(ui.ctx()).effects.title_bar {
        TitleBar::Modern => crate::app::rendering::modern::title_bar(ui, title, sprite),
        TitleBar::Striped => striped_title_bar(ui, title, sprite, band),
        TitleBar::Engraved => ornaments::engraved_title_bar(ui, title, sprite, band),
        TitleBar::Glass => aurora::glass_title_bar(ui, title, sprite, band),
        TitleBar::Gothic => gothic::title_bar(ui, title, sprite),
    }
}

fn striped_title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite, band: Color32) {
    let m = style::metrics(ui.ctx());
    let palette = Palette::of(ui);
    let height = 30.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, band);
    let ink = readable_on(band, palette);
    let rule = Rect::from_min_max(
        egui::pos2(rect.left(), rect.bottom() - m.border),
        rect.right_bottom(),
    );
    painter.rect_filled(rule, 0.0, palette.outline);
    let icon = sprites::size(ui.ctx(), sprite, ICON_SCALE);
    let icon_pos = egui::pos2(rect.left() + 10.0, rect.center().y - icon.y / 2.0 - 1.0);
    sprites::paint(painter, icon_pos, sprite, ICON_SCALE, palette.paper);
    let galley = painter.layout_no_wrap(title.to_uppercase(), label_font(ui.ctx()), ink);
    let text_pos = egui::pos2(
        icon_pos.x + icon.x + 10.0,
        rect.center().y - galley.size().y / 2.0 - 1.0,
    );
    let text_right = text_pos.x + galley.size().x;
    painter.galley(text_pos.round(), galley, ink);
    // Classic striped title bar filling the space up to three window "lights".
    let lights_left = rect.right() - 3.0 * 14.0 - 6.0;
    for row in 0..4 {
        let y = rect.top() + 8.0 + row as f32 * 4.0;
        let stripe = Rect::from_min_max(
            egui::pos2(text_right + 12.0, y),
            egui::pos2(lights_left - 10.0, y + 2.0),
        );
        if stripe.width() > 8.0 {
            painter.rect_filled(stripe, 0.0, mix(band, palette.outline, 0.35));
        }
    }
    for (index, color) in [palette.hot, palette.gold, palette.good]
        .into_iter()
        .enumerate()
    {
        let min = egui::pos2(lights_left + index as f32 * 14.0, rect.center().y - 6.0);
        let light = Rect::from_min_size(min, Vec2::splat(10.0));
        painter.rect_filled(light, 0.0, color);
        painter.rect_stroke(
            light,
            0.0,
            Stroke::new(m.border, palette.outline),
            StrokeKind::Inside,
        );
    }
}

/// Text in the display face with the skin's drop shadow or halo; `size` is scaled by the skin.
pub fn display_label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) -> Response {
    let palette = Palette::of(ui);
    let typography = &Skin::of(ui.ctx()).typography;
    let size = size * typography.display_scale;
    let font = style::face_font(ui.ctx(), size);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), color);
    let drop = (size * typography.display_drop).round();
    let glow = typography.display_glow;
    let (rect, response) = ui.allocate_exact_size(
        galley.size() + Vec2::splat(drop + glow * 2.0),
        Sense::hover(),
    );
    let painter = ui.painter();
    let origin = (rect.min + Vec2::splat(glow)).round();
    if drop > 0.0 {
        let shadow = painter.layout_no_wrap(text.to_owned(), font.clone(), palette.outline);
        painter.galley(origin + Vec2::splat(drop), shadow, palette.outline);
    }
    halo(painter, origin, text, &font, glow);
    painter.galley(origin, galley, color);
    response
}

/// Body text with the skin's halo behind it, like captions glowing on Aero glass.
pub fn glowing_label(ui: &mut egui::Ui, text: &str, color: Color32) -> Response {
    let glow = Skin::of(ui.ctx()).typography.display_glow;
    let font = TextStyle::Body.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), color);
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
    let origin = rect.min.round();
    halo(ui.painter(), origin, text, &font, glow);
    ui.painter().galley(origin, galley, color);
    response
}

/// A soft halo of `radius` pixels around text, lifting it off a busy backdrop: light behind the
/// dark ink of light palettes, dark behind the light ink of dark ones. 0 paints nothing.
pub fn halo(painter: &egui::Painter, origin: Pos2, text: &str, font: &egui::FontId, radius: f32) {
    const RAYS: usize = 8;
    if radius <= 0.0 {
        return;
    }
    let palette = Palette::of_ctx(painter.ctx());
    let glow = if luma(palette.ink) > 140.0 {
        palette.outline
    } else {
        Color32::WHITE
    };
    let galley = painter.layout_no_wrap(text.to_owned(), font.clone(), glow.gamma_multiply(0.2));
    for ring in [radius, radius * 0.5] {
        for ray in 0..RAYS {
            let offset = Vec2::angled(ray as f32 * std::f32::consts::TAU / RAYS as f32) * ring;
            painter.galley(origin + offset, galley.clone(), glow);
        }
    }
}

/// A small framed tag such as QUEUED or PROTECTED.
pub fn badge(ui: &mut egui::Ui, text: &str, fill: Color32) -> Response {
    let m = style::metrics(ui.ctx());
    let palette = Palette::of(ui);
    let ink = if Skin::of(ui.ctx()).finish == Finish::Modern {
        crate::app::rendering::modern::foreground(fill)
    } else {
        readable_on(fill, palette)
    };
    let font = TextStyle::Small.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(style::cased(ui.ctx(), text), font, ink);
    let padding = egui::vec2(6.0, 0.0);
    let (rect, response) = ui.allocate_exact_size(
        galley.size() + padding * 2.0 + Vec2::splat(m.border * 2.0),
        Sense::hover(),
    );
    let painter = ui.painter();
    if Skin::of(ui.ctx()).finish == Finish::Glass {
        glass::pill(painter, rect, fill);
        painter.galley(rect.min + padding + Vec2::splat(m.border), galley, ink);
        return response;
    }
    let radius = style::control_radius(ui.ctx());
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(m.border, palette.outline),
        StrokeKind::Inside,
    );
    painter.galley(rect.min + padding + Vec2::splat(m.border), galley, ink);
    response
}

/// A status lamp with a glint, square or diamond by skin; unlit lamps are drawn dark.
pub fn led(ui: &mut egui::Ui, color: Color32, lit: bool) -> Response {
    let m = style::metrics(ui.ctx());
    let palette = Palette::of(ui);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
    let painter = ui.painter();
    let fill = if lit {
        color
    } else {
        mix(color, palette.outline, 0.7)
    };
    match Skin::of(ui.ctx()).finish {
        Finish::Gothic => {
            gothic::lamp(painter, rect, fill);
            return response;
        }
        Finish::Pixel => {}
        Finish::Modern => {
            painter.circle_filled(rect.center(), 4.0, fill);
            return response;
        }
        Finish::Engraved => {
            ornaments::diamond_lamp(painter, rect, fill, lit);
            return response;
        }
        Finish::Glass => {
            glass::orb(painter, rect.center(), rect.width() / 2.0 - 0.5, fill, lit);
            return response;
        }
    }
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(m.border, palette.outline),
        StrokeKind::Inside,
    );
    if lit {
        let glint = Rect::from_min_size(rect.min + Vec2::splat(4.0), Vec2::splat(3.0));
        painter.rect_filled(glint, 0.0, mix(color, Color32::WHITE, 0.7));
    }
    response
}

/// An indeterminate progress bar: a comet of lit cells sweeping through a sunken track.
pub fn marquee(ui: &mut egui::Ui, width: f32, color: Color32) -> Response {
    let m = style::metrics(ui.ctx());
    if matches!(Skin::of(ui.ctx()).finish, Finish::Modern | Finish::Gothic) {
        return crate::app::rendering::modern::progress(ui, width, color);
    }
    const CELL: f32 = 8.0;
    const GAP: f32 = 2.0;
    const TRAIL: usize = 6;
    let palette = Palette::of(ui);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 18.0), Sense::hover());
    let painter = ui.painter();
    ui.ctx().request_repaint_after(MARQUEE_FRAME);
    if Skin::of(ui.ctx()).finish == Finish::Glass {
        glass::progress(painter, rect, color, ui.input(|input| input.time));
        return response;
    }
    painter.rect_filled(rect, 0.0, palette.sunken);
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(m.border, palette.outline),
        StrokeKind::Inside,
    );
    let track = rect.shrink(m.border + 2.0);
    let cells = ((track.width() + GAP) / (CELL + GAP)).floor().max(1.0) as usize;
    let step = (ui.input(|input| input.time) * 14.0) as usize % (cells + TRAIL);
    for cell in 0..cells {
        let behind = step.checked_sub(cell).filter(|distance| *distance < TRAIL);
        let fill = match behind {
            Some(distance) => mix(color, palette.sunken, distance as f32 / TRAIL as f32),
            None => mix(palette.sunken, palette.outline, 0.4),
        };
        let min = egui::pos2(track.left() + cell as f32 * (CELL + GAP), track.top());
        painter.rect_filled(
            Rect::from_min_size(min, egui::vec2(CELL, track.height())),
            0.0,
            fill,
        );
    }
    response
}

/// True during the "on" half of a blink cycle; schedules the repaint for the next flip.
pub fn blink(ui: &egui::Ui, period: f64) -> bool {
    let time = ui.input(|input| input.time);
    let phase = time % (period * 2.0);
    let until_flip = period - phase % period;
    ui.ctx()
        .request_repaint_after(Duration::from_secs_f64(until_flip.max(0.02)));
    phase < period
}

/// Four flat color bands across `rect`, the classic home-computer stripe.
pub fn stripes(painter: &egui::Painter, rect: Rect, palette: &Palette) {
    let colors = [palette.hot, palette.gold, palette.good, palette.accent];
    let band = rect.height() / colors.len() as f32;
    for (index, color) in colors.into_iter().enumerate() {
        let top = rect.top() + index as f32 * band;
        let strip = Rect::from_min_max(
            egui::pos2(rect.left(), top),
            egui::pos2(rect.right(), top + band),
        );
        painter.rect_filled(strip, 0.0, color);
    }
}

/// The frame of a side, top or bottom panel; glass skins paint their own pane instead of a fill.
pub fn panel_frame(ctx: &egui::Context, fill: Color32) -> egui::Frame {
    if Skin::of(ctx).finish == Finish::Glass {
        egui::Frame::new()
    } else if Skin::of(ctx).finish == Finish::Gothic {
        egui::Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0_f32, Palette::of_ctx(ctx).outline))
    } else {
        egui::Frame::new().fill(style::glazed(ctx, fill))
    }
}

/// Skin-specific panel decoration; call first inside the panel, before its content.
/// `density` scales the skin's panel opacity, e.g. to keep text legible over a dark sky.
pub fn panel_decoration(ui: &egui::Ui, density: f32) {
    const INSET: f32 = 6.0;
    let ctx = ui.ctx();
    if Skin::of(ctx).finish == Finish::Gothic {
        gothic::pattern(ui.painter(), ui.clip_rect().shrink(5.0));
        gothic::corners(
            ui.painter(),
            ui.clip_rect().shrink(5.0),
            Palette::of(ui).metal,
        );
        return;
    }
    if Skin::of(ctx).finish != Finish::Glass {
        return;
    }
    let rect = ui.clip_rect().shrink(INSET);
    let radius = style::metrics(ctx).radius as f32 + 4.0;
    let clear = Glaze::clear(ctx);
    let glaze = Glaze {
        opacity: (clear.opacity * density).min(1.0),
        ..clear
    };
    ui.painter().extend(glass::pane(ctx, rect, radius, &glaze));
}

/// The skin's backdrop behind every panel; call before showing any panel.
pub fn paint_underlay(ctx: &egui::Context) {
    let effects = &Skin::of(ctx).effects;
    let palette = Palette::of_ctx(ctx);
    if effects.hearts {
        midnight_neko::paint_background(ctx, palette);
    } else if effects.wallpaper {
        aurora::paint_wallpaper(ctx, palette);
    }
    if effects.bubbles {
        aurora::paint_bubbles(ctx, palette);
    }
}

/// The skin's decorations behind a central panel's content; call before adding the content.
pub fn paint_scenery(ui: &egui::Ui) {
    let effects = &Skin::of(ui.ctx()).effects;
    let palette = Palette::of(ui);
    if Skin::of(ui.ctx()).finish == Finish::Gothic {
        gothic::pattern(ui.painter(), ui.clip_rect());
    }
    if effects.grid {
        paint_grid(ui.painter(), ui.clip_rect(), palette);
    }
    if effects.embers {
        ornaments::paint_embers(ui, ui.clip_rect(), palette);
    }
}

/// The skin's screen-wide overlays; call after everything else has been laid out.
pub fn paint_overlays(ctx: &egui::Context) {
    let effects = &Skin::of(ctx).effects;
    let palette = Palette::of_ctx(ctx);
    if effects.hearts {
        midnight_neko::paint_overlays(ctx, palette);
    }
    if effects.vignette {
        ornaments::paint_vignette(ctx, palette);
    }
    if effects.scanlines {
        paint_scanlines(ctx, palette);
    }
}

/// A faint graph-paper grid behind the content.
fn paint_grid(painter: &egui::Painter, rect: Rect, palette: &Palette) {
    const STEP: f32 = 24.0;
    let stroke = Stroke::new(1.0_f32, palette.grid);
    let mut x = rect.left() - rect.left() % STEP;
    while x < rect.right() {
        painter.vline(x, rect.y_range(), stroke);
        x += STEP;
    }
    let mut y = rect.top() - rect.top() % STEP;
    while y < rect.bottom() {
        painter.hline(rect.x_range(), y, stroke);
        y += STEP;
    }
}

/// CRT scanlines over everything; painted only, so input passes straight through.
fn paint_scanlines(ctx: &egui::Context, palette: &Palette) {
    const PITCH: f32 = 3.0;
    let layer = egui::LayerId::new(egui::Order::Debug, egui::Id::new("scanlines"));
    let screen = ctx.content_rect();
    let mut mesh = egui::Mesh::default();
    let mut y = screen.top();
    while y < screen.bottom() {
        let line = Rect::from_min_max(
            egui::pos2(screen.left(), y),
            egui::pos2(screen.right(), y + 1.0),
        );
        mesh.add_colored_rect(line, palette.scanline);
        y += PITCH;
    }
    ctx.layer_painter(layer).add(mesh);
}

/// A centered pixel window with its own title bar over a dimmed backdrop.
pub fn dialog<R>(
    ctx: &egui::Context,
    window: egui::Window<'_>,
    title: &str,
    sprite: Sprite,
    band: Color32,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    paint_backdrop(ctx);
    let glassy = Skin::of(ctx).finish == Finish::Glass;
    let mut frame = egui::Frame::window(&ctx.global_style()).inner_margin(0);
    if glassy {
        // The pane is painted inside the window so it can carry reflections and a halo.
        frame = frame.fill(Color32::TRANSPARENT).stroke(Stroke::NONE);
    }
    window
        .title_bar(false)
        .frame(frame)
        .show(ctx, |ui| {
            let backing = ui.painter().add(egui::Shape::Noop);
            ui.spacing_mut().item_spacing.y = 0.0;
            title_bar(ui, title, sprite, band);
            let inner = egui::Frame::new()
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    contents(ui)
                })
                .inner;
            if glassy {
                let rect = ui.min_rect();
                ui.painter().set(backing, dialog_pane(ctx, rect));
            }
            inner
        })
        .and_then(|response| response.inner)
}

/// A row of status text; glass skins set it on a small frosted capsule so it reads over the sky.
pub fn status_row<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let ctx = ui.ctx().clone();
    if Skin::of(&ctx).finish != Finish::Glass {
        return ui.horizontal(contents).inner;
    }
    let backing = ui.painter().add(egui::Shape::Noop);
    let framed = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| ui.horizontal(contents).inner);
    let rect = framed.response.rect;
    let clear = Glaze::clear(&ctx);
    // Mostly frost: a light tint calms the sky behind the text without hiding it.
    let glaze = Glaze {
        opacity: (clear.opacity * 1.5).min(1.0),
        ..clear
    };
    ui.painter().set(
        backing,
        glass::pane(&ctx, rect, rect.height() / 2.0, &glaze),
    );
    framed.inner
}

/// A tooltip with `text` while the pointer rests on `rect`. Its hover zone is registered when
/// this is called, so call it after a clickable card or row laid over the same spot, which would
/// otherwise take the hover; the zone senses no clicks, so the card stays clickable.
pub fn tooltip(ui: &egui::Ui, rect: Rect, id: egui::Id, text: &str) {
    ui.interact(rect, id, Sense::hover()).on_hover_ui(|ui| {
        popup(ui, |ui| ui.label(text));
    });
}

/// The contents of a dropdown, menu or tooltip; glass skins frost the popup behind them.
pub fn popup<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let ctx = ui.ctx().clone();
    if Skin::of(&ctx).finish != Finish::Glass {
        return contents(ui);
    }
    // Popups may scroll or clip their contents, so the pane goes on a painter reaching the margin.
    let painter = ui.painter().with_clip_rect(ctx.content_rect());
    let backing = painter.add(egui::Shape::Noop);
    let inner = contents(ui);
    let rect = ui.min_rect() + egui::epaint::MarginF32::from(ui.spacing().menu_margin);
    let clear = Glaze::clear(&ctx);
    let glaze = Glaze {
        opacity: (clear.opacity * 1.6).min(1.0),
        ..clear
    };
    let radius = f32::from(ui.visuals().menu_corner_radius.nw);
    painter.set(backing, glass::pane(&ctx, rect, radius, &glaze));
    inner
}

/// A dialog's glass: denser than panels so its text reads over the scene, with an aqua halo.
fn dialog_pane(ctx: &egui::Context, rect: Rect) -> Vec<egui::Shape> {
    let palette = Palette::of_ctx(ctx);
    let clear = Glaze::clear(ctx);
    let glaze = Glaze {
        opacity: (clear.opacity * 1.6).min(1.0),
        glow: Some(palette.hot.gamma_multiply(0.7)),
        ..clear
    };
    glass::pane(ctx, rect, style::metrics(ctx).radius as f32, &glaze)
}

/// Dims the panels behind an open dialog without blocking them; call after all panels.
pub fn paint_backdrop(ctx: &egui::Context) {
    let palette = Palette::of_ctx(ctx);
    // Glass skins wash the scene in a bright haze instead of darkening it.
    let veil = if Skin::of(ctx).finish == Finish::Glass {
        palette.background.gamma_multiply(0.35)
    } else {
        Color32::from_black_alpha(140)
    };
    // Panels share the background layer; painting last there covers them but stays below windows.
    ctx.layer_painter(egui::LayerId::background())
        .rect_filled(ctx.content_rect(), 0.0, veil);
}

/// Light top-left and dark bottom-right edges; swapped when the surface is pressed in.
fn bevel(painter: &egui::Painter, rect: Rect, fill: Color32, inset: bool, edge: f32) {
    let light = mix(fill, Color32::WHITE, 0.35);
    let dark = mix(fill, Color32::BLACK, 0.3);
    let (top_left, bottom_right) = if inset { (dark, light) } else { (light, dark) };
    let top = Rect::from_min_size(rect.min, egui::vec2(rect.width(), edge));
    let left = Rect::from_min_size(rect.min, egui::vec2(edge, rect.height()));
    let bottom = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - edge), rect.max);
    let right = Rect::from_min_max(Pos2::new(rect.right() - edge, rect.top()), rect.max);
    painter.rect_filled(top, 0.0, top_left);
    painter.rect_filled(left, 0.0, top_left);
    painter.rect_filled(bottom, 0.0, bottom_right);
    painter.rect_filled(right, 0.0, bottom_right);
}

/// Dark ink on light fills, paper on dark ones.
pub fn readable_on(fill: Color32, palette: &Palette) -> Color32 {
    if luma(fill) > 140.0 {
        Color32::from_rgb(0x1a, 0x10, 0x2a)
    } else {
        palette.paper
    }
}

/// Perceived brightness of an opaque color from 0 to 255.
fn luma(color: Color32) -> f32 {
    0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32
}

/// A convex polygon filled with a color picked per corner, e.g. for gradients.
pub fn shaded_polygon(points: &[Pos2], shade: impl Fn(Pos2) -> Color32) -> egui::Shape {
    let sum = points
        .iter()
        .fold(Vec2::ZERO, |sum, point| sum + point.to_vec2());
    let center = (sum / points.len().max(1) as f32).to_pos2();
    shaded_fan(center, points, shade)
}

/// Like `shaded_polygon`, fanning out from `center`, which must lie inside the polygon.
pub fn shaded_fan(center: Pos2, points: &[Pos2], shade: impl Fn(Pos2) -> Color32) -> egui::Shape {
    let mut mesh = egui::Mesh::default();
    if points.len() < 3 {
        return mesh.into();
    }
    mesh.colored_vertex(center, shade(center));
    for point in points {
        mesh.colored_vertex(*point, shade(*point));
    }
    let count = points.len() as u32;
    for index in 0..count {
        mesh.add_triangle(0, index + 1, (index + 1) % count + 1);
    }
    mesh.into()
}

pub fn mix(from: Color32, to: Color32, amount: f32) -> Color32 {
    let blend = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * amount).round() as u8;
    Color32::from_rgb(
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::theme::skins;

    #[test]
    fn text_on_fills_stays_readable_in_both_palettes() {
        for palette in skins::ALL
            .iter()
            .filter(|skin| skin.finish != Finish::Modern)
            .flat_map(|skin| [&skin.dark, &skin.light])
        {
            let luminance = |color: Color32| {
                let linear = |channel: u8| {
                    let value = channel as f32 / 255.0;
                    if value <= 0.04045 {
                        value / 12.92
                    } else {
                        ((value + 0.055) / 1.055).powf(2.4)
                    }
                };
                0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
            };
            let fill = luminance(palette.gold);
            let text = luminance(readable_on(palette.gold, palette));
            assert!((fill.max(text) + 0.05) / (fill.min(text) + 0.05) >= 4.5);
            assert_eq!(readable_on(palette.outline, palette), palette.paper);
            assert_eq!(readable_on(palette.bad, palette), palette.paper);
        }
    }

    #[test]
    fn buttons_stay_within_their_minimum_footprint_in_every_skin() {
        let context = egui::Context::default();
        for (skin, dark) in skins::ALL
            .iter()
            .flat_map(|skin| [(*skin, false), (*skin, true)])
        {
            style::install(&context, skin);
            context.set_theme(if dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let size = egui::vec2(260.0, 52.0);
                    let button = PixelButton::new("New job")
                        .sprite(Sprite::Plus)
                        .min_size(size);
                    assert_eq!(ui.add(button).rect.size(), size, "skin {}", skin.id);
                });
            });
        }
    }

    #[test]
    fn tooltips_show_over_spots_inside_a_clickable_card() {
        let context = egui::Context::default();
        let spot = Rect::from_min_size(egui::pos2(40.0, 40.0), Vec2::splat(14.0));
        let mut zone_hovered = false;
        let mut card_clickable = false;
        for frame in 0..3 {
            let input = egui::RawInput {
                events: vec![egui::Event::PointerMoved(spot.center())],
                time: Some(frame as f64),
                ..Default::default()
            };
            let _ = context.run_ui(input, |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let card = Rect::from_min_size(egui::pos2(20.0, 20.0), Vec2::splat(120.0));
                    let response = ui.interact(card, egui::Id::new("card"), Sense::click());
                    tooltip(ui, spot, egui::Id::new("spot"), "Exactly 7 bytes");
                    zone_hovered = ui
                        .ctx()
                        .read_response(egui::Id::new("spot"))
                        .is_some_and(|zone| zone.hovered());
                    card_clickable = response.contains_pointer() && response.sense.senses_click();
                });
            });
        }
        assert!(zone_hovered && card_clickable);
    }

    #[test]
    fn status_rows_return_their_contents_under_every_skin() {
        let context = egui::Context::default();
        for skin in skins::ALL {
            style::install(&context, skin);
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    assert!(status_row(ui, |ui| ui.label("Enabled").rect.height() > 0.0));
                });
            });
        }
    }

    #[test]
    fn menus_return_their_contents_under_every_skin() {
        let context = egui::Context::default();
        for skin in skins::ALL {
            style::install(&context, skin);
            let _ = context.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    assert!(popup(ui, |ui| ui.label("Hours").rect.height() > 0.0));
                });
            });
        }
    }

    #[test]
    fn checkbox_and_choice_report_changes() {
        let context = egui::Context::default();
        let mut checked = false;
        let mut current = 1;
        let _ = context.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                assert!(!ui.add(PixelCheckbox::new(&mut checked, "Label")).changed());
                assert!(!choice(ui, &mut current, 2, "Two", None).changed());
            });
        });
        assert!(!checked);
        assert_eq!(current, 1);
    }
}
