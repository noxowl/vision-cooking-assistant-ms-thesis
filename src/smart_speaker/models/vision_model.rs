use std::fmt;
use std::fmt::{Debug, Formatter};
use opencv::core::Mat;
use anyhow::Result;
use crate::smart_speaker::models::message_model::SmartSpeakerI18nText;
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

    pub fn info(&self) -> String{
        if self.source.is_some() {
            format!("Capture info: {:?}", self.source.as_ref().unwrap().get_vision_type()).to_string()
        }
        else {
            format!("Capture info: None").to_string()
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

#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) struct DetectionDetail {
    pub(crate) detection_mode: DetectionMode,
    pub(crate) detectable: DetectableObject,
    pub(crate) gaze_assist: bool,
}

impl DetectionDetail {
    pub(crate) fn new(detection_mode: DetectionMode, detectable: DetectableObject, gaze_assist: bool) -> Self {
        Self {
            detection_mode,
            detectable,
            gaze_assist,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) enum DetectionMode {
    None,
    Aruco,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) enum DetectableObject {
    Carrot,
    HumanSkin,
}

impl DetectableObject {
    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            DetectableObject::Carrot => {
                SmartSpeakerI18nText::new()
                    .en("carrot")
                    .ja("にんじん")
                    .zh("胡萝卜")
                    .ko("당근")
            }
            DetectableObject::HumanSkin => {
                SmartSpeakerI18nText::new()
                    .en("human skin")
                    .ja("人肌")
                    .zh("人皮")
                    .ko("인간의 피부")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) enum VisionAction {
    None,
    ObjectDetection(DetectionDetail),
}

impl VisionAction {
    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            VisionAction::None => {
                SmartSpeakerI18nText::new()
                    .en("nothing")
                    .ja("何も")
                    .zh("什么都没有")
                    .ko("아무것도 없음")
            }
            VisionAction::ObjectDetection(object) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} with aruco", object.detectable.to_i18n().en))
                    .ja(&format!("アルコで{}を検出", object.detectable.to_i18n().ja))
                    .zh(&format!("使用 aruco 检测{}", object.detectable.to_i18n().zh))
                    .ko(&format!("aruco로 {}를 감지", object.detectable.to_i18n().ko))
            }
        }
    }

    pub(crate) fn expose_object(&self) -> Option<DetectableObject> {
        match self {
            VisionAction::None => { None }
            VisionAction::ObjectDetection(detail) => { Some(detail.detectable.clone()) }
        }
    }
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
    fn eq(&self, _: &Self) -> bool {
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
    pub(crate) shape: VisionObjectShape,
}

impl VisionObject {
    pub(crate) fn new(object_type: DetectableObject, size: VisionObjectSize, shape: VisionObjectShape) -> Self {
        Self {
            object_type,
            size,
            shape,
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VisionObjectShape {
    Triangle,
    Square,
    Rectangle,
    Circle,
    SemiCircle,
}

impl VisionObjectShape {
    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            VisionObjectShape::Triangle => {
                SmartSpeakerI18nText::new()
                    .en("triangle")
                    .ja("三角形")
                    .zh("三角形")
                    .ko("삼각형")
            }
            VisionObjectShape::Square => {
                SmartSpeakerI18nText::new()
                    .en("square")
                    .ja("正方形")
                    .zh("正方形")
                    .ko("정사각형")
            }
            VisionObjectShape::Rectangle => {
                SmartSpeakerI18nText::new()
                    .en("rectangle")
                    .ja("長方形")
                    .zh("长方形")
                    .ko("직사각형")
            }
            VisionObjectShape::Circle => {
                SmartSpeakerI18nText::new()
                    .en("circle")
                    .ja("円形")
                    .zh("圆形")
                    .ko("원형")
            }
            VisionObjectShape::SemiCircle => {
                SmartSpeakerI18nText::new()
                    .en("semi-circle")
                    .ja("半円形")
                    .zh("半圆形")
                    .ko("반원형")
            }
        }
    }
}
