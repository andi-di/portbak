//! Liquid Glass: iOS-style lenses of clear glass that bend, tint and light a flowing wallpaper.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static LIQUID_GLASS: Skin = Skin {
    id: "liquid-glass",
    name: "Liquid Glass",
    description: "Clear, light-bending glass over a flowing wallpaper, in the style of iOS.",
    dark: NIGHT,
    light: DAY,
    fonts: Fonts {
        display: Some(include_bytes!("../../../../assets/fonts/Inter-Bold.ttf")),
        body: Some(include_bytes!("../../../../assets/fonts/Inter-Regular.ttf")),
    },
    typography: Typography {
        heading: 20.0,
        body: 16.0,
        small: 13.0,
        label: 16.0,
        display_scale: 1.2,
        display_drop: 0.0,
        display_glow: 0.0,
        capitals: false,
        title_case: true,
    },
    metrics: Metrics {
        border: 1.0,
        // Glass swells when pressed instead of sinking.
        lift: 0.0,
        shadow: 4,
        shadow_blur: 24,
        // Panes round by 24, controls by half of it; buttons are capsules.
        radius: 24,
    },
    effects: Effects {
        scanlines: false,
        grid: false,
        stripes: false,
        wallpaper: true,
        bubbles: false,
        hearts: false,
        opacity: 0.4,
        title_bar: TitleBar::Liquid,
    },
    finish: Finish::Liquid,
    wording: Wording {
        mascot: Sprite::Shield,
        tagline: "",
        welcome_title: "Back up with ease",
        welcome_prompt: "Create your first backup job",
        ready: "Up to date",
    },
};

/// iOS system colors in dark mode.
const NIGHT: Palette = Palette {
    background: Color32::from_rgb(0x00, 0x00, 0x00),
    grid: Color32::from_rgb(0x1c, 0x1c, 0x1e),
    surface: Color32::from_rgb(0x1c, 0x1c, 0x1e),
    raised: Color32::from_rgb(0x2c, 0x2c, 0x2e),
    sunken: Color32::from_rgb(0x1c, 0x1c, 0x1e),
    outline: Color32::from_rgb(0x38, 0x38, 0x3a),
    highlight: Color32::from_rgb(0x3a, 0x3a, 0x3c),
    shade: Color32::from_rgb(0x48, 0x48, 0x4a),
    ink: Color32::from_rgb(0xff, 0xff, 0xff),
    dim: Color32::from_rgb(0xae, 0xae, 0xb2),
    accent: Color32::from_rgb(0x0a, 0x84, 0xff),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0x64, 0xd2, 0xff),
    gold: Color32::from_rgb(0xff, 0x9f, 0x0a),
    good: Color32::from_rgb(0x30, 0xd1, 0x58),
    bad: Color32::from_rgb(0xff, 0x45, 0x3a),
    metal: Color32::from_rgb(0x8e, 0x8e, 0x93),
    paper: Color32::from_rgb(0xff, 0xff, 0xff),
    scanline: Color32::TRANSPARENT,
    sky: Color32::from_rgb(0x03, 0x05, 0x14),
};

/// iOS system colors in light mode; secondary text is a little darker to read over glass.
const DAY: Palette = Palette {
    background: Color32::from_rgb(0xf2, 0xf2, 0xf7),
    grid: Color32::from_rgb(0xe5, 0xe5, 0xea),
    surface: Color32::from_rgb(0xff, 0xff, 0xff),
    raised: Color32::from_rgb(0xff, 0xff, 0xff),
    sunken: Color32::from_rgb(0xe9, 0xe9, 0xee),
    outline: Color32::from_rgb(0x63, 0x63, 0x66),
    highlight: Color32::from_rgb(0xff, 0xff, 0xff),
    shade: Color32::from_rgb(0xd1, 0xd1, 0xd6),
    ink: Color32::from_rgb(0x00, 0x00, 0x00),
    dim: Color32::from_rgb(0x55, 0x55, 0x5c),
    accent: Color32::from_rgb(0x00, 0x7a, 0xff),
    accent_ink: Color32::from_rgb(0xff, 0xff, 0xff),
    hot: Color32::from_rgb(0x5a, 0xc8, 0xfa),
    gold: Color32::from_rgb(0xff, 0x95, 0x00),
    good: Color32::from_rgb(0x34, 0xc7, 0x59),
    bad: Color32::from_rgb(0xff, 0x3b, 0x30),
    metal: Color32::from_rgb(0x8e, 0x8e, 0x93),
    paper: Color32::from_rgb(0xff, 0xff, 0xff),
    scanline: Color32::TRANSPARENT,
    sky: Color32::from_rgb(0xd6, 0xe6, 0xff),
};
