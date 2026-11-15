//! Short, bounded accents shared by the sidebar and gallery.

use super::Expression;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

pub(super) fn duration(expression: Expression) -> Option<f64> {
    match expression {
        Expression::Angry => Some(1.6),
        Expression::Love => Some(super::LOVE_HEART_SECONDS + super::LOVE_HEARTS[2].3),
        Expression::Celebrating => Some(2.4),
        Expression::Crazy | Expression::Killer | Expression::Energetic => Some(2.2),
        Expression::Cute => Some(2.2),
        Expression::Excited => Some(1.8),
        Expression::Exhausted => Some(2.2),
        Expression::Fed => Some(2.4),
        Expression::Drunk | Expression::Embarrassed | Expression::Flustered => Some(2.2),
        Expression::Flirty => Some(2.2),
        Expression::Hurt => Some(1.2),
        Expression::Pouting => Some(1.4),
        Expression::Pleading | Expression::Hungry => Some(2.2),
        Expression::Praying => Some(2.4),
        Expression::Question => Some(2.2),
        Expression::Wink => Some(1.5),
        _ => None,
    }
}

pub(super) fn paint(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f64) {
    match expression {
        Expression::Angry => anger_mark(ui, rect, elapsed as f32),
        Expression::Love => {
            super::paint_love_hearts(ui, rect, elapsed);
            eye_glow(ui, rect, expression, elapsed as f32);
        }
        Expression::Celebrating => confetti(ui, rect, elapsed as f32),
        Expression::Crazy | Expression::Killer | Expression::Energetic => {
            eye_glow(ui, rect, expression, elapsed as f32)
        }
        Expression::Cute => cat_play(ui, rect, elapsed as f32),
        Expression::Excited => excited(ui, rect, elapsed as f32),
        Expression::Exhausted => fatigue_lines(ui, rect, elapsed as f32),
        Expression::Fed => blood_drips(ui, rect, elapsed as f32),
        Expression::Drunk | Expression::Embarrassed | Expression::Flustered => {
            cheek_glow(ui, rect, expression, elapsed as f32)
        }
        Expression::Flirty => {
            necklace_glow(ui, rect, elapsed as f32);
            kiss_print(ui, rect, elapsed as f32);
        }
        Expression::Hurt => scratches(ui, rect, elapsed as f32),
        Expression::Pouting => exhale(ui, rect, elapsed as f32),
        Expression::Pleading | Expression::Hungry => wet_eyes(ui, rect, expression, elapsed as f32),
        Expression::Praying => holy_light(ui, rect, elapsed as f32),
        Expression::Question => fingertip_lines(ui, rect, elapsed as f32),
        Expression::Wink => wink_stars(ui, rect, elapsed as f32),
        _ => {}
    }
}

fn envelope(progress: f32) -> f32 {
    if !(0.0..1.0).contains(&progress) {
        return 0.0;
    }
    // Floating-point sin(PI) can be negative; fractional powers must stay nonnegative.
    (std::f32::consts::PI * progress).sin().max(0.0).powf(0.7)
}

fn kiss_print(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let Some(texture) = super::texture(
        ui.ctx(),
        "flirty-kiss",
        include_bytes!("../../../../assets/themes/moonveil/overlays/flirty-kiss.png"),
    ) else {
        return;
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let approach = (elapsed / 0.25).min(1.0);
    let press = 1.0 + 0.07 * (elapsed * 14.0).sin() * (-elapsed * 6.0).exp();
    let side = rect.width() * 0.31 * (0.45 + 0.55 * approach) * press;
    let center =
        rect.min + rect.size() * Vec2::new(0.475 - approach * 0.025, 0.50 + approach * 0.045);
    let opacity = (elapsed / 0.1).min(1.0) * ((2.2 - elapsed) / 0.55).min(1.0);
    let mut mesh = egui::Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(
        Rect::from_center_size(center, Vec2::splat(side)),
        Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(opacity),
    );
    // Counter the stamp's baked tilt to follow Flirty's upward-right mouth angle.
    mesh.rotate(egui::emath::Rot2::from_angle(-0.45), center);
    painter.add(mesh);
}

fn necklace_glow(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    let center = rect.min + Vec2::new(291.0, 296.0) * scale;
    let strength = envelope(elapsed / 2.2) * (0.9 + 0.1 * (elapsed * 6.0).sin());
    let mut mesh = egui::Mesh::default();
    for (extent, color) in [
        (
            Vec2::new(17.0, 22.0),
            Color32::from_rgb(236, 34, 91).gamma_multiply(strength * 0.4),
        ),
        (
            Vec2::new(5.0, 9.0),
            Color32::from_rgb(255, 144, 180).gamma_multiply(strength * 0.65),
        ),
    ] {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, color);
        for index in 0..32 {
            let angle = index as f32 * std::f32::consts::TAU / 32.0;
            mesh.colored_vertex(
                center + Vec2::angled(angle) * extent * scale,
                Color32::TRANSPARENT,
            );
            mesh.add_triangle(base, base + index + 1, base + (index + 1) % 32 + 1);
        }
    }
    painter.add(mesh);
}

