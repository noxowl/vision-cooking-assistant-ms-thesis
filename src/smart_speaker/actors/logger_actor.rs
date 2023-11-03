use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use chrono::Local;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use crate::smart_speaker::models::message_model::*;

pub(crate) struct LoggerActor {
    alive: bool,
    debug: bool,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl LoggerActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, debug: bool) -> Self {
        Self {
            alive: true,
            debug,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        log::info!("[{}] LoggerActor started", SmartSpeakerActors::LoggerActor);
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
            SmartSpeakerMessage::WriteLog(m) => {
                match m.message {
                    SmartSpeakerLogMessageType::Debug(log) => {
                        log::debug!("[{}] {}",m.send_from, log);
                    }
                    SmartSpeakerLogMessageType::Info(log) => {
                        log::info!("[{}] {}",m.send_from, log);
                    }
                    SmartSpeakerLogMessageType::Warn(log) => {
                        log::warn!("[{}] {}",m.send_from, log);
                    }
                    SmartSpeakerLogMessageType::Error(log) => {
                        log::error!("[{}] {}",m.send_from, log);
                    }
                }
            }
            _ => {}
        }
    }

    fn init_logger(&self) -> Result<(), Box<dyn std::error::Error>> {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} [{l}] - {m}\n")))
            .build(format!("log/vgv-{}.log", Local::now().format("%Y-%m-%dT%H_%M")))?;

        let mut root= Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info);
        if self.debug {
            root = Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug);
        }
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(root)?;
        log4rs::init_config(config)?;
        Ok(())
    }
}
