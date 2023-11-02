use std::fmt::{Debug, Formatter};
use anyhow::{anyhow, Error, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::utils::message_util::{Content};

pub(crate) mod cooking_task;
pub(crate) mod vision_viewing_task;
pub(crate) mod vision_cooking_task;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SmartSpeakerTaskResult {
    pub(crate) code: SmartSpeakerTaskResultCode,
    pub(crate) tts: Option<String>,
}

impl SmartSpeakerTaskResult {
    pub(crate) fn new(code: SmartSpeakerTaskResultCode) -> Self {
        SmartSpeakerTaskResult {
            code,
            tts: None,
        }
    }

    pub(crate) fn with_tts(code: SmartSpeakerTaskResultCode, tts: String) -> Self {
        SmartSpeakerTaskResult {
            code,
            tts: Some(tts),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SmartSpeakerTaskResultCode {
    Exit,
    Wait(PendingType),
    Cancelled,
}

pub(crate) trait Task: Send {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult>;
    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult>;
    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult>;
    fn internal_move_next(&mut self) -> Result<bool>;
    fn internal_rollback(&mut self) -> Result<bool>;
    fn exit(&self) -> Result<SmartSpeakerTaskResult>;
}


