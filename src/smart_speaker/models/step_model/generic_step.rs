use std::fmt::{self, Debug, Formatter};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

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
    fn eq(&self, _: &Self) -> bool {
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
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_contents: Option<IntentContent>,
}

impl DescribeExecutable {
    pub(crate) fn new(text: SmartSpeakerI18nText) -> Self {
        DescribeExecutable {
            tts_script: text,
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
                    SmartSpeakerI18nText::new()
                        .en("I can't see anything")
                        .ja("何も見えませんでした")
                        .zh("我什么都没看到")
                        .ko("아무것도 보이지 않습니다"),
                ))
            }
            Some(content) => {
                let count = content.entities.len();
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::Exit,
                    SmartSpeakerI18nText::new()
                        .en(&format!("I see {} {}", count, self.vision_action.to_string()))
                        .ja(&format!("{}個の{}が見えました", count, self.vision_action.to_string()))
                        .zh(&format!("我看到了{}个{}", count, self.vision_action.to_string()))
                        .ko(&format!("{}개의 {}가 보입니다", count, self.vision_action.to_string())),
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
