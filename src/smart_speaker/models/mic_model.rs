use pv_recorder::{Recorder, RecorderBuilder};
// use cobra::{Cobra};
use rhino::{Rhino, RhinoBuilder};
use porcupine::{Porcupine, PorcupineBuilder, BuiltinKeywords};
use cheetah::{Cheetah, CheetahBuilder};
use anyhow::{anyhow, Result};

#[derive(Clone)]
pub(crate) struct AudioListener {
    pub recorder: Recorder,
}

impl AudioListener {
    pub fn new(mic_index: u32) -> Self {
        Self {
            recorder: RecorderBuilder::new().device_index(mic_index as i32).init().unwrap(),
        }
    }

    pub fn info(&mut self) {
        dbg!(self.recorder.get_audio_devices().unwrap());
    }

    pub fn start(&mut self) {
        self.recorder.start().expect("failed to start recording");
    }

    pub fn stop(&mut self) {
        self.recorder.stop().expect("failed to stop recording");
    }

    pub fn update(&mut self) -> Result<Vec<i16>> {
        let mut pcm = vec![0; self.recorder.frame_length()];
        match self.recorder.read(&mut pcm) {
            Ok(_) => Ok(pcm),
            Err(_) => Err(anyhow!("failed to read audio frame"))
        }
    }
}

pub(crate) struct WakeWordDetector {
    pub app: Porcupine,
}

impl WakeWordDetector {
    pub fn new(api_key: String) -> Self {
        Self {
            app: PorcupineBuilder::new_with_keywords(api_key,
                                                     &[BuiltinKeywords::Jarvis, BuiltinKeywords::Alexa]).init().unwrap(),
        }
    }

    pub fn info(&mut self) {
        dbg!(self.app.version());
    }

    pub fn detect(&mut self, pcm: &Vec<i16>) -> Result<bool> {
        match self.app.process(pcm) {
            Ok(keyword_index) => {
                if keyword_index == -1 {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            Err(_) => Err(anyhow!("failed to process audio frame"))
        }
    }
}

// pub(crate) struct VoiceActivityDetector {
//     pub app: Cobra,
// }
//
// impl VoiceActivityDetector {
//     pub fn new(api_key: &String) -> Self {
//         Self {
//             app: Cobra::new(api_key.clone()).unwrap(),
//         }
//     }
//
//     pub fn detect(&mut self, pcm: &Vec<i16>) -> Result<f32> {
//         match self.app.process(pcm) {
//             Ok(probability) => Ok(probability),
//             Err(_) => Err(anyhow!("failed to process audio frame"))
//         }
//     }
// }

pub(crate) struct SpeechToIntent {
    pub app: Rhino,
}

impl SpeechToIntent {
    pub fn new(api_key: String, context_path: String) -> Self {
        Self {
            app: RhinoBuilder::new(api_key, context_path).init().unwrap(),
        }
    }

    pub fn info(&mut self) {
        dbg!(self.app.context_info());
    }

    pub fn get_inference(&mut self) -> Result<Option<String>> {
        // TODO: Change return type as enum
        if let Ok(inference) = self.app.get_inference() {
            return if inference.is_understood {
                Ok(inference.intent)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

pub(crate) struct SpeechToText {
    pub app: Cheetah,
}

impl SpeechToText {
    pub fn new(api_key: &String) -> Self {
        Self {
            app: CheetahBuilder::new().access_key(api_key.clone()).init().unwrap(),
        }
    }

    pub fn detect(&mut self, pcm: &Vec<i16>) -> Result<String> {
        match self.app.process(pcm) {
            Ok(text) => Ok(text.transcript),
            Err(_) => Err(anyhow!("failed to process audio frame"))
        }
    }
}