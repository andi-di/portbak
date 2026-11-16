use super::{aurora, liquid_wallpaper, moonveil};
use crate::app::{
    rendering::gothic,
    theme::style::{Finish, Palette, Skin},
};
use eframe::egui::{self, Rect, Stroke};

/// The skin's backdrop behind every panel; call before showing any panel.
pub fn paint_underlay(ctx: &egui::Context) {
    let effects = &Skin::of(ctx).effects;
    let palette = Palette::of_ctx(ctx);
    if effects.hearts {
        moonveil::paint_background(ctx, palette);
    } else if Skin::of(ctx).finish == Finish::Liquid {
        liquid_wallpaper::paint(ctx);
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
        moonveil::paint_scene(ui.painter(), Palette::of(ui));
        gothic::pattern(ui.painter(), ui.clip_rect());
    }
    if effects.grid {
        paint_grid(ui.painter(), ui.clip_rect(), palette);
    }
}

/// The skin's screen-wide overlays; call after everything else has been laid out.
pub fn paint_overlays(ctx: &egui::Context) {
    let effects = &Skin::of(ctx).effects;
    let palette = Palette::of_ctx(ctx);
    if effects.hearts {
        moonveil::paint_overlays(ctx, palette);
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
