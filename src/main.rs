mod config;
mod logging;
mod output;
mod state;
mod totp;

use std::env;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use config::{configuration::Args, secrets::ConfigFile};

#[cfg(feature = "onetime")]
use output::onetime::one_time_mode;

#[cfg(feature = "http")]
use output::web::server::start_server;

#[cfg(feature = "cli")]
use output::cui::console::start_console_ui;

use state::State;
use tokio::sync::oneshot;
use tokio::{signal, task::JoinSet};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Need to keep reference to _log otherwise lose the log file
    let _log = logging::setup_tracing(&args.log_file, args.std_err.unwrap());
    tracing::info!("Starting app...");
    let mut secrets_cf = ConfigFile::new(args.secrets.clone());
    if let Some(arg) = &args.one_time {
        #[cfg(feature = "onetime")]
        {
            let o = one_time_mode(&mut secrets_cf, arg).await?;
            tracing::info!("One time mode outcome: {o}");
            println!("{o}");
            return Ok(());
        }
        #[cfg(not(feature = "onetime"))]
        {
            tracing::warn!("One-time mode is not enabled in this build. Please enable the 'onetime' feature to use it.");
            eprintln!("One-time mode is not enabled in this build. Please enable the 'oneime' feature to use it.");
            return Err(anyhow::anyhow!("One-time mode is not enabled in this build. Please enable the 'onetime' feature to use it."));
        }
    }

    let mut set: JoinSet<()> = JoinSet::new();
    let secrets_cf = Arc::new(secrets_cf);
    let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
    let (ui_shutdown_tx, ui_shutdown_rx) = oneshot::channel::<()>();
    if let Some(bind) = args.bind {
        // If --bind is provided, launch the server
        #[cfg(feature = "http")]
        {
            let web_secrets_cf = Arc::clone(&secrets_cf);
            tracing::info!("Launching HTTP server at {}:{}", bind, args.port);
            let bind = bind.clone();
            let port = args.port;
            // Due to actix_web not being Send, we have to run this in a separate thread
            thread::spawn(move || {
                actix_web::rt::System::new().block_on(async move {
                    tokio::select! {
                        i = start_server(bind, port, web_secrets_cf) => {
                            match i {
                                Err(err) => {
                                    tracing::error!("HTTP server error'd: {err}")
                                }
                                Ok(i) => {

                            tracing::info!("HTTP server ended {:?}", i);
                                }
                            }
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
                println!("Received request to shutdown HTTP server via oneshot channel")
            });
        }
        #[cfg(not(feature = "http"))]
        {
            tracing::warn!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it.");
            eprintln!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it.");
            return Err(anyhow::anyhow!("HTTP server is not enabled in this build. Please enable the 'http' feature to use it."));
        }
    }
    if let Some(false) = args.no_console {
        #[cfg(feature = "cli")]
        {
            let unlock_password = env::var("UNLOCK_PASSWORD").ok();
            // Default to console UI
            let state = State::default(
                Arc::clone(&secrets_cf),
                unlock_password,
                args.lock_after,
                args.number_style,
            );
            set.spawn(async move {
                start_console_ui(state).await;
            });
        }
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
