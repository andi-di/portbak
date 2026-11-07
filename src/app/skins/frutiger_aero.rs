//! Frutiger Aero: glossy gel buttons and layered glass over a sky of auroras, hills and bubbles.

use crate::app::sprites::Sprite;
use crate::app::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static FRUTIGER_AERO: Skin = Skin {
    id: "frutiger-aero",
    name: "Frutiger Aero",
    description:
        "Glossy glass, bubbles and sunbeams over sky, sea and hills. Night brings an aurora.",
    dark: AURORA,
    light: CLEAR_SKY,
    fonts: Fonts {
        display: Some(include_bytes!("../../../assets/fonts/Selawik-Semibold.ttf")),
        body: Some(include_bytes!("../../../assets/fonts/Selawik-Regular.ttf")),
    },
    typography: Typography {
        heading: 21.0,
        body: 17.0,
        small: 14.5,
        label: 14.5,
        display_scale: 1.3,
        // A soft halo lifts text off the glass instead of a hard shadow.
        display_drop: 0.0,
        display_glow: 2.0,
        capitals: false,
    },
    metrics: Metrics {
        border: 1.0,
        lift: 2.0,
        shadow: 4,
        shadow_blur: 18,
        radius: 10,
    },
    effects: Effects {
        scanlines: false,
        grid: false,
        stripes: false,
        embers: false,
        vignette: false,
        wallpaper: true,
        bubbles: true,
        opacity: 0.24,
        title_bar: TitleBar::Glass,
    },
    finish: Finish::Glass,
    wording: Wording {
        mascot: Sprite::Floppy,
        tagline: "Clear skies for your files",
        welcome_title: "A fresh start",
        welcome_prompt: "Create your first backup job",
        ready: "All clear",
        busy: "Working",
        cursor: "",
    },
};

const AURORA: Palette = Palette {
    background: Color32::from_rgb(0x05, 0x4a, 0x66),
    grid: Color32::from_rgb(0x0c, 0x22, 0x33),
    surface: Color32::from_rgb(0x0a, 0x3c, 0x62),
    raised: Color32::from_rgb(0x18, 0x6a, 0x9a),
    sunken: Color32::from_rgb(0x03, 0x1c, 0x34),
    outline: Color32::from_rgb(0x00, 0x06, 0x14),
    highlight: Color32::from_rgb(0x1f, 0x8a, 0xc4),
    shade: Color32::from_rgb(0x16, 0x5c, 0x86),
    ink: Color32::from_rgb(0xee, 0xfa, 0xff),
    dim: Color32::from_rgb(0xa2, 0xd4, 0xee),
    accent: Color32::from_rgb(0x12, 0xa8, 0xff),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0x2a, 0xf5, 0xd8),
    gold: Color32::from_rgb(0xff, 0xd2, 0x3a),
    good: Color32::from_rgb(0x4c, 0xe8, 0x5a),
    bad: Color32::from_rgb(0xff, 0x4e, 0x52),
    metal: Color32::from_rgb(0x9a, 0xc6, 0xe0),
    paper: Color32::from_rgb(0xf4, 0xfc, 0xff),
    scanline: Color32::TRANSPARENT,
    sky: Color32::from_rgb(0x01, 0x05, 0x16),
    vignette: Color32::TRANSPARENT,
};

const CLEAR_SKY: Palette = Palette {
    background: Color32::from_rgb(0xb4, 0xec, 0xff),
    grid: Color32::from_rgb(0xc4, 0xe6, 0xf5),
    surface: Color32::from_rgb(0xe6, 0xfa, 0xff),
    raised: Color32::from_rgb(0xff, 0xff, 0xff),
    sunken: Color32::from_rgb(0xd2, 0xf2, 0xff),
    outline: Color32::from_rgb(0x06, 0x3f, 0x7a),
    highlight: Color32::from_rgb(0xff, 0xff, 0xff),
    shade: Color32::from_rgb(0x5a, 0xb4, 0xea),
    ink: Color32::from_rgb(0x03, 0x22, 0x44),
    dim: Color32::from_rgb(0x12, 0x4a, 0x7c),
    accent: Color32::from_rgb(0x00, 0x86, 0xe6),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0x00, 0xd2, 0xf0),
    gold: Color32::from_rgb(0xff, 0xa2, 0x00),
    good: Color32::from_rgb(0x22, 0x9a, 0x08),
    bad: Color32::from_rgb(0xe6, 0x1e, 0x2a),
    metal: Color32::from_rgb(0x84, 0xbc, 0xe2),
    paper: Color32::from_rgb(0xff, 0xff, 0xff),
    scanline: Color32::TRANSPARENT,
    sky: Color32::from_rgb(0x00, 0x4c, 0xc4),
    vignette: Color32::TRANSPARENT,
};
