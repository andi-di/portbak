//! Frutiger Aero scenery: a deep sky with a turning sunburst and lens flare (or stars and an aurora
//! by night), drifting clouds, light swooshes, a spinning glass globe, rolling hills and a glassy
//! sea whose waves toss up droplets; plus the glass title bars.

use super::aquarium;
use super::bubbles;
use super::glass;
use super::ornaments::{fading_rule, noise};
use super::pixel::{halo, mix, shaded_polygon};
use super::splashes;
use super::sprites::{self, Sprite};
use super::style::{self, label_font, Palette, Skin};
use super::wildlife;
use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const CURVE_STEPS: u32 = 72;
const ICON_SCALE: f32 = 2.0;
const SUN_RAYS: u32 = 18;
const STAR_COUNT: u32 = 110;
const CLOUD_COUNT: u32 = 7;
const DROPLET_COUNT: u32 = 10;

/// A rolling hill line: `base` and `swell` are fractions of the window height.
struct Hill {
    base: f32,
    swell: f32,
    waves: f32,
    phase: f32,
    /// How far the hill fades into the sky; distant hills fade more.
    haze: f32,
}

const HILLS: [Hill; 3] = [
    Hill {
        base: 0.7,
        swell: 0.06,
        waves: 0.9,
        phase: 2.6,
        haze: 0.6,
    },
    Hill {
        base: 0.76,
        swell: 0.07,
        waves: 0.65,
        phase: 0.6,
        haze: 0.25,
    },
    Hill {
        base: 0.84,
        swell: 0.05,
        waves: 0.45,
        phase: 2.2,
        haze: 0.0,
    },
];

/// Where the sea's surface rests, as a fraction of the window height.
const SEA_LEVEL: f32 = 0.88;

/// The wallpaper behind every panel; paint it before any panel so they cover it.
pub fn paint_wallpaper(ctx: &egui::Context, palette: &Palette) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen = ctx.content_rect();
    let time = ctx.input(|input| input.time) as f32;
    let night = ctx.global_style().visuals.dark_mode;
    painter.add(sky(screen, palette, night));
    if night {
        stars(&painter, screen, time);
        moon(&painter, screen, palette);
    } else {
        sunburst(&painter, screen, palette, time);
    }
    clouds(&painter, screen, night, time);
    let ribbons = if night {
        [
            palette.good,
            palette.hot,
            mix(palette.accent, palette.bad, 0.5),
        ]
    } else {
        [palette.paper, palette.hot, palette.paper]
    };
    for (index, color) in ribbons.into_iter().enumerate() {
        painter.extend(ribbon(screen, index as f32, color, time, night));
    }
    globe(&painter, screen, palette, time);
    if !night {
        wildlife::gulls(&painter, screen, palette, time);
    }
    for hill in &HILLS {
        painter.extend(hill_shapes(screen, hill, palette, night));
    }
    sea(&painter, screen, palette, time);
    aquarium::swim(ctx, &painter, screen, palette, night);
    wildlife::leaves(&painter, screen, palette, time, night);
    if !night {
        lens_flare(&painter, screen, palette);
    }
}

/// Bubbles and sparkles drifting over the scenery, plus ripples and bubbles the pointer leaves
/// behind; keeps the window repainting.
pub fn paint_bubbles(ctx: &egui::Context, palette: &Palette) {
    bubbles::paint(ctx, palette);
    splashes::paint(ctx, palette);
}

