//! Retro widgets and decorations painted from plain rectangles: bevels, hard shadows, LEDs.

use super::forge;
use super::ornaments;
use super::sprites::{self, Sprite};
use super::style::{self, display_font, label_font, Finish, Palette, Skin, TitleBar};
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

/// A chunky beveled push button; `selected` renders it latched down like a toggle key.
pub struct PixelButton {
    label: String,
    sprite: Option<Sprite>,
    tone: Tone,
    selected: bool,
    min_size: Vec2,
}

impl PixelButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            sprite: None,
            tone: Tone::Plain,
            selected: false,
            min_size: Vec2::ZERO,
        }
    }

    pub fn sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
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
        let text = self.label.to_uppercase();
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
        let engraved = Skin::of(ui.ctx()).finish == Finish::Engraved;
        // Engraved plates press straight down, so they only need room below.
        let margin = if engraved {
            Vec2::new(0.0, m.lift)
        } else {
            Vec2::splat(m.lift)
        };
        // `min_size` bounds the whole footprint, so a full-width button cannot widen its panel.
        let face_size = (content + padding * 2.0).max(self.min_size - margin);
        let (rect, response) = ui.allocate_exact_size(face_size + margin, Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, enabled, self.selected, &text)
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
        let fill = match (enabled, hovered) {
            (false, _) => palette.surface,
            (true, true) => mix(base, Color32::WHITE, 0.18),
            (true, false) => base,
        };
        let painter = ui.painter();
        let face = if engraved {
            let press = Vec2::new(0.0, if sunk { 1.0 } else { 0.0 });
            let face = Rect::from_min_size(rect.min + press, face_size);
            let look = forge::PlateLook {
                fill,
                enabled,
                pressed: sunk,
                glow,
                focused: response.has_focus(),
            };
            forge::plate(painter, face, &look);
            face
        } else {
            let offset = if sunk {
                Vec2::splat(m.lift)
            } else {
                Vec2::ZERO
            };
            let face = Rect::from_min_size(rect.min + offset, face_size);
            pixel_face(painter, face, fill, sunk, enabled, response.has_focus());
            face
        };
        let ink = if enabled {
            readable_on(fill, palette)
        } else {
            palette.dim
        };
        let mut cursor = face.center() - content / 2.0;
        if let Some(sprite) = self.sprite {
            let tint = match (enabled, self.tone, self.selected) {
                (false, ..) => palette.dim,
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
        if Skin::of(ui.ctx()).finish == Finish::Engraved {
            forge::spiked_socket(painter, square, border);
        } else {
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
        if checked {
            let tint = if enabled { palette.good } else { palette.dim };
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
    let shadow = style::drop_shadow(m, palette, m.shadow);
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(m.border, palette.outline))
        .shadow(shadow)
        .outer_margin(egui::Margin {
            left: 0,
            top: 0,
            right: m.shadow,
            bottom: m.shadow + 6,
        })
        .show(ui, |ui| {
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
        })
        .inner
}

pub fn title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite, band: Color32) {
    match Skin::of(ui.ctx()).effects.title_bar {
        TitleBar::Striped => striped_title_bar(ui, title, sprite, band),
        TitleBar::Engraved => ornaments::engraved_title_bar(ui, title, sprite, band),
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

/// Text in the display face with a hard drop shadow; `size` is scaled by the skin.
pub fn display_label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) -> Response {
    let palette = Palette::of(ui);
    let typography = &Skin::of(ui.ctx()).typography;
    let size = size * typography.display_scale;
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), display_font(size), color);
    let drop = (size * typography.display_drop).round();
    let (rect, response) =
        ui.allocate_exact_size(galley.size() + Vec2::splat(drop), Sense::hover());
    let painter = ui.painter();
    if drop > 0.0 {
        let shadow = painter.layout_no_wrap(text.to_owned(), display_font(size), palette.outline);
        painter.galley(
            rect.min.round() + Vec2::splat(drop),
            shadow,
            palette.outline,
        );
    }
    painter.galley(rect.min.round(), galley, color);
    response
}

/// A small framed tag such as QUEUED or PROTECTED.
pub fn badge(ui: &mut egui::Ui, text: &str, fill: Color32) -> Response {
    let m = style::metrics(ui.ctx());
    let palette = Palette::of(ui);
    let ink = readable_on(fill, palette);
    let font = TextStyle::Small.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.to_uppercase(), font, ink);
    let padding = egui::vec2(6.0, 0.0);
    let (rect, response) = ui.allocate_exact_size(
        galley.size() + padding * 2.0 + Vec2::splat(m.border * 2.0),
        Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(
        rect,
        0.0,
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
    if Skin::of(ui.ctx()).finish == Finish::Engraved {
        ornaments::diamond_lamp(painter, rect, fill, lit);
        return response;
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
    const CELL: f32 = 8.0;
    const GAP: f32 = 2.0;
    const TRAIL: usize = 6;
    let palette = Palette::of(ui);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 18.0), Sense::hover());
    let painter = ui.painter();
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
    ui.ctx().request_repaint_after(MARQUEE_FRAME);
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

/// The skin's decorations behind a central panel's content; call before adding the content.
pub fn paint_scenery(ui: &egui::Ui) {
    let effects = &Skin::of(ui.ctx()).effects;
    let palette = Palette::of(ui);
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
    let screen = ctx.screen_rect();
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
    let frame = egui::Frame::window(&ctx.style()).inner_margin(0);
    window
        .title_bar(false)
        .frame(frame)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            title_bar(ui, title, sprite, band);
            egui::Frame::new()
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    contents(ui)
                })
                .inner
        })
        .and_then(|response| response.inner)
}

/// Dims the panels behind an open dialog without blocking them; call after all panels.
pub fn paint_backdrop(ctx: &egui::Context) {
    // Panels share the background layer; painting last there covers them but stays below windows.
    ctx.layer_painter(egui::LayerId::background()).rect_filled(
        ctx.screen_rect(),
        0.0,
        Color32::from_black_alpha(140),
    );
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
    let luma = 0.299 * fill.r() as f32 + 0.587 * fill.g() as f32 + 0.114 * fill.b() as f32;
    if luma > 140.0 {
        Color32::from_rgb(0x1a, 0x10, 0x2a)
    } else {
        palette.paper
    }
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
    use crate::app::skins;

    #[test]
    fn text_on_fills_stays_readable_in_both_palettes() {
        for palette in skins::ALL.iter().flat_map(|skin| [&skin.dark, &skin.light]) {
            assert_eq!(readable_on(palette.gold, palette).r(), 0x1a);
            assert_eq!(readable_on(palette.outline, palette), palette.paper);
            assert_eq!(readable_on(palette.bad, palette), palette.paper);
        }
    }

    #[test]
    fn buttons_stay_within_their_minimum_footprint_in_every_skin() {
        let context = egui::Context::default();
        for skin in skins::ALL {
            style::install(&context, skin);
            let _ = context.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
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
    fn checkbox_and_choice_report_changes() {
        let context = egui::Context::default();
        let mut checked = false;
        let mut current = 1;
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert!(!ui.add(PixelCheckbox::new(&mut checked, "Label")).changed());
                assert!(!choice(ui, &mut current, 2, "Two", None).changed());
            });
        });
        assert!(!checked);
        assert_eq!(current, 1);
    }
}
