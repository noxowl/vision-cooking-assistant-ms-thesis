use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use bounded_vec_deque::BoundedVecDeque;
use opencv::{core::Mat, core::Vector, types::VectorOfVectorOfPoint2f};
use crate::smart_speaker::controllers::vision_controller;
use crate::utils::message_util::{camera_frame_message, gaze_info_message, marker_info_message, RequestCameraFrame, RequestGazeInfo, SmartSpeakerActors, SmartSpeakerMessage};

pub(crate) struct VisionActor {
    alive: bool,
    attention: bool,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    previous_frames: BoundedVecDeque<Mat>,
    previous_gaze_info: BoundedVecDeque<(f32, f32)>,
    previous_aruco_info: BoundedVecDeque<(VectorOfVectorOfPoint2f, Vector<i32>)>,
}

impl VisionActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            attention: false,
            receiver,
            sender,
            previous_frames: BoundedVecDeque::new(60),
            previous_gaze_info: BoundedVecDeque::new(60),
            previous_aruco_info: BoundedVecDeque::new(60),
        }
    }

    pub(crate) fn run(&mut self) {
        println!("VisionActor started");
        while self.alive {
            if self.alive {
                self.request_camera_frame();
                self.request_gaze_info();
            }
            let mut pending = true;
            while pending {
                if let Ok(message) = self.receiver.try_recv() {
                    self.handle_message(message);
                } else {
                    pending = false;
                }
                if self.attention {
                    let aruco_result = vision_controller::detect_aruco(&mut self.previous_frames.back().unwrap()).unwrap();
                    self.previous_aruco_info.push_back(aruco_result);
                } else {
                    self.previous_aruco_info.push_back((VectorOfVectorOfPoint2f::new(), Vector::new()));
                }
                self.send_request_marker_info();
            }
            thread::sleep(Duration::from_millis(33));
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(_) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame { send_from: _,
                                                        send_to: _,
                                                        frame_data_bytes,
                                                        height,}) => {
                self.handle_frame_data_bytes(frame_data_bytes, height);
            },
            SmartSpeakerMessage::RequestGazeInfo(RequestGazeInfo { send_from: _, send_to: _, gaze_info }) => {
                self.handle_gaze_info(gaze_info);
            },
            SmartSpeakerMessage::RequestStateUpdate(_) => {
                self.attention = true;
            },
            // SmartSpeakerMessage::AttentionFinished(_) => {
            //     self.attention = false;
            // }
            _ => {}
        }
    }

    fn handle_frame_data_bytes(&mut self, frame_data_bytes: Vec<u8>, height: i32) {
        match vision_controller::data_bytes_to_mat(frame_data_bytes, height) {
            Ok(frame) => {
                self.previous_frames.push_back(frame);
            }
            Err(_) => {}
        };
    }

    fn handle_gaze_info(&mut self, (x, y): (f32, f32)) {
        self.previous_gaze_info.push_back((x, y));
    }

    fn request_camera_frame(&self) {
        camera_frame_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::CameraActor, vec![], 0);
    }

    fn request_gaze_info(&self) {
        gaze_info_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::GazeActor, (0., 0.));
    }

    fn send_request_marker_info(&self) {
        marker_info_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::CoreActor, self.previous_aruco_info.back().unwrap().clone());
    }

    fn handle_attention(&mut self) {
        self.attention = true;
        // let capture = Capture::new(self.previous_frames.clone());
        // let gaze_info = vision_controller::get_gaze_info(capture);
        // camera_frame_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::ContextActor, vec![], 0);
    }
}

