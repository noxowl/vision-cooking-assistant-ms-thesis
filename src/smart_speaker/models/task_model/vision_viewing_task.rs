use anyhow::Result;
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::step_model::generic_step::{GenericAction, GenericStep};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
use crate::smart_speaker::models::vision_model::{VisionAction, VisionObject};
use crate::utils::message_util::{Content, IntentContent, VisionContent};

pub(crate) struct VisionViewingTask {
    pub(crate) step: Vec<GenericStep>,
    pub(crate) current_step: i16,
    pub(crate) waiting_content: PendingType
}

impl VisionViewingTask {
    pub(crate) fn new(content: IntentContent) -> Result<Self> {
        todo!()
    }
}

impl Task for VisionViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::new(SmartSpeakerTaskResultCode::Wait(PendingType::Speak)))
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
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
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            "viewing task next".to_string())
        )
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            "viewing task failed".to_string())
        )
    }

    fn internal_move_next(&mut self) -> Result<bool> {
        todo!()
    }

    fn internal_rollback(&mut self) -> Result<bool> {
        todo!()
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            "viewing task exit".to_string())
        )
    }
}
