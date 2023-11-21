use std::ptr::write;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::core_model::{WaitingInteraction, SmartSpeakerState};
use crate::smart_speaker::models::intent_model::IntentAction;
use crate::smart_speaker::models::speak_model::MachineSpeechBoilerplate;
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, Task, cooking_task::CookingTask, vision_viewing_task::VisionViewingTask, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

pub(crate) struct ContextActor {
    alive: bool,
    vision: bool,
    current_task: Option<Box<dyn Task>>,
    next_state: Option<SmartSpeakerState>,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl ContextActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, vision: bool) -> Self {
        ContextActor {
            alive: true,
            vision,
            current_task: None,
            next_state: None,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Info("ContextActor started".to_string()));
        while self.alive {
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
        match message {
            SmartSpeakerMessage::RequestShutdown(ShutdownMessage {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::IntentFinalized(IntentFinalizedMessage { send_from: _, send_to: _, result, content }) => {
                self.handle_intent(result, content);
            },
            SmartSpeakerMessage::VisionFinalized(VisionFinalizedMessage { send_from: _, send_to: _, result, contents }) => {
                self.handle_vision(result, contents);
            }
            SmartSpeakerMessage::TextToSpeechFinished(StringMessage { send_from: _, send_to: _, message: _ }) => {
                self.handle_next_state();
            }
            _ => {
                write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Error("unhandled message".to_string()));
            }
        }
    }

    fn start_new_task(&mut self, content: IntentContent) {
        match content.intent {
            IntentAction::WhatYouSee => {
                if self.vision {
                    write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Info("start vision viewing task".to_string()));
                    self.current_task = Some(Box::new(VisionViewingTask::new().unwrap()))
                } else {
                    self.request_text_to_speech_boilerplate(MachineSpeechBoilerplate::Undefined as usize);
                    self.request_state_update(SmartSpeakerState::Idle);
                }
            }
            IntentAction::CookingTask => {
                write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Info("start cooking task".to_string()));
                self.current_task = Some(Box::new(CookingTask::new(content, self.vision).unwrap()))
            }
            _ => {
                self.request_text_to_speech_boilerplate(MachineSpeechBoilerplate::Undefined as usize);
                self.request_state_update(SmartSpeakerState::Idle);
            }
        }
        if let Some(ref mut task) = &mut self.current_task {
            let result = task.init().expect("TODO: panic message");
            self.handle_task_result(result);
        }
    }

    fn handle_intent(&mut self, result: ProcessResult, content: IntentContent) {
        match result {
            ProcessResult::Success => {
                match &mut self.current_task {
                    None => {
                        write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Debug("no context. try start new context".to_string()));
                        self.start_new_task(content);
                    }
                    Some(task) => {
                        write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Debug("context exists. proceed context".to_string()));
                        let result = task.try_next(Some(Box::new(content))).unwrap();
                        self.handle_task_result(result);
                    }
                }
            },
            ProcessResult::Failure => {
                write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Debug("intent failed".to_string()));
                match &mut self.current_task {
                    None => {
                        self.request_text_to_speech_boilerplate(MachineSpeechBoilerplate::IntentFailed as usize);
                        self.request_state_update(SmartSpeakerState::Idle);
                    }
                    Some(ref mut task) => {
                        let result = task.failed(Some(Box::new(content))).unwrap();
                        self.handle_task_result(result);
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
                            let result = task.try_next(Some(Box::new(content))).unwrap();
                            self.handle_task_result(result);
                        }
                    }
                }
            },
            ProcessResult::Failure => {
                self.request_text_to_speech_boilerplate(MachineSpeechBoilerplate::VisionFailed as usize);
                if let Some(ref mut task) = &mut self.current_task {
                    let result = task.failed(None).unwrap();
                    self.handle_task_result(result);
                }
            },
        }
    }

    fn handle_task_result(&mut self, result: SmartSpeakerTaskResult) {
        match result.code {
            SmartSpeakerTaskResultCode::TaskSuccess(waitingInteraction) => {
                self.set_next_state(SmartSpeakerState::WaitingForInteraction(waitingInteraction));
            }
            SmartSpeakerTaskResultCode::TaskFailed(waitingInteraction) => {
                self.set_next_state(SmartSpeakerState::WaitingForInteraction(waitingInteraction));
            }
            SmartSpeakerTaskResultCode::Cancelled => {
                self.current_task = None;
                // self.request_text_to_speech_boilerplate(MachineSpeechBoilerplate::Aborted as usize);
                self.set_next_state(SmartSpeakerState::Idle);
            }
            SmartSpeakerTaskResultCode::Exit => {
                self.current_task = None;
                self.set_next_state(SmartSpeakerState::Idle);
            }
            _ => {
                write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Error(format!("{:?} sink!!", result)));
            }
        }
        // explain why none=handle_next_state and some=handle_tts
        // handle next state raise listen/vision request
        // so for listen safe to call handle_next_state after tts
        // state flow in if tts: request_tts -> tts_finished -> handle_next_state -> request_state_update else request_state_update
        match result.tts {
            None => {
                self.handle_next_state();
            }
            Some(script) => {
                self.request_text_to_speech(script);
            }
        }
    }

    fn set_next_state(&mut self, state: SmartSpeakerState) {
        self.next_state = Some(state);
    }

    fn handle_next_state(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Debug(format!("handle next state: {:?}", &self.next_state).to_string()));
        match &self.next_state {
            None => {}
            Some(state) => {
                let current_state = state.clone();
                self.next_state = None;
                self.request_state_update(current_state);
            }
        }
    }

    fn request_state_update(&mut self, state: SmartSpeakerState) {
        write_log_message(&self.sender, SmartSpeakerActors::ContextActor, SmartSpeakerLogMessageType::Debug(format!("request state update: {:?}", state)));
        match &state {
            SmartSpeakerState::WaitingForInteraction(p) => {
                match p {
                    WaitingInteraction::Speak => {
                        state_update_message(
                            &self.sender,
                            SmartSpeakerActors::ContextActor,
                            SmartSpeakerActors::CoreActor,
                            state,
                        )
                    }
                    WaitingInteraction::Vision(_) => {
                        state_update_message(
                            &self.sender,
                            SmartSpeakerActors::ContextActor,
                            SmartSpeakerActors::VisionActor,
                            state,
                        )
                    }
                    WaitingInteraction::Skip => {
                        match &mut self.current_task {
                            None => {}
                            Some(task) => {
                                let result = task.try_next(Some(Box::new(IntentContent::new(IntentAction::Next, vec![])))).unwrap();
                                self.handle_task_result(result);
                            }
                        }
                    }
                    WaitingInteraction::Exit => {
                        match &mut self.current_task {
                            None => {}
                            Some(task) => {
                                let result = task.exit().unwrap();
                                self.handle_task_result(result);
                            }
                        }
                    }
                }
            }
            _ => {
                state_update_message(
                    &self.sender,
                    SmartSpeakerActors::ContextActor,
                    SmartSpeakerActors::CoreActor,
                    state,
                )
            }
        }
    }

    fn request_text_to_speech(&self, text: SmartSpeakerI18nText) {
        text_to_speech_message(
            &self.sender,
            SmartSpeakerActors::ContextActor,
            SmartSpeakerActors::MachineSpeechActor,
            text,
        )
    }

    fn request_text_to_speech_boilerplate(&self, index: usize) {
        text_to_speech_boilerplate_message(
            &self.sender,
            SmartSpeakerActors::ContextActor,
            SmartSpeakerActors::MachineSpeechActor,
            index,
        )
    }
}
