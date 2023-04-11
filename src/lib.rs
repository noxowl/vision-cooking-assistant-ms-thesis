extern crate tokio_core;

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Clone)]
pub enum CommonMessage {
    HumanDetected,
    ObjectRecognizeOrder,
    DetectedObject(String),
    OrderDetected(IntentType),
    OrderCancelled,
    RequestVA(u8),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum IntentType {
    TurnOnSomething = 0,
    TurnOffSomething = 1
}

impl FromStr for IntentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "turnonsomething" => Ok(IntentType::TurnOnSomething),
            "turnoffsomething" => Ok(IntentType::TurnOffSomething),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub pv_api_key: String,
    pub pv_model_name: String,
    pub mic_index: i32,
    pub with_vision: bool,
    pub actions: Vec<AgentAction>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pv_api_key: "".to_string(),
            pv_model_name: "model.rhn".to_string(),
            with_vision: false,
            mic_index: 0,
            actions: vec![],
        }
    }
}

impl Config {
    fn new() -> Self {
        Default::default()
    }

    // fn get_action(&self, id: i32) {
    //     match self.actions.get(id) {
    //         Some(action) => {},
    //         None => {}
    //     }
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentAction {
    pub id: i32,
    pub name: String,
    pub action_type: ActionType,
    pub command: String
}

impl fmt::Display for AgentAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("Action Name: {}, Action Type: {}, Action Command: {}", self.name, self.action_type, self.command).as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "beep")]
    Beep = 0,
    #[serde(rename = "philips_hue")]
    PhilipsHue = 1,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionType::Beep => write!(f, "Beep"),
            ActionType::PhilipsHue => write!(f, "Philips Hue")
        }
    }
}
