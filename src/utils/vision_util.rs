use std::str::FromStr;
use anyhow::{anyhow, Result};
use opencv::core::{Point2f, in_range, Size, Point, bitwise_and, BORDER_DEFAULT, Vector};
use opencv::imgproc::{self, arc_length, cvt_color};
use opencv::prelude::*;
use opencv::types::VectorOfPoint2f;
use crate::smart_speaker::models::vision_model::{CameraCaptureSource, Capture, PupilCaptureSource};
use crate::utils::camera_util::Camera;
use crate::utils::pupil_util::{Pupil, PupilRemote};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VisionType {
    None,
    Pupil,
    BuiltInCamera,
}

impl FromStr for VisionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(VisionType::None),
            "false" => Ok(VisionType::None),
            "pupil" => Ok(VisionType::Pupil),
            "built-in-camera" => Ok(VisionType::BuiltInCamera),
            "camera" => Ok(VisionType::BuiltInCamera),
            "cam" => Ok(VisionType::BuiltInCamera),
            _ => Err(anyhow!("invalid vision type")),
        }
    }
}

// pub(crate) fn generate_vision_capture_source(source_type: &VisionType) -> Result<Arc<RwLock<dyn CaptureSource + 'static>>> {
//     match source_type {
//         VisionType::Pupil => {
//             let source = Arc::new(RwLock::new(
//                 PupilCaptureSource::new(
//                     Pupil::new(PupilRemote {}))));
//             Ok(source)
//         }
//         VisionType::BuiltInCamera => {
//             let source = Arc::new(RwLock::new(
//                 CameraCaptureSource::new(
//                     Camera::new()?)));
//             Ok(source)
//         }
//         _ => {
//             Err(anyhow!("invalid vision type"))
//         }
//     }
// }

pub(crate) fn set_pupil_capture(capture: &mut Capture, zmq_endpoint: String) -> Result<()> {
    capture.source = Some(Box::new(
        PupilCaptureSource::new(
            Pupil::new(PupilRemote::new(zmq_endpoint, "frame")))));
    Ok(())
}

pub(crate) fn set_camera_capture(capture: &mut Capture) -> Result<()> {
    capture.source = Some(Box::new(
        CameraCaptureSource::new(
            Camera::new()?)));
    Ok(())
}


pub(crate) fn centroid_of_frame(x: u32, y: u32) -> (f32, f32) {
    if x > 0 && x > 0 {
        ((x / 2) as f32, (y / 2) as f32)
    } else {
        (0., 0.)
    }
}

pub(crate) fn midpoint(px_x: &f32, px_y: &f32, py_x: &f32, py_y: &f32) -> (f32, f32) {
    ((px_x + py_x) / 2., (px_y + py_y) / 2.)
}

pub(crate) fn distance(px_x: &f32, px_y: &f32, py_x: &f32, py_y: &f32) -> f32 {
    ((py_x - px_x).abs().powi(2) + (py_y - px_y).abs().powi(2)).sqrt()
}

pub(crate) fn aruco_perimeter_ratio(corners: VectorOfPoint2f) -> f64 {
    let aruco_perimeter = arc_length(&corners, true).unwrap();
    return aruco_perimeter / 20.
}

/// get actual size of object by aruco marker
pub(crate) fn pixel_to_metric(target: f64, ratio: f64) -> f64 {
    target / ratio
}

pub(crate) fn mask_object(frame: &Mat, target: DetectableObject) -> Result<Mat> {
    let mut hsv = Mat::default();
    let mut mask = Mat::default();
    let mut dst = Mat::default();
    cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0).unwrap();
    match target {
        DetectableObject::Carrot => {
            in_range(
                &hsv,
                &Vector::from_slice(&[35., 155., 255.]),
                &Vector::from_slice(&[26., 255., 255.]),
                &mut mask,
            )?;
        },
        DetectableObject::HumanSkin => {
            in_range(
                &hsv,
                &Vector::from_slice(&[0., 48., 80.]),
                &Vector::from_slice(&[20., 255., 255.]),
                &mut mask,
            )?;
        }
    }
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        Size::new(3, 3), Default::default()).unwrap();
    imgproc::erode(
        &mask,
        &mut dst,
        &kernel,
        Point::new(-1, -1),
        2, 0, Default::default()).unwrap();
    imgproc::dilate(
        &dst,
        &mut mask,
        &kernel,
        Point::new(-1, -1),
        2, 0, Default::default()).unwrap();

    imgproc::gaussian_blur(&mask, &mut dst, Size::new(3, 3), 0.,0., BORDER_DEFAULT).unwrap();
    bitwise_and(frame, frame, &mut mask, &dst).unwrap();
    imgproc::cvt_color(&mask, &mut dst, imgproc::COLOR_HSV2BGR, 0).unwrap();
    imgproc::threshold(&dst, &mut mask, 60., 255., imgproc::THRESH_BINARY).unwrap();
    Ok(mask)
}

pub(crate) enum DetectableObject {
    Carrot,
    HumanSkin,
}

// pub  fn draw_debug_frame(frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>, halt: &Cell<bool>) {
//     while !halt.get() {
//         {
//             let mut frame_lock = frame.lock().unwrap();
//             let mut gaze_lock = gaze.lock().unwrap();
//             visio::debug_draw_marker(&mut *frame_lock, [gaze_lock.x as i32, gaze_lock.y as i32], [255., 0., 0.]);
//             highgui::imshow("visio - debug", &*frame_lock).unwrap();
//         }
//         highgui::wait_key(10).unwrap();
//         tokio::time::sleep(Duration::from_millis(33));
//     }
// }