use std::fmt::{self, Debug, Formatter};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::Revision;

#[derive(Debug, Clone)]
pub(crate) enum ActionType {
    None,
    Confirm,
    Vision(Vec<VisionAction>),
}

pub(crate) trait ActionExecutable: Send {
    fn execute(&self) -> Result<SmartSpeakerTaskResult>;
    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn ActionExecutable>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn get_action_type(&self) -> ActionType;
    fn get_pending_type(&self) -> PendingType {
        match self.get_action_type() {
            ActionType::None => PendingType::Speak,
            ActionType::Confirm => PendingType::Speak,
            ActionType::Vision(actions) => PendingType::Vision(actions.clone()),
        }
    }
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
}

impl GenericAction {
    pub(crate) fn new(text: SmartSpeakerI18nText) -> Self {
        GenericAction {
            tts_script: text,
            current_contents: None,
            current_revisions: None,
        }
    }
}

impl ActionExecutable for GenericAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            self.tts_script.clone(),
        ))
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
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

    fn get_action_type(&self) -> ActionType {
        ActionType::None
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
}

impl CountVisionObjectAction {
    pub(crate) fn new() -> Self {
        CountVisionObjectAction {
            vision_action: VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
            current_content: None,
        }
    }
}

impl ActionExecutable for CountVisionObjectAction {
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

    fn get_action_type(&self) -> ActionType {
        ActionType::Vision(vec![self.vision_action.clone()])
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
