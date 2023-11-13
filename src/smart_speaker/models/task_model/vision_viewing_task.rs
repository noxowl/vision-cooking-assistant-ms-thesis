use anyhow::Result;
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType, CountVisionObjectAction};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::speak_model::MachineSpeechBoilerplate;

pub(crate) struct VisionViewingTask {
    pub(crate) step: Vec<Box<dyn ActionExecutable>>,
    pub(crate) current_step: usize,
}

impl VisionViewingTask {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            step: vec![
                Box::new(CountVisionObjectAction::new()),
            ],
            current_step: 0,
        })
    }
}

impl Task for VisionViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::TaskSuccess(
                self.step[*&self.current_step + 1].get_action_trigger_type().to_waiting_interaction()),
            SmartSpeakerI18nText::new()
                .en("Checking...")
                .ja("確認しています。")
                .zh("正在确认。")
                .ko("확인 중입니다."))
        )
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action = self.step[*&self.current_step].clone();
        match content {
            None => {
                match current_action.get_action_trigger_type(){
                    ActionTriggerType::Vision(_) => {
                        return Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::TaskFailed(
                                current_action.get_action_trigger_type().to_waiting_interaction()),
                            MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                        ))
                    }
                    _ => {
                        return Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::TaskFailed(
                                current_action.get_action_trigger_type().to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                }
            }
            Some(content) => {
                current_action.feed(content, None)?;
                let result = current_action.execute();
                return result
            }
        }
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
