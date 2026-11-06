"""Render portbak icon sizes from the same geometry as assets/icon/portbak.svg."""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

GRID = 512  # design grid of portbak.svg; every coordinate below is on that grid
TILE_START = (20, 184, 166)  # #14B8A6
TILE_END = (17, 94, 89)  # #115E59
NAVY = "#172D46"
SKIN = "#FFC393"
CREAM = "#FFF1D6"
PNG_SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]

OUT = Path(__file__).resolve().parent.parent / "assets" / "icon"


def scaled(k, points):
    return [(x * k, y * k) for x, y in points]


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

    def oval(bounds, fill, outline=None, width=8):
        draw.ellipse(rect(k, *bounds), fill=fill, outline=outline, width=width * k)

    def rounded(bounds, radius, fill, outline=None, width=8):
        draw.rounded_rectangle(
            rect(k, *bounds), radius=radius * k, fill=fill,
            outline=outline, width=width * k,
        )

    # Tiny icons devote their entire silhouette to the cap and face.
    compact = target < 32
    if not compact:
        rounded((108, 324, 404, 460), 66, NAVY)
    oval((137, 208, 189, 286), SKIN, NAVY)
    oval((323, 208, 375, 286), SKIN, NAVY)
    rounded((164, 155, 348, 376 if compact else 354), 86, SKIN, NAVY)

    # An oversized work cap, with a broad visor and a simple rope band.
    rounded((124, 80, 388, 211), 60, NAVY)
    rounded((144, 153, 368, 177), 10, "#54738B")
    rounded((133, 184, 379, 223), 18, NAVY)
    draw.line(scaled(k, [(168, 197), (344, 197)]), fill=CREAM, width=8 * k)
    oval((199, 236, 215, 254), NAVY)
    oval((297, 236, 313, 254), NAVY)
    # The split moustache and oversized nose carry the character at small sizes.
    oval((188, 270, 262, 314), NAVY)
    oval((250, 270, 324, 314), NAVY)
    oval((231, 247, 281, 289), "#F49B70")
    if not compact:
        draw.arc(rect(k, 229, 299, 283, 327), 10, 170, fill=CREAM, width=8 * k)
        # A folded sheet reads as a file, rather than a shipping box.
        draw.polygon(scaled(k, [(209, 351), (285, 351), (319, 385),
                                  (319, 451), (209, 451)]), fill=CREAM)
        draw.line(scaled(k, [(209, 451), (209, 351), (285, 351),
                             (319, 385), (319, 451), (209, 451)]),
                  fill=NAVY, width=8 * k, joint="curve")
        draw.line(scaled(k, [(285, 351), (285, 385), (319, 385)]),
                  fill=NAVY, width=8 * k, joint="curve")
        draw.line(scaled(k, [(233, 411), (292, 411)]), fill="#54738B", width=8 * k)
        oval((174, 380, 223, 428), SKIN, NAVY, 6)
        oval((305, 380, 354, 428), SKIN, NAVY, 6)

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
