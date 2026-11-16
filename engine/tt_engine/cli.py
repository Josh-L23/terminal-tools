from __future__ import annotations

import argparse
import json
import os
import platform
import sys
import traceback
from contextlib import redirect_stdout
from pathlib import Path

from . import __version__
from .errors import ConversionError
from .registry import FORMAT_BY_NAME, capabilities, canonical_format, get_conversion


def _format_choices() -> tuple[str, ...]:
    return tuple(FORMAT_BY_NAME.keys())


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="tt-engine",
        description="File conversion engine for Terminal Tools",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")

    format_choices = _format_choices()
    subparsers = parser.add_subparsers(dest="command", required=True)

    convert = subparsers.add_parser("convert", help="convert a file")
    convert.add_argument("--input", "-i", required=True, type=Path)
    convert.add_argument("--from", dest="source", required=True, choices=format_choices)
    convert.add_argument("--to", dest="target", required=True, choices=format_choices)
    convert.add_argument("--output", "-o", required=True, type=Path)

    subparsers.add_parser("doctor", help="check conversion backends")
    subparsers.add_parser("capabilities", help="list supported conversions")

    return parser


def _emit(payload: dict) -> None:
    print(json.dumps(payload, ensure_ascii=False))


def _convert(arguments: argparse.Namespace) -> int:
    source = arguments.input.expanduser()
    target = arguments.output.expanduser()

    if not source.is_file():
        raise ConversionError(f"input file not found: {source}")

    source_format = canonical_format(arguments.source) or arguments.source
    target_format = canonical_format(arguments.target) or arguments.target
    spec = FORMAT_BY_NAME.get(source_format)
    if spec is None:
        raise ConversionError(f"unsupported source format: {arguments.source}")

    suffix = source.suffix.lower()
    allowed = {ext.lower() for ext in spec.extensions}
    if suffix not in allowed:
        expected = ", ".join(spec.extensions)
        raise ConversionError(
            f"input file has extension '{source.suffix}', expected one of: {expected}"
        )

    if source.resolve() == target.resolve():
        raise ConversionError("refusing to overwrite the input file")

    conversion = get_conversion(source_format, target_format)
    if conversion is None:
        raise ConversionError(
            f"no converter for {source_format} → {target_format}"
        )

    target.parent.mkdir(parents=True, exist_ok=True)
    with redirect_stdout(sys.stderr):
        backend = conversion.load()(source, target)

    if not target.is_file():
        raise ConversionError("the converter did not produce an output file")

    _emit({"ok": True, "output": str(target.resolve()), "backend": backend})
    return 0


def _doctor() -> int:
    with redirect_stdout(sys.stderr):
        report = capabilities()
    checks = [
        {"name": "Python runtime", "ok": True, "detail": platform.python_version()},
        {"name": "Engine package", "ok": True, "detail": __version__},
    ]

    for conversion in report["conversions"]:
        checks.append(
            {
                "name": f"{conversion['from']} → {conversion['to']}",
                "ok": conversion["available"],
                "detail": conversion["detail"],
            }
        )

    _emit({"ok": all(check["ok"] for check in checks), "checks": checks})
    return 0


def _capabilities() -> int:
    with redirect_stdout(sys.stderr):
        report = capabilities()
    _emit(report)
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    arguments = parser.parse_args(argv)

    try:
        if arguments.command == "convert":
            return _convert(arguments)
        if arguments.command == "doctor":
            return _doctor()
        if arguments.command == "capabilities":
            return _capabilities()
        parser.error(f"unknown command: {arguments.command}")
    except ConversionError as exc:
        _emit({"ok": False, "error": str(exc)})
        return 1
    except Exception as exc:
        if os.environ.get("TT_DEBUG"):
            traceback.print_exc()
        _emit({"ok": False, "error": f"{type(exc).__name__}: {exc}"})
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
