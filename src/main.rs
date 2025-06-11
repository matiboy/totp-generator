mod config;
mod output;
mod state;
mod totp;

use std::env;
use std::thread;

use clap::Parser;
use config::{configuration::Args, secrets::load_secrets};
use output::{cui::console::start_console_ui, onetime::one_time_mode, web::server::start_server};
use state::State;
use tokio::sync::oneshot;
use tokio::{signal, task::JoinSet};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        ) // can be overridden via RUST_LOG
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // example usage
    tracing::info!("Starting app...");
    tracing::debug!("Starting app...");
    let args = Args::parse();
    if let Some(arg) = &args.one_time {
        one_time_mode(&args.secrets, arg);
        return Ok(());
    }
    let mut set = JoinSet::new();
    let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
    let (ui_shutdown_tx, ui_shutdown_rx) = oneshot::channel::<()>();
    if let Some(bind) = args.bind {
        // If --bind is provided, launch the server
        println!("Launching HTTP server at {}:{}", bind, args.port);
        let bind = bind.clone();
        let port = args.port;
        let secrets = args.secrets.clone();
        thread::spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                tokio::select! {
                    _ = start_server(bind, port, secrets) => {
                        tracing::info!("HTTP server started");
                    }
                    _ = ui_shutdown_rx => {
                        tracing::info!("UI shutdown, stopping http too");
                    }
                };
                let _ = http_shutdown_tx.send(());
            });
        });
        set.spawn(async move {
            http_shutdown_rx.await;
        });
    }
    if let Some(false) = args.no_console {
        let unlock_password = env::var("UNLOCK_PASSWORD").ok();
        // Default to console UI
        let state = State::default(
            args.secrets.clone(),
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
