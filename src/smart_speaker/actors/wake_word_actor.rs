use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::mic_model::WakeWordDetector;
use crate::utils::message_util;
use crate::utils::message_util::{RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};

pub(crate) struct WakeWordActor {
    alive: bool,
    core: WakeWordDetector,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl WakeWordActor {
    pub(crate) fn new(core: WakeWordDetector, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("WakeWordActor started");
        self.core.info();
        while self.alive {
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message);
                },
                _ => {}
            }
            if self.alive {
                self.request_audio_stream();
            }
            thread::sleep(Duration::from_millis(33));
        }
        println!("WakeWordActor terminated");
    }

     fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from: _, send_to: _, stream }) => {
                self.detect_wake_word(stream);
            },
            _ => {
                dbg!("unhandled message");
            }
        }
    }

    fn detect_wake_word(&mut self, stream: Vec<i16>) {
        let result = self.core.detect(&stream);
        match result {
            Ok(result) => {
                if result {
                    dbg!("wake word detected");
                    self.request_attention();
                    self.terminate();
                }
            },
            Err(error) => {
                dbg!(error);
            }
        }
    }

    fn request_audio_stream(&mut self) {
        message_util::audio_stream_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }

    fn request_attention(&mut self) {
        message_util::attention_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::CoreActor)
    }

    fn terminate(&mut self) {
        message_util::terminate_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor);
        self.alive = false;
    }
}