fn blood_drips(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.4).contains(&elapsed) {
        return;
    }
    let Some(texture) = super::texture(
        ui.ctx(),
        "fed-blood",
        include_bytes!("../../../../assets/themes/moonveil/overlays/fed-blood.png"),
    ) else {
        return;
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let fade = (elapsed / 0.12).min(1.0) * ((2.4 - elapsed) / 0.45).min(1.0);
    let mut mesh = egui::Mesh::with_texture(texture.id());
    // Let different rivulets descend at slightly different rates from the top rim.
    for row in 0..=8 {
        let y = row as f32 / 8.0;
        for column in 0..=32 {
            let x = column as f32 / 32.0;
            let speed = 1.25 + 0.3 * (x * 19.0).sin();
            let flow = (elapsed / speed).clamp(0.0, 1.0);
            let depth = 0.08 + 0.92 * (1.0 - (1.0 - flow).powi(2));
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.min + rect.size() * Vec2::new(x, y * depth),
                uv: egui::pos2(x, y),
                color: Color32::WHITE.gamma_multiply(fade),
            });
            if row < 8 && column < 32 {
                let vertex = row * 33 + column;
                mesh.add_triangle(vertex, vertex + 1, vertex + 33);
                mesh.add_triangle(vertex + 1, vertex + 34, vertex + 33);
            }
        }
    }
    painter.add(mesh);
}

fn cheek_glow(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let strength = envelope(elapsed / 2.2);
    let scale = rect.width() / 512.0;
    let mut mesh = egui::Mesh::default();
    let cheeks = match expression {
        Expression::Drunk => [
            (Vec2::new(230.0, 166.0), Vec2::new(21.0, 11.0)),
            (Vec2::new(307.0, 163.0), Vec2::new(18.0, 10.0)),
        ],
        Expression::Embarrassed => [
            (Vec2::new(214.0, 201.0), Vec2::new(14.0, 8.0)),
            (Vec2::new(270.0, 190.0), Vec2::new(11.0, 8.0)),
        ],
        _ => [
            (Vec2::new(275.0, 184.0), Vec2::new(19.0, 10.0)),
            (Vec2::new(332.0, 176.0), Vec2::new(7.0, 9.0)),
        ],
    };
    for (center, extent) in cheeks {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(
            rect.min + center * scale,
            Color32::from_rgb(255, 110, 151).gamma_multiply(strength * 0.38),
        );
        for index in 0..32 {
            let angle = index as f32 * std::f32::consts::TAU / 32.0;
            mesh.colored_vertex(
                rect.min + (center + Vec2::angled(angle) * extent) * scale,
                Color32::TRANSPARENT,
            );
            mesh.add_triangle(base, base + index + 1, base + (index + 1) % 32 + 1);
        }
    }
    painter.add(mesh);
}

fn fingertip_lines(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    let center = rect.min + Vec2::new(267.0, 342.0) * scale;
    let strength = (elapsed / 0.15).min(1.0) * ((2.2 - elapsed) / 0.35).min(1.0);
    let pulse = 0.85 + 0.15 * (elapsed * 10.0).cos();
    for side in [-1.0, 1.0] {
        for index in -1..=1 {
            let direction = Vec2::new(index as f32 * 0.6, side).normalized();
            let start = center + direction * (10.0 * scale);
            let end =
                center + direction * ((19.0 + if index == 0 { 3.0 } else { 0.0 }) * scale * pulse);
            painter.line_segment(
                [start, end],
                egui::Stroke::new(
                    (1.6 * scale).max(0.7),
                    Color32::from_rgb(102, 44, 68).gamma_multiply(strength),
                ),
            );
        }
    }
}

