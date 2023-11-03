use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::mic_model::WakeWordDetector;
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;
use crate::smart_speaker::models::core_model::SmartSpeakerState;

pub(crate) struct WakeWordActor {
    alive: bool,
    core: WakeWordDetector,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    stream_before: Vec<i16>,
}

impl WakeWordActor {
    pub(crate) fn new(core: WakeWordDetector, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
            stream_before: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::WakeWordActor, SmartSpeakerLogMessageType::Info("WakeWordActor started".to_string()));
        write_log_message(&self.sender, SmartSpeakerActors::WakeWordActor, SmartSpeakerLogMessageType::Info(self.core.info()));
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
            SmartSpeakerMessage::RequestShutdown(ShutdownMessage {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestAudioStream(AudioStreamMessage { send_from: _, send_to: _, stream }) => {
                if self.stream_before != stream {
                    self.detect_wake_word(&stream);
                    self.stream_before = stream;
                }
            },
            _ => {
                dbg!("unhandled message");
            }
        }
    }

    fn detect_wake_word(&mut self, stream: &Vec<i16>) {
        let result = self.core.detect(stream);
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
        audio_stream_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }

    fn request_attention(&mut self) {
        state_update_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::CoreActor,
            SmartSpeakerState::Attention
        )
    }

    fn terminate(&mut self) {
        terminate_message(
            &self.sender,
            SmartSpeakerActors::WakeWordActor);
        self.alive = false;
    }
}