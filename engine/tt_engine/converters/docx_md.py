from __future__ import annotations

import mimetypes
import re
import sys
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

from ..errors import ConversionError


def available() -> tuple[bool, str]:
    try:
        import mammoth  # noqa: F401
        import markdownify  # noqa: F401

        mammoth_version = version("mammoth")
        markdownify_version = version("markdownify")
        return True, f"Mammoth {mammoth_version}, markdownify {markdownify_version}"
    except PackageNotFoundError as exc:
        return False, f"missing Python package: {exc}"
    except Exception as exc:
        return False, f"{type(exc).__name__}: {exc}"


def convert(source: Path, target: Path) -> str:
    try:
        import mammoth
        import markdownify
    except ImportError as exc:
        raise ConversionError(
            "the Word → Markdown backend is missing; run scripts/setup.sh"
        ) from exc

    media_directory = target.parent / f"{target.stem}_media"
    state = {"count": 0, "created": False}

    def save_image(image):
        state["count"] += 1
        extension = mimetypes.guess_extension(image.content_type or "") or ".bin"
        if extension == ".jpe":
            extension = ".jpg"
        name = f"image-{state['count']:03d}{extension}"

        with image.open() as handle:
            data = handle.read()

        if not state["created"]:
            media_directory.mkdir(parents=True, exist_ok=True)
            state["created"] = True

        (media_directory / name).write_bytes(data)

        return {"src": f"{media_directory.name}/{name}"}

    try:
        with source.open("rb") as handle:
            result = mammoth.convert_to_html(
                handle,
                convert_image=mammoth.images.img_element(save_image),
            )
    except Exception as exc:
        raise ConversionError(f"failed to read the Word document: {exc}") from exc

    for message in result.messages:
        if message.type == "warning":
            print(f"warning: {message.message}", file=sys.stderr)

    markdown = markdownify.markdownify(
        result.value,
        heading_style="ATX",
        bullets="-",
    )
    markdown = re.sub(r"\n{3,}", "\n\n", markdown).strip()

    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(markdown + "\n", encoding="utf-8")

    return "mammoth"
