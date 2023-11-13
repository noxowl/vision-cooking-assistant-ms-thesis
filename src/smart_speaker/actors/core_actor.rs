use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::actors::audio_actor::AudioActor;
use crate::smart_speaker::actors::camera_actor::CameraActor;
use crate::smart_speaker::actors::context_actor::ContextActor;
use crate::smart_speaker::actors::gaze_actor::GazeActor;
use crate::smart_speaker::actors::logger_actor::LoggerActor;
use crate::smart_speaker::actors::speech_to_intent_actor::SpeechToIntentActor;
use crate::smart_speaker::actors::machine_speech_actor::MachineSpeechActor;
// use crate::smart_speaker::actors::stream_actor::StreamActor;
use crate::smart_speaker::actors::vision_actor::VisionActor;
use crate::smart_speaker::actors::voice_activity_detect_actor::VoiceActivityDetectActor;
use crate::smart_speaker::actors::wake_word_actor::WakeWordActor;
use crate::smart_speaker::models::core_model::{WaitingInteraction, SmartSpeakerState};
use crate::smart_speaker::models::debug_model::DebugData;
use crate::smart_speaker::models::gaze_model::Gaze;
use crate::smart_speaker::models::mic_model::{AudioListener, SpeechToIntent, VoiceActivityDetector, WakeWordDetector};
use crate::smart_speaker::models::speak_model::{MachineSpeech, MachineSpeechBoilerplate};
use crate::smart_speaker::models::vision_model::Capture;
use crate::utils::config_util::Config;
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::write_log_message;
use crate::utils::vision_util;
use crate::utils::vision_util::VisionType;
#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::{NSRunLoop, NSDate, NSDefaultRunLoopMode};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl, class};

struct CoreActorManager {
    alive: bool,
    senders: HashMap<SmartSpeakerActors, mpsc::Sender<SmartSpeakerMessage>>
}

impl CoreActorManager {
    fn new() -> Self {
        Self {
            alive: true,
            senders: HashMap::new()
        }
    }

    fn add_sender(&mut self, actor: SmartSpeakerActors, sender: mpsc::Sender<SmartSpeakerMessage>) {
        self.senders.insert(actor, sender);
    }

    fn get_sender(&self, actor: SmartSpeakerActors) -> Option<&mpsc::Sender<SmartSpeakerMessage>> {
        self.senders.get(&actor)
    }

    fn remove_sender(&mut self, actor: SmartSpeakerActors) {
        if let Some(sender) = self.senders.remove(&actor) {
            drop(sender);
        }
    }

