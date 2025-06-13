use std::{fs::File, path::PathBuf};
use tracing_subscriber::prelude::*;

use tracing_appender::{self, non_blocking::WorkerGuard};
use tracing_subscriber::{fmt, EnvFilter};

pub struct LogGuards {
    _file_guard: Option<WorkerGuard>,
}

pub fn setup_tracing(log_file: &Option<PathBuf>, std_err: bool) -> LogGuards {
    let mut layers = Vec::new();
    let mut file_guard = None;

    // File logging
    if let Some(path) = log_file {
        let file = File::create(path).expect("Failed to create log file");
        let (nb_writer, guard) = tracing_appender::non_blocking(file);
        file_guard = Some(guard);
        layers.push(fmt::layer().with_writer(nb_writer).with_ansi(false).boxed());
    }

    // Stderr logging
    if std_err {
        layers.push(fmt::layer().with_writer(std::io::stderr).boxed());
    }

    // Combine and initialize
    tracing_subscriber::registry()
        .with(layers)
        .with(EnvFilter::from_default_env()) // RUST_LOG respected
        .init();

    LogGuards {
        _file_guard: file_guard,
    }
}