fn scratches(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    for index in 0..3 {
        let age = elapsed - index as f32 * 0.065;
        if age <= 0.0 || age >= 1.0 {
            continue;
        }
        let offset = Vec2::new(index as f32 * 25.0, index as f32 * 23.0);
        let start = rect.min + (Vec2::new(367.0, 63.0) + offset) * scale;
        let direction = Vec2::new(-285.0, 316.0) * scale;
        let end = start + direction * (age / 0.16).min(1.0);
        let middle = start.lerp(end, 0.58);
        let normal = Vec2::new(-direction.y, direction.x).normalized() * 6.5 * scale;
        let opacity = ((1.0 - age) / 0.55).min(1.0);
        painter.add(egui::Shape::convex_polygon(
            vec![start, middle + normal, end, middle - normal],
            Color32::from_rgb(168, 27, 56).gamma_multiply(opacity),
            egui::Stroke::NONE,
        ));
        painter.line_segment(
            [start, end],
            egui::Stroke::new(
                (1.25 * scale).max(0.65),
                Color32::from_rgb(255, 198, 204).gamma_multiply(opacity * 0.8),
            ),
        );
    }
}

fn wet_eyes(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let Some(texture) = super::texture(ui.ctx(), &format!("{expression:?}"), expression.bytes())
    else {
        return;
    };
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    let strength = envelope(elapsed / 2.2);
    let eyes = match expression {
        Expression::Hungry => [(222.0, 174.0), (289.0, 159.0)],
        _ => [(228.0, 181.0), (301.0, 171.0)],
    };
    for (x, y) in eyes {
        let center = Vec2::new(x, y);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.vertices.push(egui::epaint::Vertex {
            pos: rect.min + center * scale,
            uv: ((center + Vec2::new((elapsed * 5.0).sin() * 0.7, 0.0)) / 512.0).to_pos2(),
            color: Color32::WHITE.gamma_multiply(strength * 0.65),
        });
        for index in 0..32 {
            let angle = index as f32 * std::f32::consts::TAU / 32.0;
            let point = center + Vec2::angled(angle) * Vec2::new(9.0, 5.0);
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.min + point * scale,
                uv: (point / 512.0).to_pos2(),
                color: Color32::TRANSPARENT,
            });
            mesh.add_triangle(0, index + 1, (index + 1) % 32 + 1);
        }
        painter.add(mesh);
        let rim = (0..=16)
            .map(|index| {
                let angle = index as f32 * std::f32::consts::PI / 16.0;
                rect.min + (center + Vec2::new(angle.cos() * 9.0, angle.sin() * 4.5)) * scale
            })
            .collect();
        painter.add(egui::Shape::line(
            rim,
            egui::Stroke::new(
                scale.max(0.55),
                Color32::from_rgb(218, 235, 255).gamma_multiply(strength * 0.75),
            ),
        ));
        for (offset, radius) in [(Vec2::new(-3.0, -2.0), 1.7), (Vec2::new(4.0, 1.5), 0.9)] {
            painter.circle_filled(
                rect.min + (center + offset) * scale,
                radius * scale,
                Color32::WHITE.gamma_multiply(strength * 0.9),
            );
        }
    }
}

fn holy_light(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.4).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let strength = envelope(elapsed / 2.4);
    let mut mesh = egui::Mesh::default();
    for (offset, spread, brightness) in
        [(0.0, 0.46, 0.32), (-0.075, 0.19, 0.2), (0.08, 0.15, 0.21)]
    {
        let base = mesh.vertices.len() as u32;
        for row in 0..=8 {
            let depth = row as f32 / 8.0;
            let width = 0.065 + spread * depth;
            let center = 0.52 + offset + (elapsed * 0.8 + offset * 10.0).sin() * 0.012 * depth;
            for column in 0..=8 {
                let across = column as f32 / 4.0 - 1.0;
                let feather = (1.0 - across * across).powi(2);
                let alpha = strength * brightness * feather * (1.0 - depth).powf(0.7);
                mesh.colored_vertex(
                    rect.min + rect.size() * Vec2::new(center + width * across, depth),
                    Color32::from_rgb(255, 241, 199).gamma_multiply(alpha),
                );
                if row < 8 && column < 8 {
                    let vertex = base + row * 9 + column;
                    mesh.add_triangle(vertex, vertex + 1, vertex + 9);
                    mesh.add_triangle(vertex + 1, vertex + 10, vertex + 9);
                }
            }
        }
    }
    painter.add(mesh);
}

