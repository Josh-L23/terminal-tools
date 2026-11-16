from __future__ import annotations

from dataclasses import dataclass
from importlib import import_module
from pathlib import Path
from typing import Callable

Converter = Callable[[Path, Path], str]


@dataclass(frozen=True)
class FormatSpec:
    name: str
    label: str
    extension: str
    aliases: tuple[str, ...] = ()

    @property
    def extensions(self) -> tuple[str, ...]:
        values = [self.extension, *[f".{alias}" for alias in self.aliases]]
        # Keep order stable and unique.
        seen: set[str] = set()
        ordered: list[str] = []
        for value in values:
            if value not in seen:
                seen.add(value)
                ordered.append(value)
        return tuple(ordered)


@dataclass(frozen=True)
class Conversion:
    source: str
    target: str
    backend: str
    module: str

    def load(self) -> Converter:
        module = import_module(f"tt_engine.converters.{self.module}")
        return module.convert

    def availability(self) -> tuple[bool, str]:
        try:
            module = import_module(f"tt_engine.converters.{self.module}")
            return module.available()
        except Exception as exc:
            return False, f"{type(exc).__name__}: {exc}"


FORMATS = (
    FormatSpec("md", "Markdown", ".md", ("markdown",)),
    FormatSpec("pdf", "PDF", ".pdf"),
    FormatSpec("docx", "Word document", ".docx", ("word",)),
    FormatSpec("png", "PNG image", ".png"),
    FormatSpec("jpg", "JPEG image", ".jpg", ("jpeg",)),
    FormatSpec("webp", "WebP image", ".webp"),
    FormatSpec("bmp", "BMP image", ".bmp"),
    FormatSpec("gif", "GIF image", ".gif"),
    FormatSpec("tiff", "TIFF image", ".tiff", ("tif",)),
)

FORMAT_BY_NAME = {spec.name: spec for spec in FORMATS}

# Extension/alias token (without dot, lowercased) -> canonical format name.
EXTENSION_TO_FORMAT: dict[str, str] = {}
for spec in FORMATS:
    for token in (spec.name, *spec.aliases, spec.extension.lstrip(".")):
        EXTENSION_TO_FORMAT[token.lower()] = spec.name
    # Ensure canonical extension token is present.
    EXTENSION_TO_FORMAT[spec.extension.lstrip(".").lower()] = spec.name

IMAGE_FORMATS = ("png", "jpg", "webp", "bmp", "gif", "tiff")


def _image_conversions() -> tuple[Conversion, ...]:
    conversions: list[Conversion] = []
    for source in IMAGE_FORMATS:
        for target in IMAGE_FORMATS:
            if source == target:
                continue
            conversions.append(
                Conversion(source, target, "pillow", "image_convert")
            )
    return tuple(conversions)


CONVERSIONS = (
    Conversion("md", "pdf", "weasyprint", "md_pdf"),
    Conversion("docx", "pdf", "libreoffice", "docx_pdf"),
    Conversion("docx", "md", "mammoth", "docx_md"),
    Conversion("pdf", "md", "pymupdf4llm", "pdf_md"),
    *_image_conversions(),
)

REGISTRY = {(conversion.source, conversion.target): conversion for conversion in CONVERSIONS}


def canonical_format(value: str) -> str | None:
    if not value:
        return None
    token = value.strip().lower()
    if token.startswith("."):
        token = token[1:]
    return EXTENSION_TO_FORMAT.get(token)


def get_conversion(source: str, target: str) -> Conversion | None:
    return REGISTRY.get((source, target))


def capabilities() -> dict:
    conversions = []
    for conversion in CONVERSIONS:
        available, detail = conversion.availability()
        conversions.append(
            {
                "from": conversion.source,
                "to": conversion.target,
                "backend": conversion.backend,
                "available": available,
                "detail": detail,
            }
        )

    format_details = {}
    for spec in FORMATS:
        format_details[spec.name] = {
            "name": spec.name,
            "label": spec.label,
            "extension": spec.extension,
            "extensions": list(spec.extensions),
            "aliases": list(spec.aliases),
        }

    return {
        "ok": True,
        "formats": [spec.name for spec in FORMATS],
        "format_details": format_details,
        "conversions": conversions,
    }

