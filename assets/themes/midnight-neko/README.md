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
and punctuation. The letterbox has a fixed 56-logical-pixel body height, with
41 pixels available for text after padding and decorative trim. Its width follows
the sidebar. Text scales from 11 to at most 13 logical pixels with wider
sidebars and shrinks in half-pixel steps for longer lines, down to a minimum of 8 logical pixels. Long unbroken words may
wrap between characters. Do not insert manual line breaks.

The dialogue-fit test checks every authored line and 120 wide characters in both
brightness modes at 300, 350, and 460 logical-pixel sidebar widths. Run
`cargo test --locked dialogue_budget` after editing dialogue. Both the character
budget and rendered bounds must pass; do not enlarge the box for individual lines.
