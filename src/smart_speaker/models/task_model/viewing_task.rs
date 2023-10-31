use anyhow::Result;
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::task_model::cooking_task::StepAction;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResultCodes, Task};
use crate::smart_speaker::models::vision_model::{VisionAction, VisionObject};
use crate::utils::message_util::{Content, IntentContent, VisionContent};

pub(crate) struct ViewingTask {
    pub(crate) step: Vec<StepAction>,
    pub(crate) current_step: i16,
    pub(crate) waiting_content: PendingType
}

impl ViewingTask {
    pub(crate) fn new(content: IntentContent) -> Result<Self> {
        todo!()
    }
}

impl Task for ViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
    }

    fn execute(&mut self) -> Result<SmartSpeakerTaskResultCodes> {
        todo!()
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        match content {
            None => {}
            Some(content) => {
                match content.as_any().downcast_ref::<IntentContent>() {
                    None => {}
                    Some(intent) => {
                        match intent.intent {
                            IntentAction::Cancel => {
                                return self.exit()
                            }
                            _ => {}
                        }
                    }
                }

                match content.as_any().downcast_ref::<VisionContent>() {
                    None => {}
                    Some(vision) => {
                        match &vision.action {
                            VisionAction::None => {}
                            VisionAction::ObjectDetectionWithAruco(detectable) => {
                                for content in &vision.entities {
                                    match content.as_any().downcast_ref::<VisionObject>() {
                                        None => {}
                                        Some(vision_object) => {
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

            }
        }
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task next".to_string()))
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task failed".to_string()))
    }

    fn internal_move_next(&mut self) -> Result<bool> {
        todo!()
    }

    fn internal_rollback(&mut self) -> Result<bool> {
        todo!()
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task exit".to_string()))
    }
}
