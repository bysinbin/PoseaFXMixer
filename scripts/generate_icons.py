#!/usr/bin/env python3
"""
Posea Audio Official Icon Generator
Generates high-resolution PNG, multi-resolution Windows ICO, macOS ICNS, and resized PNG assets.
"""

import os
from PIL import Image, ImageDraw, ImageFilter

def create_posea_icon(size=512):
    scale = size / 512.0
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    pad = int(24 * scale)
    corner = int(108 * scale)
    rect = [pad, pad, size - pad, size - pad]

    # Glow effect
    glow_img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow_img)
    glow_draw.rounded_rectangle(rect, radius=corner, fill=(255, 42, 85, 40))
    glow_img = glow_img.filter(ImageFilter.GaussianBlur(radius=int(12 * scale)))
    img.paste(glow_img, (0, 0), glow_img)

    # Dark squircle badge
    draw.rounded_rectangle(rect, radius=corner, fill=(18, 19, 26, 255), outline=(255, 42, 85, 180), width=int(5 * scale))

    # Inner rim
    inner_pad = pad + int(5 * scale)
    inner_rect = [inner_pad, inner_pad, size - inner_pad, size - inner_pad]
    draw.rounded_rectangle(inner_rect, radius=int(corner * 0.95), outline=(42, 45, 60, 200), width=int(2 * scale))

    # 5 Harmonic Equalizer Bars
    bars = [
        (136, 52, (220, 30, 70), (170, 20, 50), 30),
        (196, 110, (255, 60, 100), (220, 30, 70), 30),
        (256, 160, (255, 90, 130), (255, 42, 85), 32),
        (316, 110, (255, 60, 100), (220, 30, 70), 30),
        (376, 52, (220, 30, 70), (170, 20, 50), 30),
    ]

    cy = 256 * scale
    for cx_raw, hh_raw, c_top, c_bot, w_raw in bars:
        cx = cx_raw * scale
        hh = hh_raw * scale
        w = w_raw * scale
        r = w / 2.0
        bar_rect = [cx - r, cy - hh, cx + r, cy + hh]
        draw.rounded_rectangle(bar_rect, radius=int(r), fill=c_bot)

        top_rect = [cx - r + int(2 * scale), cy - hh + int(2 * scale), cx + r - int(2 * scale), cy]
        draw.rounded_rectangle(top_rect, radius=int(r - 2 * scale), fill=c_top)

    # Apex highlight dot
    draw.ellipse([
        256 * scale - int(6 * scale),
        (256 - 160) * scale - int(1 * scale),
        256 * scale + int(6 * scale),
        (256 - 160) * scale + int(11 * scale)
    ], fill=(255, 255, 255, 240))

    return img

def main():
    out_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'assets')
    os.makedirs(out_dir, exist_ok=True)

    master = create_posea_icon(512)
    master.save(os.path.join(out_dir, 'icon.png'), 'PNG')
    master.save(os.path.join(out_dir, 'icon_512.png'), 'PNG')

    # Multi-size ICO for Windows
    master.save(
        os.path.join(out_dir, 'icon.ico'),
        format='ICO',
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )

    # ICNS for macOS
    master.save(os.path.join(out_dir, 'icon.icns'), format='ICNS')

    # Standard size PNGs
    for s in [16, 32, 48, 64, 128, 256]:
        master.resize((s, s), Image.Resampling.LANCZOS).save(os.path.join(out_dir, f'icon_{s}.png'), 'PNG')

    print(f"Successfully generated all icon assets in {out_dir}")

if __name__ == '__main__':
    main()
