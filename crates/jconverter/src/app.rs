//! The JConverter window.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;

use eframe::egui;
use tt_core::{CONVERSIONS, Check, Format, default_output_path, open_path, show_in_folder};

use crate::job::{self, Job, WorkerMessage};
use crate::settings::Settings;
use crate::support;

/// What the status area is showing.
pub enum Status {
    Idle,
    Working {
        backend: Option<String>,
    },
    Done {
        output: PathBuf,
        backend: Option<String>,
    },
    Failed {
        headline: String,
        detail: String,
    },
}

enum DoctorState {
    Loading,
    Ready(Vec<Check>),
    Failed(String),
}

/// A conversion that passed validation and is ready to hand to the worker.
struct Plan {
    input: PathBuf,
    from: Format,
    to: Format,
    output: PathBuf,
}

pub struct App {
    settings: Settings,
    settings_dirty: bool,
    source_text: String,
    target: Option<Format>,
    output_text: String,
    output_custom: bool,
    status: Status,
    busy: bool,
    jobs: Sender<Job>,
    results: Receiver<WorkerMessage>,
    doctor: Option<DoctorState>,
    show_doctor: bool,
    show_about: bool,
    show_formats: bool,
    engine_summary: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        cc.egui_ctx.set_visuals(visuals(settings.dark_theme));

        let engine_summary = match tt_core::Engine::discover() {
            Ok(engine) => engine.engine_dir().display().to_string(),
            Err(_) => "missing".to_string(),
        };
        let (jobs, results) = job::start();