/// A tinted glass band with a glossy reflection, a shimmer, a glowing rule and a haloed title.
pub fn glass_title_bar(ui: &mut egui::Ui, title: &str, sprite: Sprite, band: Color32) {
    let palette = Palette::of(ui);
    let skin = Skin::of(ui.ctx());
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), egui::Sense::hover());
    let radius = (skin.metrics.radius as f32 - skin.metrics.border).max(0.0);
    let painter = ui.painter();
    let corners = [radius, radius, 0.0, 0.0];
    let top = mix(band, Color32::WHITE, 0.45).gamma_multiply(0.85);
    let bottom = band.gamma_multiply(0.6);
    painter.add(glass::gradient(rect, corners, top, bottom));
    painter.extend(glass::sheen(ui.ctx(), rect, radius));
    let rule = Rect::from_min_max(
        egui::pos2(rect.left(), rect.bottom() - 2.0),
        rect.right_bottom(),
    );
    fading_rule(painter, rule, mix(band, Color32::WHITE, 0.3));
    let icon = sprites::size(ui.ctx(), sprite, ICON_SCALE);
    let icon_pos = egui::pos2(rect.left() + 12.0, rect.center().y - icon.y / 2.0);
    sprites::paint(painter, icon_pos, sprite, ICON_SCALE, band);
    let font = label_font(ui.ctx());
    let text = style::cased(ui.ctx(), title);
    let galley = painter.layout_no_wrap(text.clone(), font.clone(), palette.ink);
    let text_pos = egui::pos2(
        icon_pos.x + icon.x + 12.0,
        rect.center().y - galley.size().y / 2.0,
    )
    .round();
    halo(
        painter,
        text_pos,
        &text,
        &font,
        skin.typography.display_glow + 1.0,
    );
    painter.galley(text_pos, galley, palette.ink);
}

/// The sky gradient: deep at the top, brightening to `background` at the horizon.
fn sky(screen: Rect, palette: &Palette, night: bool) -> Shape {
    let middle = if night {
        mix(palette.sky, palette.surface, 0.6)
    } else {
        mix(palette.sky, palette.accent, 0.7)
    };
    let stops = [
        (0.0, palette.sky),
        (0.38, middle),
        (0.72, palette.background),
        (1.0, palette.background),
    ];
    let mut mesh = Mesh::default();
    for (index, (at, color)) in stops.into_iter().enumerate() {
        let y = screen.top() + at * screen.height();
        mesh.colored_vertex(egui::pos2(screen.left(), y), color);
        mesh.colored_vertex(egui::pos2(screen.right(), y), color);
        if index > 0 {
            let last = index as u32 * 2;
            mesh.add_triangle(last - 2, last - 1, last);
            mesh.add_triangle(last - 1, last + 1, last);
        }
    }
    mesh.into()
}

fn sun_center(screen: Rect) -> Pos2 {
    screen.left_top() + Vec2::new(screen.width() * 0.84, screen.height() * 0.1)
}

/// A blazing sun with a slowly turning wheel of light rays.
fn sunburst(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    let center = sun_center(screen);
    let reach = screen.width().max(screen.height()) * 0.9;
    let light = mix(palette.paper, palette.gold, 0.15);
    let mut rays = Mesh::default();
    for ray in 0..SUN_RAYS {
        let length = reach * if ray % 2 == 0 { 1.0 } else { 0.6 };
        let width = if ray % 2 == 0 { 0.045 } else { 0.03 };
        let angle = ray as f32 * TAU / SUN_RAYS as f32 + time * 0.03;
        let base = rays.vertices.len() as u32;
        rays.colored_vertex(center, light.gamma_multiply(0.35));
        for side in [-width, width] {
            rays.colored_vertex(
                center + Vec2::angled(angle + side) * length,
                Color32::TRANSPARENT,
            );
        }
        rays.add_triangle(base, base + 1, base + 2);
    }
    painter.add(rays);
    painter.add(glass::radial_glow(
        center,
        reach * 0.55,
        light.gamma_multiply(0.5),
    ));
    painter.add(glass::radial_glow(
        center,
        90.0,
        Color32::WHITE.gamma_multiply(0.9),
    ));
    painter.circle_filled(center, 26.0, Color32::WHITE);
}

