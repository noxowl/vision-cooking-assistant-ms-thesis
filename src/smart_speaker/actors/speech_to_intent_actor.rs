use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rhino::RhinoInference;
use crate::smart_speaker::controllers::mic_controller;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu};
use crate::smart_speaker::models::mic_model::SpeechToIntent;
use crate::utils::message_util;
use crate::utils::message_util::{IntentContent, ProcessResult, RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};

pub(crate) struct SpeechToIntentActor {
    alive: bool,
    app: SpeechToIntent,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    stream_before: Vec<i16>,
}

impl SpeechToIntentActor {
    pub(crate) fn new(app: SpeechToIntent, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            app,
            receiver,
            sender,
            stream_before: vec![],
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

    fn listen(&mut self, stream: &Vec<i16>) {
        if let Ok(finalized) = mic_controller::speech_to_intent_feed(&mut self.app, &stream) {
            if finalized {
                let mut content = IntentContent {
                    intent: IntentAction::None,
                    entities: vec![],
                };
                if let Ok(inference) = self.app.get_inference() {
                    match inference {
                        None => {}
                        Some(i) => {
                            content.intent = IntentAction::from_str(&i.intent.unwrap()).unwrap();
                            for (key, value) in i.slots {
                                match key.as_str() {
                                    "menu_name" => {
                                        content.entities.push(Box::new(IntentCookingMenu::from_str(&value).unwrap()));
                                    }
                                    &_ => {}
                                }
                            }

                        }
                    }
                    self.intent_finalized(ProcessResult::Success, content);
                } else {
                    self.intent_finalized(ProcessResult::Failure, content);
                }
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

    fn intent_finalized(&mut self, result: ProcessResult, content: IntentContent) {
        message_util::intent_finalized_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor,
            SmartSpeakerActors::ContextActor,
            result,
            content,
        )
    }

    fn terminate(&mut self) {
        message_util::terminate_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor);
        self.alive = false;
    }
}