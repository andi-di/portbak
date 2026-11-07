//! Registry of available skins. A new look is one more `Skin` value listed in `ALL`.

mod dark_fantasy;
mod frutiger_aero;
mod midnight_neko;
mod modern;
mod retro;

use crate::app::theme::style::Skin;

pub static ALL: &[&Skin] = &[
    &retro::RETRO,
    &dark_fantasy::DARK_FANTASY,
    &modern::MODERN,
    &frutiger_aero::FRUTIGER_AERO,
    &midnight_neko::MIDNIGHT_NEKO,
];

pub fn default() -> &'static Skin {
    ALL[0]
}

/// The skin saved under `id`, falling back to the default for unknown or removed skins.
pub fn find(id: &str) -> &'static Skin {
    ALL.iter()
        .copied()
        .find(|skin| skin.id == id)
        .unwrap_or_else(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_ids_are_unique_and_unknown_ids_fall_back() {
        for (index, skin) in ALL.iter().enumerate() {
            assert!(ALL[..index].iter().all(|other| other.id != skin.id));
            assert!(std::ptr::eq(find(skin.id), *skin));
        }
        assert!(std::ptr::eq(find("no-such-skin"), default()));
    }

    #[test]
    fn translucent_skins_paint_a_wallpaper_behind_their_panels() {
        for skin in ALL {
            let opacity = skin.effects.opacity;
            assert!((0.0..=1.0).contains(&opacity), "skin {}", skin.id);
            assert!(opacity == 1.0 || skin.effects.wallpaper, "skin {}", skin.id);
        }
    }
}
