use anyhow::{Result, anyhow};
use crate::utils::vision_util::VisionType;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Run,
    Help,
}

pub(crate) struct Cli {
    args: Vec<String>,
}

impl Cli {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args
        }
    }

    pub fn parse_command(&self) -> Result<Command> {
        match self.args.get(0).expect("no query given")[..].as_ref() {
            "run" => Ok(Command::Run),
            "help" => Ok(Command::Help),
            _ => Err(anyhow!("no matched command found. type help for available commands."))
        }
    }

    pub fn parse_config(&self) -> Result<Config> {
        let mut config = Config::new();
        for (i, arg) in self.args.iter().enumerate() {
            match arg.as_str() {
                "--pv-api-key" => {
                    config.pico_voice_api_key = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone();
                }
                "--pv-model-path" => {
                    config.pico_voice_rhn_model_path = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone();
                }
                "--mic-index" => {
                    config.mic_index = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone().parse::<u32>()?;
                }
                "--vision-type" => {
                    config.vision_type = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone().parse::<VisionType>()?;
                }
                "--debug" => {
                    config.debug = true;
                }
                "--vision" => {
                    config.vision = true;
                }
                "--zmq-in-endpoint" => {
                    config.zmq_in_endpoint = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone();
                }
                "--stream-out-endpoint" => {
                    config.stream_out_endpoint = self.args.get(i + 1).ok_or(anyhow!("no argument found for option")).unwrap().clone();
                }
                _ => {}
            }
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Config {
    pub pico_voice_api_key: String,
    pub pico_voice_rhn_model_path: String,
    pub mic_index: u32,
    pub vision_type: VisionType,
    pub vision: bool,
    pub debug: bool,
    pub zmq_in_endpoint: String,
    pub stream_out_endpoint: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            pico_voice_api_key: "".to_string(),
            pico_voice_rhn_model_path: "model.rhn".to_string(),
            mic_index: 0,
            vision_type: VisionType::None,
            vision: false,
            debug: false,
            zmq_in_endpoint: "".to_string(),
            stream_out_endpoint: "".to_string(),
        }
    }
}
