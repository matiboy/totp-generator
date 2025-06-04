mod config;
mod state;
mod totp;
mod output;

use std::env;

use config::{secrets::load_secrets, configuration::Args};
use output::{onetime::one_time_mode, web::server::start_server, cui::console::start_console_ui};
use clap::Parser;
use state::State;

#[actix_web::main]
async fn main()-> std::io::Result<()> {
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
    if let Err(err) = start_console_ui(state, 1).await {
        eprintln!("Error rendering UI {}", err);
    } else {
        eprintln!("Exiting CUI");
    }
    Ok(())
}
