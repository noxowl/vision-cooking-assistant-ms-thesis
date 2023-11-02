use std::fmt;
use std::fmt::{Debug, Formatter};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction};
use crate::utils::message_util::{Content, IntentContent, VisionContent};

#[derive(Debug, Clone)]
pub(crate) struct GenericStep {
    pub(crate) waiting_for: PendingType,
    pub(crate) action: GenericAction,
}

impl GenericStep {
    pub(crate) fn new(action: GenericAction) -> Self {
        let pending_type = match &action {
            GenericAction::None => PendingType::Speak,
            GenericAction::Explain(_) => PendingType::Speak,
            GenericAction::WaitForConfirm => PendingType::Speak,
            GenericAction::WaitForVision(executable) => PendingType::Vision(executable.try_expose_vision_actions().unwrap()),
        };
        GenericStep {
            waiting_for: pending_type,
            action
        }
    }
}

pub(crate) trait StepExecutable: Send {
    fn execute(&self) -> Result<SmartSpeakerTaskResult>;
    fn feed(&mut self, content: Box<dyn Content>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn StepExecutable>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>>;
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
    pub(crate) current_contents: Option<IntentContent>,
}

impl DescribeExecutable {
    pub(crate) fn new(text: String) -> Self {
        DescribeExecutable {
            tts_script: "".to_string(),
            current_contents: None,
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
                self.current_contents = Some(intent.clone());
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

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Err(anyhow!("Not a vision action"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CountVisionObjectExecutable {
    pub(crate) vision_action: VisionAction,
    pub(crate) current_content: Option<VisionContent>,
}

impl CountVisionObjectExecutable {
    pub(crate) fn new() -> Self {
        CountVisionObjectExecutable {
            vision_action: VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
            current_content: None,
        }
    }
}

impl StepExecutable for CountVisionObjectExecutable {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::Exit,
                    "I don't see anything".to_string(),
                ))
            }
            Some(content) => {
                let count = content.entities.len();
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::Exit,
                    format!("I see {} {}", count, self.vision_action.to_string()),
                ))
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>) -> Result<()> {
        match content.as_any().downcast_ref::<VisionContent>() {
            None => {}
            Some(vision) => {
                self.current_content = Some(vision.clone());
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

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Ok(vec![self.vision_action.clone()])
    }
}
