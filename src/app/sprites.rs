//! Hand-drawn pixel sprites as text grids, painted as one mesh per sprite.
//!
//! Legend: `.` transparent, `K` outline, `W` paper, `M` metal, `S` shade,
//! `A` the caller's tint, `Y` gold, `G` good, `R` bad.

use super::style::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

pub type Sprite = &'static [&'static str];

pub const FLOPPY: Sprite = &[
    "KKKKKKKKKKK.",
    "KAAMMMMMAAAK",
    "KAAMKKMMAAAK",
    "KAAMKKMMAAAK",
    "KAAMMMMMAAAK",
    "KAAAAAAAAAAK",
    "KAWWWWWWWWAK",
    "KAWKKKKKKWAK",
    "KAWWWWWWWWAK",
    "KAWKKKKWWWAK",
    "KAWWWWWWWWAK",
    "KKKKKKKKKKKK",
];

pub const FOLDER: Sprite = &[
    ".KKKKK......",
    "KYYYYYKKKKK.",
    "KYYYYYYYYYYK",
    "KKKKKKKKKKKK",
    "KYWWWWWWWWYK",
    "KYYYYYYYYYYK",
    "KYYYYYYYYYYK",
    "KYYYYYYYYYYK",
    "KSSSSSSSSSSK",
    "KKKKKKKKKKKK",
];

pub const CLOCK: Sprite = &[
    "...KKKKK...",
    "..KWWWWWK..",
    ".KWWWKWWWK.",
    "KWWWWKWWWWK",
    "KWWWWKWWWWK",
    "KWWWWKKKWWK",
    "KWWWWWWWWWK",
    "KWWWWWWWWWK",
    ".KWWWWWWWK.",
    "..KWWWWWK..",
    "...KKKKK...",
];

pub const SHIELD: Sprite = &[
    "KKKKKKKKKKK",
    "KAAAAWAAAAK",
    "KAAAAWAAAAK",
    "KAAAAWAAAAK",
    "KWWWWWWWWWK",
    "KAAAAWAAAAK",
    "KAAAAWAAAAK",
    ".KAAAWAAAK.",
    ".KAAAWAAAK.",
    "..KAAWAAK..",
    "...KAWAK...",
    "....KKK....",
];

pub const TRASH: Sprite = &[
    "...KKKK...",
    "KKKKKKKKKK",
    "KWWWWWWWWK",
    "KKKKKKKKKK",
    ".KAKAKAKK.",
    ".KAKAKAKK.",
    ".KAKAKAKK.",
    ".KAKAKAKK.",
    ".KAKAKAKK.",
    ".KKKKKKKK.",
];

pub const REWIND: Sprite = &[
    "....KK....KK",
    "...KAK...KAK",
    "..KAAK..KAAK",
    ".KAAAK.KAAAK",
    "KAAAAKKAAAAK",
    ".KAAAK.KAAAK",
    "..KAAK..KAAK",
    "...KAK...KAK",
    "....KK....KK",
];

pub const PLAY: Sprite = &[
    "KK......", "KAKK....", "KAAAKK..", "KAAAAAKK", "KAAAAAAK", "KAAAAAKK", "KAAAKK..", "KAKK....",
    "KK......",
];

pub const CHEST: Sprite = &[
    ".KKKKKKKKKK.",
    "KYAAAAAAAAYK",
    "KAAAAAAAAAAK",
    "KKKKKYYKKKKK",
    "KYAAAKKAAAYK",
    "KAAAAYYAAAAK",
    "KAAAAAAAAAAK",
    "KYAAAAAAAAYK",
    "KKKKKKKKKKKK",
];

pub const PACKAGE: Sprite = &[
    ".KKKKKKKKK.",
    "KAAAAKAAAAK",
    "KKKKKKKKKKK",
    "KAAAAKAAAAK",
    "KAAAAWAAAAK",
    "KAAAAKAAAAK",
    "KAAAAWAAAAK",
    "KAAAAKAAAAK",
    "KKKKKKKKKKK",
];

pub const SUN: Sprite = &[
    ".....Y.....",
    ".Y...Y...Y.",
    "..Y.....Y..",
    "....YYY....",
    "...YYYYY...",
    "YY.YYYYY.YY",
    "...YYYYY...",
    "....YYY....",
    "..Y.....Y..",
    ".Y...Y...Y.",
    ".....Y.....",
];

pub const MOON: Sprite = &[
    "...YYYY....",
    ".YYYY......",
    ".YYY.......",
    "YYY........",
    "YYY........",
    "YYYY.......",
    ".YYYY....Y.",
    ".YYYYYYYYY.",
    "...YYYYY...",
];

