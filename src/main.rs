use std::env;
use anyhow::Result;
use crate::utils::config_util::{Cli, Command};
extern crate opencv;

mod core;
mod core_tests;
mod utils;
mod smart_speaker;
mod query;

 fn main() -> Result<()> {
    let cli = Cli::new(env::args().skip(1).collect::<Vec<_>>());
    match cli.parse_command() {
        Ok(command) => {
            match command {
                Command::Run => {
                    core::run_smart_speaker(cli.parse_config()?);
                }
                Command::Help => {
                    println!("available commands:");
                    println!("run: run smart speaker");
                    println!("help: show this help");
                    println!("available options:");
                    println!("--pv-api-key: pico voice api key");
                    println!("--pv-model-path: pico voice rhn model path. relative path from executable file.");
                    println!("--mic-index: mic index. 0 / 1 / 2 / ...");
                    println!("--vision-type: vision type. none / pupil / camera");
                    println!("--debug: debug mode. true / false");
                    println!("--vision: vision mode. true / false. if this option is enabled, --vision-type option is required.");
                    println!("--stream-endpoint: stream endpoint");
                }
            }
        }
        _ => {
            println!("no matched query found. type help for available commands.");
        }
    }
    Ok(())
}
