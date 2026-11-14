#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Expression {
    Angry,
    Bored,
    BoredExpectant,
    Celebrating,
    Curious,
    Cute,
    Dominant,
    Embarrassed,
    Energetic,
    Excited,
    Flirty,
    Flustered,
    Happy,
    InLove,
    Killer,
    Meek,
    #[default]
    Neutral,
    Pleading,
    Pouting,
    Praying,
    Question,
    Sad,
    Seductive,
    Shy,
    Sleepy,
    Smug,
    Waiting,
}

impl Expression {
    pub(super) const REACTIONS: [Self; 26] = [
        Self::Angry,
        Self::Bored,
        Self::BoredExpectant,
        Self::Celebrating,
        Self::Curious,
        Self::Cute,
        Self::Dominant,
        Self::Embarrassed,
        Self::Energetic,
        Self::Excited,
        Self::Flirty,
        Self::Flustered,
        Self::Happy,
        Self::InLove,
        Self::Killer,
        Self::Meek,
        Self::Pleading,
        Self::Pouting,
        Self::Praying,
        Self::Question,
        Self::Sad,
        Self::Seductive,
        Self::Shy,
        Self::Sleepy,
        Self::Smug,
        Self::Waiting,
    ];

    pub(super) fn choose(pool: &[Self], current: Self, random: u128) -> Self {
        let count = pool.iter().filter(|value| **value != current).count();
        if count == 0 {
            return pool.first().copied().unwrap_or(Self::Neutral);
        }
        pool.iter()
            .copied()
            .filter(|value| *value != current)
            .nth((random % count as u128) as usize)
            .unwrap_or(Self::Neutral)
    }

    pub(super) fn next(self, random: u128) -> Self {
        Self::choose(&Self::REACTIONS, self, random)
    }

    pub(super) fn bytes(self) -> &'static [u8] {
        match self {
            Self::Angry => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/angry.png")
            }
            Self::Bored => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/bored.png")
            }
            Self::BoredExpectant => include_bytes!(
                "../../../../assets/themes/midnight-neko/expressions/bored-expectant.png"
            ),
            Self::Celebrating => include_bytes!(
                "../../../../assets/themes/midnight-neko/expressions/celebrating.png"
            ),
            Self::Curious => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/curious.png")
            }
            Self::Cute => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/cute.png")
            }
            Self::Dominant => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/dominant.png")
            }
            Self::Embarrassed => include_bytes!(
                "../../../../assets/themes/midnight-neko/expressions/embarrassed.png"
            ),
            Self::Energetic => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/energetic.png")
            }
            Self::Excited => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/excited.png")
            }
            Self::Flirty => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/flirty.png")
            }
            Self::Flustered => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/flustered.png")
            }
            Self::Happy => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/happy.png")
            }
            Self::InLove => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/in-love.png")
            }
            Self::Killer => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/killer.png")
            }
            Self::Meek => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/meek.png")
            }
            Self::Neutral => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/neutral.png")
            }
            Self::Pleading => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/pleading.png")
            }
            Self::Pouting => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/pouting.png")
            }
            Self::Praying => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/praying.png")
            }
            Self::Question => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/question.png")
            }
            Self::Sad => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/sad.png")
            }
            Self::Seductive => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/seductive.png")
            }
            Self::Shy => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/shy.png")
            }
            Self::Sleepy => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/sleepy.png")
            }
            Self::Smug => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/smug.png")
            }
            Self::Waiting => {
                include_bytes!("../../../../assets/themes/midnight-neko/expressions/waiting.png")
            }
        }
    }
}
