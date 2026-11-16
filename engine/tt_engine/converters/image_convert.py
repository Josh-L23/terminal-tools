from __future__ import annotations

from pathlib import Path

from ..errors import ConversionError

# Pillow format names for saving by our canonical format id.
_SAVE_FORMAT = {
    "png": "PNG",
    "jpg": "JPEG",
    "webp": "WEBP",
    "bmp": "BMP",
    "gif": "GIF",
    "tiff": "TIFF",
}

# Formats that cannot store an alpha channel.
_NO_ALPHA = {"jpg", "bmp"}

# Formats that do not benefit from/accept the same quality parameter.
_LOSSY = {"jpg", "webp"}


def available() -> tuple[bool, str]:
    try:
        from PIL import Image

        try:
            from importlib.metadata import PackageNotFoundError, version

            try:
                package_version = version("Pillow")
            except PackageNotFoundError:
                package_version = getattr(Image, "__version__", "unknown")
        except Exception:
            package_version = getattr(Image, "__version__", "unknown")
        return True, f"Pillow {package_version}"
    except Exception as exc:
        return False, f"Pillow is unavailable: {exc}"


def convert(source: Path, target: Path) -> str:
    try:
        from PIL import Image, ImageOps
    except ImportError as exc:
        raise ConversionError(
            "the image conversion backend is missing; run scripts/setup.sh"
        ) from exc

    target_format = target.suffix.lower().lstrip(".")
    if target_format == "jpeg":
        target_format = "jpg"
    if target_format == "tif":
        target_format = "tiff"

    save_format = _SAVE_FORMAT.get(target_format)
    if save_format is None:
        raise ConversionError(f"unsupported target image format: {target_format}")

    try:
        with Image.open(source) as image:
            image.load()
            image = ImageOps.exif_transpose(image)

            # For animated images, use the first frame for a deterministic result.
            if getattr(image, "is_animated", False):
                try:
                    image.seek(0)
                except Exception:
                    pass
                image = image.copy()

            if target_format in _NO_ALPHA:
                if image.mode in {"RGBA", "LA"} or (
                    image.mode == "P" and "transparency" in image.info
                ):
                    background = Image.new("RGB", image.size, (255, 255, 255))
                    rgba = image.convert("RGBA")
                    background.paste(rgba, mask=rgba.split()[-1])
                    image = background
                elif image.mode not in {"RGB", "L"}:
                    image = image.convert("RGB")
                elif image.mode == "L" and target_format == "jpg":
                    # Keep grayscale JPEGs compact/valid.
                    pass
                elif image.mode != "RGB" and target_format == "jpg":
                    image = image.convert("RGB")

            elif target_format in {"png", "webp", "tiff"}:
                if image.mode == "P":
                    image = image.convert("RGBA" if "transparency" in image.info else "RGB")
                elif image.mode not in {"RGB", "RGBA", "L", "LA"}:
                    image = image.convert("RGBA")
            elif target_format == "gif":
                # GIF supports palette/animation; for static conversion keep it simple.
                if image.mode not in {"P", "L"}:
                    if image.mode == "RGBA" or (
                        image.mode == "P" and "transparency" in image.info
                    ):
                        image = image.convert("RGBA")
                    else:
                        image = image.convert("RGB")
            elif target_format == "bmp" and image.mode not in {"RGB", "L"}:
                image = image.convert("RGB")

            save_kwargs: dict[str, object] = {}
            if target_format in _LOSSY:
                save_kwargs["quality"] = 92
                if target_format == "webp":
                    save_kwargs["method"] = 6
            if target_format == "jpg":
                save_kwargs["optimize"] = True
                save_kwargs["progressive"] = True
            if target_format == "png":
                save_kwargs["optimize"] = True
            if target_format == "tiff":
                save_kwargs["compression"] = "tiff_deflate"

            target.parent.mkdir(parents=True, exist_ok=True)
            image.save(target, format=save_format, **save_kwargs)
    except ConversionError:
        raise
    except Exception as exc:
        raise ConversionError(f"failed to convert the image: {exc}") from exc

    return "pillow"
