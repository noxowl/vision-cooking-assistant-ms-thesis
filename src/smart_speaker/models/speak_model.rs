use std::sync::mpsc;
use anyhow::{anyhow, Result};
use tts::{Tts, Features, Voice, UtteranceId, Error};
use crate::utils::config_util::LanguageTag;
#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::{NSRunLoop, NSDefaultRunLoopMode};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl, class};

pub(crate) struct MachineSpeech {
    app: Tts,
    pub language: LanguageTag // BCP 47
}

impl MachineSpeech {
    pub(crate) fn new(language: LanguageTag) -> Self {
        Self {
            app: Tts::default().unwrap(),
            language
        }
    }

    pub(crate) fn init(&mut self) -> Result<String> {
        let voice_participants = self.app.voices().unwrap().into_iter().filter(|v| v.language() == self.language.to_str().to_string()).collect::<Vec<Voice>>();
        let voice = voice_participants.first().ok_or(anyhow!("no voice found"))?;
        self.app.set_voice(voice)?;

        Ok(format!("{:?}", &voice_participants))
    }

    pub(crate) fn info(&self) -> String {
        let Features {
            get_voice,
            ..
        } = self.app.supported_features();
        if get_voice {
            format!("Voice: {:?}", self.app.voice().unwrap()).to_string()
        } else {
            match self.app.voice() {
                Ok(voice) => {
                    format!("Voice: {:?}", voice).to_string()
                }
                Err(err) => {
                    format!("Voice: {:?}", err).to_string()
                }
            }
        }
    }

    // pub(crate) fn speak(&mut self, text: String) -> Result<()> {
    //     let Features {
    //         utterance_callbacks,
    //         ..
    //     } = self.app.supported_features();
    //     let result = self.app.speak(text, false);
    //     Ok(())
    // }

    pub(crate) fn speak_with_callback(&mut self, text: String, callback_sender: mpsc::Sender<usize>) {
        let Features {
            utterance_callbacks,
            ..
        } = self.app.supported_features();
        let _ = self.app.speak(text, false);
        if utterance_callbacks {
            self.app.on_utterance_end(Some(Box::new(move |utterance_id: UtteranceId| {
                let _ = callback_sender.send(0);
            }))).unwrap();
        } else {
            let _ = callback_sender.send(0);
        }
        //
        // #[cfg(target_os = "macos")]
        // {
        //     let run_loop: id = unsafe { NSRunLoop::currentRunLoop() };
        //     unsafe {
        //         let date: id = msg_send![class!(NSDate), distantFuture];
        //         let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];
        //     }
        // }
    }
}

pub(crate) enum MachineSpeechBoilerplate {
    PowerOn,
    WakeUp,
    Ok,
    Undefined,
    Aborted,
    IntentFailed,
    VisionFailed,
}

impl MachineSpeechBoilerplate {
    pub(crate) fn to_string_by_language(&self, language: &LanguageTag) -> String {
        match self {
            Self::PowerOn => match language {
                LanguageTag::Japanese => { "起動しました。" }
                _ => { "Smart Speaker Activated." }
            }.to_string(),
            Self::WakeUp => match language {
                LanguageTag::Japanese => { "聞こえています。" }
                _ => { "I'm Listening." }
            }.to_string(),
            Self::Ok => match language {
                LanguageTag::Japanese => { "分かりました。" }
                _ => { "Ok." }
            }.to_string(),
            Self::Undefined => match language {
                LanguageTag::Japanese => { "すみません。わからない命令です。" }
                _ => { "Undefined command." }
            }.to_string(),
            Self::Aborted => match language {
                LanguageTag::Japanese => { "中止します。" }
                _ => { "Stop the current operation." }
            }.to_string(),
            Self::IntentFailed => match language {
                LanguageTag::Japanese => { "すみません。よく聞こえないです。もう一度おっしゃってください。" }
                _ => { "Sorry. I can't hear you very well. Please repeat your message." }
            }.to_string(),
            Self::VisionFailed => match language {
                LanguageTag::Japanese => { "すみません。よく見えないです。もう一度見せてください。" }
                _ => { "Sorry. I can't see very well. Please show me again." }
            }.to_string(),
        }
    }

    pub(crate) fn try_from(index: usize) -> Result<Self> {
        match index {
            0 => Ok(Self::PowerOn),
            1 => Ok(Self::WakeUp),
            2 => Ok(Self::Ok),
            3 => Ok(Self::Undefined),
            4 => Ok(Self::Aborted),
            5 => Ok(Self::IntentFailed),
            6 => Ok(Self::VisionFailed),
            _ => Err(anyhow!("invalid index"))
        }
    }
}
