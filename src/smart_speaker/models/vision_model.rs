use std::any::{Any, TypeId};
use ::bounded_vec_deque::BoundedVecDeque;
use opencv::core::Mat;
use anyhow::Result;
use crate::utils::camera_util::Camera;
use crate::utils::pupil_util::Pupil;
use crate::utils::vision_util::VisionType;

/// A capture source is a source of frames, such as a camera or a pupil remote.
pub(crate) struct Capture {
    pub source: Option<Box<(dyn CaptureSource + Send + 'static)>>,
    // pub frame: Option<Mat>,
    // pub previous_frames: BoundedVecDeque<Mat>,
}

impl Capture {
    /// Create a new capture source.
    pub fn new() -> Self {
        Self {
            source: None,
            // frame: None,
            // previous_frames: BoundedVecDeque::new(600),
        }
    }

    pub fn info(&self) {
        if self.source.is_some() {
            println!("Capture info: {:?}", self.source.as_ref().unwrap().get_vision_type());
        }
    }

    // pub fn update(&mut self, frame: Mat) {
    //     self.frame = Some(frame.clone());
    //     self.previous_frames.push_back(frame.clone());
    // }

    pub fn update_source(&mut self, source: Box<(dyn CaptureSource + Send + 'static)>) {
        self.source = Some(source);
    }

    // pub fn get_frame(&mut self) -> Option<Mat> {
    //     self.frame.clone()
    // }

    // pub fn get_previous_frames(&self) -> BoundedVecDeque<Mat> {
    //     self.previous_frames.clone()
    // }

    pub fn get_source_type(&self) -> Option<VisionType> {
        match &self.source {
            None => { None }
            Some(source) => {
                Some(source.get_vision_type())
            }
        }
    }
}

pub(crate) trait CaptureSource {
    fn get_vision_type(&self) -> VisionType;
    fn get_frame(&mut self) -> Result<Mat>;
}

pub(crate) struct PupilCaptureSource {
    pub pupil: Pupil,
}

impl PupilCaptureSource {
    pub fn new(pupil: Pupil) -> Self {
        Self { pupil }
    }
}

impl CaptureSource for PupilCaptureSource {
    fn get_vision_type(&self) -> VisionType {
        VisionType::Pupil
    }
    fn get_frame(&mut self) -> Result<Mat> {
        self.pupil.get_frame()
    }
}

pub(crate) struct CameraCaptureSource {
    pub camera: Camera,
}

impl CameraCaptureSource {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }
}

impl CaptureSource for CameraCaptureSource {
    fn get_vision_type(&self) -> VisionType {
        VisionType::BuiltInCamera
    }
    fn get_frame(&mut self) -> Result<Mat> {
        self.camera.get_frame()
    }
}

/// A marker is a point of interest in the frame.
pub(crate) struct Marker {

}
