use std::ops::Mul;
use opencv::{prelude::*, highgui, core::Point2f};
use opencv::core::Point;
use crate::smart_speaker::controllers::vision_controller;
use crate::smart_speaker::controllers::debug_controller;
use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::vision_model;
use crate::utils::vision_util;

pub(crate) struct DebugData {
    pub activated: bool,
    pub frame: Option<Mat>,
    pub gaze_x: f32,
    pub gaze_y: f32,
    pub state: SmartSpeakerState,
}

impl DebugData {
    pub(crate) fn new(activate: bool) -> Self {
        Self {
            activated: activate,
            frame: None,
            gaze_x: 0.,
            gaze_y: 0.,
            state: SmartSpeakerState::Idle,
        }
    }

    pub(crate) fn force_cocoa_loop(&self) {
        // Force to create a frame to display (for TTS callback)
        let display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::all(0.)).unwrap();
        highgui::imshow("Debug Screen", &display_frame).unwrap();
        highgui::wait_key(1).unwrap();
    }

    pub(crate) fn print(&mut self) {
        match &self.frame {
            Some(frame) => {
                let mut display_frame: Mat = Default::default();
                frame.copy_to(&mut display_frame).unwrap();
                match &self.state {
                    SmartSpeakerState::Idle => {
                        debug_controller::write_text_to_mat(&mut display_frame, "Waiting for wake word...", 10, 40);
                    }
                    SmartSpeakerState::Attention => {
                        debug_controller::write_text_to_mat(&mut display_frame, "Attention to user", 10, 40);
                    }
                    SmartSpeakerState::WaitingForInteraction(pending_type) => {
                        debug_controller::write_text_to_mat(&mut display_frame, &format!("pending for {}", pending_type), 10, 40);
                    }
                }
                debug_controller::write_text_to_mat(&mut display_frame, &format!("Gaze: ({}, {})", self.gaze_x, self.gaze_y), 10, 20);
                debug_controller::draw_circle_to_mat(&mut display_frame, self.gaze_x.mul(frame.cols() as f32) as i32, frame.rows() - self.gaze_y.mul(frame.rows() as f32) as i32);

                // Begin debug for object detection
                let (aruco_contours, aruco_index) = vision_controller::detect_aruco(frame).unwrap();
                let (width_ratios, height_ratios) = vision_util::get_measure_criteria_from_aruco(&aruco_contours).unwrap();

                // For ArUco debug print
                debug_controller::draw_aruco(&mut display_frame, &aruco_contours, &aruco_index);
                for i in 0..aruco_contours.len() {
                    let square = vision_util::get_min_rect2f(&aruco_contours.get(i).unwrap());
                    let mut points = [Point2f::default(); 4];
                    square.points(&mut points).unwrap();
                    let width = vision_util::distance(&points[1].x, &points[1].y, &points[2].x, &points[2].y);
                    let height = vision_util::distance(&points[0].x, &points[0].y, &points[1].x, &points[1].y);
                    let points_width = vision_util::pixel_to_metric(
                        width,
                        &width_ratios.get(i).unwrap());
                    let points_height = vision_util::pixel_to_metric(
                        height,
                        &height_ratios.get(i).unwrap());
                    debug_controller::write_text_to_mat(&mut display_frame, &format!("{}: ({:.1}x{:.1}) cm", aruco_index.get(i).unwrap() as u32, points_width, points_height), square.center.x as i32, square.center.y as i32 + 20 );
                }

                // For object detection debug print
                let masked = vision_util::mask_object(frame, vision_model::DetectableObject::Carrot).unwrap();
                match vision_controller::detect_target_objects(frame, &vision_model::DetectableObject::Carrot) {
                    Ok(objects) => {
                        let shapes = vision_controller::detect_object_shape(&objects).unwrap();
                        debug_controller::write_text_to_mat(&mut display_frame, &format!("Contour: {}", &objects.len()), 10, 60);

                        match vision_controller::measure_object_size_by_aruco(&aruco_contours, &objects) {
                            Ok(measure_result) => {
                                for i in 0..measure_result.len() {
                                    let rect = vision_util::get_min_rect2f(&objects.get(i).unwrap());
                                    let object_size = measure_result.get(i).unwrap();
                                    let shape_poly = vision_util::get_approx_poly_dp(&objects.get(i).unwrap().iter().map(|c| Point::new(c.x as i32, c.y as i32)).collect(), true);

                                    debug_controller::draw_rotated_rect_to_mat(&mut display_frame, &rect);
                                    debug_controller::draw_approx_poly_to_mat(&mut display_frame, &objects.get(i).unwrap());
                                    debug_controller::draw_approx_poly_to_mat(&mut display_frame, &shape_poly);
                                    debug_controller::write_text_to_mat(&mut display_frame, &format!("Object: {:.1} cm^2 ({:.1}x{:.1}) cm\nShape: {}", object_size.perimeter, object_size.width, object_size.height, &shapes.get(i).unwrap().to_i18n().en), rect.center.x as i32, rect.center.y as i32 + 20 );
                                }
                            }
                            Err(_) => {
                            }
                        }
                    }
                    Err(_) => {
                    }
                }

                highgui::imshow("Debug Screen", &display_frame).unwrap();
                // highgui::imshow("Debug Screen 2", &masked).unwrap();
                highgui::wait_key(1).unwrap();
            },
            None => {
                // dbg!("No frame to print"); // cause by cold start (no frame yet or real camera is not ready)
            }
        }
    }

    pub(crate) fn update_frame(&mut self, frame_data_bytes: &Vec<u8>, height: &i32) {
        if let Ok(frame) = vision_controller::data_bytes_to_mat(frame_data_bytes.clone(), height.clone()) {
            // let frame = vision_controller::resize_frame(frame);
            self.frame = Some(frame);
        }
    }

    pub(crate) fn update_gaze_info(&mut self, (gaze_x, gaze_y): &(f32, f32)) {
        self.gaze_x = gaze_x.clone();
        self.gaze_y = gaze_y.clone();
    }

    // pub(crate) fn update_marker_info(&mut self, marker_info: &(Vec<Vec<(f32, f32)>>, Vec<i32>)) {
    //     self.last_marker_info = (
    //         VectorOfVectorOfPoint2f::from_iter(marker_info.0.iter().map(|vec| {
    //             VectorOfPoint2f::from_iter(vec.iter().map(|(x, y)| {
    //                 Point2f::new(x.clone(), y.clone()) // original size
    //                 // Point2f::new(x.clone() / 2.0, y.clone() / 2.0) // resize to half
    //             }))
    //         })),
    //         Vector::from_iter(marker_info.1.iter().map(|id| {
    //             id.clone()
    //         }))
    //         );
    // }

    pub(crate) fn update_state(&mut self, state: SmartSpeakerState) {
        self.state = state;
    }
}
