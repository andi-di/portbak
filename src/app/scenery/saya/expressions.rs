macro_rules! expression_catalog {
    ($($(#[$attribute:meta])* $name:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub(super) enum Expression { $($(#[$attribute])* $name),+ }

        impl Expression {
            pub(super) const ALL: &'static [Self] = &[$(Self::$name),+];
            pub(super) const REACTIONS: [Self; Self::ALL.len() - 1] = {
                let mut result = [Self::Neutral; Self::ALL.len() - 1];
                let mut source = 0;
                let mut target = 0;
                while source < Self::ALL.len() {
                    if !matches!(Self::ALL[source], Self::Neutral) {
                        result[target] = Self::ALL[source];
                        target += 1;
                    }
                    source += 1;
                }
                result
            };
        }
    };
}

expression_catalog! {
    Angry,
    Bored,
    Celebrating,
    Crazy,
    Curious,
    Cute,
    Dominant,
    Drunk,
    Embarrassed,
    Energetic,
    Excited,
    Exhausted,
    Expectant,
    Fed,
    Flirty,
    Flustered,
    Happy,
    Hungry,
    Hurt,
    Killer,
    Love,
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
    Sweating,
    Tongue,
    Waiting,
    Wink,

}

impl Expression {
    pub(super) fn label(self) -> String {
        let name = format!("{self:?}");
        let mut label = String::new();
        for (index, character) in name.chars().enumerate() {
            if index > 0 && character.is_uppercase() {
                label.push(' ');
            }
            label.push(character);
        }
        label
    }

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
            Self::Celebrating => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/celebrating.png")
            }
            Self::Crazy => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/crazy.png")
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
            Self::Drunk => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/drunk.png")
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
            Self::Exhausted => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/exhausted.png")
            }
            Self::Expectant => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/expectant.png")
            }
            Self::Fed => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/fed.png")
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
            Self::Hungry => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/hungry.png")
            }
            Self::Hurt => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/hurt.png")
            }
            Self::Killer => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/killer.png")
            }
            Self::Love => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/love.png")
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
            Self::Sweating => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/sweating.png")
            }
            Self::Tongue => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/tongue.png")
            }
            Self::Waiting => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/waiting.png")
            }
            Self::Wink => {
                include_bytes!("../../../../assets/themes/moonveil/expressions/wink.png")
            }
        }
    }
}