fn wink_stars(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    // Outer corner of the closed eye, on the viewer's left in wink.png.
    let origin = rect.min + Vec2::new(201.0, 175.0) * scale;
    let mut mesh = egui::Mesh::default();
    for (index, (delay, travel, radius)) in [
        (0.0, Vec2::new(-49.0, -23.0), 7.0),
        (0.12, Vec2::new(-70.0, -5.0), 5.0),
        (0.25, Vec2::new(-37.0, -43.0), 6.0),
        (0.36, Vec2::new(-48.0, 11.0), 4.0),
    ]
    .into_iter()
    .enumerate()
    {
        let progress = (elapsed - delay) / 1.1;
        if !(0.0..1.0).contains(&progress) {
            continue;
        }
        let strength = envelope(progress);
        let distance = 1.0 - (1.0 - progress).powi(2);
        let center =
            origin + (travel * distance + Vec2::new(0.0, progress * progress * 8.0)) * scale;
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, Color32::WHITE.gamma_multiply(strength));
        for point in 0..10 {
            let angle = point as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::FRAC_PI_2
                + progress * (index as f32 - 1.5) * 0.7;
            let size =
                radius * scale * (0.55 + 0.45 * strength) * if point % 2 == 0 { 1.0 } else { 0.42 };
            let color = if point % 2 == 0 {
                Color32::from_rgb(255, 239, 206)
            } else {
                Color32::from_rgb(218, 148, 178)
            };
            mesh.colored_vertex(
                center + Vec2::angled(angle) * size,
                color.gamma_multiply(strength),
            );
            mesh.add_triangle(base, base + point + 1, base + (point + 1) % 10 + 1);
        }
    }
    painter.add(mesh);
}

fn fatigue_lines(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    let fade = (elapsed / 0.22).min(1.0) * ((2.2 - elapsed) / 0.4).min(1.0);
    // Follow each upper eyelid while keeping the fatigue strokes vertical.
    for (left, bottom) in [(195.0, 182.0), (266.0, 157.0)] {
        for index in 0..7 {
            let x = left + index as f32 * 3.8;
            let end_y = bottom - index as f32 * 1.3;
            let length = 8.0 + ((index * 7 + 3) % 9) as f32;
            let start = rect.min + Vec2::new(x, end_y - length) * scale;
            let end = rect.min + Vec2::new(x, end_y - 1.5) * scale;
            painter.line_segment(
                [start, end],
                egui::Stroke::new(
                    (0.9 * scale).max(0.55),
                    Color32::from_rgb(24, 17, 28).gamma_multiply(fade * 0.78),
                ),
            );
        }
    }
}

fn cat_play(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..2.2).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let scale = rect.width() / 512.0;
    let strength = (elapsed / 0.18).min(1.0) * ((2.2 - elapsed) / 0.4).min(1.0);
    let ink = Color32::from_rgb(30, 20, 31).gamma_multiply(strength);
    let at = |point: Vec2| rect.min + point * scale;
    for (start, end) in [
        ((226.0, 186.0), (184.0, 165.0)),
        ((226.0, 189.0), (181.0, 174.0)),
        ((227.0, 192.0), (182.0, 183.0)),
        ((281.0, 176.0), (323.0, 158.0)),
        ((282.0, 179.0), (326.0, 167.0)),
        ((283.0, 182.0), (328.0, 176.0)),
    ] {
        let start = Vec2::from(start);
        let end = Vec2::from(end);
        painter.line_segment(
            [
                at(start),
                at(start + (end - start) * (elapsed / 0.25).min(1.0)),
            ],
            egui::Stroke::new((1.7 * scale).max(0.7), ink),
        );
    }
    // Her raised right hand is on the viewer's left in cute.png.
    if let Some(texture) = super::texture(
        ui.ctx(),
        "cute-paw",
        include_bytes!("../../../../assets/themes/moonveil/overlays/cute-paw.png"),
    ) {
        let center = at(Vec2::new(168.0, 235.0));
        let pop = 1.0 + 0.05 * (elapsed * 9.0).sin() * (1.0 - elapsed / 2.2);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.add_rect_with_uv(
            Rect::from_center_size(center, Vec2::splat(60.0 * scale * pop)),
            Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
            Color32::WHITE.gamma_multiply(strength),
        );
        mesh.rotate(egui::emath::Rot2::from_angle(0.08), center);
        painter.add(mesh);
    }
}

