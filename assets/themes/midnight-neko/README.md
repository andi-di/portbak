# Midnight Neko artwork

These embedded PNG atlases are the approved character illustrations, created with OpenAI's built-in image generation tool. The app selects portrait regions at runtime; it does not draw over the detailed eyes or show the concept-board labels.

- `expressions.png`: neutral character with decorative scythe, nine reactions, and the matching pixel-art face icon.
- `poses.png`: bored and hand-on-cheek sideways-glance portraits.
- `yandere.png`: wide-eyed expression with tiny pupils.
- `shy.png`: shy blush and hesitant smile.

The exact prompts and reference roles are recorded in the sibling `*-prompt.md` files. Original generation outputs are retained separately; the project uses these approved copies. Portrait coordinates and display rules live in `src/app/scenery/catgirl.rs`. Jobs-view portraits are normalized at load time to 384 × 448 pixels. Measured iris centers align the eye line and face scale across all expressions without repainting their eyes. The full-figure illustration is used only on the welcome screen. Portraits use linear sampling, while the icon is extracted and sampled with nearest-neighbor filtering to retain its pixel edges.
