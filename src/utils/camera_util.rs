use opencv::core::Mat;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, CAP_ANY};
use anyhow::{anyhow, Result};

pub(crate) struct Camera {
    pub video_capture: VideoCapture,
}

impl Camera {
    #[allow(unused_mut)]
    pub fn new() -> Result<Self> {
        let mut video_capture = VideoCapture::new(0, CAP_ANY)?;
        Ok(Self { video_capture })
    }

    pub fn get_frame(&mut self) -> Result<Mat> {
        let mut frame = Mat::default();
        match self.video_capture.is_opened() {
            Ok(true) => {
                self.video_capture.grab()?;
                self.video_capture.retrieve(&mut frame, CAP_ANY)?;
                Ok(frame)
            }
            Ok(false) => {
                Err(anyhow!("camera not opened!"))
            }
            Err(_) => {
                Err(anyhow!("camera not opened!"))
            }
        }
    }
}