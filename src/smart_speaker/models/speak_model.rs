use anyhow::{anyhow, Result};
use tts::{Tts, Features, Voice};
#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSRunLoop;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use crate::utils::config_util::LanguageTag;

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

    pub(crate) fn init(&mut self) -> Result<()> {
        let voice_participants = self.app.voices().unwrap().into_iter().filter(|v| v.language() == self.language.to_str().to_string()).collect::<Vec<Voice>>();
        dbg!(&voice_participants);
        let voice = voice_participants.first().ok_or(anyhow!("no voice found"))?;
        self.app.set_voice(voice)?;
        Ok(())
    }

    pub(crate) fn info(&self) {
        let Features {
            get_voice,
            ..
        } = self.app.supported_features();
        if get_voice {
            println!("Voice: {:?}", self.app.voice().unwrap());
        }
    }

    pub(crate) fn speak(&mut self, text: String) -> Result<()> {
        let Features {
            is_speaking,
            utterance_callbacks,
            ..
        } = self.app.supported_features();
        self.app.speak(text, false)?;
        Ok(())
    }
}

pub(crate) enum MachineSpeechBoilerplate {
    PowerOn,
    WakeUp,
    Ok,
    Undefined,
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
                _ => { "Listening." }
            }.to_string(),
            Self::Ok => match language {
                LanguageTag::Japanese => { "分かりました。" }
                _ => { "Ok." }
            }.to_string(),
            Self::Undefined => match language {
                LanguageTag::Japanese => { "すみません。わからない命令です。" }
                _ => { "Undefined command." }
            }.to_string(),
        }
    }
}
