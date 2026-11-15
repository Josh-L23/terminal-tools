#!/usr/bin/env python3
"""Serve the local Terminal Tools web interface and its conversion endpoint."""

from __future__ import annotations

from email import policy
from email.parser import BytesParser
import json
import os
import re
import subprocess
import sys
import tempfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse

ROOT = Path(__file__).resolve().parents[1]
WEB = ROOT / "web"
ENGINE = ROOT / "engine"
FORMATS = {"md": ".md", "pdf": ".pdf", "docx": ".docx"}
CONVERSIONS = {"md": {"pdf"}, "pdf": {"md"}, "docx": {"pdf", "md"}}
MAX_UPLOAD = 50 * 1024 * 1024


def python_executable() -> str:
    configured = os.environ.get("TT_PYTHON")
    if configured:
        return configured
    venv_python = ROOT / ".venv" / "bin" / "python"
    return str(venv_python if venv_python.is_file() else Path(sys.executable))


class Handler(BaseHTTPRequestHandler):
    server_version = "TerminalToolsUI/1.0"

    def do_GET(self) -> None:
        path = unquote(urlparse(self.path).path)
        if path == "/":
            target = WEB / "index.html"
        elif path in {"/app.js", "/style.css"}:
            target = WEB / path.lstrip("/")
        elif path.startswith("/images/"):
            target = ROOT / path.lstrip("/")
        else:
            self.send_error(404)
            return

        try:
            resolved = target.resolve(strict=True)
            if not resolved.is_relative_to(ROOT) or not resolved.is_file():
                raise FileNotFoundError
            payload = resolved.read_bytes()
        except (OSError, ValueError):
            self.send_error(404)
            return

        content_type = {
            ".html": "text/html; charset=utf-8",
            ".js": "text/javascript; charset=utf-8",
            ".css": "text/css; charset=utf-8",
            ".png": "image/png",
        }.get(resolved.suffix, "application/octet-stream")
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        self.wfile.write(payload)

    def do_POST(self) -> None:
        if urlparse(self.path).path != "/api/convert":
            self._json_error(404, "Unknown endpoint.")
            return

        try:
            result, filename = self._convert_upload()
        except ValueError as error:
            self._json_error(400, str(error))
            return
        except RuntimeError as error:
            self._json_error(422, str(error))
            return

        self.send_response(200)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(result)))
        self.send_header("Content-Disposition", f'attachment; filename="{filename}"')
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        self.wfile.write(result)

    def _convert_upload(self) -> tuple[bytes, str]:
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError as error:
            raise ValueError("Invalid upload size.") from error
        if length <= 0 or length > MAX_UPLOAD:
            raise ValueError("Choose a file smaller than 50 MB.")
        content_type = self.headers.get("Content-Type", "")
        if not content_type.startswith("multipart/form-data"):
            raise ValueError("Upload a file using the file picker.")

        body = self.rfile.read(length)
        if len(body) != length:
            raise ValueError("The upload was incomplete. Please try again.")
        message = BytesParser(policy=policy.default).parsebytes(
            f"Content-Type: {content_type}\r\nMIME-Version: 1.0\r\n\r\n".encode("ascii") + body
        )
        fields: dict[str, str] = {}
        upload_name = ""
        upload_data = b""
        for part in message.iter_parts():
            field_name = part.get_param("name", header="content-disposition")
            if field_name == "file":
                upload_name = part.get_filename() or ""
                upload_data = part.get_payload(decode=True) or b""
            elif field_name:
                fields[field_name] = part.get_content()

        source_format = fields.get("from", "").lower()
        target_format = fields.get("to", "").lower()
        if source_format not in FORMATS or target_format not in CONVERSIONS.get(source_format, set()):
            raise ValueError("That conversion isn’t available.")
        if not upload_name:
            raise ValueError("Choose a file to convert.")

        original_name = Path(upload_name.replace("\\", "/")).name
        if Path(original_name).suffix.lower() != FORMATS[source_format]:
            raise ValueError(f"Choose a {FORMATS[source_format]} file to match the ‘from’ format.")
        stem = Path(original_name).stem or "converted-file"
        safe_stem = re.sub(r"[^A-Za-z0-9._ -]", "_", stem).strip(" .") or "converted-file"
        output_name = f"{safe_stem}.{target_format}"

        with tempfile.TemporaryDirectory(prefix="tt-web-") as temp_name:
            temp = Path(temp_name)
            input_path = temp / f"input{FORMATS[source_format]}"
            output_path = temp / output_name
            input_path.write_bytes(upload_data)
            environment = os.environ.copy()
            environment["PYTHONPATH"] = os.pathsep.join(
                [str(ENGINE), environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            environment["PYTHONIOENCODING"] = "utf-8"
            process = subprocess.run(
                [
                    python_executable(), "-m", "tt_engine", "convert",
                    "--input", str(input_path), "--from", source_format,
                    "--to", target_format, "--output", str(output_path),
                ],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=300,
                check=False,
            )
            response = next(
                (line for line in reversed(process.stdout.splitlines()) if line.startswith("{")),
                "{}",
            )
            try:
                result = json.loads(response)
            except json.JSONDecodeError:
                result = {}
            if process.returncode or not result.get("ok") or not output_path.is_file():
                detail = result.get("error") or process.stderr.strip() or "Conversion failed."
                raise RuntimeError(detail)
            return output_path.read_bytes(), output_name

    def _json_error(self, status: int, message: str) -> None:
        payload = json.dumps({"error": message}, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, format: str, *args: object) -> None:
        print(f"[tt-ui] {self.address_string()} {format % args}")


def main() -> None:
    host = os.environ.get("TT_UI_HOST", "127.0.0.1")
    port = int(os.environ.get("TT_UI_PORT", "8765"))
    server = ThreadingHTTPServer((host, port), Handler)
    print(f"Terminal Tools is ready at http://{host}:{port}")
    print("Press Ctrl+C to stop.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down Terminal Tools.")
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
