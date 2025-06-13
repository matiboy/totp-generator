mod config;
mod output;
mod state;
mod totp;
mod logging;

use std::env;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use config::{configuration::Args, secrets::load_secrets};
#[cfg(feature = "onetime")]
use output::onetime::one_time_mode;
#[cfg(feature = "http")]
use output::web::server::start_server;
#[cfg(feature = "cli")]
use output::cui::console::start_console_ui;
use state::State;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::{signal, task::JoinSet};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Need to keep reference to _log otherwise lose the log file
    let _log = logging::setup_tracing(&args.log_file, args.std_err.unwrap());
    tracing::info!("Starting app...");
    if let Some(arg) = &args.one_time {
        #[cfg(feature = "cli")]
        {
            let o = one_time_mode(&args.secrets, arg)?;
            tracing::info!("One time mode outcome: {o}");
            println!("{o}");
        }
        #[cfg(not(feature = "cli"))]
        {
            tracing::warn!("One-time mode is not enabled in this build. Please enable the 'cli' feature to use it.");
            eprintln!("One-time mode is not enabled in this build. Please enable the 'cli' feature to use it.");
            return anyhow!("One-time mode is not enabled in this build. Please enable the 'cli' feature to use it.");
        }
    }

    let secrets: Arc<Mutex<String>> = Arc::new(Mutex::new(args.secrets));
    let mut set = JoinSet::new();
    let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
    let (ui_shutdown_tx, ui_shutdown_rx) = oneshot::channel::<()>();
    if let Some(bind) = args.bind {
        // If --bind is provided, launch the server
        #[cfg(feature = "http")]
        {
            tracing::info!("Launching HTTP server at {}:{}", bind, args.port);
            let bind = bind.clone();
            let port = args.port;
            // Due to actix_web not being Send, we have to run this in a separate thread
            let secrets = Arc::clone(&secrets);
            thread::spawn(move || {
                actix_web::rt::System::new().block_on(async move {
                    tokio::select! {
                        _ = start_server(bind, port, secrets) => {
                            tracing::info!("HTTP server started");
                        }
                        _ = ui_shutdown_rx => {
                            tracing::info!("HTTP server shutdown requested");
                        }
                    };
                    let _ = http_shutdown_tx.send(());
                });
            });
            set.spawn(async move {
                http_shutdown_rx.await;
            });
        }
        #[cfg(not(feature = "http"))]
        {
            tracing::warn!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it.");
            eprintln!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it.");
            return Err(anyhow!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it."));
        }
    }
    if let Some(false) = args.no_console {
        let unlock_password = env::var("UNLOCK_PASSWORD").ok();
        // Default to console UI
        let state = State::default(
            Arc::clone(&secrets),
            unlock_password,
            args.lock_after,
            args.number_style,
        );
        set.spawn(async move {
            start_console_ui(state).await;
        });
    }
    if set.is_empty() {
        println!("Please select at least one of the modes: console/http or one-time");
        return Ok(());
    }
    tokio::select! {
        biased;
        _ = signal::ctrl_c() => {
            println!("Received Ctrl-C, aborting all tasks...");
        }
        Some(result) = set.join_next() => {
            match result {
                Ok(v) => println!("A task finished with value {v:?}"),
                Err(e) => eprintln!("A task failed: {e}"),
            }
        }
    };
    tracing::info!("Exited main select");
    ui_shutdown_tx.send(());
    Ok(())
}
