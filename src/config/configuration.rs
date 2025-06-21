use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, ValueEnum, PartialEq, Eq, Debug)]
pub enum NumberStyle {
    Standard,
    Pipe,
    Lite,
    Utf8,
}

#[derive(Parser, Debug)]
#[command(
    name = "totp-generator",
    about = "Generate TOTP tokens via CLI, UI, or HTTP API"
)]
pub struct Args {
    /// Optional path to log file
    #[arg(long, env = "TOTP_LOG_FILE")]
    pub log_file: Option<PathBuf>,

    /// Output logs to stderr. May interfere with UI.
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Log to stderr (may interfere with UI — consider redirecting)"
    )]
    pub std_err: bool,

    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Origin {
    GoogleAuthenticator,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Generate a one-time code
    OneTime {
        /// The secret name or index to use
        #[arg(required = true)]
        target: String,

        /// Path to secrets TOML file
        #[arg(short, long, env = "TOTP_SECRETS")]
        secrets: String,
    },

    /// Run the console UI and/or HTTP interface
    Interface {
        /// Path to secrets JSON file
        #[arg(short, long, env = "TOTP_SECRETS")]
        secrets: String,

        /// Bind HTTP server to this address (e.g. 127.0.0.1)
        #[arg(short, long)]
        bind: Option<String>,

        /// Disable the Console UI
        #[arg(short, long, action = ArgAction::SetTrue)]
        no_console: bool,

        /// Port to serve HTTP on (default: 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Time in seconds before locking UI (0 to disable)
        #[arg(short, long, default_value_t = 300)]
        lock_after: u16,

        /// One of: standard, pipe, lite, utf8
        #[arg(long, value_enum, default_value_t = NumberStyle::Standard)]
        number_style: NumberStyle,
    },

    /// Import a secret config from a QR code image
    Configure {
        /// Path to an image containing a QR code
        #[arg(long, value_name = "IMAGE")]
        from_image: PathBuf,

        /// Prompt for additional details interactively
        #[arg(long, action = ArgAction::SetTrue)]
        prompt: bool,

        /// Flag to use the zbar C library
        #[arg(long, action = ArgAction::SetTrue)]
        use_zbar: bool,

        /// Origin of the QR being loaded
        #[arg(long, value_enum, default_value_t = Origin::GoogleAuthenticator)]
        origin: Origin,
    },
}
