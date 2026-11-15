from __future__ import annotations

import html
from pathlib import Path

from ..errors import ConversionError

_CSS = """
@page {
  size: A4;
  margin: 20mm 18mm 22mm;
  @bottom-center {
    content: counter(page);
    color: #888;
    font-size: 9pt;
  }
}
html { font-size: 11pt; }
body {
  font-family: "DejaVu Sans", "Liberation Sans", Arial, sans-serif;
  color: #1f2328;
  line-height: 1.55;
}
h1, h2, h3, h4, h5, h6 {
  line-height: 1.25;
  margin: 1.2em 0 0.5em;
  color: #111;
}
h1 { font-size: 2em; border-bottom: 1px solid #ddd; padding-bottom: 0.25em; }
h2 { font-size: 1.5em; border-bottom: 1px solid #eee; padding-bottom: 0.2em; }
h3 { font-size: 1.25em; }
a { color: #0969da; text-decoration: none; }
p, ul, ol, table, pre, blockquote { margin: 0 0 1em; }
code {
  font-family: "DejaVu Sans Mono", "Liberation Mono", monospace;
  font-size: 0.88em;
  background: #f3f4f6;
  padding: 0.12em 0.3em;
  border-radius: 3px;
}
pre {
  background: #f6f8fa;
  border: 1px solid #e5e7eb;
  border-radius: 5px;
  padding: 0.9em 1em;
  overflow-wrap: break-word;
  white-space: pre-wrap;
}
pre code { background: none; padding: 0; font-size: 0.85em; }
blockquote {
  border-left: 4px solid #d0d7de;
  color: #57606a;
  padding: 0 1em;
}
table { border-collapse: collapse; width: 100%; font-size: 0.95em; }
th, td { border: 1px solid #d0d7de; padding: 0.45em 0.65em; text-align: left; }
th { background: #f6f8fa; }
img { max-width: 100%; height: auto; }
hr { border: none; border-top: 1px solid #d8dee4; margin: 2em 0; }
"""


def available() -> tuple[bool, str]:
    try:
        import markdown
        import weasyprint

        return True, f"Markdown {markdown.__version__}, WeasyPrint {weasyprint.__version__}"
    except Exception as exc:
        return False, f"{type(exc).__name__}: {exc}"


def convert(source: Path, target: Path) -> str:
    try:
        import markdown
        import weasyprint
    except ImportError as exc:
        raise ConversionError(
            "the Markdown → PDF backend is missing; run scripts/setup.sh"
        ) from exc

    text = source.read_text(encoding="utf-8", errors="replace")
    body = markdown.markdown(
        text,
        extensions=["extra", "toc", "sane_lists"],
        output_format="html5",
    )

    document = (
        '<!doctype html><html><head><meta charset="utf-8">'
        f"<title>{html.escape(source.stem)}</title>"
        f"<style>{_CSS}</style></head><body>{body}</body></html>"
    )

    try:
        weasyprint.HTML(
            string=document,
            base_url=str(source.parent.resolve()),
        ).write_pdf(str(target))
    except Exception as exc:
        raise ConversionError(f"failed to render the PDF: {exc}") from exc

    return "weasyprint"
