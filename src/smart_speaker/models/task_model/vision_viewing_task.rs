use anyhow::Result;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::step_model::generic_step::{CountVisionObjectExecutable, GenericAction, GenericStep};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

pub(crate) struct VisionViewingTask {
    pub(crate) step: Vec<GenericStep>,
    pub(crate) current_step: usize,
}

impl VisionViewingTask {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            step: vec![
                GenericStep::new(GenericAction::WaitForVision(Box::new(CountVisionObjectExecutable::new()))),
            ],
            current_step: 0,
        })
    }
}

impl Task for VisionViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Wait(
                self.step[*&self.current_step].waiting_for.clone()),
            "確認しています。".to_string())
        )
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action: &GenericAction = &self.step[*&self.current_step].action;
        match content {
            None => {}
            Some(content) => {
                if let Some(intent_content) = content.as_any().downcast_ref::<IntentContent>() {
                    match intent_content.intent {
                        IntentAction::Cancel => {
                            return self.exit()
                        }
                        _ => {}
                    }
                }

                if let Some(vision_content) = content.as_any().downcast_ref::<VisionContent>() {
                    match &mut current_action {
                        GenericAction::WaitForVision( executable) => {
                            let mut exe = executable.clone();
                            exe.feed(Box::new(vision_content.clone()))?;
                            let result = exe.execute();
                            return result
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(SmartSpeakerTaskResult::new(
            SmartSpeakerTaskResultCode::Wait(
                self.step[*&self.current_step].waiting_for.clone()))
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

    fn cancel(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Cancelled,
            "cooking task cancelled".to_string(),
        ))
    }
}
