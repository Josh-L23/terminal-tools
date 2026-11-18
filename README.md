# Terminal Tools

Local utilities for everyday file work. Both tools share one conversion engine:

- `tt`, a terminal-first file converter that detects the input format and lets
  you pick a target format interactively.
- **JConverter**, the same conversions in a native desktop window.

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

JConverter additionally needs a graphical session — Wayland or X11. Its file
dialogs go through the XDG desktop portal, so a portal implementation must be
installed (the usual case on GNOME, KDE and most desktop distributions). The GUI
does not need GTK development headers or any WebView.

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

### Desktop app

JConverter is built from the same workspace and uses the same Python engine, so
run `./scripts/setup.sh` first. Then:

```bash
cargo build --release -p jconverter
./target/release/jconverter
```

To put it on your `PATH`:

```bash
cargo install --path crates/jconverter
jconverter
```

Check the install without opening a window:

```bash
jconverter --self-test
```

`--self-test` discovers the engine, prints the Python version it will use, and
runs every backend check. It exits non-zero if a required backend is missing;
DOCX → PDF only warns, since LibreOffice is optional.

`jconverter --version` prints the version and `jconverter --help` lists the
options. With no argument at all, JConverter opens its window.

To create only the Python environment (without installing `tt`), use the bundle
bootstrap script:

```bash
./scripts/bootstrap_env.sh
```

It creates `.venv` from `wheelhouse/` when that directory holds wheels
(fully offline), and otherwise from `engine/requirements.lock.txt`. Pass
`--root DIR` to set up an install directory other than this checkout.

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

### Options

```text
tt convert <INPUT> [OPTIONS]

  -t, --to <FORMAT>      Target format (e.g. pdf, md, png, jpg, webp, tiff)
  -o, --output <PATH>    Output file or directory
  -f, --force            Overwrite an existing output
      --open             Open the result after conversion
  -h, --help             Show help
```

By default, output is written next to the input file with the new extension.
Existing files are never overwritten silently: `tt` asks first, or fails when
running without a terminal unless `--force` is passed.

The explicit form works too:

```bash
tt convert report.md --to pdf
```

## JConverter desktop app

A native window (Rust + `eframe`/`egui`) over `tt-core` — no Electron, no Tauri,
no WebView, and no `tt` subprocess: the GUI links `tt-core` and talks to the
Python engine directly.

```bash
jconverter
```

What the window does:

- Choose a source file with **Browse…**, type a path, or drag a file onto the
  window. The format is detected from the extension and shown under **Detected**;
  unsupported files are labelled as such.
- **Convert to** lists only the targets that this source format can actually
  reach, in registry order.
- **Output** is pre-filled with the default name next to the input and follows
  the target format until you edit it yourself. **Browse…** opens a save dialog.
- **Convert** runs on a background thread, so the window stays responsive; the
  status area shows the backend in use, then the saved path with **Show in
  folder** and **Open result** buttons.
- Incorrect choices are reported before any work starts (missing file,
  unsupported format, impossible conversion, empty output). A file that already
  exists is only replaced when **Overwrite existing output** is ticked, and the
  input file itself is never overwritten.
- Failures show a short, actionable headline with the raw engine message under
  **Technical details**.
- **Doctor…** runs the backend checks in a window and can be re-run. **Formats…**
  lists every format and its targets. **About** shows the version, the formats
  in this build, the engine folder and the Python it uses.
- **Dark theme**, **Open result when finished** and window size are remembered.

Settings live in `${XDG_CONFIG_HOME:-~/.config}/jconverter/settings.json` on
Linux (`~/Library/Application Support/jconverter/settings.json` on macOS), and
are written when they change. A missing or corrupt file falls back to defaults.

## Local web interface

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

## Architecture

```text
crates/tt-core/       Format registry, conversion matrix, engine client,
                      bundle discovery and desktop helpers (shared by both tools)
crates/tt-cli/        The `tt` command-line interface
crates/jconverter/    The JConverter desktop app (eframe/egui + rfd dialogs)
engine/tt_engine/     Python conversion backends and JSON protocol
engine/requirements.lock.txt
                      Pinned dependencies for the bundled Python 3.12 runtime
scripts/setup.sh      Creates the venv, installs dependencies, installs `tt`
scripts/bootstrap_env.sh
                      Creates `.venv` from a wheelhouse or the lock file
packaging/linux/      Desktop entry and hicolor icons for the GUI
```

