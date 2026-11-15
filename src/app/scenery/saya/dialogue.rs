use super::Expression;

impl Expression {
    pub(super) fn lines(self) -> &'static [&'static str] {
        match self {
            Self::Angry => &[
                "I wasn't waiting for you!",
                "I was speaking to you, Human. Do not ignore me.",
                "Call me cute once more. I dare you, Mortal.",
                "You could have come back sooner, Human. Not that I was waiting for you.",
            ],
            Self::Bored => &[
                "You bore me, Mortal. Surely you have something interesting to say.",
                "Two hundred years, and I have never been this bored.",
                "I expected very little entertainment from you. Yet here we are.",
                "Speak, Mortal. What do you want from me now?",
            ],
            Self::Celebrating => &[
                "Yay, you’re back! Ahem. You certainly took your time, Human.",
                "An excellent result. My influence is unmistakable.",
                "You’re staying to keep me company? Wonderful! I mean... I suppose you may stay.",
                "You did rather well, Minion. Under my guidance, of course.",
            ],
            Self::Crazy => &[
                "Tell me who hurt you, Darling. You won’t have to deal with them again.",
                "Are there any other girls, Darling? You would tell me, wouldn’t you?",
                "I can only think of you, Darling. All day. All night.",
                "Now that you’re here, Darling, we can be together forever.",
            ],
            Self::Curious => &[
                "What have you brought me, Human?",
                "That has caught my interest. Do explain.",
                "You seem rather pleased with yourself, Human. What have you been up to?",
                "What occupies that mortal mind of yours?",
            ],
            Self::Cute => &[
                "Nya~ There you are! Come keep me company.",
                "Do you like my ears? I picked the ribbons to match, nya.",
                "One little head pat wouldn’t hurt, nya?",
                "I saved you the seat beside me, nya~",
            ],
            Self::Dominant => &[
                "Come here, Mortal. I prefer having you where I can see you.",
                "Attend to your work. I shall oversee my domain.",
                "A little discipline would suit you, Servant.",
                "Sit beside me, Servant. You have my permission to stay.",
            ],
            Self::Drunk => &[
                "The room is moving. I find that terribly rude.",
                "I am perfectly composed. Why are there two of you?",
                "One more toast, Human! To... whatever we were toasting.",
                "You are excellent company. I may have said that already.",
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
            Self::Exhausted => &[
                "Even I have limits. Apparently.",
                "The rest can wait. Sit with me a moment.",
                "Two centuries old, and today still managed to exhaust me.",
                "If you need me, speak softly. Very softly.",
            ],
            Self::Expectant => &[
                "Go on. I'm listening.",
                "Well, Human? You have my attention.",
                "I trust there was a reason for that lingering glance.",
                "You may speak. I shall try to look less impressed.",
            ],
            Self::Fed => &[
                "That was a most satisfying meal.",
                "A little indulgence does wonders for my disposition.",
                "You may relax, Human. I have already dined.",
                "Something on my lips? How observant of you.",
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
            Self::Hungry => &[
                "I would be much more agreeable after a meal.",
                "Do not look so alarmed. I can ask politely.",
                "Tea is lovely, but it will not quite suffice tonight.",
                "Is that your pulse, Human? It is terribly distracting.",
            ],
            Self::Hurt => &[
                "A few scratches. The other creatures fared worse.",
                "Help me with this bandage, would you?",
                "I won. Allow me a moment to appreciate the cost.",
                "No lectures, Human. Just stay close for a while.",
            ],
            Self::Killer => &[
                "Nothing gets past me. Nothing.",
                "Should trouble find you, it will answer to me.",
                "My patience is reserved for you, Human.",
                "Behind me, Mortal. Let me look first.",
            ],
            Self::Love => &[
                "Maybe I missed you. Just a little.",
                "I find myself looking forward to your visits.",
                "The garden seems quieter when you are away.",
                "Stay beside me a little longer. If you wish.",
            ],
            Self::Meek => &[
                "Perhaps you could stay a little longer.",
                "Would you mind sitting here? Only for a moment.",
                "I thought we might share some tea... if you are free.",
                "Your company would be welcome. There, I said it.",
            ],
            Self::Neutral => &[
                "Hmph. I suppose you can stay.",
                "Make yourself useful, Human. Or make tea.",
                "My domain is quieter with you in it. How curious.",
                "You may keep me company while you work.",
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
            Self::Sweating => &[
                "Give me a moment. That was more running than I intended.",
                "Immortal does not mean tireless.",
                "A towel, Human. And perhaps a little dignity.",
                "Let me catch my breath before you ask.",
            ],
            Self::Tongue => &[
                "Such a serious face. I had to do something.",
                "Consider that my very dignified response.",
                "Oh, did you expect another lecture?",
                "You may call it childish. I call it effective.",
            ],
            Self::Waiting => &[
                "I was merely passing the time.",
                "These ribbons required my attention. I was not waiting.",
                "You took your time, Human. The tea nearly cooled.",
                "I happened to be here when you returned. How convenient.",
            ],
            Self::Wink => &[
                "That was meant for you. Try to keep up.",
                "Our little secret, Human.",
                "I trust you understood the hint.",
                "You noticed? Good. I dislike wasting a perfectly good wink.",
            ],
        }
    }
}
