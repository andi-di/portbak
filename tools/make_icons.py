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
        rounded((112, 320, 400, 460), 58, NAVY)
        rounded((145, 329, 367, 456), 32, CREAM)
        for y in (348, 376, 404, 432):
            draw.rectangle(rect(k, 150, y, 362, y + 12), fill=NAVY)
        draw.polygon(scaled(k, [(180, 325), (256, 369), (332, 325),
                                  (307, 321), (256, 347), (205, 321)]), fill=NAVY)
    oval((137, 208, 189, 286), SKIN, NAVY)
    oval((323, 208, 375, 286), SKIN, NAVY)
    rounded((164, 155, 348, 376 if compact else 354), 86, SKIN, NAVY)

    # White sailor cap and trailing ribbon distinguish him from a dock worker.
    draw.polygon(scaled(k, [(345, 176), (390, 225), (388, 167)]), fill=NAVY)
    rounded((124, 86, 388, 203), 48, CREAM, NAVY)
    rounded((140, 168, 372, 220), 14, NAVY)
    draw.line(scaled(k, [(158, 180), (354, 180)]), fill="#54738B", width=6 * k)
    oval((199, 236, 215, 254), NAVY)
    oval((297, 236, 313, 254), NAVY)
    # The split moustache and oversized nose carry the character at small sizes.
    oval((188, 270, 262, 314), NAVY)
    oval((250, 270, 324, 314), NAVY)
    oval((231, 247, 281, 289), "#F49B70")
    if not compact:
        draw.arc(rect(k, 229, 299, 283, 327), 10, 170, fill=CREAM, width=8 * k)
        # Wide tabbed folder stays recognizable when the icon is reduced.
        draw.polygon(scaled(k, [(174, 382), (174, 358), (239, 358),
                                  (257, 378), (337, 378), (337, 451), (174, 451)]),
                     fill="#E8A63F")
        draw.line(scaled(k, [(174, 451), (174, 358), (239, 358),
                             (257, 378), (337, 378), (337, 451)]),
                  fill=NAVY, width=8 * k, joint="curve")
        rounded((172, 385, 342, 453), 10, "#FFD16B", NAVY)
        draw.line(scaled(k, [(193, 400), (321, 400)]), fill=CREAM, width=6 * k)
        rounded((115, 365, 173, 449), 27, SKIN, NAVY, 6)
        rounded((339, 365, 397, 449), 27, SKIN, NAVY, 6)
        # Keep the anchor bold enough to read on the exposed forearm at 64px.
        oval((357, 378, 373, 394), None, NAVY, 5)
        draw.line(scaled(k, [(365, 392), (365, 428)]), fill=NAVY, width=6 * k)
        draw.line(scaled(k, [(352, 403), (378, 403)]), fill=NAVY, width=5 * k)
        draw.arc(rect(k, 349, 397, 381, 430), 0, 180, fill=NAVY, width=6 * k)
        draw.polygon(scaled(k, [(347, 414), (357, 416), (349, 424)]), fill=NAVY)
        draw.polygon(scaled(k, [(383, 414), (373, 416), (381, 424)]), fill=NAVY)

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
