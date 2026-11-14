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
                include_bytes!("../../../../assets/themes/moonveil/expressions/angry.png")
            }
            Self::Bored => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/bored.png")
            }
            Self::BoredExpectant => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/bored-expectant.png")
            }
            Self::Celebrating => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/celebrating.png")
            }
            Self::Curious => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/curious.png")
            }
            Self::Cute => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/cute.png")
            }
            Self::Dominant => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/dominant.png")
            }
            Self::Embarrassed => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/embarrassed.png")
            }
            Self::Energetic => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/energetic.png")
            }
            Self::Excited => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/excited.png")
            }
            Self::Flirty => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/flirty.png")
            }
            Self::Flustered => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/flustered.png")
            }
            Self::Happy => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/happy.png")
            }
            Self::InLove => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/in-love.png")
            }
            Self::Killer => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/killer.png")
            }
            Self::Meek => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/meek.png")
            }
            Self::Neutral => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/neutral.png")
            }
            Self::Pleading => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/pleading.png")
            }
            Self::Pouting => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/pouting.png")
            }
            Self::Praying => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/praying.png")
            }
            Self::Question => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/question.png")
            }
            Self::Sad => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/sad.png")
            }
            Self::Seductive => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/seductive.png")
            }
            Self::Shy => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/shy.png")
            }
            Self::Sleepy => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/sleepy.png")
            }
            Self::Smug => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/smug.png")
            }
            Self::Waiting => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/waiting.png")
            }
        }
    }
}
