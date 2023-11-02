use std::fmt;
use std::fmt::{Debug, Formatter};
use anyhow::Result;
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::utils::message_util::{Content, IntentContent};

#[derive(Debug, Clone)]
pub(crate) struct GenericStep {
    pub(crate) waiting_for: PendingType,
    pub(crate) action: GenericAction,
}

pub(crate) trait StepExecutable: Send {
    fn execute(&self) -> Result<SmartSpeakerTaskResult>;
    fn feed(&mut self, content: Box<dyn Content>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn StepExecutable>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Debug for dyn StepExecutable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "StepExecutable")
    }
}

impl PartialEq for dyn StepExecutable {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn StepExecutable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum GenericAction {
    None,
    Explain(DescribeExecutable),
    WaitForConfirm,
    WaitForVision(Box<dyn StepExecutable>),
}

#[derive(Debug, Clone)]
pub(crate) struct DescribeExecutable {
    pub(crate) tts_script: String,
    pub(crate) current_content: Option<IntentContent>,
}

impl DescribeExecutable {
    pub(crate) fn new(text: String) -> Self {
        DescribeExecutable {
            tts_script: "".to_string(),
            current_content: None,
        }
    }
}

impl StepExecutable for DescribeExecutable {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            self.tts_script.clone(),
        ))
    }

    fn feed(&mut self, content: Box<dyn Content>) -> Result<()> {
        match content.as_any().downcast_ref::<IntentContent>() {
            None => {}
            Some(intent) => {
                self.current_content = Some(intent.clone());
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn StepExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}