The CLI and the GUI both invoke the Python engine as a subprocess and exchange
JSON on stdout. `tt-core` is UI-agnostic: format detection, the conversion
matrix, default output paths, engine discovery and the "open/reveal in folder"
helpers all live there, so the terminal tool, the GUI and any future front end
share one implementation.

### Engine discovery

Both `tt` and `jconverter` look for the engine in this order, so a checkout and
an installed bundle behave the same:

1. `TT_ENGINE_DIR`, the engine directory itself
2. `TT_ROOT`, the directory containing `engine/`
3. Each parent of the current working directory
4. Each parent of the directory holding the running executable

A directory counts as the engine when it contains `tt_engine/__main__.py`. The
interpreter comes from `TT_PYTHON`, else `.venv/bin/python` beside the engine
(or the nearest `.venv` in the same search order), else `python3` on `PATH`.

Because the project is found by walking up from the working directory and the
executable, a binary installed with `cargo install` works while you are inside
the checkout. If you move the project, re-run `scripts/setup.sh`, or set
`TT_ENGINE_DIR`/`TT_ROOT`.

### Adding a converter

1. Add a module under `engine/tt_engine/converters/` with:
   - `available() -> tuple[bool, str]`
   - `convert(source: Path, target: Path) -> str` (returns the backend name)
2. Register it in `engine/tt_engine/registry.py`.
3. Add the extension/format mapping and conversion entry in
   `crates/tt-core/src/format.rs`.

Both front ends pick the new conversion up from the registry: `tt` offers it in
its target list and JConverter lists it under **Convert to** and **Formats…**.

## Packaging assets

- `packaging/linux/jconverter.desktop` — desktop entry (`Exec=jconverter`,
  `Icon=jconverter`, categories `Utility;Office;`).
- `packaging/linux/icons/hicolor/{16x16,32x32,48x48,64x64,128x128,256x256}/apps/jconverter.png`
  — icon theme sizes. Install them under `share/icons/hicolor/` and the desktop
  file under `share/applications/`.
- `scripts/generate_icons.py` — regenerates those PNGs and the window icon in
  `crates/jconverter/assets/` (`icon_rgba.raw` is embedded in the binary so no
  image decoder is needed at runtime).

There is no `.deb`, tarball or installer recipe yet; the desktop entry and icons
are staged for one. `engine/requirements.lock.txt` is pinned for the CPython
3.12 runtime that a bundle ships.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
.venv/bin/python -m compileall -q engine/tt_engine
tt doctor
jconverter --self-test
```

`cargo test` covers `tt-core` and the GUI logic (target lists, backend names,
overwrite rules, settings, error copy). The GUI can be smoke-tested without a
monitor:

```bash
env -u WAYLAND_DISPLAY xvfb-run -a -s "-screen 0 1280x800x24" \
  ./target/debug/jconverter
```

## Limitations

- Only `.docx` is supported; legacy `.doc` files are not.
- PDF → Markdown quality depends on the PDF's internal structure. Scanned
  documents need OCR before conversion.
- Image conversions are single-frame (first frame for animated GIF/WebP/TIFF).
  JPEG/BMP cannot store transparency, so alpha is composited onto white.
- HEIC/AVIF and RAW camera formats are not supported yet.
- JConverter needs a running graphical session and a desktop portal for its file
  dialogs. Without one, drag-and-drop and typing a path still work, but
  **Browse…** cannot open a dialog.
- The GUI and `tt` share the engine and backends, so the limitations above apply
  to both.
- No packaged releases yet: JConverter is built from source, and the installed
  binary discovers the engine from this project directory, `TT_ROOT` or
  `TT_ENGINE_DIR`.

## Roadmap

- More converters and terminal tools
- A unified tool-selection launcher
- A TUI built on `tt-core`
- Debian package and tarball that ship JConverter with a bundled Python runtime
