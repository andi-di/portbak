//! Icons by meaning; each skin draws them as pixel sprites or engraved vector glyphs.
//!
//! Pixel sprites are hand-drawn text grids, painted as one mesh per sprite.
//! Legend: `.` transparent, `K` outline, `W` paper, `M` metal, `S` shade,
//! `A` the caller's tint, `Y` gold, `G` good, `R` bad.

use super::glyphs;
use super::style::{Finish, Palette, Skin};
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sprite {
    Floppy,
    Folder,
    Clock,
    Shield,
    Trash,
    Rewind,
    Play,
    Chest,
    Package,
    Sun,
    Moon,
    Monitor,
    Warning,
    Power,
    Check,
    Cursor,
    Plus,
    Gear,
}

impl Sprite {
    #[cfg(test)]
    pub const ALL: [Sprite; 18] = [
        Sprite::Floppy,
        Sprite::Folder,
        Sprite::Clock,
        Sprite::Shield,
        Sprite::Trash,
        Sprite::Rewind,
        Sprite::Play,
        Sprite::Chest,
        Sprite::Package,
        Sprite::Sun,
        Sprite::Moon,
        Sprite::Monitor,
        Sprite::Warning,
        Sprite::Power,
        Sprite::Check,
        Sprite::Cursor,
        Sprite::Plus,
        Sprite::Gear,
    ];

    fn bitmap(self) -> Bitmap {
        match self {
            Sprite::Floppy => FLOPPY,
            Sprite::Folder => FOLDER,
            Sprite::Clock => CLOCK,
            Sprite::Shield => SHIELD,
            Sprite::Trash => TRASH,
            Sprite::Rewind => REWIND,
            Sprite::Play => PLAY,
            Sprite::Chest => CHEST,
            Sprite::Package => PACKAGE,
            Sprite::Sun => SUN,
            Sprite::Moon => MOON,
            Sprite::Monitor => MONITOR,
            Sprite::Warning => WARNING,
            Sprite::Power => POWER,
            Sprite::Check => CHECK,
            Sprite::Cursor => CURSOR,
            Sprite::Plus => PLUS,
            Sprite::Gear => GEAR,
        }
    }
}

type Bitmap = &'static [&'static str];

const FLOPPY: Bitmap = &[
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

const FOLDER: Bitmap = &[
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

const CLOCK: Bitmap = &[
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

const SHIELD: Bitmap = &[
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

const TRASH: Bitmap = &[
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

const REWIND: Bitmap = &[
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

const PLAY: Bitmap = &[
    "KK......", "KAKK....", "KAAAKK..", "KAAAAAKK", "KAAAAAAK", "KAAAAAKK", "KAAAKK..", "KAKK....",
    "KK......",
];

const CHEST: Bitmap = &[
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

const PACKAGE: Bitmap = &[
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

const SUN: Bitmap = &[
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

const MOON: Bitmap = &[
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

const MONITOR: Bitmap = &[
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

const WARNING: Bitmap = &[
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

const POWER: Bitmap = &[
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

const CHECK: Bitmap = &[
    "......A", ".....AA", "A...AA.", "AA.AA..", ".AAA...", "..A....",
];

const CURSOR: Bitmap = &["A...", "AA..", "AAA.", "AAAA", "AAA.", "AA..", "A..."];

const PLUS: Bitmap = &[
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

const GEAR: Bitmap = &[
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

pub fn size(ctx: &egui::Context, sprite: Sprite, scale: f32) -> Vec2 {
    let grid = match Skin::of(ctx).finish {
        Finish::Pixel => {
            let bitmap = sprite.bitmap();
            let width = bitmap.first().map_or(0, |row| row.len());
            egui::vec2(width as f32, bitmap.len() as f32)
        }
        Finish::Engraved => glyphs::grid(sprite),
        Finish::Modern => glyphs::grid(sprite) * 0.8,
    };
    grid * scale
}

/// Paints the icon with its top-left corner at `origin`, snapped to whole pixels.
pub fn paint(painter: &egui::Painter, origin: Pos2, sprite: Sprite, scale: f32, tint: Color32) {
    paint_awake(painter, origin, sprite, scale, tint, 0.0);
}

/// Like `paint`, with `wake` from 0 to 1 bringing engraved icons to life, e.g. while hovered.
pub fn paint_awake(
    painter: &egui::Painter,
    origin: Pos2,
    sprite: Sprite,
    scale: f32,
    tint: Color32,
    wake: f32,
) {
    let origin = origin.round();
    match Skin::of(painter.ctx()).finish {
        Finish::Pixel => paint_bitmap(painter, origin, sprite.bitmap(), scale, tint),
        Finish::Modern => {
            let rect = Rect::from_min_size(origin, size(painter.ctx(), sprite, scale));
            super::modern::icon(painter, rect, sprite, tint);
        }
        Finish::Engraved => {
            let rect = Rect::from_min_size(origin, size(painter.ctx(), sprite, scale));
            glyphs::paint(painter, rect, sprite, tint, wake);
        }
    }
}

fn paint_bitmap(painter: &egui::Painter, origin: Pos2, bitmap: Bitmap, scale: f32, tint: Color32) {
    let palette = Palette::of_ctx(painter.ctx());
    let mut mesh = egui::Mesh::default();
    for (y, row) in bitmap.iter().enumerate() {
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
        center - size(painter.ctx(), sprite, scale) / 2.0,
        sprite,
        scale,
        tint,
    );
}

/// Allocates room for the sprite in the layout and paints it.
pub fn show(ui: &mut egui::Ui, sprite: Sprite, scale: f32, tint: Color32) -> egui::Response {
    show_awake(ui, sprite, scale, tint, 0.0)
}

/// Like `show`, with `wake` as in `paint_awake`.
pub fn show_awake(
    ui: &mut egui::Ui,
    sprite: Sprite,
    scale: f32,
    tint: Color32,
    wake: f32,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(size(ui.ctx(), sprite, scale), egui::Sense::hover());
    paint_awake(ui.painter(), rect.min, sprite, scale, tint, wake);
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

    #[test]
    fn sprites_are_rectangular_and_use_known_colors() {
        for sprite in Sprite::ALL {
            let bitmap = sprite.bitmap();
            let width = bitmap[0].len();
            for row in bitmap {
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
