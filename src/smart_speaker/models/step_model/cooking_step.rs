use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::step_model::generic_step::StepExecutable;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{VisionAction, VisionObject};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

#[derive(Debug, Clone)]
pub(crate) struct CookingStep {
    pub(crate) waiting_for: PendingType,
    pub(crate) action: CookingAction,
}

impl CookingStep {
    pub(crate) fn new(action: CookingAction) -> Self {
        CookingStep {
            waiting_for: PendingType::Speak,
            action
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CookingAction {
    None,
    Explain(ExplainStepExecutable),
    WaitForConfirm,
    WaitForVision(Box<dyn StepExecutable>),
}

#[derive(Debug, Clone)]
pub(crate) struct ExplainStepExecutable {
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_content: Option<IntentContent>,
}

impl ExplainStepExecutable {
    pub(crate) fn new(text: SmartSpeakerI18nText) -> Self {
        ExplainStepExecutable {
            tts_script: text,
            current_content: None,
        }
    }
}

impl StepExecutable for ExplainStepExecutable {
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

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Err(anyhow!("Not a vision action"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WaitForVisionStepExecutables {
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) vision_action: VisionAction,
    pub(crate) current_content: Option<VisionContent>,
}

impl WaitForVisionStepExecutables {
    pub(crate) fn new(text: SmartSpeakerI18nText, vision_action: VisionAction) -> Self {
        WaitForVisionStepExecutables {
            tts_script: text,
            vision_action,
            current_content: None,
        }
    }

    pub(crate) fn handle_vision_contents(&self, contents: &Vec<VisionObject>) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Wait(PendingType::Vision(vec![])),
            self.tts_script.clone(),
        ))
    }
}

impl StepExecutable for WaitForVisionStepExecutables {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        return match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::Wait(PendingType::Vision(vec![])),
                    self.tts_script.clone(),
                ))
            }
            Some(content) => {
                match &content.action {
                    VisionAction::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::Wait(PendingType::Vision(vec![])),
                            self.tts_script.clone(),
                        ))
                    }
                    VisionAction::ObjectDetectionWithAruco(detectable) => {
                        self.handle_vision_contents(
                            &content.entities.iter().map(|c| c.as_any().downcast_ref::<VisionObject>().unwrap().clone()).collect::<Vec<VisionObject>>())
                    }
                }
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