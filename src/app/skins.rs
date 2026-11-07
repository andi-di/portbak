//! Registry of available skins. A new look is one more `Skin` value listed in `ALL`.

mod dark_fantasy;
mod retro;

use super::style::Skin;

pub static ALL: &[&Skin] = &[&retro::RETRO, &dark_fantasy::DARK_FANTASY];

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
}
