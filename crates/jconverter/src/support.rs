//! Branding strings, window icon, friendly error copy and the headless self-test.

use anyhow::{Result, bail};
use eframe::egui;
use tt_core::{ALL_FORMATS, Engine};

pub const APP_NAME: &str = "JConverter";
pub const APP_ID: &str = "com.jconverter.JConverter";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const TAGLINE: &str = "Convert documents and images locally";
pub const UNSUPPORTED: &str = "Unsupported file";

const ICON_SIZE: u32 = 256;

/// Window icon, embedded as raw RGBA so no image decoder is needed.
pub fn window_icon() -> egui::IconData {
    egui::IconData {
        rgba: include_bytes!("../assets/icon_rgba.raw").to_vec(),
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

pub fn about_lines() -> Vec<String> {
    let mut lines = vec![
        format!("{APP_NAME} {VERSION}"),
        format!("Command-line tool: tt {VERSION}"),
        format!(
            "Formats: {}",
            ALL_FORMATS
                .iter()
                .map(|format| format.name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ];

    match Engine::discover() {
        Ok(engine) => {
            lines.push(format!("Engine folder: {}", engine.engine_dir().display()));
            lines.push(format!(
                "Python: {}",
                engine
                    .python_version()
                    .unwrap_or_else(|| "not runnable".to_string())
            ));
        }
        Err(_) => lines.push("Engine: missing".to_string()),
    }

    lines
}

/// Map a raw engine error onto a headline a first-time user can act on.
pub fn friendly_error(raw: &str) -> (String, String) {
    let lower = raw.to_ascii_lowercase();

    let headline = if lower.contains("could not locate the python engine")
        || lower.contains("python engine not found")
    {
        "JConverter's conversion engine is missing. Reinstall JConverter."
    } else if lower.contains("failed to start the python engine")
        || (lower.contains("python") && lower.contains("no such file"))
        || lower.contains("could not create the python environment")
    {
        "The bundled Python runtime could not start. Reinstall JConverter."
    } else if lower.contains("libreoffice") || lower.contains("soffice") {
        "This conversion needs LibreOffice. Install it, then reopen JConverter."
    } else if lower.contains("input file not found") {
        "The selected file no longer exists. Choose another file."
    } else if lower.contains("refusing to overwrite the input file") {
        "JConverter will not overwrite the file it is converting. Choose another name."
    } else {
        "The conversion could not be completed."
    };

    (headline.to_string(), raw.trim().to_string())
}

pub fn print_help() {
    println!("{APP_NAME} {VERSION} — {TAGLINE}");
    println!();
    println!("Usage: jconverter [OPTION]");
    println!();
    println!("With no option, JConverter opens its window.");
    println!();
    println!("Options:");
    println!("  --self-test   check the engine and its backends, then exit");
    println!("  --version     print the version");
    println!("  --help        print this message");
    println!();
    println!("The tt command-line tool converts the same formats from a terminal.");
}

/// Headless check used by installers and CI.
pub fn self_test() -> Result<()> {
    println!("{APP_NAME} {VERSION} self-test");
    println!(
        "  platform: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let engine = match Engine::discover() {
        Ok(engine) => engine,
        Err(error) => {
            println!("  engine: missing");
            bail!("{error:#}");
        }
    };

    println!("  engine: {}", engine.engine_dir().display());
    let python_version = engine.python_version();
    println!(
        "  python: {} ({})",
        engine.python().display(),
        python_version.as_deref().unwrap_or("not runnable")
    );

    if python_version.is_none() {
        bail!(
            "the Python runtime could not start: {}",
            engine.python().display()
        );
    }

    let checks = engine.doctor()?;
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    for check in &checks {
        let optional = check.name.to_ascii_lowercase().contains("docx → pdf");
        if check.ok {
            println!("  ✓ {} — {}", check.name, check.detail);
        } else if optional {
            warnings.push(check.name.clone());
            println!("  ! {} — {} (optional)", check.name, check.detail);
        } else {
            failures.push(check.name.clone());
            println!("  ✗ {} — {}", check.name, check.detail);
        }
    }

    if !warnings.is_empty() {
        println!(
            "\n{} optional backend(s) unavailable: {}",
            warnings.len(),
            warnings.join(", ")
        );
        println!("Install LibreOffice to convert Word documents to PDF.");
    }

    if !failures.is_empty() {
        bail!(
            "{} backend check(s) failed: {}",
            failures.len(),
            failures.join(", ")
        );
    }

    println!("\nAll checks passed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branding_is_consistent() {
        assert_eq!(APP_NAME, "JConverter");
        assert_eq!(APP_ID, "com.jconverter.JConverter");
        assert!(!VERSION.is_empty());
        assert!(TAGLINE.contains("locally"));
    }

    #[test]
    fn missing_engine_is_explained() {
        let (headline, detail) = friendly_error(
            "could not locate the Python engine; set TT_ENGINE_DIR or run from the project",
        );
        assert!(headline.contains("engine is missing"));
        assert!(detail.starts_with("could not locate"));
    }

    #[test]
    fn missing_libreoffice_mentions_libreoffice() {
        let (headline, _) = friendly_error("LibreOffice (soffice) was not found on PATH");
        assert!(headline.contains("LibreOffice"));
    }

    #[test]
    fn unknown_errors_get_a_short_headline() {
        let (headline, detail) = friendly_error("Pillow: cannot identify image file");
        assert_eq!(headline, "The conversion could not be completed.");
        assert!(detail.contains("Pillow"));
    }
}
