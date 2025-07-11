// use anyhow::{anyhow, Result};
// use crate::smart_speaker::models::core_model::PendingType;
// use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu};
// use crate::smart_speaker::models::step_model::cooking_step::CookingStep;
// use crate::smart_speaker::models::step_model::generic_step::ActionExecutable;
// use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, Task};
// use crate::smart_speaker::models::task_model::cooking_task::CookingTaskIngredient;
// use crate::smart_speaker::models::message_model::*;
// use crate::utils::message_util::*;
//
// pub(crate) struct VisionCookingTask {
//     pub(crate) menu: IntentCookingMenu,
//     pub(crate) step: Vec<CookingStep>,
//     pub(crate) current_step: usize,
//     pub(crate) ingredients: Vec<CookingTaskIngredient>,
//     pub(crate) waiting_content: PendingType,
//     pub(crate) checkpoint: usize,
// }
//
// impl VisionCookingTask {
//     pub(crate) fn new(content: IntentContent) -> Result<Self> {
//         match content.entities.get(0) {
//             None => { Err(anyhow!("failed")) }
//             Some(entity) => {
//                 Ok(VisionCookingTask {
//                     menu: entity.as_any().downcast_ref::<IntentCookingMenu>().unwrap().clone(),
//                     step: vec![],
//                     current_step: 0,
//                     ingredients: vec![],
//                     waiting_content: PendingType::Speak,
//                     checkpoint: 0,
//                 })
//             }
//         }
//     }
//
//     fn update(&mut self) -> Result<()> {
//         Ok(())
//     }
// }
//
// impl Task for VisionCookingTask {
//     fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
//         dbg!("cooking task init");
//         Ok(SmartSpeakerTaskResult::new(SmartSpeakerTaskResultCode::Wait(PendingType::Speak)))
//     }
//
//     fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
//         // match content {
//         //     None => {
//         //         Ok(SmartSpeakerTaskResult::new(
//         //             SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone()))
//         //         )
//         //     }
//         //     Some(c) => {
//         //         let _ = match c.as_any().downcast_ref::<IntentContent>() {
//         //             None => {}
//         //             Some(intent) => {
//         //                 match intent.intent {
//         //                     IntentAction::Cancel => {
//         //                         let _ = self.exit();
//         //                         return self.cancel()
//         //                     }
//         //                     _ => {
//         //                     }
//         //                 }
//         //             }
//         //         };
//         //         let step = self.step.get_mut(self.current_step);
//         //         return match step {
//         //             None => {
//         //                 self.exit()
//         //             }
//         //             Some(step) => {
//         //                 match &mut step.action {
//         //                     CookingAction::None => {
//         //                         Ok(SmartSpeakerTaskResult::new(
//         //                             SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//         //                     }
//         //                     CookingAction::Explain(explain) => {
//         //                         Ok(explain.execute()?)
//         //                     }
//         //                     CookingAction::WaitForConfirm => {
//         //                         match c.as_any().downcast_ref::<IntentContent>() {
//         //                             None => {
//         //                                 Ok(SmartSpeakerTaskResult::new(
//         //                                     SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//         //                             }
//         //                             Some(intent) => {
//         //                                 match intent.intent {
//         //                                     IntentAction::Cancel => {
//         //                                         let _ = self.exit();
//         //                                         self.cancel()
//         //                                     }
//         //                                     IntentAction::Confirm => {
//         //                                         match self.internal_move_next() {
//         //                                             Ok(result) => {
//         //                                                 if result {
//         //                                                     Ok(SmartSpeakerTaskResult::new(
//         //                                                         SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//         //                                                 } else {
//         //                                                     let _ = self.exit();
//         //                                                     self.exit()
//         //                                                 }
//         //                                             }
//         //                                             Err(_) => {
//         //                                                 Err(anyhow!("failed to move next"))
//         //                                             }
//         //                                         }
//         //                                     }
//         //                                     _ => {
//         //                                         Ok(SmartSpeakerTaskResult::new(SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//         //                                     }
//         //                                 }
//         //                             }
//         //                         }
//         //                     }
//         //                     CookingAction::WaitForVision(ref mut vision) => {
//         //                         match c.as_any().downcast_ref::<VisionContent>() {
//         //                             None => {
//         //                                 Ok(SmartSpeakerTaskResult::new(
//         //                                     SmartSpeakerTaskResultCode::Wait(PendingType::Vision(vec![])))
//         //                                 )
//         //                             }
//         //                             Some(content) => {
//         //                                 let _ = vision.feed(Box::new(content.clone()));
//         //                                 Ok(vision.execute()?)
//         //                             }
//         //                         }
//         //                     }
//         //                 }
//         //             }
//         //         };
//         //     }
//         // }
//         Ok(SmartSpeakerTaskResult::new(
//             SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//     }
//
//     fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
//         Ok(SmartSpeakerTaskResult::new(
//             SmartSpeakerTaskResultCode::Wait(self.waiting_content.clone())))
//     }
//
//     fn internal_move_next(&mut self) -> Result<bool> {
//         if self.current_step < self.step.len() {
//             self.current_step += 1;
//             self.waiting_content = self.step.get(self.current_step).unwrap().waiting_for.clone();
//             Ok(true)
//         } else {
//             Ok(false)
//         }
//     }
//
//     fn internal_rollback(&mut self) -> Result<bool> {
//         self.current_step = self.checkpoint;
//         Ok(true)
//     }
//
//     fn exit(&self) -> Result<SmartSpeakerTaskResult> {
//         Ok(SmartSpeakerTaskResult::with_tts(
//             SmartSpeakerTaskResultCode::Exit,
//             SmartSpeakerI18nText::new()
//                 .en("cooking task exit")
//                 .ja("料理タスクを終了します。")
//                 .zh("退出烹饪任务。")
//                 .ko("요리 작업을 종료합니다."),
//         ))
//     }
//
//     fn cancel(&self) -> Result<SmartSpeakerTaskResult> {
//         Ok(SmartSpeakerTaskResult::with_tts(
//             SmartSpeakerTaskResultCode::Cancelled,
//             SmartSpeakerI18nText::new()
//                 .en("cooking task cancelled")
//                 .ja("料理タスクをキャンセルします。")
//                 .zh("取消烹饪任务。")
//                 .ko("요리 작업을 취소합니다."),
//         ))
//     }
// }
//
//
// // pub(crate) struct Recipe {
// //     name: String,
// //     current_step: usize,
// //     steps: Vec<Step>,
// // }
// //
// // impl Recipe {
// //     pub(crate) fn new(name: String) -> Self {
// //         Recipe {
// //             name: name.clone(),
// //             current_step: 0,
// //             steps: Recipe::load_steps(name.clone()).unwrap(),
// //         }
// //     }
// //
// //     pub(crate) fn load_steps(name: String) -> Result<Vec<Step>> {
// //         todo!()
// //     }
// //
// //     pub(crate) fn current_step(&self) -> &Step {
// //         self.steps.get(self.current_step).unwrap()
// //     }
// //
// //     pub(crate) fn next_step(&mut self) -> Result<()> {
// //         self.current_step += 1;
// //         Ok(())
// //     }
// // }
// //
// // pub(crate) struct Step {
// //     name: String,
// // }
// //
// // impl Step {
// //     pub(crate) fn new() -> Self {
// //         Step {
// //             name: "".to_string(),
// //         }
// //     }
// // }
// //
// // pub(crate) trait Action {
// //     fn execute(&self) -> Result<SmartSpeakerActionResultCodes>;
// // }
// //
// // pub(crate) enum SmartSpeakerActionResultCodes {
// //     Success,
// //     Failure,
// //     Cancelled,
// // }
