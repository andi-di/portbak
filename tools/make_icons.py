"""Render portbak icon sizes from the same geometry as assets/icon/portbak.svg."""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

GRID = 512  # design grid of portbak.svg; every coordinate below is on that grid
TILE_START = (20, 184, 166)  # #14B8A6
TILE_END = (17, 94, 89)  # #115E59
WHITE = (255, 255, 255, 255)
GREEN = (40, 200, 90, 255)  # matches the green status dot in the UI
PNG_SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

OUT = Path(__file__).resolve().parent.parent / "assets" / "icon"


def scaled(k, points):
    return [(x * k, y * k) for x, y in points]


def ellipse(k, cx, cy, r):
    return ((cx - r) * k, (cy - r) * k, (cx + r) * k, (cy + r) * k)


def rect(k, x0, y0, x1, y1):
    return (x0 * k, y0 * k, x1 * k, y1 * k)


def tile_gradient(side):
    n = 128
    pixels = bytearray()
    for y in range(n):
        for x in range(n):
            t = (x + y) / (2 * (n - 1))
            pixels += bytes(a + round((b - a) * t) for a, b in zip(TILE_START, TILE_END))
    img = Image.frombytes("RGB", (n, n), bytes(pixels))
    return img.resize((side, side), Image.BICUBIC)


def render(target):
    k = 8 if target >= 1024 else 4  # supersample, then downscale for antialiasing
    side = GRID * k
    img = Image.new("RGBA", (side, side), (0, 0, 0, 0))

    mask = Image.new("L", (side, side), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (24 * k, 24 * k, 488 * k, 488 * k), radius=104 * k, fill=255
    )
    img.paste(tile_gradient(side), (0, 0), mask)
    draw = ImageDraw.Draw(img)

    # backup-cycle arc sweeping clockwise over the folder
    draw.arc(ellipse(k, 256, 270, 165), 200, 340, fill=WHITE, width=30 * k)
    draw.ellipse(ellipse(k, 115, 219, 15), fill=WHITE)  # round start cap
    draw.polygon(scaled(k, [(409, 251), (370, 220), (419, 202)]), fill=WHITE)

    # folder: body, tab, and the slanted shoulder between them
    draw.rounded_rectangle(rect(k, 140, 260, 372, 414), fill=WHITE, radius=24 * k)
    draw.rounded_rectangle(rect(k, 140, 234, 240, 292), fill=WHITE, radius=20 * k)
    draw.polygon(scaled(k, [(226, 234), (263, 262), (226, 262)]), fill=WHITE)

    # verified checkmark
    points = scaled(k, [(214, 334), (240, 360), (298, 300)])
    draw.line(points, fill=GREEN, width=28 * k, joint="curve")
    for cx, cy in (points[0], points[-1]):
        draw.ellipse((cx - 14 * k, cy - 14 * k, cx + 14 * k, cy + 14 * k), fill=GREEN)

    return img.resize((target, target), Image.LANCZOS)


def main():
    sizes = [int(a) for a in sys.argv[1:]] or PNG_SIZES
    images = {size: render(size) for size in sizes}
    for size, img in images.items():
        path = OUT / f"portbak-{size}.png"
        img.save(path)
        print(path)
    if all(size in images for size in ICO_SIZES):
        ico = OUT / "portbak.ico"
        images[ICO_SIZES[0]].save(ico, sizes=[(size, size) for size in ICO_SIZES])
        print(ico)


if __name__ == "__main__":
    main()
