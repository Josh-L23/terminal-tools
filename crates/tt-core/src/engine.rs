use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::Format;

#[derive(Debug, Clone)]
pub struct Engine {
    python: PathBuf,
    engine_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ConversionOutcome {
    pub output: PathBuf,
    pub backend: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Response {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    output: Option<PathBuf>,
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    checks: Vec<Check>,
}

impl Engine {
    pub fn discover() -> Result<Self> {
        let project_root = find_project_root();

        let engine_dir = env::var_os("TT_ENGINE_DIR")
            .map(PathBuf::from)
            .or_else(|| project_root.as_ref().map(|root| root.join("engine")))
            .context(
                "could not locate the Python engine; set TT_ENGINE_DIR or run from the project",
            )?;

        if !engine_dir.join("tt_engine").join("__main__.py").is_file() {
            bail!(
                "Python engine not found at {}; set TT_ENGINE_DIR to the engine directory",
                engine_dir.display()
            );
        }

        let python = env::var_os("TT_PYTHON")
            .map(PathBuf::from)
            .or_else(|| {
                let candidate = project_root
                    .as_ref()?
                    .join(".venv")
                    .join("bin")
                    .join("python");
                candidate.is_file().then_some(candidate)
            })
            .unwrap_or_else(|| PathBuf::from("python3"));

        Ok(Self { python, engine_dir })
    }

    pub fn python(&self) -> &Path {
        &self.python
    }

    pub fn engine_dir(&self) -> &Path {
        &self.engine_dir
    }

    pub fn python_version(&self) -> Option<String> {
        let output = Command::new(&self.python).arg("--version").output().ok()?;
        let text = if output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr)
        } else {
            String::from_utf8_lossy(&output.stdout)
        };
        let version = text.trim();
        (!version.is_empty()).then(|| version.to_string())
    }

    pub fn convert(
        &self,
        input: &Path,
        from: Format,
        to: Format,
        output: &Path,
    ) -> Result<ConversionOutcome> {
        let args = vec![
            OsString::from("convert"),
            OsString::from("--input"),
            input.as_os_str().to_owned(),
            OsString::from("--from"),
            OsString::from(from.name()),
            OsString::from("--to"),
            OsString::from(to.name()),
            OsString::from("--output"),
            output.as_os_str().to_owned(),
        ];

        let response = self.run(args)?;
        if !response.ok {
            bail!(
                "{}",
                response
                    .error
                    .unwrap_or_else(|| "conversion failed without an error message".into())
            );
        }

        Ok(ConversionOutcome {
            output: response.output.unwrap_or_else(|| output.to_path_buf()),
            backend: response.backend,
        })
    }

    pub fn doctor(&self) -> Result<Vec<Check>> {
        let response = self.run(vec![OsString::from("doctor")])?;
        Ok(response.checks)
    }

    fn run(&self, args: Vec<OsString>) -> Result<Response> {
        let mut command = Command::new(&self.python);
        command
            .arg("-m")
            .arg("tt_engine")
            .args(&args)
            .env("PYTHONPATH", python_path(&self.engine_dir))
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONUNBUFFERED", "1");

        let output = command.output().with_context(|| {
            format!(
                "failed to start the Python engine with {}; is Python installed?",
                self.python.display()
            )
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let response_text = stdout
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| line.starts_with('{'))
            .unwrap_or("");

        if response_text.is_empty() {
            let detail = stderr.trim();
            if detail.is_empty() {
                bail!(
                    "the Python engine produced no output (exit status {})",
                    output.status
                );
            }
            bail!("the Python engine failed: {detail}");
        }

        let response: Response = serde_json::from_str(response_text).with_context(|| {
            format!(
                "could not parse the Python engine response: {}",
                response_text
            )
        })?;

        if !output.status.success() && response.error.is_none() {
            bail!(
                "the Python engine exited with {}: {}",
                output.status,
                stderr.trim()
            );
        }

        Ok(response)
    }
}

fn python_path(engine_dir: &Path) -> OsString {
    let mut paths = vec![engine_dir.to_path_buf()];
    if let Some(existing) = env::var_os("PYTHONPATH") {
        paths.extend(env::split_paths(&existing));
    }
    env::join_paths(paths).unwrap_or_else(|_| engine_dir.as_os_str().to_owned())
}

fn find_project_root() -> Option<PathBuf> {
    let mut starts = Vec::new();
    if let Ok(current_dir) = env::current_dir() {
        starts.push(current_dir);
    }
    if let Some(executable) = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
    {
        starts.push(executable);
    }
    starts.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));

    for start in starts {
        for directory in start.ancestors() {
            if directory
                .join("engine")
                .join("tt_engine")
                .join("__main__.py")
                .is_file()
            {
                return Some(
                    directory
                        .canonicalize()
                        .unwrap_or_else(|_| directory.to_path_buf()),
                );
            }
        }
    }
    None
}