    fn spawn_actor(&mut self, config: &Config, actor: SmartSpeakerActors, sender: mpsc::Sender<SmartSpeakerMessage>) {
        let (tx, rx) = mpsc::channel();
        match actor {
            SmartSpeakerActors::AudioActor => {
                let mut audio_actor = AudioActor::new(
                    AudioListener::new(config.mic_index.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    audio_actor.run();
                });
            },
            SmartSpeakerActors::CameraActor => {
                let mut capture_source = Capture::new();
                match config.vision_type {
                    VisionType::None => {}
                    VisionType::Pupil => {
                        vision_util::set_pupil_capture(&mut capture_source, config.zmq_in_endpoint.clone()).expect("TODO: panic message");
                    }
                    VisionType::BuiltInCamera => {
                        vision_util::set_camera_capture(&mut capture_source).expect("TODO: panic message");
                    }
                }
                let mut camera_actor = CameraActor::new(
                    capture_source,
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    camera_actor.run();
                });
            },
            SmartSpeakerActors::VisionActor => {
                let mut vision_actor = VisionActor::new(
                    rx,
                    sender.clone(),
                    config.debug.clone()
                );
                thread::spawn(move || {
                    vision_actor.run();
                });
            },
            SmartSpeakerActors::GazeActor => {
                let gaze = Gaze::new(config.vision_type.clone(), 0.5, 0.5, config.zmq_in_endpoint.clone()).unwrap();
                let mut gaze_actor = GazeActor::new(
                    gaze,
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    gaze_actor.run();
                });
            }
            SmartSpeakerActors::WakeWordActor => {
                let mut wake_word_actor = WakeWordActor::new(
                    WakeWordDetector::new(config.pico_voice_api_key.clone(), config.pico_voice_ppn_model_path.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    wake_word_actor.run();
                    drop(wake_word_actor);
                });
            },
            SmartSpeakerActors::SpeechToIntentActor => {
                let mut speech_to_intent_actor = SpeechToIntentActor::new(
                    SpeechToIntent::new(config.pico_voice_api_key.clone(),
                                        config.pico_voice_rhn_model_path.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    speech_to_intent_actor.run();
                    drop(speech_to_intent_actor);
                });
            },
            SmartSpeakerActors::MachineSpeechActor => {
                let mut machine_speech_actor = MachineSpeechActor::new(
                    MachineSpeech::new(config.language.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    machine_speech_actor.run();
                });
            },
            SmartSpeakerActors::VoiceActivityDetectActor => {
                let mut voice_activity_detect_actor = VoiceActivityDetectActor::new(
                    VoiceActivityDetector::new(config.pico_voice_api_key.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    voice_activity_detect_actor.run();
                });
            }
            SmartSpeakerActors::ContextActor => {
                let mut context_actor = ContextActor::new(
                    rx,
                    sender.clone(),
                    config.vision.clone()
                );
                thread::spawn(move || {
                    context_actor.run();
                });
            }
            SmartSpeakerActors::LoggerActor => {
                let mut logger_actor = LoggerActor::new(
                    rx,
                    sender.clone(),
                    config.debug.clone()
                );
                thread::spawn(move || {
                    logger_actor.run();
                });
            }
            // SmartSpeakerActors::StreamActor => {
            //     let mut stream_actor = StreamActor::new(
            //         rx,
            //         sender.clone(),
            //     );
            //     thread::spawn(move || {
            //         stream_actor.run();
            //     });
            // }
            _ => {}
        }
        self.add_sender(actor, tx);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CoreActorState {
    ActorTerminated {
        actor: SmartSpeakerActors,
    },
    NewActorRequested {
        actor: SmartSpeakerActors,
        custom_args: Option<String>,
    },
    WaitForNextMessage {},
    ShutdownRequested {},
}

pub(crate) struct CoreActorMessageHandler {
    pub(crate) debug: DebugData,
}

