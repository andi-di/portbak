# Saya interface icons

Each function has its own transparent PNG; do not combine icons into a sprite
sheet. Generation prompts and masters live locally under
`agent-local/refinements/characters/saya/ui-icons/`, outside runtime assets.
The archive emblem and shield-shaped job controls are in `v3/`.
The current Saya job heads, normalization palette, previews and prompts live in
`agent-local/refinements/characters/saya/pixel-head/job-icons-v5/`.

## Resolution and size

- Functional icons are exactly **128 × 128 pixels**, RGBA PNG.
- Saya heads (active, paused, job title and app header) use **128 × 128 pixels**,
  with a centered 104-pixel maximum visible extent and no added pale contour.
  Their reference palette is identical across light and dark modes.
  Favor a coarse 32-pixel design grid and broad flat color clusters for faces.
- Fit the visible symbol proportionally inside a **104 × 104 pixel** area.
  Its longest dimension is 104 pixels; never stretch it to fill both dimensions.
- Center the visible bounds on the canvas, leaving at least **12 pixels of
  transparent padding** on each side along the longest dimension.
- Empty background pixels have alpha zero. Remove stray background specks before
  measuring bounds; preserve intentional internal transparent gaps.
- Keep the same canvas and occupied-area rule for replacements and new icons.
  Check readability at 24, 32, and 48 pixels before use.
- Utility buttons render at 32 logical pixels; New job and Check now use 40.
  Job status heads render at 72 logical pixels in taller list entries; the
  job-title head renders at 80 logical pixels.
- Generate each functional silhouette individually. Derive the `dark/` (white)
  and `light/` (black) base masks from the same alpha mask so their
  geometry stays identical. The renderer adds a shallow inset silver bevel and
  broad diagonal metallic reflections and stepped facets: light silver for dark surfaces, dark silver for light
  surfaces. Clamp shading toward the readable foreground wherever needed to
  preserve 4.5:1 contrast. Keep Saya status and job-title heads in their original colors.
  Buttons select the variant from their actual fill,
  including hover and pressed states; standalone icons follow the theme mode.
- Use a coarse 32-pixel design grid, thick strokes and broad transparent gaps.
  Readability at 24 pixels takes priority over decorative detail. No fine
  engraving, interior gems, ribbons, gradients or multiple nested symbols.
- Job heads retain Saya's character colors. Aggressive eyes, visible vampire fangs and a play badge
  mean automatic backups are enabled; sleeping closed eyes and a pause badge
  mean automatic backups are disabled.
  Use the same fixed reference palette in both modes, with no white or silver
  outer contour. Favor broad faces and clear eyes. Keep status necks short and
  tapered, with clear separation between chin and black choker, plus the same
  red ruby in a silver mount on every head. Render the 23-pixel play/pause
  badges separately so their symbols remain crisp at list size.
- Utility controls show their frame only on hover, keyboard focus or selection.
- Validate enabled button text and icon contrast at 4.5:1 or higher across both
  palettes and normal, hovered and pressed states. Checkboxes use the same
  filled-surface variant selection with a broad checkmark.

## Visual direction

Saya's plum-black (`#35252C`, `#2C1D23`), burgundy (`#671D36`), shaded silver
(`#B5ABAB`), ivory (`#F1D0CA`), crimson (`#6B0F1F`), and moss green (`#48613B`).
Keep character colors in the Saya heads and surrounding controls. Functional
icons use high-contrast silver shading, bold silhouettes and crisp pixel clusters.
No text, backdrop, glow or cast shadow.
