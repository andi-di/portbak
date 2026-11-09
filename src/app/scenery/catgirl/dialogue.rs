use super::Expression;

impl Expression {
    pub(super) fn lines(self) -> &'static [&'static str] {
        match self {
            Self::Neutral => &[
                "Hmph. I suppose you can stay.",
                "Make yourself useful, Human. Or make tea.",
                "My domain is quieter with you in it. How curious.",
                "You may keep me company while you work.",
            ],
            Self::Angry => &[
                "I wasn't waiting for you!",
                "Do not mistake my patience for permission to tease me.",
                "Call me cute once more. I dare you, Mortal.",
                "Hmph! You are testing two centuries of restraint.",
            ],
            Self::Bored => &[
                "Still here. Still keeping watch.",
                "Even eternity has its slow afternoons.",
                "A book, some strong tea... surely you can arrange that.",
                "Have you finished? I have counted every rose outside.",
            ],
            Self::BoredExpectant => &[
                "Go on. I'm listening.",
                "Well, Human? You have my attention.",
                "I trust there was a reason for that lingering glance.",
                "You may speak. I shall try to look less impressed.",
            ],
            Self::Celebrating => &[
                "Naturally, everything went according to my plan.",
                "An excellent result. My influence is unmistakable.",
                "A triumph worthy of tea in the garden!",
                "You did rather well, Minion. Under my guidance, of course.",
            ],
            Self::Curious => &[
                "What have you brought me, Human?",
                "That has caught my interest. Do explain.",
                "A small mystery? How considerate of you.",
                "What occupies that mortal mind of yours?",
            ],
            Self::Cute => &[
                "Don't call me cute.",
                "This is regal composure. The ears are beside the point.",
                "You saw no such thing. Especially not that smile.",
                "Nya. A deliberate flourish, Human. Control yourself.",
            ],
            Self::Dominant => &[
                "Your files are under my protection, Mortal.",
                "Attend to your work. I shall oversee my domain.",
                "A little discipline would suit you, Servant.",
                "You may relax. I am keeping watch.",
            ],
            Self::Embarrassed => &[
                "You needn't stare, Human.",
                "That compliment was entirely unnecessary... thank you.",
                "It is merely warm in here. Do not draw conclusions.",
                "I had a perfectly dignified reply a moment ago.",
            ],
            Self::Energetic => &[
                "Shall we get to work?",
                "Come along, Human. The evening will not wait.",
                "I have had my tea. You may now attempt to keep up.",
                "One task at a time. We shall make splendid progress.",
            ],
            Self::Excited => &[
                "You came back! I mean... good timing!",
                "Oh! You must tell me everything. Briefly, of course.",
                "At last, something worthy of our attention!",
                "I knew you would return. I simply did not expect to smile.",
            ],
            Self::Flirty => &[
                "You do enjoy my company, Human.",
                "Another glance? I shall allow it.",
                "You are finding it rather difficult to leave, aren't you?",
                "Such attentive company. I could become accustomed to this.",
            ],
            Self::Flustered => &[
                "Nya?! You startled me!",
                "I was not rehearsing what to say to you!",
                "You cannot simply say things like that without warning!",
                "Give me a moment. My composure has been inconvenienced.",
            ],
            Self::Happy => &[
                "It's nice having you around.",
                "The evening is rather pleasant with you here.",
                "I saved you a place. It seemed practical.",
                "You may stay for another cup of tea.",
            ],
            Self::InLove => &[
                "Maybe I missed you. Just a little.",
                "I find myself looking forward to your visits.",
                "The garden seems quieter when you are away.",
                "Stay beside me a little longer. If you wish.",
            ],
            Self::Killer => &[
                "Nothing gets past me. Nothing.",
                "Should trouble find you, it will answer to me.",
                "My patience is reserved for you, Human.",
                "Behind me, Mortal. Let me look first.",
            ],
            Self::Meek => &[
                "Perhaps you could stay a little longer.",
                "Would you mind sitting here? Only for a moment.",
                "I thought we might share some tea... if you are free.",
                "Your company would be welcome. There, I said it.",
            ],
            Self::Pleading => &[
                "You will return, will you not?",
                "One more moment, Human. The tea is still warm.",
                "Must you go already? I had another story to tell.",
                "Promise you will visit the garden with me next time.",
            ],
            Self::Pouting => &[
                "A little attention would not hurt...",
                "Your screen has enjoyed quite enough of your devotion.",
                "I am being remarkably patient about being ignored.",
                "Hmph. Even the bats noticed my new ribbons.",
            ],
            Self::Praying => &[
                "May your files remain safe.",
                "A quiet evening and a little good fortune for you.",
                "I shall wish you well. Quietly, of course.",
                "May tomorrow be kinder to you, Human.",
            ],
            Self::Question => &[
                "Might I ask you something?",
                "Would you... care to join me for tea later?",
                "Do you prefer the garden by moonlight? I was merely curious.",
                "There is something I wished to ask. Please do not laugh.",
            ],
            Self::Sad => &[
                "Do take care of yourself, Human.",
                "You need not pretend to be tireless on my account.",
                "Some evenings are heavy. You may sit with me.",
                "I would rather hear that you rested than that you struggled alone.",
            ],
            Self::Seductive => &[
                "I seem to have your attention.",
                "Come closer, Human. I do not intend to raise my voice.",
                "Perhaps the moonlit garden can wait another moment.",
                "You may linger. I find your company rather tempting.",
            ],
            Self::Shy => &[
                "You noticed me...?",
                "I was hoping you might sit here. Beside me.",
                "You remembered my favorite tea? How... thoughtful.",
                "I am pleased you came. Please do not make me repeat that.",
            ],
            Self::Sleepy => &[
                "Five more minutes... nya...",
                "The book was fascinating. The hour was unreasonable.",
                "My hair is perfectly presentable... is it not?",
                "I am resting my eyes with considerable dignity.",
            ],
            Self::Smug => &[
                "Of course I noticed. I notice everything.",
                "You may thank me after you finish admiring my foresight.",
                "Another excellent decision: keeping me company.",
                "I did tell you I had impeccable judgment.",
            ],
            Self::Waiting => &[
                "I was merely passing the time.",
                "These ribbons required my attention. I was not waiting.",
                "You took your time, Human. The tea nearly cooled.",
                "I happened to be here when you returned. How convenient.",
            ],
        }
    }
}
