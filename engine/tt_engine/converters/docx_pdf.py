from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from ..errors import ConversionError


def _soffice() -> str | None:
    return shutil.which("soffice") or shutil.which("libreoffice")


def available() -> tuple[bool, str]:
    executable = _soffice()
    if executable:
        return True, f"LibreOffice at {executable}"
    return False, "LibreOffice (soffice) was not found on PATH"


def convert(source: Path, target: Path) -> str:
    executable = _soffice()
    if not executable:
        raise ConversionError("LibreOffice (soffice) was not found on PATH")

    try:
        with tempfile.TemporaryDirectory(prefix="tt-docx-") as workspace:
            workspace_path = Path(workspace)
            profile = (workspace_path / "profile").resolve()
            command = [
                executable,
                "--headless",
                "--norestore",
                "--nolockcheck",
                "--nodefault",
                f"-env:UserInstallation={profile.as_uri()}",
                "--convert-to",
                "pdf",
                "--outdir",
                str(workspace_path),
                str(source.resolve()),
            ]

            environment = os.environ.copy()
            environment["SAL_USE_VCLPLUGIN"] = "svp"
            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                timeout=180,
                env=environment,
                check=False,
            )

            pdfs = [path for path in workspace_path.glob("*.pdf") if path.is_file()]
            if result.returncode != 0 or not pdfs:
                detail = (result.stderr or result.stdout or "no output").strip()
                raise ConversionError(
                    f"LibreOffice failed to convert the document: {detail}"
                )

            produced = next(
                (path for path in pdfs if path.name == f"{source.stem}.pdf"),
                pdfs[0],
            )
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(produced), str(target))
    except subprocess.TimeoutExpired as exc:
        raise ConversionError("LibreOffice timed out after 180 seconds") from exc

    return "libreoffice"