/// Ghostly rings and discs strung along a line from the sun through the middle of the window.
fn lens_flare(painter: &egui::Painter, screen: Rect, palette: &Palette) {
    let sun = sun_center(screen);
    let axis = screen.center() - sun;
    let flares = [
        (0.35, 18.0, palette.gold, 0.25),
        (0.6, 40.0, palette.hot, 0.14),
        (0.75, 12.0, palette.paper, 0.35),
        (1.1, 70.0, palette.good, 0.1),
        (1.35, 28.0, palette.accent, 0.2),
    ];
    for (along, radius, color, strength) in flares {
        let center = sun + axis * along;
        painter.add(glass::radial_glow(
            center,
            radius,
            color.gamma_multiply(strength * 2.0),
        ));
        painter.circle_stroke(
            center,
            radius * 0.8,
            Stroke::new(1.5_f32, color.gamma_multiply(strength)),
        );
    }
}

/// A quiet moon with a wide halo.
fn moon(painter: &egui::Painter, screen: Rect, palette: &Palette) {
    let center = sun_center(screen);
    painter.add(glass::radial_glow(
        center,
        screen.height() * 0.35,
        palette.hot.gamma_multiply(0.25),
    ));
    painter.add(glass::radial_glow(
        center,
        60.0,
        palette.paper.gamma_multiply(0.6),
    ));
    painter.circle_filled(center, 20.0, mix(palette.paper, palette.hot, 0.15));
}

/// Stars twinkling in the upper night sky.
fn stars(painter: &egui::Painter, screen: Rect, time: f32) {
    for index in 0..STAR_COUNT {
        let random = |salt: u32| noise(index, salt + 40);
        let at = egui::pos2(
            screen.left() + random(0) * screen.width(),
            screen.top() + random(1).powf(1.4) * screen.height() * 0.7,
        );
        let twinkle = 0.5 + 0.5 * (time * (1.0 + random(2) * 3.0) + random(3) * TAU).sin();
        let size = 0.6 + random(4) * 1.4;
        painter.circle_filled(at, size, Color32::WHITE.gamma_multiply(0.3 + 0.7 * twinkle));
        if random(5) > 0.9 {
            glass::sparkle(painter, at, 7.0 * twinkle, Color32::WHITE);
        }
    }
}

/// Puffy cumulus clouds drifting slowly to the right; by night only a faint mist.
fn clouds(painter: &egui::Painter, screen: Rect, night: bool, time: f32) {
    let strength = if night { 0.12 } else { 0.85 };
    for index in 0..CLOUD_COUNT {
        let random = |salt: u32| noise(index, salt + 70);
        let scale = 40.0 + random(0) * 60.0;
        let span = screen.width() + scale * 6.0;
        let x = (random(1) * span + time * (4.0 + random(2) * 8.0)) % span - scale * 3.0;
        let y = screen.top() + screen.height() * (0.08 + random(3) * 0.42);
        let base = egui::pos2(screen.left() + x, y);
        for puff in 0..6 {
            let pick = |salt: u32| noise(index * 16 + puff, salt + 90);
            let offset = Vec2::new(
                (pick(0) - 0.5) * scale * 3.6,
                -pick(1) * scale * 0.7 * (1.0 - ((pick(0) - 0.5) * 2.0).abs()),
            );
            let radius = scale * (0.55 + pick(2) * 0.6);
            painter.add(glass::radial_glow(
                base + offset,
                radius * 1.35,
                Color32::WHITE.gamma_multiply(strength),
            ));
        }
    }
}

