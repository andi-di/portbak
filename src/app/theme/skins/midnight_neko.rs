//! Midnight Neko: matte gothic panels, crimson accents and an illustrated catgirl companion.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static MIDNIGHT_NEKO: Skin = Skin {
    id: "midnight-neko",
    name: "Midnight Neko",
    description: "Matte gothic panels, crimson hearts and an emerald-eyed catgirl companion.",
    dark: MOONLIGHT,
    light: PALE_STONE,
    fonts: Fonts {
        display: Some(include_bytes!(
            "../../../../assets/fonts/Cinzel-Regular.ttf"
        )),
        body: Some(include_bytes!(
            "../../../../assets/fonts/Selawik-Regular.ttf"
        )),
    },
    typography: Typography {
        heading: 20.0,
        body: 16.0,
        small: 14.5,
        label: 14.5,
        display_scale: 1.15,
        display_drop: 0.0,
        display_glow: 0.0,
        capitals: false,
    },
    metrics: Metrics {
        border: 1.0,
        lift: 0.0,
        shadow: 2,
        shadow_blur: 6,
        radius: 4,
    },
    effects: Effects {
        scanlines: false,
        grid: false,
        stripes: false,
        embers: false,
        vignette: false,
        wallpaper: true,
        bubbles: false,
        hearts: true,
        opacity: 1.0,
        title_bar: TitleBar::Gothic,
    },
    finish: Finish::Gothic,
    wording: Wording {
        mascot: Sprite::Floppy,
        tagline: "",
        welcome_title: "Your files, forever cherished",
        welcome_prompt: "Create your first backup job",
        ready: "Keeping watch",
    },
};

const MOONLIGHT: Palette = Palette {
    background: Color32::from_rgb(0x17, 0x15, 0x1c),
    grid: Color32::from_rgb(0x21, 0x1b, 0x27),
    surface: Color32::from_rgb(0x21, 0x1c, 0x25),
    raised: Color32::from_rgb(0x30, 0x23, 0x2e),
    sunken: Color32::from_rgb(0x15, 0x12, 0x18),
    outline: Color32::from_rgb(0x57, 0x45, 0x4f),
    highlight: Color32::from_rgb(0x4c, 0x35, 0x45),
    shade: Color32::from_rgb(0x12, 0x0e, 0x15),
    ink: Color32::from_rgb(0xf4, 0xe9, 0xe4),
    dim: Color32::from_rgb(0xbc, 0xb0, 0xb8),
    accent: Color32::from_rgb(0x8e, 0x24, 0x45),
    accent_ink: Color32::from_rgb(0xff, 0xf5, 0xee),
    hot: Color32::from_rgb(0xdf, 0x8b, 0x9e),
    gold: Color32::from_rgb(0xde, 0xd2, 0xc0),
    good: Color32::from_rgb(0x86, 0xc7, 0xa3),
    bad: Color32::from_rgb(0xcf, 0x53, 0x66),
    metal: Color32::from_rgb(0xbc, 0xb0, 0xbc),
    paper: Color32::from_rgb(0xff, 0xf5, 0xef),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0x23, 0x1a, 0x29),
    vignette: Color32::from_rgb(0x00, 0x00, 0x00),
};

const PALE_STONE: Palette = Palette {
    background: Color32::from_rgb(0xe6, 0xdf, 0xda),
    grid: Color32::from_rgb(0xda, 0xcb, 0xd2),
    surface: Color32::from_rgb(0xf5, 0xee, 0xe7),
    raised: Color32::from_rgb(0xff, 0xfa, 0xf4),
    sunken: Color32::from_rgb(0xe6, 0xdb, 0xd6),
    outline: Color32::from_rgb(0x9c, 0x84, 0x88),
    highlight: Color32::from_rgb(0xea, 0xd1, 0xd5),
    shade: Color32::from_rgb(0xc7, 0xb7, 0xbd),
    ink: Color32::from_rgb(0x30, 0x22, 0x2c),
    dim: Color32::from_rgb(0x6c, 0x5b, 0x62),
    accent: Color32::from_rgb(0x8c, 0x21, 0x3f),
    accent_ink: Color32::from_rgb(0xff, 0xf5, 0xef),
    hot: Color32::from_rgb(0x85, 0x25, 0x3c),
    gold: Color32::from_rgb(0x58, 0x43, 0x4a),
    good: Color32::from_rgb(0x28, 0x63, 0x4a),
    bad: Color32::from_rgb(0xb2, 0x29, 0x47),
    metal: Color32::from_rgb(0x72, 0x66, 0x71),
    paper: Color32::from_rgb(0xff, 0xff, 0xff),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0xe6, 0xdf, 0xda),
    vignette: Color32::from_rgb(0x00, 0x00, 0x00),
};
