"""Render portbak icon sizes from the same geometry as assets/icon/portbak.svg."""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

GRID = 512  # design grid of portbak.svg; every coordinate below is on that grid
TILE_START = (20, 184, 166)  # #14B8A6
TILE_END = (17, 94, 89)  # #115E59
NAVY = "#172D46"
GOLD = "#FFD16B"
CREAM = "#FFF1D6"
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

    # Broad silhouettes keep all three symbols readable at small sizes.
    draw.polygon(scaled(k, [(78, 172), (174, 172), (204, 208),
                            (322, 208), (322, 390), (78, 390)]), fill=NAVY)
    draw.polygon(scaled(k, [(90, 184), (168, 184), (198, 220),
                            (310, 220), (310, 378), (90, 378)]), fill="#E8A63F")
    draw.rounded_rectangle(rect(k, 78, 218, 330, 398), radius=22 * k, fill=NAVY)
    draw.rounded_rectangle(rect(k, 90, 230, 318, 386), radius=10 * k, fill=GOLD)

    draw.arc(ellipse(k, 202, 308, 46), 45, 315, fill=NAVY, width=18 * k)
    draw.polygon(scaled(k, [(254, 270), (254, 310), (214, 284)]), fill=NAVY)

    # A cream keyline separates the overlapping anchor from the folder.
    for color, width in ((CREAM, 44), (NAVY, 24)):
        draw.ellipse(ellipse(k, 370, 158, 31 if color == CREAM else 25),
                     outline=color, width=(24 if color == CREAM else 12) * k)
        draw.line(scaled(k, [(370, 184), (370, 368)]), fill=color, width=width * k)
        draw.line(scaled(k, [(332, 228), (408, 228)]), fill=color, width=width * k)
        bounds = (300, 240, 440, 380) if color == CREAM else (310, 250, 430, 370)
        draw.arc(rect(k, *bounds), 0, 180, fill=color, width=width * k)
    for points in ([(294, 292), (334, 306), (307, 330)],
                   [(446, 292), (406, 306), (433, 330)]):
        draw.polygon(scaled(k, points), fill=CREAM)
    for points in ([(303, 302), (326, 310), (310, 324)],
                   [(437, 302), (414, 310), (430, 324)]):
        draw.polygon(scaled(k, points), fill=NAVY)

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
        images[max(ICO_SIZES)].save(
            ico, sizes=[(size, size) for size in ICO_SIZES],
            append_images=[images[size] for size in ICO_SIZES[:-1]],
        )
        print(ico)


if __name__ == "__main__":
    main()
