use std::sync::mpsc;
use crate::smart_speaker::actors::core_actor::CoreActor;
use crate::utils::config_util::Config;
use crate::utils::vision_util::VisionType;

/// Smart speaker trait
pub(crate) trait SmartSpeaker {
    fn start(&mut self);
    fn stop(&mut self);
}

/// Non-vision smart speaker
pub(crate) struct NonVisionSmartSpeaker {
    config: Config,
}

impl NonVisionSmartSpeaker {
    pub(crate) fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl SmartSpeaker for NonVisionSmartSpeaker {
    fn start(&mut self) {
        println!("NonVisionSmartSpeaker started");
        let (tx, rx) = mpsc::channel();
        let mut core_actor = CoreActor::new(self.config.clone(), tx, rx);
        core_actor.run();
    }

     fn stop(&mut self) {
        println!("NonVisionSmartSpeaker stopped");
    }
}

/// Vision smart speaker
pub(crate) struct VisionSmartSpeaker {
    config: Config,
}

impl VisionSmartSpeaker {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl SmartSpeaker for VisionSmartSpeaker {
     fn start(&mut self) {
        println!("VisionSmartSpeaker started");
    }

     fn stop(&mut self) {
        println!("VisionSmartSpeaker stopped");
    }
}

/// Run smart speaker
pub(crate) fn run_smart_speaker(config: Config) {
    if config.vision {
        match config.vision_type {
            VisionType::None => {
                panic!("Vision type is None!");
            }
            VisionType::Pupil => {
                VisionSmartSpeaker::new(config).start();
            }
            VisionType::BuiltInCamera => {
                VisionSmartSpeaker::new(config).start();
            }
        }
    } else {
        match config.vision_type {
            VisionType::None => {
                println!("The vision type is none. The smart speaker works, but there is no recording of the experimental environment.");
                NonVisionSmartSpeaker::new(config).start();
            }
            VisionType::Pupil => {
                NonVisionSmartSpeaker::new(config).start();
            }
            VisionType::BuiltInCamera => {
                NonVisionSmartSpeaker::new(config).start();
            }
        }
    };
}
