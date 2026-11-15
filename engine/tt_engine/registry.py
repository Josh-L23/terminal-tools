from __future__ import annotations

from dataclasses import dataclass
from importlib import import_module
from pathlib import Path
from typing import Callable

Converter = Callable[[Path, Path], str]


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


CONVERSIONS = (
    Conversion("md", "pdf", "weasyprint", "md_pdf"),
    Conversion("docx", "pdf", "libreoffice", "docx_pdf"),
    Conversion("docx", "md", "mammoth", "docx_md"),
    Conversion("pdf", "md", "pymupdf4llm", "pdf_md"),
)

REGISTRY = {(conversion.source, conversion.target): conversion for conversion in CONVERSIONS}


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

    return {
        "ok": True,
        "formats": ["md", "pdf", "docx"],
        "conversions": conversions,
    }
