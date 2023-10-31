use std::fmt;
use std::fmt::{Debug, Formatter};
use anyhow::{anyhow, Error, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::utils::message_util::{Content};

pub(crate) mod cooking_task;
pub(crate) mod viewing_task;

#[derive(Debug, PartialEq)]
pub(crate) enum SmartSpeakerTaskResultCodes {
    Exit(String),
    Wait(PendingType),
    TTS(String),
}

pub(crate) trait Task: Send {
    fn init(&mut self) -> Result<SmartSpeakerTaskResultCodes>;
    fn execute(&mut self) -> Result<SmartSpeakerTaskResultCodes>;
    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes>;
    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes>;
    fn internal_move_next(&mut self) -> Result<bool>;
    fn internal_rollback(&mut self) -> Result<bool>;
    fn exit(&self) -> Result<SmartSpeakerTaskResultCodes>;
}


