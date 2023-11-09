use anyhow::Result;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::step_model::generic_step::{CountVisionObjectExecutable, GenericAction, GenericStep};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
use crate::smart_speaker::models::message_model::*;

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
            SmartSpeakerI18nText::new()
                .en("Checking...")
                .ja("確認しています。")
                .zh("正在确认。")
                .ko("확인 중입니다."))
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
                            return self.cancel()
                        }
                        _ => {}
                    }
                }

                if let Some(vision_content) = content.as_any().downcast_ref::<VisionContent>() {
                    match &mut current_action {
                        GenericAction::WaitForVision(executable) => {
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
            SmartSpeakerI18nText::new()
                .en("task failed")
                .ja("タスクに失敗しました。")
                .zh("任务失败了。")
                .ko("작업에 실패했습니다."))
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
            SmartSpeakerI18nText::new()
                .en("task finished")
                .ja("タスクを完了しました。")
                .zh("任务完成了。")
                .ko("작업을 완료했습니다."))
        )
    }

    fn cancel(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Cancelled,
            SmartSpeakerI18nText::new()
                .en("task cancelled")
                .ja("タスクをキャンセルしました。")
                .zh("任务取消了。")
                .ko("작업이 취소되었습니다."))
        )
    }
}
