mod config;
mod topt;
mod output;

use topt::generate_totp;
use config::{secrets::load_config, configuration::Args};
use output::onetime::one_time_mode;
use clap::Parser;

fn main() {
    let args = Args::parse();
    if let Some(arg) = &args.one_time {
        one_time_mode(&args.secrets, arg);
        return;
    }
}
