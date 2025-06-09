mod config;
mod state;
mod totp;
mod output;

use std::env;

use config::{secrets::load_secrets, configuration::Args};
use output::{onetime::one_time_mode, web::server::start_server, cui::console::start_console_ui};
use clap::Parser;
use state::State;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[actix_web::main]
async fn main()-> std::io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy()
            ) // can be overridden via RUST_LOG
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // example usage
    tracing::info!("Starting app...");
    tracing::debug!("Starting app...");
    let args = Args::parse();
    if let Some(arg) = &args.one_time {
        one_time_mode(&args.secrets, arg);
        return Ok(());
    }
    if let Some(bind) = args.bind {
        // If --bind is provided, launch the server
        println!("Launching HTTP server at {}:{}", bind, args.port);
        start_server(&bind, args.port, args.secrets.clone())
        .await?;
    }
    let unlock_password = env::var("UNLOCK_PASSWORD").ok();
    // Default to console UI
    let state = State::default(args.secrets.clone(), unlock_password, args.lock_after, args.number_style);
    if let Err(err) = start_console_ui(state).await {
        eprintln!("Error rendering UI {}", err);
    } else {
        eprintln!("Exiting CUI");
    }
    Ok(())
}
