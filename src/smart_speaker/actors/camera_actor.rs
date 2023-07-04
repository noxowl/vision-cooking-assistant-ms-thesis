use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use opencv::prelude::*;
use opencv::core::Mat;
use crate::smart_speaker::controllers::camera_controller;
use crate::smart_speaker::models::vision_model::Capture;
use crate::utils::message_util::{camera_frame_message, RequestCameraFrame, SmartSpeakerActors, SmartSpeakerMessage};
use crate::utils::vision_util::VisionType;

pub(crate) struct CameraActor {
    alive: bool,
    core: Capture,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    frame_data_bytes: Vec<u8>,
    height: i32,
}

impl CameraActor {
    pub(crate) fn new(core: Capture, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
            frame_data_bytes: vec![],
            height: 0,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("CameraActor started");
        self.core.info();
        while self.alive {
            match &mut self.core.source {
                None => {}
                Some(source) => {
                    match camera_controller::capture_frame(source) {
                        Ok(frame) => {
                            self.handle_frame(frame);
                            if let Ok(message) = self.receiver.try_recv() {
                                self.handle_message(message);
                            }
                        },
                        _ => {}
                    }
                }
            }
            if let Some(vision_type) = self.core.get_source_type() {
                if vision_type == VisionType::Pupil {
                    // The pupil world camera frame rate is 60fps.
                    thread::sleep(Duration::from_millis(14));
                }
            } else {
                thread::sleep(Duration::from_millis(33));
            }
        }
    }

    fn handle_frame(&mut self, frame: Mat) {
        self.frame_data_bytes = frame.data_bytes().unwrap().to_vec();
        self.height = frame.size().unwrap().height;
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(_) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame {
                                                        send_from,
                                                        send_to: _,
                                                        frame_data_bytes: _,
                                                        height: _, }) => {
                camera_frame_message(
                    &self.sender,
                    SmartSpeakerActors::CameraActor,
                    send_from,
                    self.frame_data_bytes.clone(),
                    self.height.clone());
            },
            _ => {}
        }
    }
}