        Self {
            settings,
            settings_dirty: false,
            source_text: String::new(),
            target: None,
            output_text: String::new(),
            output_custom: false,
            status: Status::Idle,
            busy: false,
            jobs,
            results,
            doctor: None,
            show_doctor: false,
            show_about: false,
            show_formats: false,
            engine_summary,
        }
    }

    /// Validate the current form and hand the work to the worker thread.
    fn start(&mut self) {
        let plan = match self.plan() {
            Ok(plan) => plan,
            Err(headline) => {
                self.status = Status::Failed {
                    headline,
                    detail: String::new(),
                };
                return;
            }
        };

        let backend = backend_for(plan.from, plan.to);
        if let Some(parent) = plan.output.parent() {
            if !parent.as_os_str().is_empty() {
                self.settings.last_output_dir = Some(parent.to_path_buf());
                self.settings_dirty = true;
            }
        }

        self.status = Status::Working {
            backend: backend.clone(),
        };
        self.busy = true;

        let job = Job::Convert {
            input: plan.input,
            from: plan.from,
            to: plan.to,
            output: plan.output,
            open_after: self.settings.open_after,
        };

        if self.jobs.send(job).is_err() {
            self.busy = false;
            self.status = Status::Failed {
                headline: "JConverter's background worker stopped. Restart JConverter.".to_string(),
                detail: String::new(),
            };
        }
    }

    fn start_doctor(&mut self) {
        self.doctor = Some(DoctorState::Loading);
        self.show_doctor = true;
        let _ = self.jobs.send(Job::Doctor);
    }

    fn plan(&self) -> Result<Plan, String> {
        let source_text = self.source_text.trim();
        if source_text.is_empty() {
            return Err("Choose a file to convert.".to_string());
        }

        let input = PathBuf::from(source_text);
        if !input.is_file() {
            return Err("The selected file no longer exists. Choose another file.".to_string());
        }

        let from = match Format::from_path(&input) {
            Some(from) => from,
            None => {
                return Err(format!(
                    "JConverter does not support that kind of file yet. Supported: {}.",
                    Format::supported_extension_list()
                ));
            }
        };

        let to = self
            .target
            .ok_or_else(|| "Choose an output format.".to_string())?;

        if !from.can_convert_to(to) {
            return Err(format!(
                "{} cannot be converted to {}.",
                from.label(),
                to.label()
            ));
        }

        let output_text = self.output_text.trim();
        if output_text.is_empty() {
            return Err("Choose where to save the result.".to_string());
        }
        let output = PathBuf::from(output_text);
        check_overwrite(&input, &output, self.settings.overwrite)?;

        Ok(Plan {
            input,
            from,
            to,
            output,
        })
    }

    fn set_source(&mut self, path: &Path) {
        self.source_text = path.display().to_string();
        self.output_custom = false;
        if let Some(parent) = path.parent() {
            self.settings.last_source_dir = Some(parent.to_path_buf());
            self.settings_dirty = true;
        }
        self.refresh_source();
    }

    /// Re-derive the target list and default output from the source field.
    fn refresh_source(&mut self) {
        self.status = Status::Idle;

        if self.source_text.trim().is_empty() {
            self.target = None;
            self.output_text.clear();
            self.output_custom = false;
            return;
        }

        let path = PathBuf::from(self.source_text.trim());
        let Some(from) = Format::from_path(&path) else {
            self.target = None;
            return;
        };

        let targets = targets_for(from);
        if !self
            .target
            .is_some_and(|current| targets.contains(&current))
        {
            self.target = targets.first().copied();
        }

        self.refresh_output();
    }

    /// Keep the output path in step with the target unless the user chose it.
    fn refresh_output(&mut self) {
        if self.output_custom {
            return;
        }
        let Some(target) = self.target else {
            return;
        };
        let path = PathBuf::from(self.source_text.trim());
        if path.as_os_str().is_empty() {
            return;
        }
        self.output_text = default_output_path(&path, target).display().to_string();
    }

    fn poll(&mut self, ctx: &egui::Context) {
        loop {
            match self.results.try_recv() {
                Ok(WorkerMessage::Converted {
                    output,
                    backend,
                    error,
                }) => {
                    self.busy = false;
                    match error {
                        Some(raw) => {
                            let (headline, detail) = support::friendly_error(&raw);
                            self.status = Status::Failed { headline, detail };
                        }
                        None => {
                            self.status = Status::Done { output, backend };
                        }
                    }
                }
                Ok(WorkerMessage::Doctor(result)) => {
                    self.doctor = Some(match result {
                        Ok(checks) => DoctorState::Ready(checks),
                        Err(error) => DoctorState::Failed(error),
                    });
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.busy = false;
                    break;
                }
            }
        }

        if self.busy || matches!(self.doctor, Some(DoctorState::Loading)) {
            ctx.request_repaint_after(Duration::from_millis(120));
        }
    }

    fn handle_drop(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .find_map(|file| file.path.clone())
        });
        if let Some(path) = dropped {
            self.set_source(&path);
        }
    }

    fn save_settings(&mut self) {
        let _ = self.settings.save();
        self.settings_dirty = false;
    }

    fn remember_window(&mut self, ctx: &egui::Context) {
        let size = ctx.screen_rect().size();
        if size.x > 400.0 && size.y > 300.0 {
            self.settings.window_size = Some([size.x, size.y]);
        }
        self.save_settings();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        self.handle_drop(ctx);

        let detected = Format::from_path(Path::new(self.source_text.trim()));
        let available_targets = detected.map(targets_for).unwrap_or_default();
        let ready = !self.busy
            && detected.is_some()
            && self.target.is_some()
            && !self.output_text.trim().is_empty();

        let mut source_edited = false;
        let mut output_edited = false;
        let mut target_changed = false;
        let mut browse_source = false;
        let mut browse_output = false;
        let mut convert_clicked = false;
        let mut doctor_clicked = false;
        let mut show_folder: Option<PathBuf> = None;
        let mut open_result: Option<PathBuf> = None;

        let open_before = self.settings.open_after;
        let overwrite_before = self.settings.overwrite;
        let theme_before = self.settings.dark_theme;

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(support::APP_NAME);
                ui.add_space(8.0);
                ui.label(egui::RichText::new(support::TAGLINE).weak());
            });
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Formats…").clicked() {
                    self.show_formats = true;
                }
                if ui.button("Doctor…").clicked() {
                    doctor_clicked = true;
                }
                if ui.button("About").clicked() {
                    self.show_about = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.settings.dark_theme, "Dark theme");
                    ui.label(
                        egui::RichText::new(format!("Engine: {}", self.engine_summary))
                            .weak()
                            .small(),
                    );
                });
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("form")
                .num_columns(3)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Source file");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.source_text)
                            .desired_width(f32::INFINITY)
                            .hint_text("Choose a file, or drop one here"),
                    );
                    source_edited = response.changed();
                    if ui.button("Browse…").clicked() {
                        browse_source = true;
                    }
                    ui.end_row();

                    ui.label("Detected");
                    let detected_text = match detected {
                        Some(format) => format.label().to_string(),
                        None if self.source_text.trim().is_empty() => "—".to_string(),
                        None => support::UNSUPPORTED.to_string(),
                    };
                    ui.label(detected_text);
                    ui.label("");
                    ui.end_row();

                    ui.label("Convert to");
                    let selected = match self.target {
                        Some(format) if available_targets.contains(&format) => format.label(),
                        _ => "—",
                    };
                    egui::ComboBox::from_id_salt("target-format")
                        .selected_text(selected)
                        .width(280.0)
                        .show_ui(ui, |ui| {
                            if available_targets.is_empty() {
                                ui.label("Choose a supported file first");
                            }
                            for format in &available_targets {
                                if ui
                                    .selectable_label(self.target == Some(*format), format.label())
                                    .clicked()
                                {
                                    self.target = Some(*format);
                                    target_changed = true;
                                }
                            }
                        });
                    ui.label("");
                    ui.end_row();

                    ui.label("Output");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.output_text)
                            .desired_width(f32::INFINITY)
                            .hint_text("Where the converted file is saved"),
                    );
                    output_edited = response.changed();
                    if ui.button("Browse…").clicked() {
                        browse_output = true;
                    }
                    ui.end_row();
                });

            ui.add_space(8.0);
            ui.checkbox(&mut self.settings.open_after, "Open result when finished");
            ui.checkbox(&mut self.settings.overwrite, "Overwrite existing output");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(ready, egui::Button::new("Convert"))
                    .clicked()
                {
                    convert_clicked = true;
                }
                if self.busy {
                    ui.spinner();
                }
            });

            ui.add_space(6.0);
            ui.separator();

            match &self.status {
                Status::Idle => {
                    ui.label("Status: Ready");
                }
                Status::Working { backend } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(match backend {
                            Some(backend) => format!("Converting with {backend}…"),
                            None => "Working…".to_string(),
                        });
                    });
                }
                Status::Done { output, backend } => {
                    ui.label(egui::RichText::new(format!("Saved {}", output.display())).strong());
                    ui.label(
                        egui::RichText::new(
                            backend
                                .clone()
                                .unwrap_or_else(|| "conversion finished".to_string()),
                        )
                        .weak(),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Show in folder").clicked() {
                            show_folder = Some(output.clone());
                        }
                        if ui.button("Open result").clicked() {
                            open_result = Some(output.clone());
                        }
                    });
                }
                Status::Failed { headline, detail } => {
                    ui.label(
                        egui::RichText::new(headline).color(egui::Color32::from_rgb(200, 70, 70)),
                    );
                    if !detail.is_empty() {
                        egui::CollapsingHeader::new("Technical details")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.monospace(detail);
                            });
                    }
                }
            }
        });

        if self.show_doctor {
            let mut open = true;
            let mut rerun = false;
            egui::Window::new("Doctor")
                .open(&mut open)
                .default_width(520.0)
                .resizable(true)
                .show(ctx, |ui| {
                    match &self.doctor {
                        None => {
                            ui.label("No checks have run yet.");
                        }
                        Some(DoctorState::Loading) => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Checking backends…");
                            });
                        }
                        Some(DoctorState::Failed(error)) => {
                            ui.label(
                                egui::RichText::new(error)
                                    .color(egui::Color32::from_rgb(200, 70, 70)),
                            );
                        }
                        Some(DoctorState::Ready(checks)) => {
                            egui::ScrollArea::vertical()
                                .max_height(320.0)
                                .show(ui, |ui| {
                                    for check in checks {
                                        ui.horizontal(|ui| {
                                            let (mark, color) = if check.ok {
                                                ("✓", egui::Color32::from_rgb(40, 150, 90))
                                            } else {
                                                ("✗", egui::Color32::from_rgb(200, 70, 70))
                                            };
                                            ui.label(
                                                egui::RichText::new(mark).color(color).strong(),
                                            );
                                            ui.label(&check.name);
                                            if !check.detail.is_empty() {
                                                ui.label(egui::RichText::new(&check.detail).weak());
                                            }
                                        });
                                    }
                                });
                        }
                    }

                    ui.add_space(8.0);
                    if ui.button("Run checks again").clicked() {
                        rerun = true;
                    }
                });
            self.show_doctor = open;
            if rerun {
                self.start_doctor();
            }
        }

        if self.show_about {
            let mut open = true;
            egui::Window::new(format!("About {}", support::APP_NAME))
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading(support::APP_NAME);
                    ui.label(support::TAGLINE);
                    ui.add_space(8.0);
                    for line in support::about_lines() {
                        ui.label(line);
                    }
                });
            self.show_about = open;
        }

        if self.show_formats {
            let mut open = true;
            egui::Window::new("Formats")
                .open(&mut open)
                .default_width(460.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for format in tt_core::ALL_FORMATS {
                                let targets = targets_for(*format);
                                if targets.is_empty() {
                                    continue;
                                }
                                ui.label(egui::RichText::new(format.label()).strong());
                                let list = targets
                                    .iter()
                                    .map(|target| target.name())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                ui.label(egui::RichText::new(format!("  → {list}")).weak());
                                ui.add_space(4.0);
                            }
                        });
                });
            self.show_formats = open;
        }

        if source_edited {
            self.output_custom = false;
            self.refresh_source();
        }
        if target_changed {
            self.refresh_output();
        }
        if output_edited {
            self.output_custom = true;
        }

        if browse_source {
            let start = self.settings.last_source_dir.clone();
            if let Some(path) = pick_file(start.as_deref()) {
                self.set_source(&path);
            }
        }
        if browse_output {
            let start = self.settings.last_output_dir.clone();
            if let Some(path) = pick_output(start.as_deref(), &self.output_text) {
                self.output_text = path.display().to_string();
                self.output_custom = true;
            }
        }

        if doctor_clicked {
            self.start_doctor();
        }
        if convert_clicked {
            self.start();
        }
        if let Some(path) = show_folder {
            if let Err(error) = show_in_folder(&path) {
                let (headline, detail) = support::friendly_error(&format!("{error:#}"));
                self.status = Status::Failed { headline, detail };
            }
        }
        if let Some(path) = open_result {
            if let Err(error) = open_path(&path) {
                let (headline, detail) = support::friendly_error(&format!("{error:#}"));
                self.status = Status::Failed { headline, detail };
            }
        }

        if theme_before != self.settings.dark_theme {
            ctx.set_visuals(visuals(self.settings.dark_theme));
            self.settings_dirty = true;
        }
        if open_before != self.settings.open_after || overwrite_before != self.settings.overwrite {
            self.settings_dirty = true;
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            self.remember_window(ctx);
        } else if self.settings_dirty {
            self.save_settings();
        }
    }
}