fn eye_glow(ui: &egui::Ui, rect: Rect, expression: Expression, elapsed: f32) {
    let duration = duration(expression).unwrap_or(0.0) as f32;
    if !(0.0..duration).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let (eyes, tilt) = match expression {
        Expression::Love => ([(221.0, 164.0), (287.0, 148.0)], -0.22),
        Expression::Energetic => ([(221.0, 165.0), (287.0, 149.0)], -0.22),
        Expression::Crazy => ([(220.0, 174.0), (280.0, 149.0)], -0.38),
        Expression::Killer => ([(225.0, 182.0), (290.0, 174.0)], -0.12),
        _ => return,
    };
    let strength = envelope(elapsed / duration) * (0.9 + 0.1 * (elapsed * 7.0).sin());
    let rose = expression == Expression::Love;
    let outer = if expression == Expression::Energetic {
        Color32::from_rgb(250, 208, 80)
    } else if rose {
        Color32::from_rgb(249, 111, 170)
    } else {
        Color32::from_rgb(215, 8, 36)
    };
    let inner = if expression == Expression::Energetic {
        Color32::from_rgb(255, 250, 196)
    } else if rose {
        Color32::from_rgb(255, 184, 210)
    } else {
        Color32::from_rgb(255, 25, 43)
    };
    let scale = rect.width() / 512.0;
    let rotation = egui::emath::Rot2::from_angle(tilt);
    let mut mesh = egui::Mesh::default();
    for (x, y) in eyes {
        let center = rect.min + Vec2::new(x, y) * scale;
        for (extent, color) in [
            (Vec2::new(17.0, 7.0), outer.gamma_multiply(strength * 0.6)),
            (Vec2::new(6.0, 4.0), inner.gamma_multiply(strength * 0.85)),
        ] {
            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(center, color);
            for index in 0..32 {
                let angle = index as f32 * std::f32::consts::TAU / 32.0;
                let offset = rotation * (Vec2::angled(angle) * extent * scale);
                mesh.colored_vertex(center + offset, Color32::TRANSPARENT);
                mesh.add_triangle(base, base + index + 1, base + (index + 1) % 32 + 1);
            }
        }
    }
    painter.add(mesh);
}

