# Midnight Neko artwork

Theme assets:

- `saya-full-body.png`: static pixel-art welcome figure.
- `saya-head.png`: simplified 64 × 64 pixel portrait beside the application name.
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
