use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::actors::audio_actor::AudioActor;
use crate::smart_speaker::actors::speech_to_intent_actor::SpeechToIntentActor;
use crate::smart_speaker::actors::wake_word_actor::WakeWordActor;
use crate::smart_speaker::models::mic_model::{AudioListener, SpeechToIntent, WakeWordDetector};
use crate::utils::config_util::Config;
use crate::utils::message_util::{AttentionFinished, QueryMessage, ReportTerminated, RequestAttention, RequestAudioStream, RequestCameraFrame, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};


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
        self.senders.remove(&actor);
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
            // SmartSpeakerActors::CameraActor => {
            //     let mut camera_actor = CameraActor::new(
            //         Camera::new().expect("TODO: panic message"),
            //         self.receiver.clone(),
            //         sender.clone(),
            //     );
            //     thread::spawn(move || {
            //         camera_actor.run();
            //     })
            // },
            SmartSpeakerActors::WakeWordActor => {
                let mut wake_word_actor = WakeWordActor::new(
                    WakeWordDetector::new(config.pico_voice_api_key.clone()),
                    rx,
                    sender.clone(),
                );
                thread::spawn(move || {
                    wake_word_actor.run();
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
                });
            },
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
    },
    WaitForNextMessage {},
    ShutdownRequested {},
}

pub(crate) struct CoreActorMessageHandler {}

impl CoreActorMessageHandler {
    pub fn handle_message(&self, senders: HashMap<SmartSpeakerActors, mpsc::Sender<SmartSpeakerMessage>>, message: SmartSpeakerMessage) -> CoreActorState {
        return match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                for (_, sender) in senders.iter() {
                    sender.send(SmartSpeakerMessage::RequestShutdown(RequestShutdown {})).expect("TODO: panic message");
                }
                CoreActorState::ShutdownRequested {}
            },
            SmartSpeakerMessage::ReportTerminated(ReportTerminated { send_from, send_to }) => {
                println!("ReportTerminated from {:?} to {:?}", &send_from, &send_to);
                CoreActorState::ActorTerminated {
                    actor: send_from,
                }
            },
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from, send_to, stream }) => {
                if let Some(sender) = senders.get(&send_to) {
                    match sender.send(SmartSpeakerMessage::RequestAudioStream(RequestAudioStream {
                        send_from: send_from.clone(),
                        send_to: send_to.clone(),
                        stream,
                    })) {
                        Ok(_) => {},
                        Err(err) => {
                            println!("RequestAudioStream error: {:?} to {:?}", &send_from, &send_to);
                        }
                    }
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame { send_from, send_to, frame }) => {
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame {
                        send_from,
                        send_to,
                        frame,
                    })).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestQuery(QueryMessage { send_from, send_to, message }) => {
                if let Some(sender) = senders.get(&send_to) {
                    sender.send(SmartSpeakerMessage::RequestQuery(QueryMessage {
                        send_from,
                        send_to,
                        message,
                    })).expect("TODO: panic message");
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::RequestAttention(RequestAttention { send_from: _, send_to: _ }) => {
                println!("RequestAttention");
                if senders.get(&SmartSpeakerActors::SpeechToIntentActor).is_none() {
                    return CoreActorState::NewActorRequested {
                        actor: SmartSpeakerActors::SpeechToIntentActor,
                    }
                }
                CoreActorState::WaitForNextMessage {}
            },
            SmartSpeakerMessage::AttentionFinished(AttentionFinished { send_from: _, send_to: _ }) => {
                if senders.get(&SmartSpeakerActors::WakeWordActor).is_none() {
                    return CoreActorState::NewActorRequested {
                        actor: SmartSpeakerActors::WakeWordActor,
                    }
                }
                CoreActorState::WaitForNextMessage {}
            },
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
            config,
            sender,
            receiver,
            manager: CoreActorManager::new(),
            message_handler: CoreActorMessageHandler {},
        }
    }

    fn init(&mut self) {
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::AudioActor, self.sender.clone());
        self.manager.spawn_actor(&self.config, SmartSpeakerActors::WakeWordActor, self.sender.clone());
    }

    pub(crate) fn run(&mut self) {
        self.init();
        while self.manager.alive {
            if let Ok(message) = self.receiver.try_recv() {
                match self.message_handler.handle_message(self.manager.senders.clone(), message) {
                    CoreActorState::ActorTerminated { actor } => {
                        self.manager.remove_sender(actor);
                    },
                    CoreActorState::NewActorRequested { actor } => {
                        self.manager.spawn_actor(&self.config, actor, self.sender.clone());
                    },
                    CoreActorState::WaitForNextMessage {} => {},
                    CoreActorState::ShutdownRequested {} => {
                        self.manager.alive = false;
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}