/// A wavy band of light that is brightest along its crest and trails off below, with a crisp
/// thread of light along the crest like the swooshes on a glossy wallpaper.
fn ribbon(screen: Rect, index: f32, color: Color32, time: f32, night: bool) -> [Shape; 2] {
    let height = screen.height();
    let strength = if night { 0.6 } else { 0.38 };
    let mut mesh = Mesh::default();
    let mut crest_line = Vec::with_capacity(CURVE_STEPS as usize + 1);
    for step in 0..=CURVE_STEPS {
        let t = step as f32 / CURVE_STEPS as f32;
        let x = screen.left() + t * screen.width();
        let wave = (t * 4.2 + index * 1.7 + time * 0.12).sin() * 0.07
            + (t * 9.0 - index + time * 0.07).sin() * 0.02;
        let crest = screen.top() + height * (0.2 + index * 0.12 + wave);
        let half = height * (0.03 + 0.03 * (t * 5.0 + index * 2.3 + time * 0.1).sin().abs());
        let envelope = (t * PI).sin().max(0.0);
        let core = color.gamma_multiply(envelope * strength);
        mesh.colored_vertex(egui::pos2(x, crest - half), Color32::TRANSPARENT);
        mesh.colored_vertex(egui::pos2(x, crest), core);
        mesh.colored_vertex(egui::pos2(x, crest + half * 2.4), Color32::TRANSPARENT);
        crest_line.push(egui::pos2(x, crest));
        if step > 0 {
            let (before, now) = ((step - 1) * 3, step * 3);
            for row in 0..2 {
                mesh.add_triangle(before + row, before + row + 1, now + row);
                mesh.add_triangle(before + row + 1, now + row + 1, now + row);
            }
        }
    }
    let thread = Stroke::new(1.5_f32, Color32::WHITE.gamma_multiply(strength * 0.9));
    [mesh.into(), Shape::line(crest_line, thread)]
}

/// A big glass globe turning slowly, with a lit grid, a fresnel rim and a glinting orbit.
fn globe(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    const TILT: f32 = 0.38;
    let radius = screen.height() * 0.2;
    let center = screen.left_top() + Vec2::new(screen.width() * 0.8, screen.height() * 0.52);
    let turn = Vec2::angled(TILT);
    let project = |latitude: f32, longitude: f32| {
        let x = latitude.cos() * longitude.sin();
        let y = -latitude.sin();
        let depth = latitude.cos() * longitude.cos();
        let flat = Vec2::new(x * turn.x - y * turn.y, x * turn.y + y * turn.x) * radius;
        (center + flat, depth)
    };
    painter.add(glass::radial_glow(
        center,
        radius * 1.5,
        palette.hot.gamma_multiply(0.35),
    ));
    painter.circle_filled(center, radius, palette.accent.gamma_multiply(0.22));
    let grid = |front: bool| {
        let ink = Color32::WHITE.gamma_multiply(if front { 0.55 } else { 0.15 });
        Stroke::new(if front { 1.4_f32 } else { 1.0 }, ink)
    };
    let mut lines = Vec::new();
    for meridian in 0..10 {
        let longitude = meridian as f32 * PI / 5.0 + time * 0.2;
        let points: Vec<(Pos2, f32)> = (0..=24)
            .map(|step| project(-FRAC_PI_2 + step as f32 * PI / 24.0, longitude))
            .collect();
        lines.push(points);
    }
    for parallel in [-60.0_f32, -30.0, 0.0, 30.0, 60.0] {
        let latitude = parallel.to_radians();
        let points: Vec<(Pos2, f32)> = (0..=48)
            .map(|step| project(latitude, step as f32 * TAU / 48.0))
            .collect();
        lines.push(points);
    }
    // Back lines first and faint, so the front of the grid reads on top.
    for front in [false, true] {
        for line in &lines {
            for pair in line.windows(2) {
                let ((from, depth_a), (to, depth_b)) = (pair[0], pair[1]);
                if ((depth_a + depth_b) > 0.0) == front {
                    painter.line_segment([from, to], grid(front));
                }
            }
        }
    }
    fresnel_rim(painter, center, radius, palette.paper);
    let shine = glass::ellipse(
        center + Vec2::new(-0.3, -0.5) * radius,
        Vec2::new(0.5, 0.26) * radius,
        -0.45,
    );
    painter.add(shaded_polygon(&shine, |point| {
        let t = ((point.y - (center.y - radius * 0.8)) / (radius * 0.55)).clamp(0.0, 1.0);
        Color32::WHITE.gamma_multiply(0.75 * (1.0 - t))
    }));
    let orbit = |angle: f32| {
        let flat = Vec2::new(angle.cos() * radius * 1.35, angle.sin() * radius * 0.32);
        center
            + Vec2::new(
                flat.x * turn.x - flat.y * turn.y,
                flat.x * turn.y + flat.y * turn.x,
            )
    };
    let path: Vec<Pos2> = (0..=64)
        .map(|step| orbit(step as f32 * TAU / 64.0))
        .collect();
    painter.add(Shape::line(
        path,
        Stroke::new(1.5_f32, Color32::WHITE.gamma_multiply(0.35)),
    ));
    glass::sparkle(painter, orbit(time * 0.9), 14.0, palette.hot);
}

