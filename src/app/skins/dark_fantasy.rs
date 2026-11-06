//! Dark Fantasy: carved capitals, crimson on ash-black nights and candlelit parchment days.

use crate::app::sprites::Sprite;
use crate::app::style::{
    Effects, Fonts, IconStyle, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static DARK_FANTASY: Skin = Skin {
    id: "dark-fantasy",
    name: "Dark Fantasy",
    description: "Carved capitals, crimson embers and the long night. Day brings ashen parchment.",
    dark: ABYSS,
    light: PARCHMENT,
    fonts: Fonts {
        display: Some(include_bytes!("../../../assets/fonts/Cinzel-Regular.ttf")),
        body: Some(include_bytes!(
            "../../../assets/fonts/EBGaramond-Regular.ttf"
        )),
    },
    // Cinzel sets small next to the pixel face it replaces, so fixed display sizes grow.
    typography: Typography {
        heading: 22.0,
        body: 20.0,
        small: 17.0,
        label: 14.0,
        display_scale: 1.4,
        // A hairline engraving; a deep hard shadow would double the thin serifs.
        display_drop: 0.04,
    },
    metrics: Metrics {
        border: 1.0,
        lift: 2.0,
        shadow: 6,
    },
    effects: Effects {
        scanlines: false,
        grid: false,
        stripes: false,
        embers: true,
        vignette: true,
        title_bar: TitleBar::Engraved,
    },
    icons: IconStyle::Engraved,
    wording: Wording {
        mascot: Sprite::Power,
        tagline: "KEEPER OF FORGOTTEN FILES",
        welcome_title: "THE VAULT LIES EMPTY",
        welcome_prompt: "KINDLE THE FIRST FLAME",
        ready: "KINDLED",
        busy: "FORGING",
        cursor: "",
    },
};

const ABYSS: Palette = Palette {
    background: Color32::from_rgb(0x0b, 0x09, 0x09),
    grid: Color32::from_rgb(0x14, 0x0f, 0x0f),
    surface: Color32::from_rgb(0x1c, 0x15, 0x15),
    raised: Color32::from_rgb(0x2b, 0x1f, 0x1e),
    sunken: Color32::from_rgb(0x07, 0x05, 0x05),
    outline: Color32::from_rgb(0x00, 0x00, 0x00),
    highlight: Color32::from_rgb(0x45, 0x22, 0x22),
    shade: Color32::from_rgb(0x3a, 0x2a, 0x26),
    ink: Color32::from_rgb(0xe8, 0xdc, 0xc8),
    dim: Color32::from_rgb(0x8f, 0x80, 0x75),
    accent: Color32::from_rgb(0xb3, 0x12, 0x1f),
    accent_ink: Color32::from_rgb(0xf5, 0xe9, 0xd6),
    hot: Color32::from_rgb(0xe0, 0x46, 0x1b),
    gold: Color32::from_rgb(0xc9, 0xa2, 0x4a),
    good: Color32::from_rgb(0x6f, 0x8f, 0x4e),
    bad: Color32::from_rgb(0xd4, 0x20, 0x2c),
    metal: Color32::from_rgb(0x8c, 0x8a, 0x90),
    paper: Color32::from_rgb(0xf5, 0xe9, 0xd6),
    scanline: Color32::TRANSPARENT,
    vignette: Color32::from_rgba_premultiplied(0, 0, 0, 190),
};

const PARCHMENT: Palette = Palette {
    background: Color32::from_rgb(0xd6, 0xcb, 0xb6),
    grid: Color32::from_rgb(0xcc, 0xbf, 0xa8),
    surface: Color32::from_rgb(0xe8, 0xdf, 0xcc),
    raised: Color32::from_rgb(0xf2, 0xea, 0xd9),
    sunken: Color32::from_rgb(0xfa, 0xf5, 0xea),
    outline: Color32::from_rgb(0x1a, 0x10, 0x10),
    highlight: Color32::from_rgb(0xff, 0xfa, 0xf0),
    shade: Color32::from_rgb(0xb5, 0xa7, 0x8f),
    ink: Color32::from_rgb(0x1f, 0x14, 0x12),
    dim: Color32::from_rgb(0x5e, 0x4f, 0x48),
    accent: Color32::from_rgb(0x9e, 0x10, 0x20),
    accent_ink: Color32::from_rgb(0xfa, 0xf5, 0xea),
    hot: Color32::from_rgb(0xb8, 0x3a, 0x14),
    gold: Color32::from_rgb(0xd4, 0xa5, 0x37),
    good: Color32::from_rgb(0x4a, 0x7a, 0x3a),
    bad: Color32::from_rgb(0xb3, 0x12, 0x1f),
    metal: Color32::from_rgb(0x8a, 0x85, 0x80),
    paper: Color32::from_rgb(0xfa, 0xf5, 0xea),
    scanline: Color32::TRANSPARENT,
    // Premultiplied scorched brown, darkening the parchment edges like old burns.
    vignette: Color32::from_rgba_premultiplied(0x1e, 0x0c, 0x05, 0x64),
};
