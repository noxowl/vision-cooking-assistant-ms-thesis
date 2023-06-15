use anyhow::{anyhow, Result};
use opencv::{objdetect, imgproc};
use opencv::prelude::*;
use opencv::core::{Point, Point2f, Scalar, Vector};
use opencv::types::{VectorOfi32, VectorOfPoint2f, VectorOfVectorOfPoint2f};
use crate::smart_speaker::models::vision_model::{CameraCaptureSource, Capture, PupilCaptureSource};
use crate::utils::camera_util::Camera;
use crate::utils::pupil_util::{Pupil, PupilRemote};
use crate::utils::vision_util::VisionType;

pub(crate) fn vision_loop(capture: &mut Capture) -> Result<()> {
    loop {
        match capture.source.as_mut() {
            Some(source) => {
                let frame = source.get_frame()?;
                // let (corners, ids) = find_aruco(&frame)?;
                match capture.get_source_type() {
                    None => {}
                    Some(source_type) => {
                        match source_type {
                            VisionType::BuiltInCamera => {
                                // let frame = draw_aruco(&frame, &corners, &ids)?;
                                capture.update(frame);
                            }
                            VisionType::Pupil => {
                                // let frame = draw_aruco(&frame, &corners, &ids)?;
                                capture.update(frame);
                            }
                            _ => {}
                        }
                    }
                }
                // let gaze = find_gaze(&frame)?;
                // let nearest_aruco = find_nearest_aruco(&gaze, &corners, &ids)?;
                // println!("nearest_aruco: {:?}", nearest_aruco);
            }
            None => {
                println!("no source");
            }
        }
    }
}

pub(crate) fn set_pupil_capture(capture: &mut Capture) -> Result<()> {
    capture.source = Some(Box::new(
        PupilCaptureSource::new(
            Pupil::new(PupilRemote {}))));
    Err(anyhow!("not implemented"))
}

pub(crate) fn set_camera_capture(capture: &mut Capture) -> Result<()> {
    capture.source = Some(Box::new(
        CameraCaptureSource::new(
            Camera::new()?)));
    Ok(())
}




pub(crate) struct DetectedMarker {
    pub corner: VectorOfPoint2f,
    pub id: i32,
    pub centroid: Point2f,
}

impl DetectedMarker {
    fn new(corner: Vector<Point2f>, id: i32, centroid: Point2f) -> Self {
        Self {
            corner,
            id,
            centroid,
        }
    }

    fn update(mut self, corner: Vector<Point2f>, id: i32, centroid: Point2f) {
        self.corner = corner;
        self.id = id;
        self.centroid = centroid;
    }

    fn default() -> Self {
        Self {
            corner: Default::default(),
            id: 0,
            centroid: Default::default(),
        }
    }
}

// pub  fn find_nearest_aruco(gaze: &(f32, f32), corners: &VectorOfVectorOfPoint2f, ids: &Vector<i32>) -> Result<DetectedMarker> {
//     let mut nearest_index = 0;
//     if ids.len() > 0 {
//         let mut nearest_distance: f32 = 0.0;
//         for i in 0..ids.len() {
//             let square = corners.get(i).unwrap();
//             let square_mid = vision_util::midpoint(
//                 &square.get(0).unwrap(),
//                 &square.get(2).unwrap());
//             let dist = vision_util::distance(
//                 &gaze.0, &gaze.1,
//                 &);
//             if i == 0 || &dist < &nearest_distance {
//                 nearest_distance = dist;
//                 *&nearest_index.clone_from(&i);
//             }
//         }
//     }
//     Ok(DetectedMarker::new(
//         corners.get(nearest_index).unwrap(),
//         ids.get(nearest_index).unwrap(),
//         vision_util::midpoint(&corners.get(nearest_index).unwrap().get(0).unwrap(),&corners.get(nearest_index).unwrap().get(2).unwrap())
//     ))
// }

pub(crate) fn detect_aruco(frame: &Mat, max_markers: usize) -> Result<(VectorOfVectorOfPoint2f, Vector<i32>)> {
    let parameters = opencv::objdetect::DetectorParameters::default()?;
    let dictionary = opencv::objdetect::get_predefined_dictionary(objdetect::PredefinedDictionaryType::DICT_4X4_50)?;
    let mut corners: VectorOfVectorOfPoint2f = Default::default();
    let mut rejected: VectorOfVectorOfPoint2f = Default::default();
    let mut ids = VectorOfi32::default();
    let detector = objdetect::ArucoDetector::new(&dictionary, &parameters,
                                                 objdetect::RefineParameters::new(10., 3., true).unwrap())?;
    detector.detect_markers(frame, &mut corners, &mut ids, &mut rejected).expect("TODO: panic message");
    Ok((corners, ids))
}

pub(crate) fn debug_draw_aruco(frame: &mut Mat, corners: &VectorOfVectorOfPoint2f, ids: &VectorOfi32) {
    objdetect::draw_detected_markers(frame, corners, ids, Scalar::new(255., 0., 0., 255.)).unwrap();
}

pub(crate) fn debug_put_text(frame: &mut Mat, text: &str, pt: [i32; 2]) {
    imgproc::put_text(frame, text, Point::new(pt[0], pt[1]), imgproc::FONT_ITALIC, 0.5, Scalar::new(0., 255., 0., 255.),
                      2, 0, false).unwrap();
}

pub(crate) fn debug_draw_marker(frame: &mut Mat, pt: [i32; 2], colour: [f64; 3]) {
    imgproc::draw_marker(frame, Point::new(pt[0], pt[1]),
                         Scalar::new(colour[0],
                                     colour[1],
                                     colour[2], 255.), 0, 0, 3, 0).unwrap();
}
