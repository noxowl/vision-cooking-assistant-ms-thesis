use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use anyhow::Result;
use crate::smart_speaker::models::context_model::{CookingTask, SmartSpeakerTaskResultCodes, Task, ViewingTask};
use crate::smart_speaker::models::core_model::{PendingType, SmartSpeakerState};
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::utils::message_util;
use crate::utils::message_util::{IntentContent, IntentFinalized, ProcessResult, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage, VisionContent, VisionFinalized};

pub(crate) struct ContextActor {
    alive: bool,
    vision: bool,
    current_context: Option<Box<dyn Task>>,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl ContextActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, vision: bool) -> Self {
        ContextActor {
            alive: true,
            vision,
            current_context: None,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("ContextActor started");
        while self.alive {
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message);
                },
                _ => {}
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
            SmartSpeakerMessage::VisionFinalized(VisionFinalized { send_from: _, send_to: _, result, content }) => {
                self.handle_vision(result, content);
            }
            _ => {
                dbg!("unhandled message");
            }
        }
    }

    fn start_new_context(&mut self, content: IntentContent) {
        match content.intent {
            IntentAction::WhatYouSee => {
                self.current_context = Some(Box::new(ViewingTask::new(content).unwrap()))
            }
            IntentAction::CookingTask => {
                self.current_context = Some(Box::new(CookingTask::new(content).unwrap()))
            }
            _ => {}
        }
        if let Some(ref mut context) = &mut self.current_context {
            let result = context.init().expect("TODO: panic message");
            match result {
                SmartSpeakerTaskResultCodes::Exit(_) => {}
                SmartSpeakerTaskResultCodes::Wait(wait_type) => {
                    self.request_state_update(SmartSpeakerState::Pending(wait_type))
                }
            }
        }
    }

    fn handle_intent(&mut self, result: ProcessResult, content: IntentContent) {
        match result {
            ProcessResult::Success => {
                match &mut self.current_context {
                    None => {
                        dbg!("no context. start new context");
                        self.start_new_context(content);
                    }
                    Some(ref mut context) => {
                        dbg!("context exists. proceed context");
                        match context.next(Some(Box::new(content))).unwrap() {
                            SmartSpeakerTaskResultCodes::Exit(_) => {
                                dbg!("context exit");
                                self.current_context = None;
                                self.request_state_update(SmartSpeakerState::Idle);
                            }
                            SmartSpeakerTaskResultCodes::Wait(pending_type) => {
                                dbg!("context wait");
                                self.request_state_update(SmartSpeakerState::Pending(pending_type));
                            }
                        }
                    }
                }
            },
            ProcessResult::Failure => {
                match &mut self.current_context {
                    None => {}
                    Some(ref mut context) => {
                        match context.failed(Some(Box::new(content))).unwrap() {
                            SmartSpeakerTaskResultCodes::Exit(_) => {
                                dbg!("context exit");
                                self.current_context = None;
                                self.request_state_update(SmartSpeakerState::Idle);
                            }
                            SmartSpeakerTaskResultCodes::Wait(pending_type) => {
                                dbg!("context wait");
                                self.request_state_update(SmartSpeakerState::Pending(pending_type));
                            }
                        }
                    }
                }

            },
        }
    }

    fn handle_vision(&mut self, result: ProcessResult, content: VisionContent) {
        match result {
            ProcessResult::Success => {
                match content {
                    VisionContent { .. } => {}
                }
            },
            ProcessResult::Failure => {
                self.current_context = None;
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
}
