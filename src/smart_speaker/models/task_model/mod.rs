use std::fmt::Debug;
use anyhow::Result;
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::Revision;

pub(crate) mod cooking_task;
pub(crate) mod vision_viewing_task;
pub(crate) mod vision_cooking_task;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SmartSpeakerTaskResult {
    pub(crate) code: SmartSpeakerTaskResultCode,
    pub(crate) tts: Option<SmartSpeakerI18nText>,
    pub(crate) revision: Option<Box<dyn Revision>>
}

impl SmartSpeakerTaskResult {
    pub(crate) fn new(code: SmartSpeakerTaskResultCode) -> Self {
        SmartSpeakerTaskResult {
            code,
            tts: None,
            revision: None,
        }
    }

    pub(crate) fn with_tts(code: SmartSpeakerTaskResultCode, tts: SmartSpeakerI18nText) -> Self {
        SmartSpeakerTaskResult {
            code,
            tts: Some(tts),
            revision: None,
        }
    }

    pub(crate) fn with_tts_and_revision(code: SmartSpeakerTaskResultCode, tts: SmartSpeakerI18nText, revision: Box<dyn Revision>) -> Self {
        SmartSpeakerTaskResult {
            code,
            tts: Some(tts),
            revision: Some(revision),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SmartSpeakerTaskResultCode {
    Exit,
    RepeatPrevious,
    StepSuccess,
    StepFailed,
    TaskSuccess(WaitingInteraction),
    TaskFailed(WaitingInteraction),
    Cancelled,
}

pub(crate) trait Task: Send {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult>;
    fn next_index(&self) -> Option<usize>;
    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult>;
    fn handle_result(&mut self, result: SmartSpeakerTaskResult) -> Result<SmartSpeakerTaskResult>;
    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult>;
    fn internal_move_next(&mut self) -> Result<bool>;
    fn internal_rollback(&mut self) -> Result<bool>;

    fn exit(&self) -> Result<SmartSpeakerTaskResult>;
    fn cancel(&self) -> Result<SmartSpeakerTaskResult>;
}


