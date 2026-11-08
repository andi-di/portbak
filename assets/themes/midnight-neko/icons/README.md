# Saya interface icons

Each function has its own transparent PNG and generation prompt; do not combine
the icons into a sprite sheet.

## Resolution and size

- Every production icon is exactly **128 × 128 pixels**, RGBA PNG.
- Fit the visible symbol proportionally inside a **104 × 104 pixel** area.
  Its longest dimension is 104 pixels; never stretch it to fill both dimensions.
- Center the visible bounds on the canvas, leaving at least **12 pixels of
  transparent padding** on each side along the longest dimension.
- Empty background pixels have alpha zero. Remove stray background specks before
  measuring bounds; preserve intentional internal transparent gaps.
- Keep the same canvas and occupied-area rule for replacements and new icons.
  Check readability at 24, 32, and 48 pixels before use.

## Visual direction

Saya's plum-black (`#35252C`, `#2C1D23`), burgundy (`#671D36`), shaded silver
(`#B5ABAB`), ivory (`#F1D0CA`), crimson (`#6B0F1F`), and moss green (`#48613B`).
Use bold functional silhouettes, crisp pixel clusters, restrained highlights and
small crescent, ribbon or gemstone details. No text, backdrop, glow or cast shadow.
