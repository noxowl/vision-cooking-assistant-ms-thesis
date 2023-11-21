use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType, CountVisionObjectAction, GenericAction};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, SmartSpeakerTaskType, Task};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::speak_model::MachineSpeechBoilerplate;

pub(crate) struct VisionViewingTask {
    pub(crate) step: Vec<Box<dyn ActionExecutable>>,
    pub(crate) current_step: usize,
    pub(crate) previous_success_result: Option<SmartSpeakerTaskResult>,
}

impl VisionViewingTask {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            step: vec![
                Box::new(GenericAction::new(SmartSpeakerI18nText::new()
                    .en("Checking...")
                    .ja("確認しています。")
                    .zh("正在确认。")
                    .ko("확인 중입니다."))),
                Box::new(CountVisionObjectAction::new()),
            ],
            current_step: 0,
            previous_success_result: None,
        })
    }
}

impl Task for VisionViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        self.try_next(Some(Box::new(IntentContent::new(IntentAction::Next, vec![])))
        )
    }

    fn next_index(&self) -> Option<usize> {
        todo!()
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action = self.step[self.current_step].clone();
        match content {
            None => {
                let trigger = current_action.get_action_trigger_type();
                return match trigger {
                    ActionTriggerType::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Confirm => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Vision(_) => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                        ))
                    }
                }
            }
            Some(content) => {
                let _ = current_action.feed(content, None);
                let result = current_action.execute();
                match result {
                    Ok(r) => {
                        return self.handle_result(r)
                    }
                    Err(_) => {
                        return self.failed(None)
                    }
                }
            }
        }
    }

    fn handle_result(&mut self, result: SmartSpeakerTaskResult) -> Result<SmartSpeakerTaskResult> {
        match result.code {
            SmartSpeakerTaskResultCode::StepSuccess => {
                if let Ok(move_next_success) = self.internal_move_next() {
                    if move_next_success {
                        // replace with next action
                        let next_action = self.step[self.current_step].clone();
                        let mut updated_result = result.clone();
                        updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(next_action.get_action_trigger_type().to_waiting_interaction());
                        self.previous_success_result = Some(updated_result.clone());
                        return Ok(updated_result)
                    } else {
                        let mut updated_result = result.clone();
                        updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(WaitingInteraction::Exit);
                        return Ok(updated_result)
                    }
                }
                return self.exit()
            }
            SmartSpeakerTaskResultCode::StepFailed => {
                return Ok(result)
            }
            SmartSpeakerTaskResultCode::RepeatPrevious => {
                if let Some(previous) = self.previous_success_result.clone() {
                    return Ok(previous)
                }
                Err(anyhow!("failed to repeat previous action"))
            }
            SmartSpeakerTaskResultCode::Cancelled => {
                return self.cancel()
            }
            SmartSpeakerTaskResultCode::Exit => {
                return self.exit()
            }
            _ => {
                Err(anyhow!("task execution failed"))
            }
        }
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskType::NonVision,
            SmartSpeakerTaskResultCode::Exit,
            SmartSpeakerI18nText::new()
                .en("task failed")
                .ja("タスクに失敗しました。")
                .zh("任务失败了。")
                .ko("작업에 실패했습니다."))
        )
    }

    fn internal_move_next(&mut self) -> Result<bool> {
        if self.current_step < self.step.len() - 1 {
            self.current_step += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn internal_rollback(&mut self) -> Result<bool> {
        todo!()
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskType::NonVision,
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
            SmartSpeakerTaskType::NonVision,
            SmartSpeakerTaskResultCode::Cancelled,
            SmartSpeakerI18nText::new()
                .en("task cancelled")
                .ja("タスクをキャンセルしました。")
                .zh("任务取消了。")
                .ko("작업이 취소되었습니다."))
        )
    }
}
