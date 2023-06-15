use std::str::FromStr;
use std::sync::{Arc, RwLock};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::vision_model::{CameraCaptureSource, CaptureSource, PupilCaptureSource};
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

pub(crate) fn generate_vision_capture_source(source_type: &VisionType) -> Result<Arc<RwLock<dyn CaptureSource + 'static>>> {
    match source_type {
        VisionType::Pupil => {
            let source = Arc::new(RwLock::new(
                PupilCaptureSource::new(
                    Pupil::new(PupilRemote {}))));
            Ok(source)
        }
        VisionType::BuiltInCamera => {
            let source = Arc::new(RwLock::new(
                CameraCaptureSource::new(
                    Camera::new()?)));
            Ok(source)
        }
        _ => {
            Err(anyhow!("invalid vision type"))
        }
    }
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