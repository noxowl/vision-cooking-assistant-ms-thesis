use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use chrono::Local;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use crate::smart_speaker::models::message_model::*;

pub(crate) struct SubtaskActor {
    alive: bool,
    debug: bool,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl SubtaskActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, debug: bool) -> Self {
        Self {
            alive: true,
            debug,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        log::info!("[{}] SubtaskActor started", SmartSpeakerActors::LoggerActor);
        let _ = self.init_logger();
        while self.alive {
            if let Ok(message) = self.receiver.try_recv() {
                self.handle_message(message);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(_) => {
                self.alive = false;
            },
            SmartSpeakerMessage::SubTaskStart(m) => {

            }
            _ => {}
        }
    }
}
