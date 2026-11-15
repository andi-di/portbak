# Moonveil artwork

Theme assets:

- `saya-full-body.png`: static pixel-art welcome figure.
- `saya-emblem.png`: Kurobane family crest beside the application and job names.
- `expressions/`: registered pixel busts for the sidebar; click to show a
  non-neutral expression without immediate repeats. Hovering holds the
  expression; it resets after the pointer has been away for three seconds.
- `scenery/`: moonlit rose garden, bat flight/perching sprites, petal and crescent.
- `icons/`: individually generated Saya-inspired interface icons and prompts.
  See its README for the shared resolution and visible-size rules.

All use transparent backgrounds and preserve their proportions. Expressions are
individual 512 × 512 PNGs. The welcome figure is static and has no click reaction.
Prompts are stored alongside assets; refinement masters remain in agent-local.

Scenery remains behind controls. Bats avoid the pointer and scatter on clicks;
one approaches at a distance. Flying bats deform one consistent sprite smoothly
and mirror with direction. Three petal shapes tumble and vary in size; petals,
crescents and silver sparks form a bounded trail. All effects stay enabled.

## Saya dialogue budget

Keep each dialogue line to **120 Unicode characters maximum**, including spaces
and punctuation. The unified companion panel has one outer frame around a square
portrait, plain letterbox and caret strip. A single divider separates the portrait
from the letterbox; the portrait and outer frame stay undecorated. The strip contains only a centered caret.

The letterbox stays 60 logical pixels tall, with 46 pixels available for text and
7 pixels of top and bottom padding. This fits three lines at the normal font size.
Text scales from 12 to 13.5 logical pixels as the sidebar widens, then shrinks in
half-pixel steps for long dialogue, down to 9 pixels. Long unbroken words may wrap
between characters. Do not insert manual line breaks.

The dialogue-fit test checks every authored line and 120 wide characters in both
brightness modes at 300, 350, and 460 logical-pixel sidebar widths. Run
`cargo test --locked dialogue_budget` after editing dialogue. Both the character
budget and rendered bounds must pass; never resize the box for individual lines.

Minimizing and maximizing use a 240 ms eased slide and fade. The caret strip
stays fixed at the bottom, while the job list smoothly gains or gives up space.
The animation can reverse without jumping. Saya starts expanded on every launch.


Scenery textures are prepared at a maximum edge of 1024 pixels. Castle flicker uses
`castle.png` plus the transparent `castle-windows.png` mask and 16-pixel cell list
in `castle-windows.json`. Regenerate derived scenery with
`python tools/prepare_moonveil_assets.py` (requires Pillow).

The expression catalog in `src/app/scenery/saya/expressions.rs` is the source of
truth for the enum, gallery, preloader and non-neutral click pool. Add variants
alphabetically, their artwork, dialogue and overlay, then refine page/action pools.
The gallery derives its count and grid from that catalog; it has no fixed capacity.
Hold the header emblem for two seconds to open it.

Excited uses the transparent `overlays/excited-sparkles.png` sprite. Eye glints are
anchored to pupil centers in the 512px expression artwork; update those anchors
when replacing that portrait. Overlays play once and are clipped to the portrait.

Love uses the transparent `overlays/love-hearts.png` ruby-heart cluster, rising
and fading in three staggered bursts with its original one-shot timing.

Celebrating renders 64 folded metallic confetti pieces in one shared mesh. Its
2.4-second shower falls from the top with staggered arrivals, flutter and shaded folds;
no additional texture or persistent particle simulation is needed.

Pouting uses the compact custom `overlays/pouting-exhale.png` anime huff sprite,
with its open neck anchored beside her lips.
