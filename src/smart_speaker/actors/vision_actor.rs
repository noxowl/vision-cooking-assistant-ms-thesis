use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use anyhow::{anyhow, Result};
use bounded_vec_deque::BoundedVecDeque;
use opencv::{core::Mat, core::Vector, types::VectorOfVectorOfPoint2f};
use opencv::prelude::MatTraitConst;
use crate::smart_speaker::controllers::vision_controller;
use crate::smart_speaker::models::core_model::{WaitingInteraction, SmartSpeakerState};
use crate::smart_speaker::models::vision_model::{DetectableObject, DetectionDetail, DetectionMode, VisionAction, VisionObject, VisionSlot};
use crate::smart_speaker::models::message_model::*;
use crate::utils::message_util::*;
use crate::utils::vision_util;

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
            previous_frames: BoundedVecDeque::new(30),
            previous_gaze_info: BoundedVecDeque::new(30),
            previous_aruco_info: BoundedVecDeque::new(30),
        }
    }

    pub(crate) fn run(&mut self) {
        write_log_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerLogMessageType::Info("VisionActor started".to_string()));
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
            SmartSpeakerMessage::RequestCameraFrame(CameraFrameMessage { send_from: _,
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
            SmartSpeakerMessage::RequestGazeInfo(GazeInfoMessage { send_from: _, send_to: _, gaze_info }) => {
                self.handle_gaze_info(gaze_info);
            },
            SmartSpeakerMessage::RequestStateUpdate(StateUpdateMessage { send_from: _, send_to: _, state }) => {
                match state {
                    SmartSpeakerState::WaitingForInteraction(p) => {
                        match p {
                            WaitingInteraction::Vision(actions) => {
                                let mut result: Vec<VisionContent> = Vec::new();
                                for action in actions {
                                    match action {
                                        VisionAction::None => {}
                                        VisionAction::ObjectDetection(detail) => {
                                            match detail.detection_mode {
                                                DetectionMode::None => {}
                                                DetectionMode::Aruco => {
                                                    match self.handle_object_detection_with_aruco(detail.clone()) {
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
                                    }
                                }
                                self.send_vision_finalized(ProcessResult::Success, result);
                            }
                            _ => {
                                self.send_vision_finalized(ProcessResult::Failure, vec![]);
                            }
                        }
                    }
                    _ => {
                        self.send_vision_finalized(ProcessResult::Failure, vec![]);
                    }
                }
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

    fn handle_object_detection_with_aruco(&self, detail: DetectionDetail) -> Result<VisionContent> {
        match self.previous_aruco_info.back() {
            None => {
                Err(anyhow!("failed to detect target objects: no aruco data"))
            }
            Some((aruco, aruco_index)) => {
                match self.previous_frames.back() {
                    Some(frame) => {
                        match vision_controller::detect_target_objects(frame, &detail.detectable) {
                            Ok(objects) => {
                                write_log_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerLogMessageType::Debug(format!("Detected objects: {}", &objects.len())));
                                let shapes = vision_controller::detect_object_shape(&objects).unwrap();
                                match vision_controller::measure_object_size_by_aruco(aruco, &objects) {
                                    Ok(measure_result) => {
                                        write_log_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerLogMessageType::Debug(format!("Measured objects: {:?}", &measure_result)));
                                        if detail.gaze_assist {
                                            match self.previous_gaze_info.back() {
                                                Some((x, y)) => {
                                                    let gaze_as_pxf = vision_util::gaze_to_pxf(&(*x, *y), &(frame.cols(), frame.rows()));
                                                    let gaze_assist_result = vision_controller::find_nearest_object_from_gaze(&gaze_as_pxf, &objects);
                                                    match gaze_assist_result {
                                                        Ok(result) => {
                                                            let content_result = VisionContent::new(
                                                                VisionAction::ObjectDetection(detail.clone()),
                                                                vec![Box::new(VisionObject::new(
                                                                    detail.detectable.clone(),
                                                                    measure_result.get(result.0).unwrap().clone(),
                                                                    shapes.get(result.0).unwrap().clone(),
                                                                )) as Box<dyn VisionSlot>]
                                                            );
                                                            write_log_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerLogMessageType::Debug(format!("Content: {:?}", &content_result)));
                                                            Ok(content_result)
                                                        }
                                                        Err(e) => {
                                                            Err(anyhow!("failed to measure target objects: {}", e))
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    Err(anyhow!("failed to measure target objects: no gaze data"))
                                                }
                                            }
                                        } else {
                                            let content_result = VisionContent::new(
                                                VisionAction::ObjectDetection(detail.clone()),
                                                measure_result.iter().enumerate()
                                                    .map(|(i, object)| {
                                                        Box::new(VisionObject::new(
                                                            detail.detectable.clone(),
                                                            object.clone(),
                                                            shapes.get(i).unwrap().clone(),
                                                        )) as Box<dyn VisionSlot>
                                                    }).collect()
                                                );
                                            write_log_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerLogMessageType::Debug(format!("Content: {:?}", &content_result)));
                                            Ok(content_result)
                                        }
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

    fn send_vision_finalized(&self, result: ProcessResult, contents: Vec<VisionContent>) {
        vision_finalized_message(&self.sender, SmartSpeakerActors::VisionActor, SmartSpeakerActors::ContextActor, result, contents);
    }
}

