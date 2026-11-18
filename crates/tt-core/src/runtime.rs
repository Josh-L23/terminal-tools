//! Locating the bundle layout and launching the desktop helpers.
//!
//! A JConverter/tarball install and a project checkout share one layout: a root
//! that holds `engine/`, `web/`, `scripts/` and a `.venv`. Discovery resolves
//! that root from `TT_ROOT`, then from the current directory and the executable
//! location, so the app works both inside the repo and from an install prefix.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// `engine/tt_engine/__main__.py` marks a directory as the Python engine.
pub fn is_engine_dir(directory: &Path) -> bool {
    directory.join("tt_engine").join("__main__.py").is_file()
}

/// Candidate roots to probe, most explicit first.
pub fn search_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Some(root) = env::var_os("TT_ROOT")
        .map(PathBuf::from)
        .filter(|root| !root.as_os_str().is_empty())
    {
        roots.push(root);
    }

    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(current_dir) = env::current_dir() {
        starts.push(current_dir);
    }
    if let Some(executable) = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
    {
        starts.push(executable);
    }

    for start in starts {
        for ancestor in start.ancestors() {
            roots.push(ancestor.to_path_buf());
        }
    }

    let mut unique: Vec<PathBuf> = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

/// Absolute path of the Python engine directory, if one can be found.
pub fn find_engine_dir() -> Option<PathBuf> {
    for root in search_roots() {
        let engine = root.join("engine");
        if is_engine_dir(&engine) {
            return Some(engine.canonicalize().unwrap_or(engine));
        }
    }
    None
}

/// Interpreter inside a bundle/project venv. Venvs are not relocatable, so the
/// path is always derived from the root that is in use right now.
pub fn venv_python(root: &Path) -> PathBuf {
    let venv = root.join(".venv");
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Python for the discovered engine, preferring the venv that sits beside it.
pub fn find_venv_python(engine_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(engine_dir) = engine_dir {
        if let Some(root) = engine_dir.parent() {
            let candidate = venv_python(root);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    search_roots()
        .into_iter()
        .map(|root| venv_python(&root))
        .find(|candidate| candidate.is_file())
}

/// Last resort when no venv exists: whatever Python is on `PATH`.
pub fn default_python() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("python")
    } else {
        PathBuf::from("python3")
    }
}

/// Open a file with the desktop's default application.
pub fn open_path(path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", ""]).arg(path);
        command
    };

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("opening files is not supported on this platform");
    }

    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", target_os = "windows")),
        allow(unreachable_code)
    )]
    command
        .spawn()
        .with_context(|| format!("could not open {}", path.display()))?;
    Ok(())
}

/// Reveal a file in the desktop's file manager.
pub fn show_in_folder(path: &Path) -> Result<()> {
    let folder = path.parent().unwrap_or(path);

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(folder);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg("-R").arg(path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(format!("/select,{}", path.display()));
        command
    };

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("showing files in a folder is not supported on this platform");
    }

    #[cfg_attr(
        not(any(target_os = "linux", target_os = "macos", target_os = "windows")),
        allow(unreachable_code)
    )]
    command
        .spawn()
        .with_context(|| format!("could not open {}", folder.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venv_python_is_platform_aware() {
        let python = venv_python(Path::new("/opt/jconverter"));
        if cfg!(windows) {
            assert!(python.ends_with("Scripts/python.exe"));
        } else {
            assert!(python.ends_with(".venv/bin/python"));
        }
    }

    #[test]
    fn engine_marker_detection() {
        let root = env::temp_dir().join(format!("tt-runtime-test-{}", std::process::id()));
        let engine = root.join("engine").join("tt_engine");
        std::fs::create_dir_all(&engine).unwrap();
        assert!(!is_engine_dir(&root.join("engine")));

        std::fs::write(engine.join("__main__.py"), "").unwrap();
        assert!(is_engine_dir(&root.join("engine")));

        std::fs::remove_dir_all(&root).unwrap();
    }
}
