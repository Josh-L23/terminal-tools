mod convert;
mod doctor;
mod formats;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use console::style;
use tt_core::{Format, parse_format};

#[derive(Debug, Parser)]
#[command(
    name = "tt",
    version,
    about = "Terminal Tools — local file converters",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Convert a file; the source format is detected from its extension
    #[command(visible_alias = "c")]
    Convert(ConvertArgs),
    /// List supported conversions, optionally for one file
    Formats { input: Option<PathBuf> },
    /// Check that every conversion backend is installed
    Doctor,
}

#[derive(Debug, Args)]
pub struct ConvertArgs {
    /// File to convert
    pub input: PathBuf,

    /// Target format (e.g. pdf, md, png, jpg, webp)
    #[arg(short, long, value_parser = parse_format)]
    pub to: Option<Format>,

    /// Output file or directory (defaults to the input name with a new extension)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite an existing output file without asking
    #[arg(short, long)]
    pub force: bool,

    /// Open the converted file when finished
    #[arg(long)]
    pub open: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{} {error:#}", style("error:").red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let arguments = normalize_args(env::args_os().collect());
    let cli = Cli::parse_from(arguments);

    match cli.command {
        Command::Convert(arguments) => convert::execute(arguments),
        Command::Formats { input } => formats::execute(input),
        Command::Doctor => doctor::execute(),
    }
}

fn normalize_args(mut arguments: Vec<OsString>) -> Vec<OsString> {
    if let Some(candidate) = arguments.get(1) {
        let text = candidate.to_string_lossy();
        let is_subcommand = matches!(
            text.as_ref(),
            "convert" | "c" | "formats" | "doctor" | "help"
        );

        if !is_subcommand && !text.starts_with('-') && Path::new(candidate).is_file() {
            arguments.insert(1, OsString::from("convert"));
        }
    }

    arguments
}