fn visuals(dark: bool) -> egui::Visuals {
    if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    }
}

/// Targets offered for a source format, in registry order.
fn targets_for(source: Format) -> Vec<Format> {
    let mut targets = Vec::new();
    for conversion in CONVERSIONS.iter().filter(|item| item.from == source) {
        if !targets.contains(&conversion.to) {
            targets.push(conversion.to);
        }
    }
    targets
}

fn backend_for(from: Format, to: Format) -> Option<String> {
    CONVERSIONS
        .iter()
        .find(|item| item.from == from && item.to == to)
        .map(|item| item.backend.to_string())
}

/// Decide whether writing `output` is allowed, without touching the filesystem.
fn check_overwrite(input: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }

    let same_file = match (input.canonicalize(), output.canonicalize()) {
        (Ok(input), Ok(output)) => input == output,
        _ => input == output,
    };
    if same_file {
        return Err(
            "JConverter will not overwrite the file it is converting. Choose another name."
                .to_string(),
        );
    }

    if overwrite {
        return Ok(());
    }

    Err("A file already exists there. Enable overwrite or choose another name.".to_string())
}

fn pick_file(start: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title("Choose a file to convert");
    if let Some(directory) = start {
        dialog = dialog.set_directory(directory);
    }
    dialog
        .add_filter(
            "Supported files",
            &[
                "md", "pdf", "docx", "png", "jpg", "jpeg", "webp", "bmp", "gif", "tif", "tiff",
            ],
        )
        .add_filter("All files", &["*"])
        .pick_file()
}

