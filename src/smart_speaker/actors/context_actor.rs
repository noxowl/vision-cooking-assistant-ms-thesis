use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use anyhow::Result;
use crate::smart_speaker::models::core_model::{PendingType, SmartSpeakerState};
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResultCodes, Task};
use crate::smart_speaker::models::task_model::cooking_task::CookingTask;
use crate::smart_speaker::models::task_model::viewing_task::ViewingTask;
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction, VisionObject, VisionSlot};
use crate::utils::message_util;
use crate::utils::message_util::{IntentContent, IntentFinalized, ProcessResult, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage, VisionContent, VisionFinalized};

pub(crate) struct ContextActor {
    alive: bool,
    vision: bool,
    current_task: Option<Box<dyn Task>>,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl ContextActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, vision: bool) -> Self {
        ContextActor {
            alive: true,
            vision,
            current_task: None,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("ContextActor started");
        while self.alive {
            match &mut self.current_task {
                None => {}
                Some(task) => {
                    match task.execute() {
                        Ok(result) => {
                            match result {
                                SmartSpeakerTaskResultCodes::Exit(_) => {}
                                SmartSpeakerTaskResultCodes::Wait(_) => {}
                                SmartSpeakerTaskResultCodes::TTS(text) => {
                                    self.request_text_to_speech(text);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
            let mut pending = true;
            while pending {
                if let Ok(message) = self.receiver.try_recv() {
                    self.handle_message(message);
                } else {
                    pending = false;
                }
            }
            thread::sleep(Duration::from_millis(33));
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        println!("ContextActor received message");
        match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::IntentFinalized(IntentFinalized { send_from: _, send_to: _, result, content }) => {
                self.handle_intent(result, content);
            },
            SmartSpeakerMessage::VisionFinalized(VisionFinalized { send_from: _, send_to: _, result, contents }) => {
                self.handle_vision(result, contents);
            }
            _ => {
                dbg!("unhandled message");
            }
        }
    }

    fn start_new_task(&mut self, content: IntentContent) {
        match content.intent {
            IntentAction::WhatYouSee => {
                self.current_task = Some(Box::new(ViewingTask::new(content).unwrap()))
            }
            IntentAction::CookingTask => {
                self.current_task = Some(Box::new(CookingTask::new(content).unwrap()))
            }
            _ => {}
        }
        if let Some(ref mut task) = &mut self.current_task {
            let result = task.init().expect("TODO: panic message");
            match result {
                SmartSpeakerTaskResultCodes::Wait(wait_type) => {
                    self.request_state_update(SmartSpeakerState::Pending(wait_type))
                }
                _ => {}
            }
        }
    }

    fn handle_intent(&mut self, result: ProcessResult, content: IntentContent) {
        match result {
            ProcessResult::Success => {
                match &mut self.current_task {
                    None => {
                        dbg!("no context. start new context");
                        self.start_new_task(content);
                    }
                    Some(task) => {
                        dbg!("context exists. proceed context");
                        match task.try_next(Some(Box::new(content))).unwrap() {
                            SmartSpeakerTaskResultCodes::Exit(_) => {
                                dbg!("context exit");
                                self.current_task = None;
                                self.request_state_update(SmartSpeakerState::Idle);
                            }
                            SmartSpeakerTaskResultCodes::Wait(pending_type) => {
                                dbg!("context wait");
                                match pending_type {
                                    PendingType::Speak => {
                                        self.request_state_update(SmartSpeakerState::Pending(pending_type));
                                    }
                                    PendingType::Vision(actions) => {
                                        self.request_vision_action(actions);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            },
            ProcessResult::Failure => {
                match &mut self.current_task {
                    None => {}
                    Some(ref mut task) => {
                        match task.failed(Some(Box::new(content))).unwrap() {
                            SmartSpeakerTaskResultCodes::Exit(_) => {
                                dbg!("context exit");
                                self.current_task = None;
                                self.request_state_update(SmartSpeakerState::Idle);
                            }
                            SmartSpeakerTaskResultCodes::Wait(pending_type) => {
                                dbg!("context wait");
                                self.request_state_update(SmartSpeakerState::Pending(pending_type));
                            }
                            _ => {}
                        }
                    }
                }

            },
        }
    }

    fn handle_vision(&mut self, result: ProcessResult, contents: Vec<VisionContent>) {
        match result {
            ProcessResult::Success => {
                for content in contents {
                    match &mut self.current_task {
                        None => {}
                        Some(task) => {
                            task.try_next(Some(Box::new(content))).unwrap();
                        }
                    }

                }
            },
            ProcessResult::Failure => {
                self.current_task = None;
            },
        }
    }

    fn request_state_update(&self, state: SmartSpeakerState) {
        message_util::state_update_message(
            &self.sender,
            SmartSpeakerActors::ContextActor,
            SmartSpeakerActors::CoreActor,
            state,
        )
    }

    fn request_text_to_speech(&self, text: String) {
        message_util::text_to_speech_message(
            &self.sender,
            SmartSpeakerActors::ContextActor,
            SmartSpeakerActors::MachineSpeechActor,
            text,
        )
    }

    fn request_vision_action(&self, actions: Vec<VisionAction>) {
        message_util::vision_action_message(
            &self.sender,
            SmartSpeakerActors::ContextActor,
            SmartSpeakerActors::VisionActor,
            actions,
        )
    }
}
