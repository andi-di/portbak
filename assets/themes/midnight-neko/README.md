# Midnight Neko artwork

Theme assets:

- `saya-full-body.png`: static pixel-art welcome figure.
- `saya-emblem.png`: Kurobane family crest beside the application and job names.
- `expressions/`: all 27 refined pixel busts for the sidebar; click to show one
  of the 26 non-neutral expressions without immediate repeats. Hovering holds the
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
from the letterbox; the outer frame stays plain. A separate transparent silver-vine and burgundy-rose
overlay forms a symmetrical inner lining rooted at the portrait border. The strip contains
a centered caret and small matching corner caps.

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

`portrait-ornament.png` follows the original `saya-frame.png` style with fine
bevelled silver rails, muted burgundy enamel, tiny curled corner caps, restrained
thorn stems and small ribbon accents. Sparse, narrow details keep Saya dominant. It is drawn edge-to-edge
inside the portrait, without enlargement or floating margins. One quadrant is
reflected across both axes for exact symmetry.

The min/max pane uses only small mirrored corner caps sampled from the same
frame artwork, with the center left clear for the caret. The frame was made with
built-in imagegen; `portrait-ornament.prompt.txt` records the prompt. Textures
larger than 1024 pixels are reduced when loaded for GPU compatibility.
