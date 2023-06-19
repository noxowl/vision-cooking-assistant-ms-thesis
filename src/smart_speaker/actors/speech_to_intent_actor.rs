use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::controllers::mic_controller;
use crate::smart_speaker::models::mic_model::SpeechToIntent;
use crate::utils::message_util;
use crate::utils::message_util::{RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};

pub(crate) struct SpeechToIntentActor {
    alive: bool,
    app: SpeechToIntent,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl SpeechToIntentActor {
    pub(crate) fn new(app: SpeechToIntent, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            app,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("SpeechToIntentActor started");
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
             SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                 self.alive = false;
             },
             SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from: _, send_to: _, stream }) => {
                 self.listen(stream);
             },
             _ => {
                 dbg!("unhandled message");
             }
         }
    }

    fn listen(&mut self, stream: Vec<i16>) {
        if let Ok(finalized) = mic_controller::speech_to_intent_feed(&mut self.app, &stream) {
            if finalized {
                if let Ok(inference) = self.app.get_inference() {
                    dbg!(inference);
                }
                self.attention_finished();
                self.terminate();
            }
        }
    }

    fn request_audio_stream(&mut self) {
        message_util::audio_stream_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }

    fn attention_finished(&mut self) {
        message_util::attention_finished_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor,
            SmartSpeakerActors::CoreActor)
    }

    fn terminate(&mut self) {
        message_util::terminate_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor);
        self.alive = false;
    }
}