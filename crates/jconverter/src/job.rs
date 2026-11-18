use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use tt_core::{Check, Engine, Format, open_path};

/// Work handed to the background thread. Conversions never run on the UI thread.
pub enum Job {
    Convert {
        input: PathBuf,
        from: Format,
        to: Format,
        output: PathBuf,
        open_after: bool,
    },
    Doctor,
}

/// Results handed back to the UI thread.
pub enum WorkerMessage {
    Converted {
        output: PathBuf,
        backend: Option<String>,
        error: Option<String>,
    },
    Doctor(Result<Vec<Check>, String>),
}

/// Start the worker thread and return the two halves of the channel pair.
pub fn start() -> (Sender<Job>, Receiver<WorkerMessage>) {
    let (job_sender, job_receiver) = mpsc::channel::<Job>();
    let (result_sender, result_receiver) = mpsc::channel::<WorkerMessage>();

    thread::Builder::new()
        .name("jconverter-worker".into())
        .spawn(move || {
            while let Ok(job) = job_receiver.recv() {
                let message = match job {
                    Job::Convert {
                        input,
                        from,
                        to,
                        output,
                        open_after,
                    } => convert(&input, from, to, &output, open_after),
                    Job::Doctor => WorkerMessage::Doctor(doctor()),
                };

                if result_sender.send(message).is_err() {
                    break;
                }
            }
        })
        .expect("could not start the conversion worker");

    (job_sender, result_receiver)
}

fn convert(
    input: &Path,
    from: Format,
    to: Format,
    output: &Path,
    open_after: bool,
) -> WorkerMessage {
    let engine = match Engine::discover() {
        Ok(engine) => engine,
        Err(error) => {
            return WorkerMessage::Converted {
                output: output.to_path_buf(),
                backend: None,
                error: Some(format!("{error:#}")),
            };
        }
    };

    match engine.convert(input, from, to, output) {
        Ok(outcome) => {
            if open_after {
                if let Err(error) = open_path(&outcome.output) {
                    eprintln!("{error:#}");
                }
            }

            WorkerMessage::Converted {
                output: outcome.output,
                backend: outcome.backend,
                error: None,
            }
        }
        Err(error) => WorkerMessage::Converted {
            output: output.to_path_buf(),
            backend: None,
            error: Some(format!("{error:#}")),
        },
    }
}

fn doctor() -> Result<Vec<Check>, String> {
    let engine = Engine::discover().map_err(|error| format!("{error:#}"))?;
    engine.doctor().map_err(|error| format!("{error:#}"))
}
