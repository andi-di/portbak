//! Retro: pixel fonts, a night-arcade dark mode and a beige handheld light mode.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static RETRO: Skin = Skin {
    id: "retro",
    name: "Retro",
    description: "Pixel fonts, neon arcade nights and beige handheld days.",
    dark: NIGHT,
    light: DAY,
    fonts: Fonts {
        display: Some(include_bytes!(
            "../../../../assets/fonts/PressStart2P-Regular.ttf"
        )),
        body: Some(include_bytes!("../../../../assets/fonts/VT323-Regular.ttf")),
    },
    // The display face is drawn on an 8 px grid, so its sizes stay near multiples of 8.
    typography: Typography {
        heading: 16.0,
        body: 20.0,
        small: 17.0,
        label: 10.0,
        display_scale: 1.0,
        display_drop: 0.125,
        display_glow: 0.0,
        capitals: true,
        title_case: false,
    },
    metrics: Metrics {
        border: 2.0,
        lift: 3.0,
        shadow: 5,
        shadow_blur: 0,
        radius: 0,
    },
    effects: Effects {
        scanlines: true,
        grid: true,
        stripes: true,
        wallpaper: false,
        bubbles: false,
        hearts: false,
        opacity: 1.0,
        title_bar: TitleBar::Striped,
    },
    finish: Finish::Pixel,
    wording: Wording {
        mascot: Sprite::Floppy,
        tagline: "PORTABLE BACKUP SYSTEM",
        welcome_title: "INSERT DISK",
        welcome_prompt: "PRESS START",
        ready: "READY",
    },
};

const NIGHT: Palette = Palette {
    background: Color32::from_rgb(0x0e, 0x08, 0x1d),
    grid: Color32::from_rgb(0x17, 0x0f, 0x2c),
    surface: Color32::from_rgb(0x1c, 0x12, 0x37),
    raised: Color32::from_rgb(0x2b, 0x1e, 0x52),
    sunken: Color32::from_rgb(0x08, 0x04, 0x12),
    outline: Color32::from_rgb(0x03, 0x01, 0x08),
    highlight: Color32::from_rgb(0x4c, 0x3b, 0x85),
    shade: Color32::from_rgb(0x0d, 0x07, 0x1c),
    ink: Color32::from_rgb(0xf4, 0xec, 0xff),
    dim: Color32::from_rgb(0xa3, 0x95, 0xcf),
    accent: Color32::from_rgb(0x2c, 0xe8, 0xf5),
    accent_ink: Color32::from_rgb(0x05, 0x02, 0x0c),
    hot: Color32::from_rgb(0xff, 0x3f, 0xa4),
    gold: Color32::from_rgb(0xff, 0xd2, 0x3f),
    good: Color32::from_rgb(0x3d, 0xf5, 0x8b),
    bad: Color32::from_rgb(0xff, 0x4d, 0x5e),
    metal: Color32::from_rgb(0xc2, 0xc3, 0xd6),
    paper: Color32::from_rgb(0xf4, 0xec, 0xff),
    scanline: Color32::from_rgba_premultiplied(0, 0, 0, 34),
    sky: Color32::from_rgb(0x0e, 0x08, 0x1d),
};

const DAY: Palette = Palette {
    background: Color32::from_rgb(0xe4, 0xd8, 0xbc),
    grid: Color32::from_rgb(0xd9, 0xcb, 0xab),
    surface: Color32::from_rgb(0xf3, 0xea, 0xd3),
    raised: Color32::from_rgb(0xfb, 0xf5, 0xe4),
    sunken: Color32::from_rgb(0xff, 0xfd, 0xf6),
    outline: Color32::from_rgb(0x2b, 0x1d, 0x3a),
    highlight: Color32::from_rgb(0xff, 0xff, 0xff),
    shade: Color32::from_rgb(0xc4, 0xb3, 0x90),
    ink: Color32::from_rgb(0x2b, 0x1d, 0x3a),
    dim: Color32::from_rgb(0x6e, 0x5e, 0x7c),
    accent: Color32::from_rgb(0x1f, 0x8f, 0xa8),
    accent_ink: Color32::from_rgb(0xff, 0xfd, 0xf6),
    hot: Color32::from_rgb(0xd6, 0x33, 0x6c),
    gold: Color32::from_rgb(0xe0, 0x9a, 0x00),
    good: Color32::from_rgb(0x2b, 0x9a, 0x48),
    bad: Color32::from_rgb(0xd2, 0x37, 0x37),
    metal: Color32::from_rgb(0x9a, 0x9a, 0xae),
    paper: Color32::from_rgb(0xff, 0xfd, 0xf6),
    scanline: Color32::from_rgba_premultiplied(0, 0, 0, 12),
    sky: Color32::from_rgb(0xe4, 0xd8, 0xbc),
};
