use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu};
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::speak_model::MachineSpeechBoilerplate;
use crate::smart_speaker::models::step_model::cooking_step::CookingStepBuilder;

#[derive(Debug, Clone)]
pub(crate) struct CookingIngredient {
    pub(crate) name: CookingIngredientName,
    pub(crate) unit: CookingIngredientAmount,
}

impl CookingIngredient {
    pub(crate) fn new(name: CookingIngredientName, unit: CookingIngredientAmount) -> Self {
        CookingIngredient {
            name,
            unit,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CookingIngredientName {
    Salt,
    Pepper,
    Sugar,
    SoySauce,
    Sesame,
    SesameOil,
    Miso,
    Sake,
    Mirin,
    Carrot,
    Onion,
}

#[derive(Debug, Clone)]
pub(crate) enum CookingIngredientAmount {
    MilliGram(i32),
    MilliLiter(i32),
    Piece(i32),
}

pub(crate) struct CookingTask {
    pub(crate) menu: IntentCookingMenu,
    pub(crate) step: Vec<Box<dyn ActionExecutable>>,
    pub(crate) current_step: usize,
    pub(crate) last_revision: Option<Box<dyn Revision>>,
    pub(crate) checkpoint: usize,
}

impl CookingTask {
    pub(crate) fn new(content: IntentContent, vision: bool) -> Result<Self> {
        match content.entities.get(0) {
            None => { Err(anyhow!("failed")) }
            Some(entity) => {
                let menu = entity.as_any().downcast_ref::<IntentCookingMenu>().unwrap().clone();
                Ok(CookingTask {
                    menu,
                    step: CookingStepBuilder::new(vision).build(menu),
                    current_step: 0,
                    last_revision: None,
                    checkpoint: 0,
                })
            }
        }
    }
}

impl Task for CookingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        self.try_next(Some(Box::new(IntentContent::new(IntentAction::Next, vec![]))))
    }

    fn next_index(&self) -> Option<usize> {
        if self.current_step < self.step.len() - 1 {
            Some(self.current_step + 1)
        } else {
            None
        }
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action = self.step[self.current_step].clone();
        match content {
            None => {
                let trigger = current_action.get_action_trigger_type();
                return match trigger {
                    ActionTriggerType::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Confirm => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Vision(_) => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                        ))
                    }
                }
            }
            Some(content) => {
                if let Some(revision) = &self.last_revision {
                    let _ = current_action.feed(content, Some(revision.clone_box()));
                } else {
                    let _ = current_action.feed(content, None);
                }
                let result = current_action.execute();
                match result {
                    Ok(r) => {
                        return self.handle_result(r)
                    }
                    Err(_) => {
                        return self.failed(None)
                    }
                }

                // let next = self.next_index();
                // match next {
                //     None => {
                //         if self.current_step == self.step.len() {
                //             if let Some(revision) = &self.last_revision {
                //                 let _ = current_action.feed(content, Some(revision.clone_box()));
                //             } else {
                //                 let _ = current_action.feed(content, None);
                //             }
                //             let result = current_action.execute();
                //         } else {
                //             return self.exit()
                //         }
                //     }
                //     Some(i) => {
                //
                //     }
                // }
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
                        return Ok(updated_result)
                    } else {
                        let mut updated_result = result.clone();
                        updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(WaitingInteraction::None);
                        return Ok(updated_result)
                    }
                }
                return self.exit()
            }
            SmartSpeakerTaskResultCode::StepFailed => {
                return Ok(result)
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
        let mut current_action = self.step[self.current_step].clone();
        match current_action.get_action_trigger_type() {
            ActionTriggerType::Vision(_) => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::TaskFailed(
                        current_action.get_action_trigger_type().to_waiting_interaction()),
                    MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                ))
            }
            _ => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::TaskFailed(
                        current_action.get_action_trigger_type().to_waiting_interaction()),
                    MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                ))
            }
        }
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
        self.current_step = self.checkpoint;
        Ok(true)
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Exit,
            SmartSpeakerI18nText::new()
                .en("cooking task exit")
                .ja("料理タスクを終了します。")
                .zh("退出烹饪任务。")
                .ko("요리 작업을 종료합니다."),
        ))
    }

    fn cancel(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::Cancelled,
            SmartSpeakerI18nText::new()
                .en("cooking task cancelled")
                .ja("料理タスクをキャンセルします。")
                .zh("取消烹饪任务。")
                .ko("요리 작업을 취소합니다."),
        ))
    }
}
