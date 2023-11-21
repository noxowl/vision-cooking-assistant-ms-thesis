use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::controllers::mic_controller;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu};
use crate::smart_speaker::models::mic_model::SpeechToIntent;
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

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
        write_log_message(&self.sender, SmartSpeakerActors::SpeechToIntentActor, SmartSpeakerLogMessageType::Info("SpeechToIntentActor started".to_string()));
        // write_log_message(&self.sender, SmartSpeakerActors::SpeechToIntentActor, SmartSpeakerLogMessageType::Debug(self.app.info()));
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

    fn listen(&mut self, stream: &Vec<i16>) {
        if let Ok(finalized) = mic_controller::speech_to_intent_feed(&mut self.app, &stream) {
            if finalized {
                let mut content = IntentContent {
                    intent: IntentAction::None,
                    entities: vec![],
                };
                if let Ok(inference) = self.app.get_inference() {
                    match inference {
                        None => {
                            dbg!("inference is none");
                            write_log_message(&self.sender, SmartSpeakerActors::SpeechToIntentActor, SmartSpeakerLogMessageType::Error(format!("failed to parse intent: None")));
                            self.intent_finalized(ProcessResult::Failure, content);
                        }
                        Some(i) => {
                            let intent = &i.intent.unwrap();
                            match IntentAction::from_str(intent) {
                                Ok(action) => {
                                    content.intent = action;
                                    for (key, value) in i.slots {
                                        match key.as_str() {
                                            "menu_name" => {
                                                content.entities.push(Box::new(IntentCookingMenu::from_str(&value).unwrap()));
                                            }
                                            &_ => {}
                                        }
                                    }
                                    self.intent_finalized(ProcessResult::Success, content);
                                }
                                Err(_) => {
                                    write_log_message(&self.sender, SmartSpeakerActors::SpeechToIntentActor, SmartSpeakerLogMessageType::Error(format!("failed to parse intent: {}", intent)));
                                    self.intent_finalized(ProcessResult::Failure, content);
                                }
                            }
                        }
                    }
                } else {
                    self.intent_finalized(ProcessResult::Failure, content);
                }
                self.terminate();
            }
        }
    }

    fn request_audio_stream(&mut self) {
        audio_stream_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }

    fn intent_finalized(&mut self, result: ProcessResult, content: IntentContent) {
        intent_finalized_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor,
            SmartSpeakerActors::ContextActor,
            result,
            content,
        )
    }

    fn terminate(&mut self) {
        terminate_message(
            &self.sender,
            SmartSpeakerActors::SpeechToIntentActor);
        self.alive = false;
    }
}