fn anger_mark(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    if !(0.0..1.6).contains(&elapsed) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let fade = (elapsed / 0.1).min(1.0) * ((1.6 - elapsed) / 0.3).min(1.0);
    let pulse = 1.0 + 0.09 * (elapsed * 17.0).sin();
    let scale = rect.width() / 512.0 * pulse * (elapsed / 0.14).min(1.0);
    let center = rect.min + rect.size() * Vec2::new(0.57, 0.25);
    for [a, b, c] in [
        [
            Vec2::new(-8.0, -19.0),
            Vec2::new(-3.0, -1.0),
            Vec2::new(-22.0, 0.0),
        ],
        [
            Vec2::new(8.0, -19.0),
            Vec2::new(3.0, -1.0),
            Vec2::new(22.0, 0.0),
        ],
        [
            Vec2::new(-16.0, 13.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(13.0, 23.0),
        ],
    ] {
        let points: Vec<_> = (0..=16)
            .map(|index| {
                let t = index as f32 / 16.0;
                center + (a * (1.0 - t).powi(2) + b * (2.0 * (1.0 - t) * t) + c * t * t) * scale
            })
            .collect();
        for (width, color) in [
            (9.0, Color32::from_rgb(49, 14, 25)),
            (5.8, Color32::from_rgb(206, 32, 54)),
        ] {
            painter.add(egui::Shape::line(
                points.clone(),
                egui::Stroke::new(width * scale, color.gamma_multiply(fade)),
            ));
        }
    }
}

fn exhale(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let progress = elapsed / 1.4;
    if !(0.0..1.0).contains(&progress) {
        return;
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let strength = envelope(progress);
    let scale = rect.width() / 512.0;
    // Pouting faces left; the puff leaves her pursed lips at (202, 183).
    let mouth = rect.min + Vec2::new(202.0, 183.0) * scale;
    let travel = 1.0 - (1.0 - progress).powi(3);
    let Some(texture) = super::texture(
        ui.ctx(),
        "pouting-exhale",
        include_bytes!("../../../../assets/themes/moonveil/overlays/pouting-exhale.png"),
    ) else {
        return;
    };
    let side = 49.0 * scale * (0.75 + 0.25 * travel);
    let opening = mouth + Vec2::new(-travel * 12.0, -travel * 2.0) * scale;
    let min = opening - Vec2::new(0.77, 0.34) * side;
    let mut mesh = egui::Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(
        Rect::from_min_size(min, Vec2::splat(side)),
        Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(strength),
    );
    mesh.rotate(egui::emath::Rot2::from_angle(0.45), opening);
    painter.add(mesh);
}

fn confetti(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let colors = [
        Color32::from_rgb(221, 222, 239),
        Color32::from_rgb(157, 39, 75),
        Color32::from_rgb(228, 150, 175),
        Color32::from_rgb(173, 152, 203),
        Color32::from_rgb(244, 220, 178),
    ];
    let mut mesh = egui::Mesh::default();
    // Deterministic trajectories avoid random jitter and per-frame particle state.
    for index in 0..64 {
        let random = |channel: u32| {
            let mut bits = (index as u32 + 1).wrapping_mul(0x9e37_79b9) ^ channel;
            bits = (bits ^ (bits >> 16)).wrapping_mul(0x7feb_352d);
            bits = (bits ^ (bits >> 15)).wrapping_mul(0x846c_a68b);
            (bits ^ (bits >> 16)) as f32 / u32::MAX as f32
        };
        let phase = random(1) * std::f32::consts::TAU;
        let age = elapsed - random(2) * 1.05;
        if age <= 0.0 {
            continue;
        }
        let sway = 0.012 + random(3) * 0.038;
        let flutter = 2.0 + random(4) * 5.0;
        let x = 0.04
            + random(5) * 0.92
            + ((phase + age * flutter).sin() - phase.sin()) * sway
            + (random(6) - 0.5) * 0.07 * age;
        let y = -0.06 + (0.32 + random(7) * 0.42) * age + (0.07 + random(8) * 0.1) * age * age;
        let center = rect.min + rect.size() * Vec2::new(x, y);
        let opacity = (age * 10.0).min(1.0) * ((2.4 - elapsed) / 0.6).clamp(0.0, 1.0);
        let color = colors[index % colors.len()].gamma_multiply(opacity);
        let spin = (1.5 + random(9) * 5.5) * if random(10) < 0.5 { -1.0 } else { 1.0 };
        let rotation = egui::emath::Rot2::from_angle(phase + age * spin);
        let width = rect.width() * (0.006 + random(11) * 0.01);
        let flip = (age * (4.0 + random(12) * 8.0) + phase).cos();
        let length = width * (1.1 + random(13) * 2.3);
        let base = mesh.vertices.len() as u32;
        // Folded foil has a bright crease and a darker underside as it tumbles.
        for (point, shade) in [
            (Vec2::new(-width * flip, -length), 0.65),
            (Vec2::new(width * flip, -length * 0.7), 0.9),
            (Vec2::new(width * flip * 0.6, length), 0.6),
            (Vec2::new(-width * flip, length * 0.7), 0.8),
            (Vec2::ZERO, 1.0),
        ] {
            mesh.colored_vertex(center + rotation * point, color.gamma_multiply(shade));
        }
        for corner in 0..4 {
            mesh.add_triangle(base + 4, base + corner, base + (corner + 1) % 4);
        }
    }
    painter.add(mesh);
}

fn excited(ui: &egui::Ui, rect: Rect, elapsed: f32) {
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
    let at = |x, y| rect.min + rect.size() * Vec2::new(x, y);
    if let Some(texture) = super::texture(
        ui.ctx(),
        "excited-sparkles",
        include_bytes!("../../../../assets/themes/moonveil/overlays/excited-sparkles.png"),
    ) {
        for (x, y, size, delay, angle) in [
            (0.23, 0.24, 0.19, 0.0, -0.16),
            (0.79, 0.19, 0.23, 0.18, 0.13),
            (0.12, 0.48, 0.15, 0.42, -0.10),
            (0.88, 0.43, 0.17, 0.62, 0.18),
        ] {
            let progress = (elapsed - delay) / 1.15;
            if !(0.0..1.0).contains(&progress) {
                continue;
            }
            let strength = envelope(progress);
            let center = at(x, y - progress * 0.035);
            let side = rect.width() * size * (0.72 + strength * 0.28);
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                Rect::from_center_size(center, Vec2::splat(side)),
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE.gamma_multiply(strength),
            );
            mesh.rotate(
                egui::emath::Rot2::from_angle(angle * (progress - 0.5)),
                center,
            );
            painter.add(mesh);
        }
    }

    // Exact centers of the existing pupil stars in excited.png (512 × 512).
    for (x, y, phase) in [(225.0, 162.0, 0.0), (293.0, 162.0, 0.22)] {
        let strength = envelope(elapsed / 1.8) * (0.72 + 0.28 * (elapsed * 12.0 + phase).cos());
        let center = at(x / 512.0, y / 512.0);
        eye_glare(&painter, center, rect.width() / 512.0, elapsed, phase);
        let radius = rect.width() / 512.0 * (2.3 + 1.2 * strength);
        let color = Color32::from_rgb(255, 250, 224).gamma_multiply(strength);
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(center, color);
        for index in 0..8 {
            let angle = index as f32 * std::f32::consts::FRAC_PI_4;
            let length = if index % 2 == 0 {
                radius
            } else {
                radius * 0.22
            };
            mesh.colored_vertex(center + Vec2::angled(angle) * length, color);
            mesh.add_triangle(0, index + 1, (index + 1) % 8 + 1);
        }
        painter.add(mesh);
        painter.circle_filled(
            center,
            radius * 0.19,
            Color32::WHITE.gamma_multiply(strength),
        );
    }
}

fn eye_glare(painter: &egui::Painter, center: Pos2, scale: f32, elapsed: f32, phase: f32) {
    let flash = envelope((elapsed - 0.12 - phase * 0.15) / 1.2);
    if flash <= 0.0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    for (extent, color) in [
        (
            Vec2::new(12.0, 3.8),
            Color32::from_rgb(217, 226, 255).gamma_multiply(flash * 0.42),
        ),
        (
            Vec2::new(14.0, 0.9),
            Color32::from_rgb(255, 252, 234).gamma_multiply(flash * 0.85),
        ),
    ] {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, color);
        for index in 0..24 {
            let angle = index as f32 * std::f32::consts::TAU / 24.0;
            let offset = Vec2::angled(angle) * extent * scale * (0.75 + flash * 0.25);
            mesh.colored_vertex(center + offset, Color32::TRANSPARENT);
            mesh.add_triangle(base, base + index + 1, base + (index + 1) % 24 + 1);
        }
    }
    painter.add(mesh);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_is_finite_at_and_outside_its_boundaries() {
        for progress in [-1.0, 0.0, f32::EPSILON, 0.5, 1.0 - f32::EPSILON, 1.0, 2.0] {
            assert!((0.0..=1.0).contains(&envelope(progress)));
        }
        assert_eq!(envelope(1.0), 0.0);
        assert_eq!(envelope(2.0), 0.0);
    }

    #[test]
    fn overlays_render_through_their_fades_without_invalid_geometry() {
        let ctx = egui::Context::default();
        for expression in Expression::ALL
            .iter()
            .copied()
            .filter(|value| duration(*value).is_some())
        {
            for frame in 0..=180 {
                let output = ctx.run_ui(egui::RawInput::default(), |ui| {
                    paint(
                        ui,
                        Rect::from_min_size(Pos2::ZERO, Vec2::splat(300.0)),
                        expression,
                        frame as f64 / 60.0,
                    );
                });
                for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
                    if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                        assert!(mesh.is_valid(), "{expression:?} at frame {frame}");
                        assert!(
                            mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()),
                            "{expression:?} at frame {frame}"
                        );
                    }
                }
            }
        }
    }
}
