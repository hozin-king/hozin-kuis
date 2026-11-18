#!/usr/bin/env python3
"""Generate launcher icons for Hozin Kuis from the bundled Pikachu sprite.

Master 512x512: Pikachu (assets/pokemon/25.png, pixel art 96x96) upscaled
NEAREST onto the app gradient (GRAD_TOP -> GRAD_MID -> GRAD_BOT, same
colors as src/lib.rs). Density buckets are downscaled from the master
with LANCZOS.

Output:
  tools/ic_launcher_512.png        (master, not packaged)
  android_res/mipmap-*/ic_launcher.png  (48/72/96/144/192, packaged by cargo-apk)

Run: python3 tools/gen_icon.py
"""
import os

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPRITE = os.path.join(ROOT, "assets", "pokemon", "25.png")
OUT = os.path.join(ROOT, "android_res")

TOP = (10, 18, 48)  # GRAD_TOP
MID = (26, 22, 70)  # GRAD_MID
BOT = (56, 20, 84)  # GRAD_BOT
MID_POS = 0.55


def lerp(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def gradient(size):
    img = Image.new("RGB", (size, size))
    px = img.load()
    for y in range(size):
        t = y / (size - 1)
        if t < MID_POS:
            c = lerp(TOP, MID, t / MID_POS)
        else:
            c = lerp(MID, BOT, (t - MID_POS) / (1 - MID_POS))
        for x in range(size):
            px[x, y] = c
    return img


def main():
    master_size = 512
    bg = gradient(master_size).convert("RGBA")
    sprite = Image.open(SPRITE).convert("RGBA")
    # Potong padding transparan supaya Pikachu mengisi ikon (lebih terbaca
    # di ukuran launcher kecil), lalu perbesar NEAREST agar pixel art tetap tajam.
    bbox = sprite.getbbox()
    if bbox is not None:
        sprite = sprite.crop(bbox)
    poke_size = 380
    sprite = sprite.resize((poke_size, poke_size), Image.NEAREST)
    off = (master_size - poke_size) // 2
    bg.alpha_composite(sprite, (off, off))
    bg.save(os.path.join(ROOT, "tools", "ic_launcher_512.png"))
    buckets = {
        "mipmap-mdpi": 48,
        "mipmap-hdpi": 72,
        "mipmap-xhdpi": 96,
        "mipmap-xxhdpi": 144,
        "mipmap-xxxhdpi": 192,
    }
    for name, size in buckets.items():
        d = os.path.join(OUT, name)
        os.makedirs(d, exist_ok=True)
        bg.resize((size, size), Image.LANCZOS).save(
            os.path.join(d, "ic_launcher.png")
        )
    print("icon master + buckets generated")


if __name__ == "__main__":
    main()
