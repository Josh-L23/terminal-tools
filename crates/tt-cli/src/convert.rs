use std::fs;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use indicatif::{ProgressBar, ProgressDrawTarget};
use tt_core::{Engine, Format, default_output_path};

use crate::ConvertArgs;

pub fn execute(mut arguments: ConvertArgs) -> Result<()> {
    let input = arguments.input.clone();

    if !input.is_file() {
        bail!("input file not found: {}", input.display());
    }

    let source = Format::from_path(&input).with_context(|| {
        format!(
            "unsupported input file '{}'; expected a .md, .pdf, or .docx file",
            input.display()
        )
    })?;

    let target = match arguments.to.take() {
        Some(target) => target,
        None => prompt_target(source)?,
    };

    if !source.can_convert_to(target) {
        let supported = source
            .targets()
            .iter()
            .map(|format| format.name())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "cannot convert {} to {}; supported targets: {}",
            source.label(),
            target.label(),
            supported
        );
    }

    let mut output = arguments
        .output
        .take()
        .unwrap_or_else(|| default_output_path(&input, target));

    if output.is_dir() {
        if let Some(file_name) = default_output_path(&input, target).file_name() {
            output.push(file_name);
        }
    }

    if !confirm_output(&input, &output, arguments.force)? {
        println!("Cancelled.");
        return Ok(());
    }

    let engine = Engine::discover()?;
    let spinner = new_spinner();

    let outcome = match engine.convert(&input, source, target, &output) {
        Ok(outcome) => {
            spinner.finish_and_clear();
            outcome
        }
        Err(error) => {
            spinner.finish_and_clear();
            return Err(error);
        }
    };

    println!(
        "{} {} → {} {}",
        style("✓").green().bold(),
        input.display(),
        style(outcome.output.display().to_string()).cyan(),
        style(format!(
            "({})",
            outcome.backend.as_deref().unwrap_or("done")
        ))
        .dim()
    );

    if arguments.open {
        open_path(&outcome.output)?;
    }

    Ok(())
}

fn prompt_target(source: Format) -> Result<Format> {
    let targets = source.targets();

    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        let choices = targets
            .iter()
            .map(|format| format.name())
            .collect::<Vec<_>>()
            .join(", ");
        bail!("no target format specified; pass --to <{choices}>");
    }

    let labels = targets
        .iter()
        .map(|format| format.label())
        .collect::<Vec<_>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Convert {} to", source.label()))
        .items(&labels)
        .default(0)
        .interact()?;

    Ok(targets[selection])
}

fn confirm_output(input: &Path, output: &Path, force: bool) -> Result<bool> {
    if !output.exists() {
        return Ok(true);
    }

    if let (Ok(input), Ok(output)) = (fs::canonicalize(input), fs::canonicalize(output))
        && input == output
    {
        bail!("refusing to overwrite the input file");
    }

    if force {
        return Ok(true);
    }

    if !io::stdin().is_terminal() {
        bail!(
            "output already exists: {}; pass --force to overwrite it",
            output.display()
        );
    }

    Confirm::new()
        .with_prompt(format!("{} already exists. Overwrite?", output.display()))
        .default(false)
        .interact()
        .map_err(Into::into)
}

fn new_spinner() -> ProgressBar {
    if !io::stderr().is_terminal() {
        return ProgressBar::hidden();
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_draw_target(ProgressDrawTarget::stderr());
    spinner.set_message("Converting…");
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

fn open_path(path: &Path) -> Result<()> {
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
        eprintln!("note: opening files is not supported on this platform");
        return Ok(());
    }

    command
        .spawn()
        .with_context(|| format!("failed to open {}", path.display()))?;
    Ok(())
}
