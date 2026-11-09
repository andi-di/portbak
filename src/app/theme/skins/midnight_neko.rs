//! Midnight Neko: matte gothic panels, burgundy accents and an illustrated catgirl companion.

use crate::app::rendering::sprites::Sprite;
use crate::app::theme::style::{
    Effects, Finish, Fonts, Metrics, Palette, Skin, TitleBar, Typography, Wording,
};
use eframe::egui::Color32;

pub static MIDNIGHT_NEKO: Skin = Skin {
    id: "midnight-neko",
    name: "Midnight Neko",
    description: "Plum-black panels, burgundy ribbons, silver accents and Saya, a moss-eyed gothic companion.",
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
    background: Color32::from_rgb(0x1c, 0x14, 0x19),
    grid: Color32::from_rgb(0x28, 0x1e, 0x22),
    surface: Color32::from_rgb(0x2c, 0x1d, 0x23),
    raised: Color32::from_rgb(0x35, 0x25, 0x2c),
    sunken: Color32::from_rgb(0x21, 0x17, 0x1c),
    outline: Color32::from_rgb(0x6c, 0x55, 0x5e),
    highlight: Color32::from_rgb(0x50, 0x3a, 0x45),
    shade: Color32::from_rgb(0x16, 0x0f, 0x14),
    ink: Color32::from_rgb(0xf1, 0xd0, 0xca),
    dim: Color32::from_rgb(0xb5, 0xab, 0xab),
    accent: Color32::from_rgb(0x67, 0x1d, 0x36),
    accent_ink: Color32::from_rgb(0xf1, 0xd0, 0xca),
    hot: Color32::from_rgb(0xc6, 0x8d, 0x87),
    gold: Color32::from_rgb(0xd8, 0xcc, 0xcc),
    good: Color32::from_rgb(0x9b, 0xb5, 0x8c),
    bad: Color32::from_rgb(0x9e, 0x36, 0x51),
    metal: Color32::from_rgb(0xb5, 0xab, 0xab),
    paper: Color32::from_rgb(0xf1, 0xd0, 0xca),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0x28, 0x1e, 0x22),
};

const PALE_STONE: Palette = Palette {
    background: Color32::from_rgb(0xe5, 0xce, 0xcb),
    grid: Color32::from_rgb(0xd7, 0xbd, 0xbc),
    surface: Color32::from_rgb(0xf3, 0xe2, 0xde),
    raised: Color32::from_rgb(0xfc, 0xf2, 0xee),
    sunken: Color32::from_rgb(0xe6, 0xcf, 0xca),
    outline: Color32::from_rgb(0x94, 0x7b, 0x80),
    highlight: Color32::from_rgb(0xf9, 0xed, 0xea),
    shade: Color32::from_rgb(0xcb, 0xb1, 0xb4),
    ink: Color32::from_rgb(0x35, 0x25, 0x2c),
    dim: Color32::from_rgb(0x6c, 0x55, 0x5e),
    accent: Color32::from_rgb(0x67, 0x1d, 0x36),
    accent_ink: Color32::from_rgb(0xf1, 0xd0, 0xca),
    hot: Color32::from_rgb(0x6b, 0x0f, 0x1f),
    gold: Color32::from_rgb(0x58, 0x46, 0x4e),
    good: Color32::from_rgb(0x48, 0x61, 0x3b),
    bad: Color32::from_rgb(0x8d, 0x23, 0x3d),
    metal: Color32::from_rgb(0x65, 0x59, 0x60),
    paper: Color32::from_rgb(0xfc, 0xf2, 0xee),
    scanline: Color32::from_rgb(0x00, 0x00, 0x00),
    sky: Color32::from_rgb(0xe5, 0xce, 0xcb),
};
