use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::speak_model::{MachineSpeech, MachineSpeechBoilerplate};
use crate::utils::message_util::{RequestShutdown, SmartSpeakerMessage};

pub(crate) struct MachineSpeechActor {
    alive: bool,
    app: MachineSpeech,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl MachineSpeechActor {
    pub(crate) fn new(app: MachineSpeech, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            app,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("MachineSpeechActor started");
        self.app.init().unwrap();
        self.app.info();
        self.speech(MachineSpeechBoilerplate::PowerOn.to_string_by_language(&self.app.language));
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

    fn speech(&mut self, text: String) {
        match self.app.speak(text) {
            Ok(_) => {},
            Err(e) => {
                dbg!(e);
            }
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                self.alive = false;
            },
            // SmartSpeakerMessage::RequestStateUpdate(_) => {
            //     self.speech(MachineSpeechBoilerplate::WakeUp.to_string_by_language(&self.app.language));
            // },
            // SmartSpeakerMessage::AttentionFinished(AttentionFinished { send_from: _, send_to: _, result }) => {
            //     match result {
            //         AttentionResult::Success => {
            //             self.speech(MachineSpeechBoilerplate::Ok.to_string_by_language(&self.app.language));
            //         },
            //         AttentionResult::Failure => {
            //             self.speech(MachineSpeechBoilerplate::Undefined.to_string_by_language(&self.app.language));
            //         }
            //     }
            // },
            SmartSpeakerMessage::RequestTextToSpeech(message) => {
                self.speech(message.message);
            },
            SmartSpeakerMessage::StringMessage(message) => {
                self.speech(message.message);
            },
            _ => {
                dbg!("unhandled message");
            }
        }
    }
}
