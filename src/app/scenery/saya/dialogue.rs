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
                "Come heeere, Human! I missed you! ...Did I say that out loud? Hehehe!",
                "I am perfectly composed. Why are there two of you?",
                "Cheersh, Darling! To you, and me, and... hehehe, us!",
                "You’re staying? Yay! This is the besht evening in... in ages!",
            ],
            Self::Embarrassed => &[
                "You needn't stare, Human.",
                "You think I look lovely? W-well... you have surprisingly good taste, Human.",
                "It is merely warm in here. Do not draw conclusions.",
                "I was only straightening your collar. Stop smiling like that.",
            ],
            Self::Energetic => &[
                "Shall we get to work?",
                "You said you wanted something from me, Human? Well, here I am!",
                "I have so many plans for tonight! You weren’t about to leave, were you?",
                "Can I help you with this task? Come on, let me try!",
            ],
            Self::Excited => &[
                "You came back! I mean... good timing!",
                "Oh! You must tell me everything. Briefly, of course.",
                "You remembered what I like! I... that was very thoughtful of you!",
                "I knew you would return, Human! Come, tell me what you have been up to!",
            ],
            Self::Exhausted => &[
                "Even I have limits. Apparently.",
                "Stay a little, Human. I am too tired to pretend I prefer being alone.",
                "Two centuries old, and today still managed to exhaust me.",
                "If you need me, speak softly. Very softly.",
            ],
            Self::Expectant => &[
                "Go on. I'm listening.",
                "Well, Human? You have my attention.",
                "I trust there was a reason for that lingering glance.",
                "Tell me more about it, Human. You have caught my interest.",
            ],
            Self::Fed => &[
                "That was a most satisfying meal.",
                "A little indulgence does wonders for my disposition.",
                "You may relax, Human. I have already dined.",
                "Something on my lips? How observant of you.",
            ],
            Self::Flirty => &[
                "More wine, Darling? Or shall we find somewhere a little more private?",
                "Another glance? I shall allow it.",
                "The evening needn’t end here. My chambers are just upstairs.",
                "Come sit closer, Mortal. I should hate to spend our date admiring you from afar.",
            ],
            Self::Flustered => &[
                "Nya?! You startled me!",
                "You did what?! You might have warned me before announcing it like that!",
                "You cannot simply say things like that without warning!",
                "W-wait! Give me a moment. You have thoroughly ruined my train of thought.",
            ],
            Self::Happy => &[
                "It's nice having you around.",
                "The evening is rather pleasant with you here.",
                "I saved you a place. It seemed practical.",
                "Come, sit with me, Human. I was hoping we might spend some time together.",
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
                "Who was that girl, Mortal? You seemed very pleased to see her.",
                "Look at me, Human. I dislike repeating myself.",
                "Try ignoring me again, Mortal. I dare you.",
            ],
            Self::Love => &[
                "Maybe I missed you, Darling. Just a little.",
                "There you are, my dear. Come closer; I have missed your company.",
                "You have become rather dear to me, Human. I trust you know that.",
                "Stay beside me, Love. I am not ready to let this evening end.",
            ],
            Self::Meek => &[
                "Perhaps you could stay a little longer.",
                "Would you mind sitting here? Only for a moment.",
                "Could you help me for a moment, Human? I would be grateful.",
                "Your company would be welcome. There, I said it.",
            ],
            Self::Neutral => &[
                "Hmph. I suppose you can stay, Human.",
                "Well, Human? Make yourself comfortable. I have a little time to spare.",
                "You are in my domain now, Mortal. Try to behave yourself.",
                "You may keep me company while you work, Mortal.",
            ],
            Self::Pleading => &[
                "You will return, will you not?",
                "Stay just a little longer, Human. I have grown rather fond of your company.",
                "Must you go already? We have barely had a moment together.",
                "Promise you will visit the garden with me next time.",
            ],
            Self::Pouting => &[
                "A little attention would not hurt...",
                "Fine. I won’t come with you. See if I care.",
                "I am being remarkably patient about being ignored.",
                "Hmph. Even the bats noticed my new ribbons.",
            ],
            Self::Praying => &[
                "May your files remain safe.",
                "A quiet evening and a little good fortune for you.",
                "Just this once, let everything turn out well for you. Please.",
                "I shall keep you in my thoughts, Mortal. You need not make a fuss about it.",
            ],
            Self::Question => &[
                "Might I ask you something?",
                "Would you... care to join me for tea later?",
                "Do you prefer the garden by moonlight? I was merely curious.",
                "There is something I wished to ask. Please do not laugh.",
            ],
            Self::Sad => &[
                "You are leaving already? I thought we might have a little longer.",
                "The halls feel rather empty after you go, Human.",
                "Some evenings feel terribly long. Would you sit with me for a while?",
                "I know you have other places to be. I had hoped you might choose to stay.",
            ],
            Self::Seductive => &[
                "You are quite intriguing, Mortal. I should like to know you a little better.",
                "Come closer, Human. I do not intend to raise my voice.",
                "I have a quieter place in mind, Mortal. Come, and I shall show you.",
                "You may linger. I find your company rather tempting.",
            ],
            Self::Shy => &[
                "Do you really think that, Human? I... do not quite know what to say.",
                "You shouldn’t have... but I shall treasure it.",
                "I... want to tell you something. You won’t laugh, will you?",
                "Would you mind looking away for a moment? You are making it difficult to speak.",
            ],
            Self::Sleepy => &[
                "Mm... juss five more minutes, Human...",
                "The book was fascinating. The hour was unreasonable.",
                "My hair is perfectly presentable... is it not?",
                "Don’ go yet... you’re warm. Jus’ stay there a little.",
            ],
            Self::Smug => &[
                "That look on your face. Have you finally realized whom you are dealing with?",
                "You may thank me after you finish admiring my elegance.",
                "Another excellent decision: visiting me in my domain.",
                "You have been staring for quite a while, Human. I can hardly blame you.",
            ],
            Self::Sweating => &[
                "Give me a moment. That was more running than I intended.",
                "Immortal does not mean tireless.",
                "A towel, Human. And perhaps a little dignity.",
                "Let me catch my breath before you ask.",
            ],
            Self::Tongue => &[
                "Nya~ What was that about behaving like a proper lady?",
                "You make such a funny face when I do that, Human.",
                "Oh, lighten up, Mortal. You are far too easy to tease.",
                "You may call it childish. I call it effective.",
            ],
            Self::Waiting => &[
                "Oh, you’re here. I was just about to find something else to do.",
                "You certainly know how to keep a lady waiting. Hypothetically, of course.",
                "You took your time, Human. The tea nearly cooled.",
                "I happened to be here when you returned. How convenient.",
            ],
            Self::Wink => &[
                "That was meant for you. Try to keep up.",
                "Our little secret, Human.",
                "Consider that a little encouragement, Human. You are doing rather well.",
                "You noticed? Good. I dislike wasting a perfectly good wink.",
            ],
        }
    }
}
