"""Prepare embedded Moonveil scenery at its runtime resolution (requires Pillow)."""
from pathlib import Path
import json
from PIL import Image

root = Path(__file__).resolve().parents[1] / "assets/themes/moonveil/scenery"
for name in ("cloud-bank.png", "mist-veil.png"):
    with Image.open(root / name) as source:
        source.thumbnail((1024, 1024))
        source.save(root / name)

with Image.open(root / "castle.png") as source:
    castle = source.convert("RGBA")
mask = Image.new("RGBA", castle.size)
cells = set()
for y in range(castle.height * 2 // 3):
    for x in range(castle.width):
        red, green, blue, alpha = castle.getpixel((x, y))
        if red - green > 35 and alpha:
            mask.putpixel((x, y), (min(255, red + 35), min(255, green + 12), blue, alpha))
            cells.add((x // 16, y // 16))
mask.save(root / "castle-windows.png")
(root / "castle-windows.json").write_text(
    json.dumps(sorted(cells, key=lambda cell: (cell[1], cell[0]))), encoding="utf8"
)
