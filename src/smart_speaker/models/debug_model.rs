use std::ops::Mul;
use opencv::{prelude::*, highgui, core::Point2f, core::Vector, types::VectorOfPoint2f, types::VectorOfVectorOfPoint2f};
use crate::smart_speaker::controllers::vision_controller;
use crate::smart_speaker::controllers::debug_controller;
use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::marker_model::{IngredientMarker};
use crate::utils::vision_util;

pub(crate) struct DebugData {
    pub activated: bool,
    pub frame: Option<Mat>,
    pub gaze_x: f32,
    pub gaze_y: f32,
    pub state: SmartSpeakerState,
    pub last_marker_info: (VectorOfVectorOfPoint2f, Vector<i32>)
}

impl DebugData {
    pub(crate) fn new(activate: bool) -> Self {
        Self {
            activated: activate,
            frame: None,
            gaze_x: 0.,
            gaze_y: 0.,
            state: SmartSpeakerState::Idle,
            last_marker_info: (VectorOfVectorOfPoint2f::new(), Vector::new()),
        }
    }

    pub(crate) fn print(&mut self) {
        match &mut self.frame {
            Some(frame) => {
                match &self.state {
                    SmartSpeakerState::Idle => {
                        debug_controller::write_text_to_mat(frame, "Waiting for wake word...", 10, 40);
                    }
                    SmartSpeakerState::Attention => {
                        debug_controller::write_text_to_mat(frame, "Attention to user", 10, 40);
                    }
                    SmartSpeakerState::Pending(pending_type) => {
                        debug_controller::write_text_to_mat(frame, &format!("pending for {}", pending_type), 10, 40);
                    }
                }
                debug_controller::write_text_to_mat(frame, &format!("Gaze: ({}, {})", self.gaze_x, self.gaze_y), 10, 20);
                debug_controller::draw_circle_to_mat(frame, self.gaze_x.mul(frame.cols() as f32) as i32, frame.rows() - self.gaze_y.mul(frame.rows() as f32) as i32);
                debug_controller::draw_aruco(frame, &self.last_marker_info.0, &self.last_marker_info.1);
                for i in 0..self.last_marker_info.1.len() {
                    let square = self.last_marker_info.0.get(i).unwrap();
                    let square_mid = vision_util::midpoint(
                        &square.get(0).unwrap().x,
                        &square.get(0).unwrap().y,
                        &square.get(2).unwrap().x,
                        &square.get(2).unwrap().y
                    );
                    debug_controller::write_text_to_mat(frame, &format!("{}: ({}, {})", IngredientMarker::from(self.last_marker_info.1.get(i).unwrap() as u32), square_mid.0, square_mid.1), 10, 60 + i as i32 * 20);
                }
                highgui::imshow("Debug Screen", frame).unwrap();
                highgui::wait_key(1).unwrap();
            },
            None => {
                // dbg!("No frame to print"); // cause by cold start (no frame yet or real camera is not ready)
            }
        }
    }

    pub(crate) fn update_frame(&mut self, frame_data_bytes: &Vec<u8>, height: i32) {
        if let Ok(frame) = vision_controller::data_bytes_to_mat(frame_data_bytes.clone(), height) {
            // let frame = vision_controller::resize_frame(frame);
            self.frame = Some(frame);
        }
    }

    pub(crate) fn update_gaze_info(&mut self, (gaze_x, gaze_y): &(f32, f32)) {
        self.gaze_x = gaze_x.clone();
        self.gaze_y = gaze_y.clone();
    }

    pub(crate) fn update_marker_info(&mut self, marker_info: &(Vec<Vec<(f32, f32)>>, Vec<i32>)) {
        self.last_marker_info = (
            VectorOfVectorOfPoint2f::from_iter(marker_info.0.iter().map(|vec| {
                VectorOfPoint2f::from_iter(vec.iter().map(|(x, y)| {
                    Point2f::new(x.clone(), y.clone()) // original size
                    // Point2f::new(x.clone() / 2.0, y.clone() / 2.0) // resize to half
                }))
            })),
            Vector::from_iter(marker_info.1.iter().map(|id| {
                id.clone()
            }))
            );
    }

    pub(crate) fn update_state(&mut self, state: SmartSpeakerState) {
        self.state = state;
    }
}
