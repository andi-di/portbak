//! Kawaii: candy glass, an interactive catgirl companion and a sky full of floating hearts.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static KAWAII: Skin = Skin {
    id: "kawaii",
    name: "Kawaii",
    description: "A watchful catgirl, glossy red hearts, candy glass and a subtle sparkle trail.",
    dark: MOONLIGHT,
    light: STRAWBERRY_MILK,
    fonts: Fonts {
        display: Some(include_bytes!(
            "../../../../assets/fonts/Selawik-Semibold.ttf"
        )),
        body: Some(include_bytes!(
            "../../../../assets/fonts/Selawik-Regular.ttf"
        )),
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
        radius: 18,
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
        opacity: 0.62,
        title_bar: TitleBar::Glass,
    },
    finish: Finish::Glass,
    wording: Wording {
        mascot: Sprite::Floppy,
        tagline: "",
        welcome_title: "Your files, forever cherished",
        welcome_prompt: "Create your first backup job",
        ready: "Safe and loved",
        busy: "Working",
        cursor: "",
    },
};

const MOONLIGHT: Palette = Palette {
    background: Color32::from_rgb(0x24, 0x16, 0x31),
    grid: Color32::from_rgb(0x35, 0x22, 0x3f),
    surface: Color32::from_rgb(0x39, 0x24, 0x43),
    raised: Color32::from_rgb(0x51, 0x34, 0x5f),
    sunken: Color32::from_rgb(0x28, 0x1a, 0x35),
    outline: Color32::from_rgb(0x6d, 0x3f, 0x68),
    highlight: Color32::from_rgb(0xff, 0xe5, 0xf5),
    shade: Color32::from_rgb(0x71, 0x46, 0x6f),
    ink: Color32::from_rgb(0xff, 0xf0, 0xfa),
    dim: Color32::from_rgb(0xdf, 0xb8, 0xdc),
    accent: Color32::from_rgb(0xba, 0x39, 0x75),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0xff, 0xb6, 0xdc),
    gold: Color32::from_rgb(0xff, 0xe2, 0xa0),
    good: Color32::from_rgb(0x74, 0xdd, 0xb6),
    bad: Color32::from_rgb(0xc5, 0x2d, 0x64),
    metal: Color32::from_rgb(0xd7, 0xb1, 0xe9),
    paper: Color32::from_rgb(0xff, 0xf2, 0xfa),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0x52, 0x31, 0x5f),
    vignette: Color32::from_rgb(0x00, 0x00, 0x00),
};

const STRAWBERRY_MILK: Palette = Palette {
    background: Color32::from_rgb(0xff, 0xf0, 0xf7),
    grid: Color32::from_rgb(0xf5, 0xd7, 0xe8),
    surface: Color32::from_rgb(0xff, 0xf5, 0xfa),
    raised: Color32::from_rgb(0xff, 0xff, 0xff),
    sunken: Color32::from_rgb(0xf7, 0xe3, 0xef),
    outline: Color32::from_rgb(0x99, 0x6b, 0x94),
    highlight: Color32::from_rgb(0xff, 0xff, 0xff),
    shade: Color32::from_rgb(0xd8, 0xa8, 0xc7),
    ink: Color32::from_rgb(0x39, 0x1a, 0x43),
    dim: Color32::from_rgb(0x73, 0x51, 0x6f),
    accent: Color32::from_rgb(0xb9, 0x32, 0x76),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0xa1, 0x36, 0x85),
    gold: Color32::from_rgb(0x8d, 0x59, 0x1d),
    good: Color32::from_rgb(0x17, 0x6b, 0x53),
    bad: Color32::from_rgb(0xad, 0x24, 0x4c),
    metal: Color32::from_rgb(0xc2, 0x9a, 0xd6),
    paper: Color32::from_rgb(0xff, 0xff, 0xff),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0xe6, 0xd4, 0xfa),
    vignette: Color32::from_rgb(0x00, 0x00, 0x00),
};
