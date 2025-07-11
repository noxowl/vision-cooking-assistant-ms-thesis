use opencv::core::Mat;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, CAP_ANY};
use anyhow::{anyhow, Result};

pub(crate) struct Camera {
    pub video_capture: VideoCapture,
}

impl Camera {
    pub fn new() -> Result<Self> {
        let video_capture = VideoCapture::new(0, CAP_ANY)?;
        Ok(Self { video_capture })
    }

    pub fn get_frame(&mut self) -> Result<Mat> {
        let mut frame = Mat::default();
        match self.video_capture.is_opened() {
            Ok(true) => {
                if self.video_capture.read(&mut frame).unwrap() {
                    Ok(frame)
                } else {
                    Err(anyhow!("failed to read frame"))
                }
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
