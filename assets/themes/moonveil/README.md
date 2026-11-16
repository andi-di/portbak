# Moonveil artwork

## Asset overview

- `saya-full-body.png`: static pixel-art welcome figure.
- `saya-emblem.png`: Kurobane family crest beside the application and job names.
- `expressions/`: registered pixel busts for the sidebar; click to show a
  non-neutral expression without immediate repeats. Hovering holds the
  expression; it resets after the pointer has been away for three seconds.
- [Scenery](scenery/README.md): moonlit rose garden, bat flight/perching sprites,
  petal and crescent.
- [Interface icons](icons/README.md): individually generated Saya-inspired icons,
  with shared resolution and visible-size rules.

All use transparent backgrounds and preserve their proportions. Expressions are
individual 512 × 512 PNGs. The welcome figure is static and has no click reaction.

## Scenery

Scenery remains behind controls. Bats avoid the pointer and scatter on clicks;
one approaches at a distance. Flying bats deform one consistent sprite smoothly
and mirror with direction. Three petal shapes tumble and vary in size; petals,
crescents and silver sparks form a bounded trail. All effects stay enabled.

Scenery textures are prepared at a maximum edge of 1024 pixels. Castle flicker uses
`castle.png` plus the transparent `castle-windows.png` mask and 16-pixel cell list
in `castle-windows.json`. Regenerate derived scenery with
`python tools/prepare_moonveil_assets.py` (requires Pillow).

## Saya dialogue budget

Keep each dialogue line to **120 Unicode characters maximum**, including spaces
and punctuation. The unified companion panel has one outer frame around a square
portrait, plain letterbox and caret strip. A single divider separates the portrait
from the letterbox; the portrait and outer frame stay undecorated. The strip
contains only a centered caret.

The letterbox stays 60 logical pixels tall, with 46 pixels available for text and
7 pixels of top and bottom padding. This fits three lines at the normal font size.
Text scales from 12 to 13.5 logical pixels as the sidebar widens, then shrinks in
half-pixel steps for long dialogue, down to 9 pixels. Long unbroken words may wrap
between characters. Do not insert manual line breaks.

The dialogue-fit test checks every authored line and 120 wide characters in both
brightness modes at 300, 350, and 460 logical-pixel sidebar widths. Run
`cargo test --locked dialogue_budget` after editing dialogue. Both the character
budget and rendered bounds must pass; never resize the box for individual lines.

### Panel animation

Minimizing and maximizing use a 240 ms eased slide and fade. The caret strip
stays fixed at the bottom, while the job list smoothly gains or gives up space.
The animation can reverse without jumping. Saya starts expanded on every launch.

## Expression catalog

The expression catalog in `src/app/scenery/saya/expressions.rs` is the source of
truth for the enum, gallery, preloader and non-neutral click pool. Add variants
alphabetically, their artwork, dialogue and overlay, then refine page/action pools.
The gallery derives its count and grid from that catalog; it has no fixed capacity.
Hold the header emblem for two seconds to open it.

### Expression overlays

Overlays play once and are clipped to the portrait. Sprite paths below are
relative to `overlays/`.

| Expression | Sprite | Appearance and animation |
| --- | --- | --- |
| Celebrating | Procedural mesh | 64 folded metallic confetti pieces fall from the top over 2.4 seconds, with staggered arrivals, flutter and shaded folds. No additional texture or persistent particle simulation is needed. |
| Cute | `cute-paw.png`, `cute-whiskers.png` | A shaded black-and-pink paw stamp fits over her raised right hand. Two curved ink whisker clusters are fitted independently around her nose. |
| Excited | `excited-sparkles.png` | Transparent sparkles with eye glints anchored to pupil centers in the 512 × 512 expression artwork. Update the anchors when replacing the portrait. |
| Fed | `fed-blood.png` | Organic crimson rivulets and wet highlights flow downward from the top at different rates across the image. |
| Flirty | `flirty-kiss.png` | A burgundy lipstick imprint appears as a foreground camera-kiss stamp. |
| Hurt | `hurt-scratches.png` | A detailed crimson claw decal is revealed diagonally across the portrait before fading. |
| Love | `love-hearts.png` | A transparent ruby-heart cluster rises and fades in three staggered bursts with its original one-shot timing. |
| Pouting | `pouting-exhale.png` | A compact anime huff sprite has its open neck anchored beside her lips. |
| Question | `question-lines.png` | Tapered manga emphasis rays appear above and below her touching fingertips. |
