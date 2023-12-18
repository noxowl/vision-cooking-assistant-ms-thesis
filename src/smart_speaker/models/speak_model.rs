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
use crate::smart_speaker::models::message_model::SmartSpeakerI18nText;

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
                    #[cfg(target_os = "macos")]
                    {
                        return format!("Voice: {:?}", self.app.voices().unwrap().into_iter().filter(|v| v.language() == self.language.to_str().to_string()).collect::<Vec<Voice>>().first()).to_string()
                    }
                    format!("Voice: {:?}", err).to_string()
                }
            }
        }
    }

    pub(crate) fn speak_with_callback(&mut self, i18n_text: SmartSpeakerI18nText, callback_sender: mpsc::Sender<usize>) {
        let Features {
            rate,
            utterance_callbacks,
            ..
        } = self.app.supported_features();
        if rate {
            let normal_rate = self.app.normal_rate();
            match &self.language {
                LanguageTag::English => {
                    if &i18n_text.en.chars().count() > &100 {
                        self.app.set_rate(normal_rate - 0.1).unwrap();
                    } else {
                        self.app.set_rate(normal_rate).unwrap();
                    }
                }
                LanguageTag::Japanese => {
                    if &i18n_text.ja.chars().count() > &60 {
                        self.app.set_rate(normal_rate - 0.1).unwrap();
                    } else {
                        self.app.set_rate(normal_rate).unwrap();
                    }
                }
                LanguageTag::Chinese => {
                    if &i18n_text.zh.chars().count() > &50 {
                        self.app.set_rate(normal_rate - 0.1).unwrap();
                    } else {
                        self.app.set_rate(normal_rate).unwrap();
                    }
                }
                LanguageTag::Korean => {
                    if &i18n_text.ko.chars().count() > &60 {
                        self.app.set_rate(normal_rate - 0.1).unwrap();
                    } else {
                        self.app.set_rate(normal_rate).unwrap();
                    }
                }
            }
        }
        let _ = self.app.speak(i18n_text.get(&self.language), false);
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

    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            Self::PowerOn => SmartSpeakerI18nText::new()
                .en("I started up.")
                .ja("起動しました。")
                .zh("我已经启动了。")
                .ko("기동했습니다."),
            Self::WakeUp => SmartSpeakerI18nText::new()
                .en("I'm listening.")
                .ja("聞いています。")
                .zh("我在听。")
                .ko("듣고 있습니다."),
            Self::Ok => SmartSpeakerI18nText::new()
                .en("Ok.")
                .ja("わかりました。")
                .zh("好的。")
                .ko("알겠습니다."),
            Self::Undefined => SmartSpeakerI18nText::new()
                .en("Sorry. I don't understand.")
                .ja("すみません。わからない命令です。")
                .zh("对不起。我不明白。")
                .ko("죄송합니다. 이해할 수 없는 명령입니다."),
            Self::Aborted => SmartSpeakerI18nText::new()
                .en("Aborted.")
                .ja("中止します。")
                .zh("中止。")
                .ko("중단합니다."),
            Self::IntentFailed => SmartSpeakerI18nText::new()
                .en("Sorry. I can't hear you very well. Please repeat your message")
                .ja("すみません。よく聞こえないです。もう一度おっしゃってください。")
                .zh("对不起。我听不清楚。请再说一遍。")
                .ko("죄송합니다. 잘 들리지 않습니다. 다시 말씀해주세요."),
            Self::VisionFailed => SmartSpeakerI18nText::new()
                .en("Sorry. I can't see very well. Please show me again.")
                .ja("すみません。よく見えないです。もう一度見せてください。")
                .zh("对不起。我看不清楚。请再给我看一遍。")
                .ko("죄송합니다. 잘 보이지 않습니다. 다시 보여주세요."),
        }
    }
}
