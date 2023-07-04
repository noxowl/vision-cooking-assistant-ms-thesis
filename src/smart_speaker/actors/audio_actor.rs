use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::mic_model::AudioListener;
use crate::smart_speaker::controllers::mic_controller;
use crate::utils::message_util::{audio_stream_message, RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage, StringMessage};

pub(crate) struct AudioActor {
    alive: bool,
    core: AudioListener,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    stream: Vec<i16>,
}

impl AudioActor {
    pub(crate) fn new(core: AudioListener, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
            stream: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        println!("AudioActor started");
        self.core.info();
        let _ = self.core.start();
        while self.alive {
            match mic_controller::listen_mic(&mut self.core) {
                Ok(stream) => {
                    self.stream = stream;
                    let mut pending = true;
                    while pending {
                        if let Ok(message) = self.receiver.try_recv() {
                            self.handle_message(message);
                        } else {
                            pending = false;
                        }
                    }
                },
                _ => {}
            }
            thread::sleep(Duration::from_millis(33));
        }
        let _ = self.core.stop();
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::StringMessage(StringMessage { send_from, send_to: _, message: _ }) => {
                self.sender.send(SmartSpeakerMessage::StringMessage(StringMessage {
                    send_from: SmartSpeakerActors::AudioActor,
                    send_to: send_from,
                    message: "pong".to_string(),
                })).expect("TODO: panic message");
            },
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from, send_to: _, stream: _ }) => {
                audio_stream_message(&self.sender, SmartSpeakerActors::AudioActor, send_from, self.stream.clone());
            }
            _ => {
                dbg!("unhandled message");
            }
        }
    }
}
