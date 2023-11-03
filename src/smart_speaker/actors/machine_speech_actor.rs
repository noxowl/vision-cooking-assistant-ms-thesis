use std::sync::{mpsc};
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::speak_model::{MachineSpeech, MachineSpeechBoilerplate};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

pub(crate) struct MachineSpeechActor {
    alive: bool,
    app: MachineSpeech,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    callback_sender: mpsc::Sender<SmartSpeakerActors>,
    callback_receiver: mpsc::Receiver<SmartSpeakerActors>
}

impl MachineSpeechActor {
    pub(crate) fn new(app: MachineSpeech, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            alive: true,
            app,
            receiver,
            sender,
            callback_sender: tx,
            callback_receiver: rx
        }
    }

    pub(crate) fn run(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::MachineSpeechActor, SmartSpeakerLogMessageType::Info("MachineSpeechActor started".to_string()));
        self.app.init().unwrap();
        write_log_message(&self.sender, SmartSpeakerActors::MachineSpeechActor, SmartSpeakerLogMessageType::Info(self.app.info()));
        self.speech(TextToSpeechMessageType::Boilerplate(MachineSpeechBoilerplate::PowerOn as usize), Some(SmartSpeakerActors::CoreActor));
        while self.alive {
            match self.callback_receiver.try_recv() {
                Ok(actor) => {
                    self.text_to_speech_finished_message(actor);
                },
                _ => {}
            }
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.handle_message(message);
                },
                _ => {}
            }
            thread::sleep(Duration::from_millis(33));
        }
    }

    fn speech(&mut self, message: TextToSpeechMessageType, request_from: Option<SmartSpeakerActors>) {
        let (micro_tx, micro_rx) = mpsc::channel();
        let mut speech_callback_actor = MachineSpeechCallbackMicroActor {
            receiver: micro_rx,
            sender: self.callback_sender.clone(),
            message: request_from
        };
        thread::spawn(move || {
            speech_callback_actor.run();
        });
        match message {
            TextToSpeechMessageType::Normal(text) => {
                self.app.speak_with_callback(text, micro_tx);
            }
            TextToSpeechMessageType::Boilerplate(index) => {
                self.app.speak_with_callback(
                    MachineSpeechBoilerplate::try_from(index).unwrap().to_string_by_language(&self.app.language),
                    micro_tx);
            }
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(ShutdownMessage {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestTextToSpeech(message) => {
                self.speech(message.message, Some(message.send_from));
            },
            SmartSpeakerMessage::StringMessage(message) => {
                self.speech(TextToSpeechMessageType::Normal(message.message), None);
            },
            _ => {
                dbg!("unhandled message");
            }
        }
    }

    fn text_to_speech_finished_message(&mut self, request_from: SmartSpeakerActors) {
        text_to_speech_finished_message(
            &self.sender,
            SmartSpeakerActors::MachineSpeechActor,
            request_from,
            "".to_string()
        )
    }
}

pub(crate) struct MachineSpeechCallbackMicroActor {
    receiver: mpsc::Receiver<usize>,
    sender: mpsc::Sender<SmartSpeakerActors>,
    message: Option<SmartSpeakerActors>
}

impl MachineSpeechCallbackMicroActor {
    fn run(&mut self) {
        dbg!(format!("MachineSpeechCallbackMicroActor started with {:?}", &self.message));
        while let Ok(message) = self.receiver.recv() {
            dbg!(format!("MachineSpeechCallbackMicroActor({:?}) got callback({:?})", &self.message, message));
            if let Some(actor) = &self.message {
                self.sender.send(actor.clone()).unwrap();
            }
        }
    }
}