use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    pub secrets: String, // mandatory
    #[arg(short, long)]
    pub one_time: Option<String>, // optional, can be int or code
    #[arg(short, long)]
    pub bind: Option<String>, // optional
    #[arg(short, long, default_value = "3000")]
    pub port: u16, // default to 3000
}
