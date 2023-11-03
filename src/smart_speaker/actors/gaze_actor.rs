use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::gaze_model::Gaze;
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;

pub(crate) struct GazeActor {
    alive: bool,
    core: Gaze,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    gaze_x: f32,
    gaze_y: f32,
}

impl GazeActor {
    pub(crate) fn new(core: Gaze, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
            gaze_x: 0.,
            gaze_y: 0.,
        }
    }

    pub(crate) fn run(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::GazeActor, SmartSpeakerLogMessageType::Info("GazeActor started".to_string()));
        while self.alive {
            self.core.update_gaze();
            self.handle_gaze(self.core.get_gaze());
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
            SmartSpeakerMessage::RequestGazeInfo(GazeInfoMessage {send_from, send_to: _, gaze_info: _}) => {
                gaze_info_message(
                    &self.sender,
                    SmartSpeakerActors::GazeActor,
                    send_from,
                    (self.gaze_x, self.gaze_y)
                )
            },
            _ => {}
        }
    }

    fn handle_gaze(&mut self, (gaze_x, gaze_y): (f32, f32)) {
        self.gaze_x = gaze_x;
        self.gaze_y = gaze_y;
    }
}
