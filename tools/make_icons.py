"""Render the SVG master with librsvg, then package PNGs and the Windows ICO."""

import io
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image

PIXEL_GRID = 16
PNG_SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
OUT = Path(__file__).resolve().parent.parent / "assets" / "icon"


def render(target):
    result = subprocess.run(
        ["rsvg-convert", "--width", str(PIXEL_GRID), "--height", str(PIXEL_GRID)],
        input=(OUT / "portbak.svg").read_bytes(), capture_output=True, check=True,
    )
    with Image.open(io.BytesIO(result.stdout)) as image:
        return image.convert("RGBA").resize((target, target), Image.Resampling.NEAREST)


def main():
    if not shutil.which("rsvg-convert"):
        raise SystemExit("Install librsvg's rsvg-convert to render the SVG master.")
    sizes = [int(arg) for arg in sys.argv[1:]] or PNG_SIZES
    if any(size <= 0 for size in sizes):
        raise SystemExit("Icon sizes must be positive integers.")
    images = {size: render(size) for size in sizes}
    for size, image in images.items():
        path = OUT / f"portbak-{size}.png"
        image.save(path)
        print(path)
    if all(size in images for size in ICO_SIZES):
        path = OUT / "portbak.ico"
        images[max(ICO_SIZES)].save(
            path, sizes=[(size, size) for size in ICO_SIZES],
            append_images=[images[size] for size in ICO_SIZES[:-1]],
        )
        print(path)


if __name__ == "__main__":
    main()
