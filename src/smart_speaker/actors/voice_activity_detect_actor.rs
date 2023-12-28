use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::mic_model::VoiceActivityDetector;
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

pub(crate) struct VoiceActivityDetectActor {
    alive: bool,
    app: VoiceActivityDetector,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    stream_before: Vec<i16>,
}

impl VoiceActivityDetectActor {
    pub(crate) fn new(app: VoiceActivityDetector, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            app,
            receiver,
            sender,
            stream_before: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        println!("VoiceActivityDetectActor started");
        self.app.info();
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
    }

     fn handle_message(&mut self, message: SmartSpeakerMessage) {
         match message {
             SmartSpeakerMessage::RequestShutdown(ShutdownMessage {}) => {
                 self.alive = false;
             },
             SmartSpeakerMessage::RequestAudioStream(AudioStreamMessage { send_from: _, send_to: _, stream }) => {
                 if self.stream_before != stream {
                     self.listen(&stream);
                     self.stream_before = stream;
                 }
             },
             _ => {
                 dbg!("unhandled message");
             }
         }
    }

    fn request_audio_stream(&self) {
        audio_stream_message(
            &self.sender,
            SmartSpeakerActors::VoiceActivityDetectActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }

    fn request_attention(&mut self) {
        state_update_message(
            &self.sender,
            SmartSpeakerActors::VoiceActivityDetectActor,
            SmartSpeakerActors::CoreActor,
            SmartSpeakerState::Attention
        )
    }

    fn listen(&mut self, stream: &Vec<i16>) {
        if let Ok(probability) = self.app.detect(&stream) {
            // dbg!(&probability);
            if probability > 0.5 {
                self.request_attention();
                self.terminate();
            }
        }
    }

    fn terminate(&mut self) {
        terminate_message(
            &self.sender,
            SmartSpeakerActors::VoiceActivityDetectActor);
        self.alive = false;
    }
}