pub const MONITOR: Sprite = &[
    "KKKKKKKKKKKK",
    "KMMMMMMMMMMK",
    "KMKKKKKKKKMK",
    "KMKAAAAAAKMK",
    "KMKAKAAAAKMK",
    "KMKAAAAAAKMK",
    "KMKKKKKKKKMK",
    "KMMMMMMMMMMK",
    "KKKKKKKKKKKK",
    "....KMMK....",
    "..KKKKKKKK..",
];

pub const WARNING: Sprite = &[
    ".....K.....",
    "....KYK....",
    "....KYK....",
    "...KYKYK...",
    "...KYKYK...",
    "..KYYKYYK..",
    "..KYYYYYK..",
    ".KYYYKYYYK.",
    ".KYYYYYYYK.",
    "KKKKKKKKKKK",
];

pub const POWER: Sprite = &[
    ".....AA.....",
    "..A..AA..A..",
    ".AA..AA..AA.",
    "AA...AA...AA",
    "AA........AA",
    "AA........AA",
    ".AA......AA.",
    "..AAA..AAA..",
    "...AAAAAA...",
];

pub const CHECK: Sprite = &[
    "......A", ".....AA", "A...AA.", "AA.AA..", ".AAA...", "..A....",
];

pub const CURSOR: Sprite = &["A...", "AA..", "AAA.", "AAAA", "AAA.", "AA..", "A..."];

pub const PLUS: Sprite = &[
    "...KKKK...",
    "...KAAK...",
    "...KAAK...",
    "KKKKAAKKKK",
    "KAAAAAAAAK",
    "KAAAAAAAAK",
    "KKKKAAKKKK",
    "...KAAK...",
    "...KAAK...",
    "...KKKK...",
];

pub const GEAR: Sprite = &[
    "....KKK....",
    ".KK.KMK.KK.",
    ".KMKKMKKMK.",
    "..KMMMMMK..",
    "KKKMKKKMKKK",
    "KMMMK.KMMMK",
    "KKKMKKKMKKK",
    "..KMMMMMK..",
    ".KMKKMKKMK.",
    ".KK.KMK.KK.",
    "....KKK....",
];

pub fn size(sprite: Sprite, scale: f32) -> Vec2 {
    let width = sprite.first().map_or(0, |row| row.len());
    egui::vec2(width as f32, sprite.len() as f32) * scale
}

/// Paints the sprite with its top-left corner at `origin`, snapped to whole pixels.
pub fn paint(painter: &egui::Painter, origin: Pos2, sprite: Sprite, scale: f32, tint: Color32) {
    let palette = Palette::of_ctx(painter.ctx());
    let origin = origin.round();
    let mut mesh = egui::Mesh::default();
    for (y, row) in sprite.iter().enumerate() {
        for (x, cell) in row.chars().enumerate() {
            let Some(color) = cell_color(cell, palette, tint) else {
                continue;
            };
            let min = origin + egui::vec2(x as f32, y as f32) * scale;
            mesh.add_colored_rect(Rect::from_min_size(min, egui::Vec2::splat(scale)), color);
        }
    }
    painter.add(mesh);
}

pub fn paint_centered(
    painter: &egui::Painter,
    center: Pos2,
    sprite: Sprite,
    scale: f32,
    tint: Color32,
) {
    paint(
        painter,
        center - size(sprite, scale) / 2.0,
        sprite,
        scale,
        tint,
    );
}

/// Allocates room for the sprite in the layout and paints it.
pub fn show(ui: &mut egui::Ui, sprite: Sprite, scale: f32, tint: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size(sprite, scale), egui::Sense::hover());
    paint(ui.painter(), rect.min, sprite, scale, tint);
    response
}

fn cell_color(cell: char, palette: &Palette, tint: Color32) -> Option<Color32> {
    Some(match cell {
        'K' => palette.outline,
        'W' => palette.paper,
        'M' => palette.metal,
        'S' => palette.shade,
        'A' => tint,
        'Y' => palette.gold,
        'G' => palette.good,
        'R' => palette.bad,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Sprite; 18] = [
        FLOPPY, FOLDER, CLOCK, SHIELD, TRASH, REWIND, PLAY, CHEST, PACKAGE, SUN, MOON, MONITOR,
        WARNING, POWER, CHECK, CURSOR, PLUS, GEAR,
    ];

    #[test]
    fn sprites_are_rectangular_and_use_known_colors() {
        for sprite in ALL {
            let width = sprite[0].len();
            for row in sprite {
                assert_eq!(row.len(), width, "ragged sprite row {row:?}");
                for cell in row.chars() {
                    assert!(
                        cell == '.'
                            || cell_color(cell, &crate::app::skins::default().dark, Color32::RED)
                                .is_some(),
                        "unknown sprite cell {cell:?}"
                    );
                }
            }
        }
    }
}