/// A ring of light hugging the inside of a sphere's edge, as real glass brightens at grazing angles.
fn fresnel_rim(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    const SEGMENTS: u32 = 64;
    let mut mesh = Mesh::default();
    for segment in 0..SEGMENTS {
        let direction = Vec2::angled(segment as f32 * TAU / SEGMENTS as f32);
        // Brighter on the lower right, opposite the highlight.
        let lit = 0.45 + 0.35 * direction.dot(Vec2::new(0.6, 0.8));
        mesh.colored_vertex(center + direction * radius * 0.78, Color32::TRANSPARENT);
        mesh.colored_vertex(center + direction * radius, color.gamma_multiply(lit));
    }
    for segment in 0..SEGMENTS {
        let (a, b) = (segment * 2, (segment + 1) % SEGMENTS * 2);
        mesh.add_triangle(a, a + 1, b + 1);
        mesh.add_triangle(a, b + 1, b);
    }
    painter.add(mesh);
}

/// A grassy hill reaching down to the window bottom, with light catching its crest.
fn hill_shapes(screen: Rect, hill: &Hill, palette: &Palette, night: bool) -> [Shape; 2] {
    let lush = if night {
        mix(palette.good, palette.surface, 0.55)
    } else {
        mix(palette.good, Color32::from_rgb(0xa8, 0xf0, 0x20), 0.45)
    };
    let grass = mix(lush, palette.background, hill.haze);
    let crest_color = mix(grass, Color32::WHITE, 0.25);
    let foot_color = mix(grass, palette.outline, 0.45);
    let crest: Vec<Pos2> = (0..=CURVE_STEPS)
        .map(|step| {
            let t = step as f32 / CURVE_STEPS as f32;
            let swell = (t * hill.waves * TAU + hill.phase).sin() * hill.swell;
            egui::pos2(
                screen.left() + t * screen.width(),
                screen.top() + screen.height() * (hill.base + swell),
            )
        })
        .collect();
    let mut mesh = Mesh::default();
    for (index, point) in crest.iter().enumerate() {
        mesh.colored_vertex(*point, crest_color);
        mesh.colored_vertex(egui::pos2(point.x, screen.bottom()), foot_color);
        if index > 0 {
            let last = index as u32 * 2;
            mesh.add_triangle(last - 2, last - 1, last);
            mesh.add_triangle(last - 1, last + 1, last);
        }
    }
    let shine = Stroke::new(2.5_f32, Color32::WHITE.gamma_multiply(0.45));
    [mesh.into(), Shape::line(crest, shine)]
}

/// The height of the sea's surface at `x` and `time`.
fn sea_surface(screen: Rect, x: f32, time: f32, layer: f32) -> f32 {
    let t = (x - screen.left()) / screen.width().max(1.0);
    let wave = (t * 7.0 + time * 0.9 + layer * 2.0).sin() * 0.012
        + (t * 15.0 - time * 1.4 + layer).sin() * 0.006;
    screen.top() + screen.height() * (SEA_LEVEL + layer * 0.03 + wave)
}