impl CoreActorMessageHandler {
    pub fn handle_message(&mut self, senders: &HashMap<SmartSpeakerActors, mpsc::Sender<SmartSpeakerMessage>>, message: SmartSpeakerMessage) -> CoreActorState {
        return match &message {
            SmartSpeakerMessage::RequestShutdown(ShutdownMessage {}) => {
                for (_, sender) in senders.iter() {
                    sender.send(message.clone()).expect("TODO: panic message");
                }
                CoreActorState::ShutdownRequested {}
            },
            SmartSpeakerMessage::ReportTerminated(ReportTerminated { send_from, send_to }) => {
                write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                  SmartSpeakerActors::CoreActor,
                                  SmartSpeakerLogMessageType::Debug(
                                      format!("ReportTerminated from {:?} to {:?}", &send_from, &send_to).to_string()));
                CoreActorState::ActorTerminated {
                    actor: send_from.clone(),
                }
            },
            SmartSpeakerMessage::RequestAudioStream(AudioStreamMessage { send_from, send_to, stream: _ }) => {
                if senders.get(&send_from).is_none() {
                    write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                      SmartSpeakerActors::CoreActor,
                                      SmartSpeakerLogMessageType::Error(
                                          format!("RequestAudioStream find sender error: {:?} to {:?}", &send_from, &send_to).to_string()));
                }
                if let Some(sender) = senders.get(&send_to) {
                    match sender.send(message.clone()) {
                        Ok(_) => {},
                        Err(_) => {
                            write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                              SmartSpeakerActors::CoreActor,
                                              SmartSpeakerLogMessageType::Error(
                                                  format!("RequestAudioStream error: {:?} to {:?}", &send_from, &send_to).to_string()));
                        }
                    }
                } else {
                    write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                      SmartSpeakerActors::CoreActor,
                                      SmartSpeakerLogMessageType::Error(
                                          format!("RequestAudioStream error: {:?} to {:?}", &send_from, &send_to).to_string()));
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestCameraFrame(CameraFrameMessage { send_from, send_to,
                                                        frame_data_bytes, height }) => {
                if self.debug.activated && send_from == &SmartSpeakerActors::CameraActor {
                    self.debug.update_frame(&frame_data_bytes, height);
                }
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestGazeInfo(GazeInfoMessage { send_from, send_to, gaze_info }) => {
                if self.debug.activated && send_from == &SmartSpeakerActors::GazeActor {
                    self.debug.update_gaze_info(&gaze_info);
                }
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestQuery(QueryMessage { send_from, send_to, message: _ }) => {
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestStateUpdate(StateUpdateMessage { send_from, send_to, state }) => {
                write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                  SmartSpeakerActors::CoreActor,
                                  SmartSpeakerLogMessageType::Debug(
                                      format!("RequestStateUpdate from {:?} to {:?}", &send_from, &state).to_string()));
                if self.debug.activated {
                    self.debug.update_state(state.clone());
                }
                match send_from {
                    SmartSpeakerActors::WakeWordActor => {
                        if let Some(sender) = senders.get(&SmartSpeakerActors::MachineSpeechActor) {
                            sender.send(SmartSpeakerMessage::RequestTextToSpeech(TextToSpeechMessage {
                                send_from: SmartSpeakerActors::WakeWordActor,
                                send_to: SmartSpeakerActors::MachineSpeechActor,
                                message: TextToSpeechMessageType::Boilerplate(MachineSpeechBoilerplate::WakeUp as usize),
                            })).expect("TODO: panic message");
                        }
                    },
                    SmartSpeakerActors::VoiceActivityDetectActor => {
                        if senders.get(&SmartSpeakerActors::SpeechToIntentActor).is_none() {
                            return CoreActorState::NewActorRequested {
                                actor: SmartSpeakerActors::SpeechToIntentActor,
                                custom_args: None,
                            }
                        }
                    },
                    SmartSpeakerActors::ContextActor => {
                        match state {
                            SmartSpeakerState::Idle => {
                                if senders.get(&SmartSpeakerActors::WakeWordActor).is_none() {
                                    return CoreActorState::NewActorRequested {
                                        actor: SmartSpeakerActors::WakeWordActor,
                                        custom_args: None,
                                    }
                                }
                            }
                            SmartSpeakerState::WaitingForInteraction(pending_type) => {
                                match pending_type {
                                    WaitingInteraction::Speak => {
                                        if senders.get(&SmartSpeakerActors::VoiceActivityDetectActor).is_none() {
                                            return CoreActorState::NewActorRequested {
                                                actor: SmartSpeakerActors::VoiceActivityDetectActor,
                                                custom_args: None,
                                            }
                                        }
                                    }
                                    WaitingInteraction::Vision(action) => {
                                        if let Some(sender) = senders.get(&SmartSpeakerActors::VisionActor) {
                                            sender.send(message).expect("TODO: panic message");
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestTextToSpeech(TextToSpeechMessage { send_from, send_to, message: _ }) => {
                if let Some(sender) = senders.get(&SmartSpeakerActors::MachineSpeechActor) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::IntentFinalized(IntentFinalizedMessage { send_from: _, send_to: _, result: _, content: _ }) => {
                if let Some(sender) = senders.get(&SmartSpeakerActors::ContextActor) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::VisionFinalized(VisionFinalizedMessage { send_from, send_to, result, contents }) => {
                if let Some(sender) = senders.get(&SmartSpeakerActors::ContextActor) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::TextToSpeechFinished(StringMessage{ send_from, send_to, message: _ }) => {
                write_log_message(&senders.get(&SmartSpeakerActors::LoggerActor).unwrap(),
                                  SmartSpeakerActors::CoreActor,
                                  SmartSpeakerLogMessageType::Debug(
                                      format!("TextToSpeechFinished from {:?} to {:?}", &send_from, &send_to).to_string()));
                match &send_to {
                    SmartSpeakerActors::WakeWordActor => {
                        if senders.get(&SmartSpeakerActors::WakeWordActor).is_none() && senders.get(&SmartSpeakerActors::SpeechToIntentActor).is_none() {
                            return CoreActorState::NewActorRequested {
                                actor: SmartSpeakerActors::SpeechToIntentActor,
                                custom_args: None,
                            }
                        }
                    },
                    SmartSpeakerActors::CoreActor => {
                        // consume here
                    },
                    _ => {
                        if let Some(sender) = senders.get(&SmartSpeakerActors::ContextActor) {
                            sender.send(message).expect("TODO: panic message");
                        }
                    },
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::StringMessage(StringMessage { send_from, send_to, message: _ }) => {
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::WriteLog(LogMessage { send_from, send_to, message: _ }) => {
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(message).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            }
            _ => {
                CoreActorState::WaitForNextMessage {}
            }
        }
    }
}

pub(crate) struct CoreActor {
    config: Config,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    manager: CoreActorManager,
    message_handler: CoreActorMessageHandler,
}

impl CoreActor {
    pub(crate) fn new(config: Config, sender: mpsc::Sender<SmartSpeakerMessage>, receiver: mpsc::Receiver<SmartSpeakerMessage>) -> Self {
        Self {
            config: config.clone(),
            sender,
            receiver,
            manager: CoreActorManager::new(),
            message_handler: CoreActorMessageHandler {
                debug: match config.debug {
                    true => DebugData::new(true),
                    false => DebugData::new(false),
                }
            },
        }
    }

    fn init(&mut self) {
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::LoggerActor, self.sender.clone());
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::AudioActor, self.sender.clone());
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::WakeWordActor, self.sender.clone());
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::ContextActor, self.sender.clone());
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::MachineSpeechActor, self.sender.clone());
        if self.config.vision_type != VisionType::None {
            self.manager.spawn_actor(&self.config, SmartSpeakerActors::CameraActor, self.sender.clone());
            // self.manager.spawn_actor(&self.config, SmartSpeakerActors::StreamActor, self.sender.clone());
        }
        if self.config.vision {
            self.manager.spawn_actor(&self.config, SmartSpeakerActors::VisionActor, self.sender.clone());
            self.manager.spawn_actor(&self.config, SmartSpeakerActors::GazeActor, self.sender.clone());
        }

    }

    pub(crate) fn run(&mut self) {
        dbg!(&self.config);
        self.init();
        while self.manager.alive {
            let mut pending = true;
            while pending {
                if let Ok(message) = self.receiver.try_recv() {
                    match self.message_handler.handle_message(&self.manager.senders, message) {
                        CoreActorState::ActorTerminated { actor } => {
                            self.manager.remove_sender(actor);
                        },
                        CoreActorState::NewActorRequested { actor, custom_args } => {
                            self.manager.spawn_actor(&self.config, actor, self.sender.clone());
                        },
                        CoreActorState::WaitForNextMessage {} => {
                        },
                        CoreActorState::ShutdownRequested {} => {
                            self.manager.alive = false;
                        }
                    }
                } else {
                    pending = false;
                }
            }
            if self.config.debug && self.config.vision {
                self.message_handler.debug.print();
            } else {
                #[cfg(target_os = "macos")]
                {
                    // This block for macOS. without this block, the TTS callback will not be called.
                    // but in debug mode, the highgui window will execute this block internally.
                    self.message_handler.debug.force_cocoa_loop();
                }
            }
            thread::sleep(Duration::from_micros(1));
        }
    }
}
