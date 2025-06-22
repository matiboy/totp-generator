mod config;
mod logging;
mod output;
mod qr;
mod state;
mod totp;

use std::sync::Arc;
use std::thread;
use std::env;

use clap::Parser;
use config::{configuration::Args, secrets::ConfigFile};

#[cfg(feature = "onetime")]
use output::onetime::one_time_mode;

#[cfg(feature = "http")]
use output::web::server::start_server;

#[cfg(feature = "cli")]
use output::cui::console::start_console_ui;

#[cfg(feature = "configure")]
use qr::prompt::generate_configuration;

use state::State;
use tokio::sync::oneshot;
use tokio::{signal, task::JoinSet};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Need to keep reference to _log otherwise lose the log file
    let _log = logging::setup_tracing(&args.log_file, args.std_err);
    match args.mode {
        config::configuration::Mode::OneTime { target, secrets } => {
            #[cfg(feature = "onetime")]
            {
                let mut secrets_cf = ConfigFile::new(secrets.clone());
                let o = one_time_mode(&mut secrets_cf, &target).await?;
                tracing::info!("One time mode outcome: {o}");
                println!("{o}");
                Ok(())
            }
            #[cfg(not(feature = "onetime"))]
            {
                tracing::warn!(
                    "One-time mode is not enabled in this build. Please enable the 'onetime' feature to use it."
                );
                eprintln!(
                    "One-time mode is not enabled in this build. Please enable the 'onetime' feature to use it."
                );
                Err(anyhow::anyhow!(
                    "One-time mode is not enabled in this build. Please enable the 'onetime' feature to use it."
                ))
            }
        }
        config::configuration::Mode::Interface {
            secrets,
            bind,
            no_console,
            port,
            lock_after,
            number_style,
        } => {
            let mut set: JoinSet<()> = JoinSet::new();
            let secrets_cf = Arc::new(ConfigFile::new(secrets.clone()));
            let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
            let (ui_shutdown_tx, ui_shutdown_rx) = oneshot::channel::<()>();
            if let Some(bind) = bind {
                // If --bind is provided, launch the server
                #[cfg(feature = "http")]
                {
                    let web_secrets_cf = Arc::clone(&secrets_cf);
                    tracing::info!("Launching HTTP server at {}:{}", bind, port);
                    let bind = bind.clone();
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
                        let _ = http_shutdown_rx.await;
                        println!("Received request to shutdown HTTP server via oneshot channel")
                    });
                }
                #[cfg(not(feature = "http"))]
                {
                    tracing::warn!(
                        "HTTP server is not enabled in this build. Please enable the 'http' feature to use it."
                    );
                    eprintln!(
                        "HTTP server is not enabled in this build. Please enable the 'http' feature to use it."
                    );
                    return Err(anyhow::anyhow!(
                        "HTTP server is not enabled in this build. Please enable the 'http' feature to use it."
                    ));
                }
            }
            if !no_console {
                #[cfg(feature = "cli")]
                {
                    let unlock_password = env::var("UNLOCK_PASSWORD").ok();
                    // Default to console UI
                    let state = State::default(
                        Arc::clone(&secrets_cf),
                        unlock_password,
                        lock_after,
                        number_style,
                    );
                    set.spawn(async move {
                        let _ = start_console_ui(state).await;
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
            let _ = ui_shutdown_tx.send(());
            Ok(())
        }
        config::configuration::Mode::Configure {
            from_image,
            prompt,
            origin,
            use_zbar,
        } => {
            #[cfg(feature = "configure")]
            {
                generate_configuration(from_image, prompt, origin, use_zbar)
                    .await
            }
            #[cfg(not(feature = "configure"))]
            {
                let _ = ( from_image, prompt, origin, use_zbar ); // This is to avoid unused
                                                                  // variable warnings
                tracing::warn!(
                    "Configuration mode is not enabled in this build. Please enable the 'configure' feature to use it."
                );
                Err(anyhow::anyhow!(
                    "Configuration mode is not enabled in this build. Please enable the 'configure' feature to use it."
                ))
            }
        }
    }
}
