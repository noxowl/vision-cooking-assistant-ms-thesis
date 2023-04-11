#![feature(exclusive_range_pattern)]
mod lib;
mod auditio;
mod visio;
mod logic;
mod debug;

use std::env;
use anyhow::Result;
use crate::lib::Config;

struct Cli {
    command: String,
    option: Option<String>
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = env::args().nth(1).expect("no query given");
    let option = env::args().nth(2);
    let args = Cli {
        command,
        option,
    };
    match &args.command[..] {
        "run" => {
            logic::run_va().await?
        },
        "init" => {
            logic::init_marker().await?
        },
        _ => {
            println!("no matched query found. type help for available commands.");
        }
    }
    Ok(())
}
