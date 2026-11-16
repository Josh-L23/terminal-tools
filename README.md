# Terminal Tools

Local, terminal-first utilities for everyday file work. The first tool is `tt`,
a file converter that detects the input format and lets you pick a target
format interactively.

## Supported conversions

### Documents

| From             | To       | Backend                          |
| ---------------- | -------- | -------------------------------- |
| Markdown (`.md`) | PDF      | WeasyPrint                       |
| Word (`.docx`)   | PDF      | LibreOffice                      |
| Word (`.docx`)   | Markdown | Mammoth + markdownify            |
| PDF              | Markdown | PyMuPDF4LLM (`pdftotext` fallback) |

DOCX images are extracted next to the output Markdown file into
`<output-name>_media/`.

### Images

| From / To                           | Backend |
| ----------------------------------- | ------- |
| PNG, JPEG (`.jpg`/`.jpeg`), WebP, BMP, GIF, TIFF (`.tif`/`.tiff`) — convert between any of these formats | Pillow  |

Image conversions preserve orientation (EXIF transpose), use the first frame for
animated inputs, and flatten transparency to white for formats without alpha
(e.g. JPEG/BMP).

## Requirements

- Linux or macOS (Windows support is untested)
- Rust 1.85+ via [rustup](https://rustup.rs)
- Python 3.11+
- LibreOffice (`soffice`) for DOCX → PDF
- `pdftotext` from Poppler is optional, used only if PyMuPDF is unavailable

## Install

```bash
cd /path/to/terminal-tools
./scripts/setup.sh
```

The setup script is safe to re-run. It:

1. Creates a project-local `.venv`
2. Installs the Python conversion engine and its dependencies
3. Builds and installs the `tt` binary to `~/.cargo/bin`
4. Prints the PATH command if `~/.cargo/bin` is not already available

Verify everything is ready:

```bash
tt doctor
```

## Usage

```bash
# Point at a file and choose the target interactively
tt report.md
tt photo.png

# Specify the target directly
tt report.md --to pdf
tt report.docx --to md
tt document.pdf --to md
tt photo.png --to webp
tt image.jpeg --to png

# Choose an output file or an existing directory
tt report.docx --to md -o converted/
tt report.md --to pdf -o final-report.pdf
tt photo.png -o exports/

# Overwrite an existing output without prompting
tt report.docx --to md --force

# Open the result when finished
tt report.md --to pdf --open

# Inspect available targets for a file
tt formats report.docx
tt formats photo.png

# Check backends and dependencies
tt doctor
```

### Local web interface

Start the local interface from the project directory:

```bash
./.venv/bin/python scripts/web_ui.py
```

Then open <http://127.0.0.1:8765>. You can drop or choose a supported document
or image file, pick source and destination formats, and download the converted
file. The interface loads available formats from the engine (`/api/capabilities`)
and uses the same conversion backends as `tt`. Uploaded files are kept in a
temporary local directory while converting.

Environment variables: `TT_UI_PORT` (default `8765`), `TT_UI_HOST` (default
`127.0.0.1`), `TT_PYTHON` (interpreter for conversions), `TT_UI_MAX_UPLOAD`
(max upload size in bytes, default 100 MB). Stop with Ctrl+C.

By default, output is written next to the input file with the new extension.
Existing files are never overwritten silently: `tt` asks first, or fails when
running without a terminal unless `--force` is passed.

The explicit form works too:

```bash
tt convert report.md --to pdf
```

### Options

```text
tt convert <INPUT> [OPTIONS]

  -t, --to <FORMAT>      Target format (e.g. pdf, md, png, jpg, webp, tiff)
  -o, --output <PATH>    Output file or directory
  -f, --force            Overwrite an existing output
      --open             Open the result after conversion
  -h, --help             Show help
```

## Architecture

```text
crates/tt-core/       Format registry, conversion matrix, Python engine client
crates/tt-cli/        The `tt` command-line interface
engine/tt_engine/     Python conversion backends and JSON protocol
engine/tt_engine/converters/
                      One module per conversion, each exposing `convert` and
                      `available`
scripts/setup.sh      Creates the venv, installs dependencies, installs `tt`
```

The CLI invokes the Python engine as a subprocess and exchanges JSON on stdout.
The engine directory is discovered automatically from the project location. Set
`TT_ENGINE_DIR` and `TT_PYTHON` to override discovery when needed.

`tt-core` is deliberately UI-agnostic so future tools (TUI or web GUI) can reuse
the same conversion logic and Python backend protocol.

### Adding a converter

1. Add a module under `engine/tt_engine/converters/` with:
   - `available() -> tuple[bool, str]`
   - `convert(source: Path, target: Path) -> str` (returns the backend name)
2. Register it in `engine/tt_engine/registry.py`.
3. Add the extension/format mapping and conversion entry in
   `crates/tt-core/src/format.rs`.
4. Add `tt` tests or run an end-to-end conversion manually.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
.venv/bin/python -m compileall -q engine/tt_engine
tt doctor
```

## Limitations

- Only `.docx` is supported; legacy `.doc` files are not.
- PDF → Markdown quality depends on the PDF's internal structure. Scanned
  documents need OCR before conversion.
- Image conversions are single-frame (first frame for animated GIF/WebP/TIFF).
  JPEG/BMP cannot store transparency, so alpha is composited onto white.
- HEIC/AVIF and RAW camera formats are not supported yet.
- The installed binary discovers the engine from this project directory or
  `TT_ENGINE_DIR`. If the project is moved, re-run `scripts/setup.sh` or set the
  environment variable.

## Roadmap

- More converters and terminal tools
- A unified tool-selection launcher
- TUI and local web GUI built on `tt-core`
- Packaging and release binaries
