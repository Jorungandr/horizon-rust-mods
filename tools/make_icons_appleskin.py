"""Generate the two registry icons: 128 px square, flat, readable at 32 px."""
from PIL import Image, ImageDraw

SIZE = 128
SS = 4  # supersample


def rounded(bg):
    img = Image.new("RGBA", (SIZE * SS, SIZE * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle(
        [0, 0, SIZE * SS - 1, SIZE * SS - 1], radius=26 * SS, fill=bg
    )
    return img, d


def down(img, path):
    img.resize((SIZE, SIZE), Image.LANCZOS).save(path)


def appleskin(path):
    img, d = rounded((32, 24, 18, 255))
    s = SS
    # three hunger icons along the bottom, the right one half full
    gold = (217, 162, 27, 255)
    dark = (58, 42, 20, 255)
    y = 92 * s
    for i in range(3):
        x = (26 + i * 26) * s
        d.rounded_rectangle([x, y, x + 22 * s, y + 10 * s], radius=3 * s, fill=dark)
        fill_w = 22 if i < 2 else 11
        d.rounded_rectangle([x, y, x + fill_w * s, y + 10 * s], radius=3 * s, fill=gold)
    # an apple: two lobes, a leaf and a stem
    red = (226, 74, 61, 255)
    d.ellipse([30 * s, 26 * s, 62 * s, 66 * s], fill=red)
    d.ellipse([58 * s, 26 * s, 90 * s, 66 * s], fill=red)
    d.rectangle([58 * s, 30 * s, 62 * s, 60 * s], fill=red)
    d.ellipse([42 * s, 38 * s, 50 * s, 48 * s], fill=(255, 150, 140, 255))
    d.line([60 * s, 30 * s, 64 * s, 14 * s], fill=(120, 78, 40, 255), width=4 * s)
    d.ellipse([64 * s, 12 * s, 88 * s, 26 * s], fill=(94, 173, 74, 255))
    down(img, path)


def dynamicfps(path):
    img, d = rounded((18, 26, 38, 255))
    s = SS
    # a gauge arc, a needle and a small "paused" pair of bars
    cx, cy, r = 64 * s, 78 * s, 36 * s
    d.arc([cx - r, cy - r, cx + r, cy + r], 180, 360, fill=(84, 112, 158, 255), width=8 * s)
    d.arc([cx - r, cy - r, cx + r, cy + r], 210, 268, fill=(96, 208, 176, 255), width=8 * s)
    d.line([cx, cy, cx - 20 * s, cy - 22 * s], fill=(240, 244, 250, 255), width=6 * s)
    d.ellipse([cx - 7 * s, cy - 7 * s, cx + 7 * s, cy + 7 * s], fill=(240, 244, 250, 255))
    # three steps down: the frame rate the mod leaves the window at
    steps = [((38, 88), (56, 104), (96, 208, 176, 255)),
             ((62, 94), (80, 104), (72, 150, 150, 255)),
             ((86, 99), (104, 104), (84, 112, 158, 255))]
    for (x0, y0), (x1, y1), colour in steps:
        d.rounded_rectangle([x0 * s, y0 * s, x1 * s, y1 * s], radius=2 * s, fill=colour)
    down(img, path)


if __name__ == "__main__":
    import sys

    out = sys.argv[1]
    appleskin(f"{out}/appleskin.png")
    dynamicfps(f"{out}/dynamicfps.png")
    print("icons written")
