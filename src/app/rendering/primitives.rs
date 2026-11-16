use crate::app::theme::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

/// Light top-left and dark bottom-right edges; swapped when the surface is pressed in.
pub(super) fn bevel(painter: &egui::Painter, rect: Rect, fill: Color32, inset: bool, edge: f32) {
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
pub(super) fn luma(color: Color32) -> f32 {
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
    use crate::app::theme::{skins, style::Finish};
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
}
