use anyhow::{anyhow, Result};
use crate::utils::pupil_util::{Pupil, PupilRemote};
use crate::utils::vision_util::VisionType;

pub(crate) struct Gaze {
    pub source: Option<Box<(dyn GazeSource + Send + 'static)>>,
    pub fallback_x: f32,
    pub fallback_y: f32,
    pub x: f32,
    pub y: f32
}

impl Gaze {
    pub(crate) fn new(vision_type: VisionType, fallback_x: f32, fallback_y: f32, zmq_endpoint: String) -> Result<Self> {
        match vision_type {
            VisionType::Pupil => {
                let source = PupilGazeSource::new(zmq_endpoint);
                Ok(Self {
                    source: Some(Box::new(source)),
                    fallback_x,
                    fallback_y,
                    x: fallback_x,
                    y: fallback_y,
                })
            },
            _ => {
                Ok(Self {
                    source: None,
                    fallback_x,
                    fallback_y,
                    x: fallback_x,
                    y: fallback_y,
                })
            }
        }

    }

    pub(crate) fn update_gaze(&mut self) {
        match &mut self.source {
            None => {
                self.x = self.fallback_x;
                self.y = self.fallback_y;
            },
            Some(source) => {
                match source.get_gaze() {
                    Ok(result) => {
                        (self.x, self.y) = result;
                    }
                    Err(_) => {}
                }
            }
        }
    }

    pub(crate) fn get_gaze(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

pub(crate) trait GazeSource {
    fn get_vision_type(&self) -> VisionType;
    fn get_gaze(&mut self) -> Result<(f32, f32)>;
}

pub(crate) struct PupilGazeSource {
    pub pupil: Pupil,
}

impl PupilGazeSource {
    pub fn new(zmq_endpoint: String) -> Self {
        Self {
            pupil: Pupil::new(PupilRemote::new(zmq_endpoint, "gaze")),
        }
    }
}

impl GazeSource for PupilGazeSource {
    fn get_vision_type(&self) -> VisionType {
        VisionType::Pupil
    }

    fn get_gaze(&mut self) -> Result<(f32, f32)> {
        self.pupil.get_gaze()
    }
}
