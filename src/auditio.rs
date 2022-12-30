use std::borrow::BorrowMut;
use anyhow::{anyhow, Result};
use tokio;
use std::time::{Duration, Instant};
use std::future::{pending, Pending};
use cobra::{Cobra, CobraError};
use cheetah::{Cheetah, CheetahBuilder, CheetahError};
// use rhino::{Rhino, RhinoBuilder};
use pv_recorder::{Recorder, RecorderBuilder};
use tokio::sync::mpsc::{Receiver, Sender};


pub struct AudioRecorder {
    recorder: Recorder,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            recorder: RecorderBuilder::new().device_index(option_env!("PICOVOICE_MIC_INDEX").unwrap_or("0").parse().unwrap()).init().unwrap(),
        }
    }

    pub fn start_listen(&mut self) {
        self.recorder.start().expect("failed to start recording");
    }

    pub async fn listen(&mut self, vad: &mut Cobra) -> Result<bool> {
        let mut pcm = vec![0; self.recorder.frame_length()];
        self.recorder.read(&mut pcm).expect("failed to read audio frame");
        let voice_probability = vad.process(&pcm).unwrap();
        match voice_probability {
            0.5..1.0 => {
                Ok(true)
            }
            _ => { Ok(false) }
        }
    }
}

pub fn create_vad() -> Result<Cobra, CobraError> {
    Cobra::new(option_env!("PICOVOICE_ACCESS_KEY").unwrap_or(""))
}

pub fn create_stt() -> Result<Cheetah, CheetahError> {
    CheetahBuilder::new().access_key(option_env!("PICOVOICE_ACCESS_KEY").unwrap_or("")).init()
}

fn next_audio_frame() {

}

