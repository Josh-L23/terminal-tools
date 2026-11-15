from __future__ import annotations

import shutil
import subprocess
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path

from ..errors import ConversionError


def _pdftotext() -> str | None:
    return shutil.which("pdftotext")


def available() -> tuple[bool, str]:
    try:
        import pymupdf4llm  # noqa: F401

        try:
            package_version = version("pymupdf4llm")
        except PackageNotFoundError:
            package_version = "unknown"
        return True, f"PyMuPDF4LLM {package_version}"
    except Exception as exc:
        if _pdftotext():
            return (
                True,
                f"PyMuPDF unavailable ({type(exc).__name__}); using pdftotext fallback",
            )
        return False, f"PyMuPDF4LLM and pdftotext are both unavailable: {exc}"


def convert(source: Path, target: Path) -> str:
    try:
        import pymupdf4llm
    except Exception:
        pymupdf4llm = None

    if pymupdf4llm is not None:
        try:
            markdown = pymupdf4llm.to_markdown(str(source))
        except Exception as exc:
            raise ConversionError(
                f"failed to extract Markdown from the PDF: {exc}"
            ) from exc
        backend = "pymupdf4llm"
    else:
        executable = _pdftotext()
        if not executable:
            raise ConversionError(
                "the PDF → Markdown backend is missing; install PyMuPDF4LLM or poppler-utils"
            )

        result = subprocess.run(
            [executable, "-layout", str(source), "-"],
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        if result.returncode != 0:
            detail = (result.stderr or result.stdout or "no output").strip()
            raise ConversionError(f"pdftotext failed: {detail}")

        markdown = result.stdout
        backend = "pdftotext"

    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(markdown.strip() + "\n", encoding="utf-8")

    return backend
