use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, env = "AUTHENTICATOR_SECRETS")]
    pub secrets: String, // mandatory
    #[arg(short, long)]
    pub one_time: Option<String>, // optional, can be int or code
    #[arg(short, long)]
    pub bind: Option<String>, // optional
    #[arg(short, long, default_value = "3000")]
    pub port: u16, // default to 3000
    #[arg(short, long, default_value = "300", help="Time in seconds before locking the user interface; set it to 0 to disable")]
    pub lock_after: u8,
}
