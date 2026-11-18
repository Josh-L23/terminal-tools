#!/usr/bin/env python3
"""Generate the JConverter icon assets.

Writes the 256px application icon and the raw RGBA blob used for the window
icon, plus the hicolor sizes that the desktop install consumes.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "crates" / "jconverter" / "assets"
ICON_THEME = ROOT / "packaging" / "linux" / "icons" / "hicolor"

SIZES = (16, 32, 48, 64, 128, 256)
MASTER = 1024
TOP_COLOR = (63, 81, 181)
BOTTOM_COLOR = (0, 150, 136)
MARK_COLOR = (255, 255, 255, 235)


def _gradient(size: int) -> Image.Image:
    image = Image.new("RGBA", (size, size))
    draw = ImageDraw.Draw(image)
    for y in range(size):
        ratio = y / (size - 1)
        color = tuple(
            round(start + (end - start) * ratio)
            for start, end in zip(TOP_COLOR, BOTTOM_COLOR)
        )
        draw.line([(0, y), (size, y)], fill=(*color, 255))
    return image


def _arrow(
    draw: ImageDraw.ImageDraw,
    *,
    left: int,
    right: int,
    center_y: int,
    thickness: int,
    head: int,
    pointing_right: bool,
) -> None:
    top = center_y - thickness // 2
    bottom = top + thickness
    if pointing_right:
        shaft = (left, top, right - head // 2, bottom)
        tip = [(right - head, center_y - head // 2), (right, center_y), (right - head, center_y + head // 2)]
    else:
        shaft = (left + head // 2, top, right, bottom)
        tip = [(left + head, center_y - head // 2), (left, center_y), (left + head, center_y + head // 2)]
    draw.rounded_rectangle(shaft, radius=thickness // 3, fill=MARK_COLOR)
    draw.polygon(tip, fill=MARK_COLOR)


def render_master() -> Image.Image:
    canvas = Image.new("RGBA", (MASTER, MASTER), (0, 0, 0, 0))
    mask = Image.new("L", (MASTER, MASTER), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        [0, 0, MASTER - 1, MASTER - 1], radius=int(MASTER * 0.23), fill=255
    )
    canvas.paste(_gradient(MASTER), (0, 0), mask)

    draw = ImageDraw.Draw(canvas)
    margin = int(MASTER * 0.19)
    thickness = int(MASTER * 0.08)
    head = int(MASTER * 0.16)
    _arrow(
        draw,
        left=margin,
        right=MASTER - margin,
        center_y=int(MASTER * 0.38),
        thickness=thickness,
        head=head,
        pointing_right=True,
    )
    _arrow(
        draw,
        left=margin,
        right=MASTER - margin,
        center_y=int(MASTER * 0.62),
        thickness=thickness,
        head=head,
        pointing_right=False,
    )
    return canvas


def main() -> None:
    master = render_master()
    ASSETS.mkdir(parents=True, exist_ok=True)

    icon = master.resize((256, 256), Image.LANCZOS)
    icon.save(ASSETS / "jconverter.png", optimize=True)
    (ASSETS / "icon_rgba.raw").write_bytes(icon.tobytes())

    for size in SIZES:
        directory = ICON_THEME / f"{size}x{size}" / "apps"
        directory.mkdir(parents=True, exist_ok=True)
        master.resize((size, size), Image.LANCZOS).save(
            directory / "jconverter.png", optimize=True
        )

    print(f"Wrote {ASSETS / 'jconverter.png'} and {len(SIZES)} hicolor sizes")


if __name__ == "__main__":
    main()
