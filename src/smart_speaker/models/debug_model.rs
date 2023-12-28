use std::ops::Mul;
use opencv::{prelude::*, highgui, core::Point2f, imgproc};
use opencv::core::{Point, Scalar};
use crate::smart_speaker::controllers::vision_controller;
use crate::smart_speaker::controllers::debug_controller;
use crate::smart_speaker::models::core_model::{SmartSpeakerState, WaitingInteraction};
use crate::smart_speaker::models::message_model::SmartSpeakerActors;
use crate::smart_speaker::models::vision_model;
use crate::utils::vision_util;

pub(crate) struct DebugData {
    pub activated: bool,
    pub frame: Option<Mat>,
    pub gaze_x: f32,
    pub gaze_y: f32,
    pub gaze_as_px: (i32, i32),
    pub state: (SmartSpeakerState, SmartSpeakerActors),
}

impl DebugData {
    pub(crate) fn new(activate: bool) -> Self {
        Self {
            activated: activate,
            frame: None,
            gaze_x: 0.,
            gaze_y: 0.,
            gaze_as_px: (0, 0),
            state: (SmartSpeakerState::Idle, SmartSpeakerActors::CoreActor),
        }
    }

    pub(crate) fn force_cocoa_loop(&self) {
        // Force to create a frame to display (for TTS callback)
        let display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::all(0.)).unwrap();
        highgui::imshow("Debug Screen", &display_frame).unwrap();
        highgui::wait_key(1).unwrap();
    }

    pub(crate) fn indicator_loop(&self) {
        // Force to create a frame to display (for TTS callback)
        let mut display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::all(0.)).unwrap();
        match &self.state.0 {
            SmartSpeakerState::Idle => {
                // Black screen
                display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::all(0.)).unwrap();
                imgproc::put_text(&mut display_frame, "Waiting for Wake word 'Hey Ringo'ウェイクワードをお待ちしております / 等待唤醒", Point::new(240, 320), 1, 1., Scalar::new(255., 255., 255., 255.), 1, 0, false).unwrap();
            }
            SmartSpeakerState::Attention => {
                // Green Screen
                match &self.state.1 {
                    SmartSpeakerActors::SpeechToIntentActor => {
                        display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::new(0., 255., 0., 255.)).unwrap();
                        imgproc::put_text(&mut display_frame,  "Waiting for your command. 命令を言ってください / 说出命令", Point::new(240, 320), 1, 1., Scalar::new(0., 0., 0., 255.), 1, 0, false).unwrap();
                    }
                    SmartSpeakerActors::VoiceActivityDetectActor => {
                        display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::all(0.)).unwrap();
                        imgproc::put_text(&mut display_frame, "Waiting for your speech begin.  お話の準備ができたかどうかお待ちしております / 我们在等着看你是否准备好发言了", Point::new(240, 320), 1, 1., Scalar::new(255., 255., 255., 255.), 1, 0, false).unwrap();
                    }
                    _ => {}
                }

            }
            SmartSpeakerState::WaitingForInteraction(pending_type) => {
                // Yellow screen
                match pending_type {
                    WaitingInteraction::Speak => {
                        display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::new(0., 255., 255., 255.)).unwrap();
                        imgproc::put_text(&mut display_frame, "Waiting for your speech begin. お話の準備ができたかどうかお待ちしております / 我们在等着看你是否准备好发言了", Point::new(240, 320), 1, 1., Scalar::new(0., 0., 0., 255.), 1, 0, false).unwrap();
                    }
                    WaitingInteraction::Vision(_) => {
                        display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::new(0., 255., 255., 255.)).unwrap();
                        imgproc::put_text(&mut display_frame, "Wait for Vision...", Point::new(240, 320), 1, 1., Scalar::new(0., 0., 0., 255.), 1, 0, false).unwrap();
                    }
                    _ => {
                    }
                }
            }
            SmartSpeakerState::Speaking => {
                // Navy screen
                display_frame = Mat::new_rows_cols_with_default(480, 640, opencv::core::CV_8UC3, opencv::core::Scalar::new(128., 0., 0., 255.)).unwrap();
                imgproc::put_text(&mut display_frame, "I'm talking. 私が話しています / 我说的是", Point::new(240, 320), 1, 1., Scalar::new(255., 255., 255., 255.), 1, 0, false).unwrap();
            }
        }
        highgui::imshow("Indicator Screen", &display_frame).unwrap();
        highgui::wait_key(1).unwrap();
    }

    pub(crate) fn print(&mut self) {
        let verbose = false;
        match &self.frame {
            Some(frame) => {
                let mut display_frame: Mat = Default::default();
                frame.copy_to(&mut display_frame).unwrap();
                self.gaze_as_px = vision_util::gaze_to_px(&(self.gaze_x, self.gaze_y), &(frame.cols(), frame.rows()));
                match &self.state.0 {
                    SmartSpeakerState::Idle => {
                        debug_controller::write_text_to_mat(&mut display_frame, "Waiting for wake word...", 10, 40);
                    }
                    SmartSpeakerState::Attention => {
                        debug_controller::write_text_to_mat(&mut display_frame, "Attention to user", 10, 40);
                    }
                    SmartSpeakerState::WaitingForInteraction(pending_type) => {
                        debug_controller::write_text_to_mat(&mut display_frame, &format!("pending for {}", pending_type), 10, 40);
                    }
                    SmartSpeakerState::Speaking => {
                        debug_controller::write_text_to_mat(&mut display_frame, "Speaking", 10, 40);
                    }
                }
                debug_controller::write_text_to_mat(&mut display_frame, &format!("Gaze: ({}, {})", self.gaze_x, self.gaze_y), 10, 20);
                debug_controller::draw_circle_to_mat(&mut display_frame, self.gaze_as_px.0, self.gaze_as_px.1);

                // Begin debug for object detection
                let (aruco_contours, aruco_index) = vision_controller::detect_aruco(frame).unwrap();
                let (width_ratios, height_ratios) = vision_util::get_measure_criteria_from_aruco(&aruco_contours).unwrap();

                // For ArUco debug print
                debug_controller::draw_aruco(&mut display_frame, &aruco_contours, &aruco_index);
                if verbose {
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
                                if measure_result.len() > 0 {
                                    for i in 0..measure_result.len() {
                                        let rect = vision_util::get_min_rect2f(&objects.get(i).unwrap());
                                        let object_size = measure_result.get(i).unwrap();
                                        let shape_poly = vision_util::get_approx_poly_dp(&objects.get(i).unwrap().iter().map(|c| Point::new(c.x as i32, c.y as i32)).collect(), true);

                                        debug_controller::draw_rotated_rect_to_mat(&mut display_frame, &rect);
                                        debug_controller::draw_approx_poly_to_mat(&mut display_frame, &objects.get(i).unwrap());
                                        debug_controller::draw_approx_poly_to_mat(&mut display_frame, &shape_poly);
                                        debug_controller::write_text_to_mat(&mut display_frame, &format!("Object: {:.1} cm^2 ({:.1}x{:.1}) cm\nShape: {}", object_size.perimeter, object_size.width, object_size.height, &shapes.get(i).unwrap().to_i18n().en), rect.center.x as i32, rect.center.y as i32 + 20 );
                                    }
                                    if aruco_index.len() > 0 {
                                        let (width_ratios, height_ratios) = vision_util::get_measure_criteria_from_aruco(&aruco_contours).unwrap();
                                        let ratios = width_ratios.iter().zip(height_ratios.iter()).map(|(a, b)| a * b).collect::<Vec<f32>>();
                                        let gaze_as_pxf = vision_util::gaze_to_pxf(&(self.gaze_x, self.gaze_y), &(frame.cols(), frame.rows()));
                                        let nearest_info = vision_controller::find_nearest_object_from_gaze(&(gaze_as_pxf.0, gaze_as_pxf.1), &objects).unwrap();
                                        let mut distance_candidates = vec![];
                                        for r in ratios {
                                            distance_candidates.push(vision_util::pixel_to_metric(
                                                nearest_info.1,
                                                &r));
                                        }
                                        let distance_as_metric = distance_candidates.iter().sum::<f32>() / distance_candidates.len() as f32;
                                        let rect = vision_util::get_min_rect2f(&objects.get(nearest_info.0).unwrap());
                                        debug_controller::draw_line_to_mat(&mut display_frame, self.gaze_as_px.0, self.gaze_as_px.1, rect.center.x as i32, rect.center.y as i32);
                                        debug_controller::draw_rotated_rect_to_mat(&mut display_frame, &rect);
                                        debug_controller::write_text_to_mat(&mut display_frame, &format!("Nearest from gaze: {} cm", distance_as_metric), rect.center.x as i32, rect.center.y as i32 + 50 );



                                    }
                                }
                            }
                            Err(_) => {
                            }
                        }
                    }
                    Err(_) => {
                    }
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

    pub(crate) fn update_state(&mut self, state: SmartSpeakerState, actor: SmartSpeakerActors) {
        self.state = (state, actor);
    }
}
