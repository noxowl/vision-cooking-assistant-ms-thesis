use std::fmt::{self, Debug, Formatter};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, SmartSpeakerTaskType};
use crate::smart_speaker::models::vision_model::{DetectableObject, DetectionDetail, DetectionMode, VisionAction};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::Revision;

#[derive(Debug, Clone)]
pub(crate) enum ActionTriggerType {
    None,
    Confirm,
    Vision(Vec<VisionAction>),
}

impl ActionTriggerType {
    pub(crate) fn to_waiting_interaction(&self) -> WaitingInteraction {
        match self {
            ActionTriggerType::None => WaitingInteraction::Skip,
            ActionTriggerType::Confirm => WaitingInteraction::Speak,
            ActionTriggerType::Vision(actions) => WaitingInteraction::Vision(actions.clone()),
        }
    }

    pub(crate) fn to_task_type(&self) -> SmartSpeakerTaskType {
        match self {
            ActionTriggerType::None => SmartSpeakerTaskType::NonVision,
            ActionTriggerType::Confirm => SmartSpeakerTaskType::NonVision,
            ActionTriggerType::Vision(_) => SmartSpeakerTaskType::Vision,
        }
    }
}

pub(crate) trait ActionExecutable: Send {
    fn execute(&self) -> Result<SmartSpeakerTaskResult>;
    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn ActionExecutable>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn get_action_trigger_type(&self) -> ActionTriggerType;

    fn has_cancelled(&self) -> bool {
        self.get_cancelled()
    }

    fn check_cancelled(&mut self, content: &Box<dyn Content>) -> Result<()> {
        if let Some(content) = content.as_any().downcast_ref::<IntentContent>() {
            match content.intent {
                IntentAction::Cancel => {
                    self.set_cancelled();
                }
                _ => {
                }
            }
        } else {
        }
        Ok(())
    }

    fn get_cancelled(&self) -> bool;
    fn set_cancelled(&mut self) -> Result<()>;

    fn has_request_repeat(&self) -> bool {
        self.get_repeated()
    }

    fn check_request_repeat(&mut self, content: &Box<dyn Content>) -> Result<()> {
        if let Some(content) = content.as_any().downcast_ref::<IntentContent>() {
            match content.intent {
                IntentAction::Repeat => {
                    self.set_repeated();
                }
                _ => {
                }
            }
        } else {
        }
        Ok(())
    }

    fn get_repeated(&self) -> bool;

    fn set_repeated(&mut self) -> Result<()>;

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText>;
    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>>;
}

impl Debug for dyn ActionExecutable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ActionExecutable")
    }
}

impl PartialEq for dyn ActionExecutable {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn ActionExecutable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GenericAction {
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_contents: Option<IntentContent>,
    pub(crate) current_revisions: Option<Box<dyn Revision>>,
    cancelled: bool,
    repeat_requested: bool,
}

impl GenericAction {
    pub(crate) fn new(text: SmartSpeakerI18nText) -> Self {
        GenericAction {
            tts_script: text,
            current_contents: None,
            current_revisions: None,
            cancelled: false,
            repeat_requested: false,
        }
    }
}

impl ActionExecutable for GenericAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::Cancelled));
        }
        if self.has_request_repeat() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::RepeatPrevious));
        }
        Ok(SmartSpeakerTaskResult::with_tts(
            self.get_action_trigger_type().to_task_type(),
            SmartSpeakerTaskResultCode::StepSuccess,
            self.tts_script.clone(),
        ))
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
        self.check_cancelled(&content)?;
        match content.as_any().downcast_ref::<IntentContent>() {
            None => {}
            Some(intent) => {
                self.current_contents = Some(intent.clone());
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ActionExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_action_trigger_type(&self) -> ActionTriggerType {
        ActionTriggerType::None
    }

    fn get_cancelled(&self) -> bool {
        self.cancelled
    }

    fn set_cancelled(&mut self) -> Result<()> {
        self.cancelled = true;
        Ok(())
    }

    fn get_repeated(&self) -> bool {
        self.repeat_requested
    }

    fn set_repeated(&mut self) -> Result<()> {
        self.repeat_requested = true;
        Ok(())
    }

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText> {
        Ok(self.tts_script.clone())
    }

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Err(anyhow!("Not a vision action"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CountVisionObjectAction {
    pub(crate) vision_action: VisionAction,
    pub(crate) current_content: Option<VisionContent>,
    cancelled: bool,
    repeat_requested: bool,
}

impl CountVisionObjectAction {
    pub(crate) fn new() -> Self {
        CountVisionObjectAction {
            vision_action: VisionAction::ObjectDetection(DetectionDetail::new(
                DetectionMode::Aruco,
                DetectableObject::Carrot,
                false,
            )),
            current_content: None,
            cancelled: false,
            repeat_requested: false,
        }
    }
}

impl ActionExecutable for CountVisionObjectAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    self.get_action_trigger_type().to_task_type(),
                    SmartSpeakerTaskResultCode::StepSuccess,
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
                    self.get_action_trigger_type().to_task_type(),
                    SmartSpeakerTaskResultCode::StepSuccess,
                    SmartSpeakerI18nText::new()
                        .en(&format!("I see {} {}", count, self.vision_action.to_i18n().en))
                        .ja(&format!("{}個の{}が見えました", count, self.vision_action.to_i18n().ja))
                        .zh(&format!("我看到了{}个{}", count, self.vision_action.to_i18n().zh))
                        .ko(&format!("{}개의 {}가 보입니다", count, self.vision_action.to_i18n().ko)),
                ))
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
        match content.as_any().downcast_ref::<VisionContent>() {
            None => {}
            Some(vision) => {
                self.current_content = Some(vision.clone());
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ActionExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_action_trigger_type(&self) -> ActionTriggerType {
        ActionTriggerType::Vision(vec![self.vision_action.clone()])
    }

    fn get_cancelled(&self) -> bool {
        self.cancelled
    }

    fn set_cancelled(&mut self) -> Result<()> {
        self.cancelled = true;
        Ok(())
    }

    fn get_repeated(&self) -> bool {
        self.repeat_requested
    }

    fn set_repeated(&mut self) -> Result<()> {
        self.repeat_requested = true;
        Ok(())
    }

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText> {
        Ok(SmartSpeakerI18nText::new()
            .en("I can't see anything")
            .ja("何も見えませんでした")
            .zh("我什么都没看到")
            .ko("아무것도 보이지 않습니다"))
    }

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Ok(vec![self.vision_action.clone()])
    }
}
