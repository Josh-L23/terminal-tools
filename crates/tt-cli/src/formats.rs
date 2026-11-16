use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use console::style;
use tt_core::{CONVERSIONS, Format};

pub fn execute(input: Option<PathBuf>) -> Result<()> {
    match input {
        Some(input) => {
            let source = Format::from_path(&input).with_context(|| {
                format!(
                    "unsupported input file '{}'; supported formats: {}",
                    input.display(),
                    Format::supported_names()
                )
            })?;

            println!(
                "{} can be converted to:",
                style(input.display().to_string()).cyan()
            );

            let mut found = false;
            for conversion in CONVERSIONS
                .iter()
                .filter(|conversion| conversion.from == source)
            {
                found = true;
                println!(
                    "  {} {} {}",
                    style("→").green(),
                    conversion.to.label(),
                    style(format!("({})", conversion.backend)).dim()
                );
            }

            if !found {
                bail!("no conversions available for {}", source.label());
            }
        }
        None => {
            println!("{}", style("Supported conversions:").bold());
            for conversion in CONVERSIONS {
                println!(
                    "  {:>4} → {:<4}  {}",
                    conversion.from.name(),
                    conversion.to.name(),
                    style(format!("({})", conversion.backend)).dim()
                );
            }
        }
    }

    Ok(())
}
