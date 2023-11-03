use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use anyhow::{anyhow, Result};
use bounded_vec_deque::BoundedVecDeque;
use opencv::{core::Mat, core::Vector, types::VectorOfVectorOfPoint2f};
use crate::smart_speaker::controllers::vision_controller;
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction, VisionObject, VisionSlot};
use crate::utils::message_util::{camera_frame_message, gaze_info_message, RequestCameraFrame, RequestGazeInfo, SmartSpeakerActors, SmartSpeakerMessage, RequestVisionAction, VisionContent, vision_finalized_message, ProcessResult};

pub(crate) struct VisionActor {
    alive: bool,
    debug: bool,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    previous_frames: BoundedVecDeque<Mat>,
    previous_gaze_info: BoundedVecDeque<(f32, f32)>,
    previous_aruco_info: BoundedVecDeque<(VectorOfVectorOfPoint2f, Vector<i32>)>,
}

impl VisionActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>, debug: bool) -> Self {
        Self {
            alive: true,
            debug,
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
                match self.previous_frames.back() {
                    Some(frame) => {
                        let aruco_result = vision_controller::detect_aruco(frame).unwrap();
                        self.previous_aruco_info.push_back(aruco_result);
                    }
                    None => {
                        self.previous_aruco_info.push_back((VectorOfVectorOfPoint2f::new(), Vector::new()));
                    }
                }
            },
            SmartSpeakerMessage::RequestGazeInfo(RequestGazeInfo { send_from: _, send_to: _, gaze_info }) => {
                self.handle_gaze_info(gaze_info);
            },
            SmartSpeakerMessage::RequestVisionAction(RequestVisionAction { send_from: _, send_to: _, actions }) => {
                let mut result: Vec<VisionContent> = Vec::new();
                for action in actions {
                    match action {
                        VisionAction::None => {}
                        VisionAction::ObjectDetectionWithAruco(target) => {
                            match self.handle_object_detection_with_aruco(target) {
                                Ok(content) => {
                                    result.push(content);
                                }
                                Err(_) => {
                                    self.send_vision_finalized(ProcessResult::Failure, vec![]);
                                }
                            }
                        }
                    }
                }
                self.send_vision_finalized(ProcessResult::Success, result);
            },
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

    fn handle_object_detection_with_aruco(&self, target: DetectableObject) -> Result<VisionContent> {
        match self.previous_aruco_info.back() {
            None => {
                Err(anyhow!("failed to detect target objects: no aruco data"))
            }
            Some((aruco, aruco_index)) => {
                match self.previous_frames.back() {
                    Some(frame) => {
                        match vision_controller::detect_target_objects(frame, &target) {
                            Ok(objects) => {
                                println!("Detected objects: {}", &objects.len());
                                match vision_controller::measure_object_size_by_aruco(aruco, &objects) {
                                    Ok(measure_result) => {
                                        let content_result = VisionContent::new(
                                            VisionAction::ObjectDetectionWithAruco(target.clone()),
                                            measure_result.iter()
                                                .map(|object| {
                                                    Box::new(VisionObject::new(
                                                        target.clone(),
                                                        object.clone())) as Box<dyn VisionSlot>
                                                }).collect()
                                        );
                                        Ok(content_result)
                                    }
                                    Err(_) => {
                                        Err(anyhow!("failed to measure target objects"))
                                    }
                                }
                            }
                            Err(_) => {
                                Err(anyhow!("failed to detect target objects"))
                            }
                        }
                    }
                    None => {
                        Err(anyhow!("failed to detect target objects: no frame data"))
                    }
                }
            }
        }
    }

    fn request_camera_frame(&self) {
        camera_frame_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::CameraActor, vec![], 0);
    }

    fn request_gaze_info(&self) {
        gaze_info_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::GazeActor, (0., 0.));
    }

    // fn send_request_marker_info(&self) {
    //     marker_info_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::CoreActor, self.previous_aruco_info.back().unwrap().clone());
    // }

    fn send_vision_finalized(&self, result: ProcessResult, contents: Vec<VisionContent>) {
        vision_finalized_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::ContextActor, result, contents);
    }
}