/// Two layers of clear blue water rolling along the bottom, with glinting crests and droplets.
fn sea(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    for layer in [0.0, 1.0] {
        let top = mix(palette.hot, Color32::WHITE, 0.25).gamma_multiply(0.55 + layer * 0.15);
        let bottom = mix(palette.accent, palette.outline, 0.25).gamma_multiply(0.85);
        let crest: Vec<Pos2> = (0..=CURVE_STEPS)
            .map(|step| {
                let x = screen.left() + step as f32 / CURVE_STEPS as f32 * screen.width();
                egui::pos2(x, sea_surface(screen, x, time, layer))
            })
            .collect();
        let mut mesh = Mesh::default();
        for (index, point) in crest.iter().enumerate() {
            mesh.colored_vertex(*point, top);
            mesh.colored_vertex(egui::pos2(point.x, screen.bottom()), bottom);
            if index > 0 {
                let last = index as u32 * 2;
                mesh.add_triangle(last - 2, last - 1, last);
                mesh.add_triangle(last - 1, last + 1, last);
            }
        }
        painter.add(mesh);
        painter.add(Shape::line(
            crest.clone(),
            Stroke::new(2.0_f32, Color32::WHITE.gamma_multiply(0.85)),
        ));
        // Short glints riding just under the crest.
        for (index, point) in crest.iter().enumerate().skip(2).step_by(5) {
            // Grows from nothing and fades back, so glints never pop in as hard lines.
            let flicker = ((time * 2.5 + index as f32 * 1.3 + layer).sin() - 0.3) / 0.7;
            if flicker > 0.0 {
                let glint = Rect::from_center_size(
                    *point + Vec2::new(0.0, 7.0 + layer * 4.0),
                    egui::vec2(16.0 * flicker, 2.0),
                );
                painter.rect_filled(glint, 1.0, Color32::WHITE.gamma_multiply(0.5 * flicker));
            }
        }
    }
    droplets(painter, screen, palette, time);
}

/// Water drops leaping out of the waves in little arcs.
fn droplets(painter: &egui::Painter, screen: Rect, palette: &Palette, time: f32) {
    const FLIGHT: f32 = 1.4;
    for index in 0..DROPLET_COUNT {
        let random = |salt: u32| noise(index, salt + 120);
        let period = FLIGHT + 2.0 + random(0) * 4.0;
        let age = (time + random(1) * period) % period;
        if age > FLIGHT {
            continue;
        }
        let start = screen.left() + random(2) * screen.width();
        let x = start + (random(3) - 0.5) * 80.0 * age;
        let lift = 140.0 + random(4) * 120.0;
        let rise = lift * age - 0.5 * (2.0 * lift / FLIGHT) * age * age;
        let y = sea_surface(screen, start, time, 0.0) - rise;
        let radius = 2.5 + random(5) * 3.5;
        glass::sphere(
            painter,
            egui::pos2(x, y),
            radius,
            mix(palette.hot, Color32::WHITE, 0.3),
            0.5,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::skins;

    #[test]
    fn scenery_paints_under_every_skin_and_theme() {
        let context = egui::Context::default();
        for skin in skins::ALL {
            style::install(&context, skin);
            for theme in [egui::Theme::Dark, egui::Theme::Light] {
                context.set_theme(theme);
                let _ = context.run_ui(egui::RawInput::default(), |ui| {
                    let ctx = &ui.ctx().clone();
                    paint_wallpaper(ctx, Palette::of_ctx(ctx));
                    paint_bubbles(ctx, Palette::of_ctx(ctx));
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        glass_title_bar(ui, "Folders", Sprite::Folder, Palette::of(ui).accent);
                    });
                });
            }
        }
    }

    #[test]
    fn hills_span_the_window_and_stay_inside_it() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let palette = &skins::default().light;
        for hill in &HILLS {
            let [_, Shape::Path(crest)] = hill_shapes(screen, hill, palette, false) else {
                panic!("the crest is a line");
            };
            assert_eq!(crest.points.first().map(|point| point.x), Some(0.0));
            assert_eq!(crest.points.last().map(|point| point.x), Some(800.0));
            assert!(crest.points.iter().all(|point| screen.contains(*point)));
        }
    }

    #[test]
    fn the_sea_stays_in_the_lower_part_of_the_window() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        for step in 0..=40 {
            let x = step as f32 * 20.0;
            for layer in [0.0, 1.0] {
                let y = sea_surface(screen, x, step as f32 * 0.7, layer);
                assert!((480.0..600.0).contains(&y), "{y}");
            }
        }
    }
}
