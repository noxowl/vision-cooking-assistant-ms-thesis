use anyhow::{anyhow, Result};
use cobra::{Cobra, CobraError};
use cheetah::{Cheetah, CheetahBuilder, CheetahError};
// use rhino::{Rhino, RhinoBuilder};
use pv_recorder::{Recorder, RecorderBuilder};
use rhino::{Rhino, RhinoBuilder, RhinoError, RhinoInference};


pub struct AudioRecorder {
    recorder: Recorder,
}

impl AudioRecorder {
    pub fn new(mic_index: i32) -> Self {
        Self {
            recorder: RecorderBuilder::new().device_index(mic_index).init().unwrap(),
        }
    }

    pub fn start_listen(&mut self) {
        self.recorder.start().expect("failed to start recording");
    }

    pub async fn listen(&mut self) -> Result<Vec<i16>> {
        let mut pcm = vec![0; self.recorder.frame_length()];
        match self.recorder.read(&mut pcm) {
            Ok(_) => Ok(pcm),
            Err(_) => Err(anyhow!("failed to read audio frame"))
        }
    }
}

pub fn is_human_voice(pcm: &Vec<i16>, vad: &mut Cobra) -> Result<bool> {
    match vad.process(&pcm) {
        Ok(probability) => {
            dbg!(&probability);
            match probability {
                0.4..1.0 => {
                    Ok(true)
                }
                _ => { Ok(false) }
            }
        },
        Err(_) => {
            Err(anyhow!(""))
        }
    }
}

pub fn process_sti(pcm: &Vec<i16>, sti: &mut Rhino) -> Result<bool>  {
    match sti.process(&pcm) {
        Ok(finalized) => {
            Ok(finalized)
        },
        Err(_) => Err(anyhow!("error on sti processing!"))
    }
}

pub fn get_inference(sti: &mut Rhino) -> Result<RhinoInference> {
    match sti.get_inference() {
        Ok(inference) => Ok(inference),
        Err(_) => Err(anyhow!("error on getting inference!"))
    }
}

pub fn create_vad(api_key: &str) -> Result<Cobra, CobraError> {
    Cobra::new(api_key)
}

pub fn create_sti(api_key: &str, context_path: &str) -> Result<Rhino, RhinoError> {
    RhinoBuilder::new(
        api_key,
        context_path
    ).init()
}

pub fn create_stt(api_key: &str) -> Result<Cheetah, CheetahError> {
    CheetahBuilder::new().access_key(api_key).init()
}

fn next_audio_frame() {

}

