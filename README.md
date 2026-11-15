# Terminal Tools

Local, terminal-first utilities for everyday file work. The first tool is `tt`,
a document converter that detects the input format and lets you pick a target
format interactively.

## Supported conversions

| From             | To       | Backend                          |
| ---------------- | -------- | -------------------------------- |
| Markdown (`.md`) | PDF      | WeasyPrint                       |
| Word (`.docx`)   | PDF      | LibreOffice                      |
| Word (`.docx`)   | Markdown | Mammoth + markdownify            |
| PDF              | Markdown | PyMuPDF4LLM (`pdftotext` fallback) |

DOCX images are extracted next to the output Markdown file into
`<output-name>_media/`.

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

# Specify the target directly
tt report.md --to pdf
tt report.docx --to md
tt document.pdf --to md

# Choose an output file or an existing directory
tt report.docx --to md -o converted/
tt report.md --to pdf -o final-report.pdf

# Overwrite an existing output without prompting
tt report.docx --to md --force

# Open the result when finished
tt report.md --to pdf --open

# Inspect available targets for a file
tt formats report.docx

# Check backends and dependencies
tt doctor
```

### Local web interface

Start the monochrome local interface from the project directory:

```bash
./.venv/bin/python scripts/web_ui.py
```

Then open <http://127.0.0.1:8765>. You can drop or choose an `.md`, `.pdf`, or
`.docx` file, choose the source and destination formats, and download the
converted file. The interface uses the same conversion engine as `tt` and
keeps uploaded files in a temporary local directory while converting. Stop it
with Ctrl+C. Set `TT_UI_PORT` to use a different port, or `TT_PYTHON` to select
the Python interpreter for the engine.

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

  -t, --to <FORMAT>      Target format: md, pdf, or docx
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
- The installed binary discovers the engine from this project directory or
  `TT_ENGINE_DIR`. If the project is moved, re-run `scripts/setup.sh` or set the
  environment variable.

## Roadmap

- More converters and terminal tools
- A unified tool-selection launcher
- TUI and local web GUI built on `tt-core`
- Packaging and release binaries
