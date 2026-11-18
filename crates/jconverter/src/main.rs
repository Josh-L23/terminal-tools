#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod job;
mod settings;
mod support;

use anyhow::Result;
use eframe::egui;

use crate::settings::Settings;

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        None => run_gui(),
        Some("--self-test") => support::self_test(),
        Some("--version" | "-V") => {
            println!("{} {}", support::APP_NAME, support::VERSION);
            Ok(())
        }
        Some("--help" | "-h") => {
            support::print_help();
            Ok(())
        }
        Some(argument) => Err(anyhow::anyhow!(
            "unrecognized argument '{argument}'; try --help"
        )),
    }
}

fn run_gui() -> Result<()> {
    let settings = Settings::load();
    let size = settings.window_size.unwrap_or([780.0, 560.0]);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(support::APP_NAME)
            .with_app_id(support::APP_ID)
            .with_inner_size(size)
            .with_min_inner_size([560.0, 440.0])
            .with_icon(support::window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        support::APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, settings)))),
    )
    .map_err(|error| anyhow::anyhow!("could not open the JConverter window: {error}"))
}