fn pick_output(start: Option<&Path>, suggested: &str) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title("Choose where to save the result");
    if let Some(directory) = start {
        dialog = dialog.set_directory(directory);
    }
    if let Some(name) = Path::new(suggested).file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy().to_string());
    }
    dialog.save_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_offers_image_targets_but_not_docx() {
        let targets = targets_for(Format::Png);
        assert!(targets.contains(&Format::Webp));
        assert!(targets.contains(&Format::Jpg));
        assert!(!targets.contains(&Format::Pdf));
        assert!(!targets.contains(&Format::Docx));
        assert!(!targets.contains(&Format::Png));
    }

    #[test]
    fn word_documents_only_reach_pdf_and_markdown() {
        assert_eq!(
            targets_for(Format::Docx),
            vec![Format::Pdf, Format::Markdown]
        );
    }

    #[test]
    fn every_source_format_offers_at_least_one_target() {
        for format in tt_core::ALL_FORMATS {
            assert!(
                !targets_for(*format).is_empty(),
                "{format:?} has no targets"
            );
        }
    }

    #[test]
    fn backends_come_from_the_registry() {
        assert_eq!(
            backend_for(Format::Png, Format::Webp).as_deref(),
            Some("Pillow")
        );
        assert_eq!(
            backend_for(Format::Docx, Format::Pdf).as_deref(),
            Some("LibreOffice")
        );
        assert!(backend_for(Format::Png, Format::Docx).is_none());
    }

    #[test]
    fn missing_output_needs_no_confirmation() {
        let input = Path::new("/tmp/report.docx");
        let output = Path::new("/tmp/does-not-exist-jconverter.md");
        assert!(check_overwrite(input, output, false).is_ok());
    }

    #[test]
    fn refusing_to_overwrite_the_input() {
        let input = std::env::temp_dir().join("jconverter-overwrite-input.png");
        std::fs::write(&input, b"x").unwrap();
        let error = check_overwrite(&input, &input, true).unwrap_err();
        assert!(error.contains("will not overwrite"));
        std::fs::remove_file(&input).unwrap();
    }

    #[test]
    fn existing_output_requires_the_overwrite_setting() {
        let input = std::env::temp_dir().join("jconverter-overwrite-source.png");
        let output = std::env::temp_dir().join("jconverter-overwrite-target.webp");
        std::fs::write(&input, b"x").unwrap();
        std::fs::write(&output, b"x").unwrap();

        let error = check_overwrite(&input, &output, false).unwrap_err();
        assert!(error.contains("Enable overwrite"));
        assert!(check_overwrite(&input, &output, true).is_ok());

        std::fs::remove_file(&input).unwrap();
        std::fs::remove_file(&output).unwrap();
    }
}
