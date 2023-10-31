use std::fmt;
use std::fmt::{Debug, Formatter};
use opencv::core::{Mat, Point2f};
use anyhow::Result;
use opencv::types::{VectorOfPoint2f, VectorOfVectorOfPoint2f};
use crate::utils::camera_util::Camera;
use crate::utils::pupil_util::Pupil;
use crate::utils::vision_util::VisionType;

/// A capture source is a source of frames, such as a camera or a pupil remote.
pub(crate) struct Capture {
    pub source: Option<Box<(dyn CaptureSource + Send + 'static)>>,
}

impl Capture {
    /// Create a new capture source.
    pub fn new() -> Self {
        Self {
            source: None,
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


// pub(crate) struct DetectedMarker {
//     pub corner: VectorOfPoint2f,
//     pub id: i32,
//     pub centroid: Point2f,
// }
//
// impl DetectedMarker {
//     fn new(corner: Vector<Point2f>, id: i32, centroid: Point2f) -> Self {
//         Self {
//             corner,
//             id,
//             centroid,
//         }
//     }
//
//     fn update(mut self, corner: Vector<Point2f>, id: i32, centroid: Point2f) {
//         self.corner = corner;
//         self.id = id;
//         self.centroid = centroid;
//     }
//
//     fn default() -> Self {
//         Self {
//             corner: Default::default(),
//             id: 0,
//             centroid: Default::default(),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DetectableObject {
    Carrot,
    HumanSkin,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VisionAction {
    None,
    ObjectDetectionWithAruco(DetectableObject),
}

pub(crate) trait VisionSlot: Send {
    fn clone_box(&self) -> Box<dyn VisionSlot>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Debug for dyn VisionSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "IntentSlot")
    }
}

impl PartialEq for dyn VisionSlot {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn VisionSlot> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionObject {
    pub(crate) object_type: DetectableObject,
    pub(crate) size: VisionObjectSize,
}

impl VisionObject {
    pub(crate) fn new(object_type: DetectableObject, size: VisionObjectSize) -> Self {
        Self {
            object_type,
            size
        }
    }
}

impl VisionSlot for VisionObject {
    fn clone_box(&self) -> Box<dyn VisionSlot> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionObjectSize {
    pub(crate) perimeter: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl VisionObjectSize {
    pub(crate) fn new(perimeter: f32, width: f32, height: f32) -> Self {
        Self {
            perimeter,
            width,
            height,
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub(crate) enum VisionObject {
// }

