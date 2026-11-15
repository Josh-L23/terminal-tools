use anyhow::Result;
use console::style;
use tt_core::Engine;

pub fn execute() -> Result<()> {
    println!("{}", style("Terminal Tools doctor").bold());
    println!(
        "  {} {:<16} {}",
        mark(true),
        "Rust CLI",
        env!("CARGO_PKG_VERSION")
    );

    let engine = match Engine::discover() {
        Ok(engine) => engine,
        Err(error) => {
            println!("  {} {:<16} {error}", mark(false), "Python engine");
            println!(
                "\nRun {} to set up the Python engine.",
                style("scripts/setup.sh").cyan()
            );
            std::process::exit(1);
        }
    };

    println!(
        "  {} {:<16} {}",
        mark(true),
        "Engine",
        engine.engine_dir().display()
    );

    let version = engine.python_version();
    let python_ok = version.is_some();
    let version = version
        .as_deref()
        .and_then(|version| version.strip_prefix("Python "))
        .unwrap_or("not runnable");
    println!("  {} {:<16} {version}", mark(python_ok), "Python");
    println!(
        "  {} {:<16} {}",
        mark(python_ok),
        "Interpreter",
        engine.python().display()
    );

    let mut failures = usize::from(!python_ok);
    for check in engine.doctor()? {
        if check.name == "Python runtime" || check.name == "Engine package" {
            continue;
        }

        if !check.ok {
            failures += 1;
        }
        println!("  {} {:<16} {}", mark(check.ok), check.name, check.detail);
    }

    if failures > 0 {
        println!(
            "\n{} check(s) failed. Run {} to install the Python dependencies.",
            failures,
            style("scripts/setup.sh").cyan()
        );
        std::process::exit(1);
    }

    println!("\n{} Everything is ready.", style("✓").green().bold());
    Ok(())
}

fn mark(ok: bool) -> String {
    if ok {
        style("✓").green().bold().to_string()
    } else {
        style("✗").red().bold().to_string()
    }
}
