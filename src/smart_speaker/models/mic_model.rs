use pv_recorder::{PvRecorder, PvRecorderBuilder, };
use cobra::{Cobra};
use rhino::{Rhino, RhinoBuilder, RhinoInference};
use porcupine::{Porcupine, PorcupineBuilder, BuiltinKeywords};
// use cheetah::{Cheetah, CheetahBuilder};
use anyhow::{anyhow, Result};

#[derive(Clone)]
pub(crate) struct AudioListener {
    pub recorder: PvRecorder,
}

impl AudioListener {
    pub fn new(mic_index: u32) -> Self {
        Self {
            recorder: PvRecorderBuilder::default().device_index(mic_index as i32).init().unwrap(),
        }
    }

    pub fn info(&mut self) -> String {
        let audio_devices = PvRecorderBuilder::default().get_available_devices().unwrap();
        format!("{:?}", audio_devices).to_string()
    }

    pub fn start(&mut self) {
        self.recorder.start().expect("failed to start recording");
    }

    pub fn stop(&mut self) {
        self.recorder.stop().expect("failed to stop recording");
    }

    pub fn update(&mut self) -> Result<Vec<i16>> {
        match self.recorder.read() {
            Ok(pcm) => Ok(pcm),
            Err(_) => Err(anyhow!("failed to read audio frame"))
        }
    }
}

pub(crate) struct WakeWordDetector {
    pub app: Porcupine,
}

impl WakeWordDetector {
    pub fn new(api_key: String, context_path: String) -> Self {
        if context_path == "default" {
            Self {
                app: PorcupineBuilder::new_with_keywords(api_key,
                                                         &[BuiltinKeywords::Jarvis, BuiltinKeywords::Alexa]).init().unwrap(),
            }
        } else {
            Self {
                app: PorcupineBuilder::new_with_keyword_paths(api_key, &[context_path]).model_path("picovoice_data/porcupine_params_ja_v3_0_0.pv").init().unwrap(),
            }
        }

    }

    pub fn info(&mut self) -> String {
        self.app.version().to_string()
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

pub(crate) struct VoiceActivityDetector {
    pub app: Cobra,
}

impl VoiceActivityDetector {
    pub fn new(api_key: String) -> Self {
        Self {
            app: Cobra::new(api_key).unwrap(),
        }
    }

    pub fn info(&mut self) -> String {
        self.app.version().to_string()
    }

    pub fn detect(&mut self, pcm: &Vec<i16>) -> Result<f32> {
        match self.app.process(pcm) {
            Ok(probability) => Ok(probability),
            Err(_) => Err(anyhow!("failed to process audio frame"))
        }
    }
}

pub(crate) struct SpeechToIntent {
    pub app: Rhino,
}

impl SpeechToIntent {
    pub fn new(api_key: String, context_path: String) -> Self {
        Self {
            app: RhinoBuilder::new(api_key, context_path).model_path("picovoice_data/rhino_params_ja_v3_0_0.pv").init().unwrap(),
        }
    }

    pub fn info(&mut self) -> String {
        self.app.context_info().to_string()
    }

    pub fn get_inference(&mut self) -> Result<Option<RhinoInference>> {
        // TODO: Change return type as enum
        if let Ok(inference) = self.app.get_inference() {
            return if inference.is_understood {
                dbg!(&inference.intent);
                dbg!(&inference.slots);
                Ok(
                    Some(inference)
                )
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

// pub(crate) struct SpeechToText {
//     pub app: Cheetah,
// }
//
// impl SpeechToText {
//     pub fn new(api_key: &String) -> Self {
//         Self {
//             app: CheetahBuilder::new().access_key(api_key.clone()).init().unwrap(),
//         }
//     }
//
//     pub fn detect(&mut self, pcm: &Vec<i16>) -> Result<String> {
//         match self.app.process(pcm) {
//             Ok(text) => Ok(text.transcript),
//             Err(_) => Err(anyhow!("failed to process audio frame"))
//         }
//     }
// }
