use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentSlot};
use crate::smart_speaker::models::vision_model::{VisionAction, VisionSlot};
use crate::utils::config_util::LanguageTag;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) enum SmartSpeakerActors {
    AudioActor,
    CameraActor,
    ContextActor,
    CoreActor,
    GazeActor,
    LoggerActor,
    MachineSpeechActor,
    QueryActor,
    SpeechToIntentActor,
    StreamActor,
    VisionActor,
    VoiceActivityDetectActor,
    WakeWordActor,
}

impl std::fmt::Display for SmartSpeakerActors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmartSpeakerActors::AudioActor => write!(f, "AudioActor"),
            SmartSpeakerActors::CameraActor => write!(f, "CameraActor"),
            SmartSpeakerActors::ContextActor => write!(f, "ContextActor"),
            SmartSpeakerActors::CoreActor => write!(f, "CoreActor"),
            SmartSpeakerActors::GazeActor => write!(f, "GazeActor"),
            SmartSpeakerActors::LoggerActor => write!(f, "LoggerActor"),
            SmartSpeakerActors::MachineSpeechActor => write!(f, "MachineSpeechActor"),
            SmartSpeakerActors::QueryActor => write!(f, "QueryActor"),
            SmartSpeakerActors::SpeechToIntentActor => write!(f, "SpeechToIntentActor"),
            SmartSpeakerActors::StreamActor => write!(f, "StreamActor"),
            SmartSpeakerActors::VisionActor => write!(f, "VisionActor"),
            SmartSpeakerActors::VoiceActivityDetectActor => write!(f, "VoiceActivityDetectActor"),
            SmartSpeakerActors::WakeWordActor => write!(f, "WakeWordActor"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerSubtask {

}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerMessage {
    StringMessage(StringMessage),
    RequestActorGenerate(ActorGenerateMessage),
    RequestAudioStream(AudioStreamMessage),
    RequestCameraFrame(CameraFrameMessage),
    RequestGazeInfo(GazeInfoMessage),
    ReportTerminated(ReportTerminated),
    RequestQuery(QueryMessage),
    RequestShutdown(ShutdownMessage),
    RequestStateUpdate(StateUpdateMessage),
    RequestTextToSpeech(TextToSpeechMessage),
    RequestVisionAction(VisionActionMessage),
    IntentFinalized(IntentFinalizedMessage),
    VisionFinalized(VisionFinalizedMessage),
    TextToSpeechFinished(StringMessage),
    SubTaskStart(SmartSpeakerSubtask),
    SubTaskFinished(SmartSpeakerSubtask),
    WriteLog(LogMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StringMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActorGenerateMessage {
    pub send_from: SmartSpeakerActors,
    pub request: SmartSpeakerActors
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AudioStreamMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub stream: Vec<i16>
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CameraFrameMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub frame_data_bytes: Vec<u8>,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GazeInfoMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub gaze_info: (f32, f32)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReportTerminated {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QueryMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShutdownMessage {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StateUpdateMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub state: SmartSpeakerState,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextToSpeechMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: TextToSpeechMessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TextToSpeechMessageType {
    Normal(SmartSpeakerI18nText),
    Boilerplate(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SmartSpeakerI18nText {
    pub en: String,
    pub ja: String,
    pub zh: String,
    pub ko: String,
}

impl SmartSpeakerI18nText {
    pub(crate) fn new() -> Self {
        Self {
            en: "".to_string(),
            ja: "".to_string(),
            zh: "".to_string(),
            ko: "".to_string(),
        }
    }

    pub(crate) fn en(mut self, en: &str) -> Self {
        self.en = en.to_string();
        self
    }

    pub(crate) fn ja(mut self, ja: &str) -> Self {
        self.ja = ja.to_string();
        self
    }

    pub(crate) fn zh(mut self, zh: &str) -> Self {
        self.zh = zh.to_string();
        self
    }

    pub(crate) fn ko(mut self, ko: &str) -> Self {
        self.ko = ko.to_string();
        self
    }

    pub(crate) fn get(&self, language: &LanguageTag) -> String {
        match language {
            LanguageTag::English => self.en.clone(),
            LanguageTag::Japanese => self.ja.clone(),
            LanguageTag::Chinese => self.zh.clone(),
            LanguageTag::Korean => self.ko.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: SmartSpeakerLogMessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerLogMessageType {
    Debug(String),
    Info(String),
    Warn(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionActionMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub actions: Vec<VisionAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IntentFinalizedMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub result: ProcessResult,
    pub content: IntentContent,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionFinalizedMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub result: ProcessResult,
    pub contents: Vec<VisionContent>,
}

pub(crate) trait Content {
    fn clone_box(&self) -> Box<dyn Content>;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContentType {
    None,
    Intent,
    Vision,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IntentContent {
    pub intent: IntentAction,
    pub entities: Vec<Box<dyn IntentSlot>>,
}

impl Content for IntentContent {
    fn clone_box(&self) -> Box<dyn Content> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntentContent {
    pub(crate) fn new(intent: IntentAction, entities: Vec<Box<dyn IntentSlot>>) -> Self {
        IntentContent {
            intent,
            entities,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionContent {
    pub action: VisionAction,
    pub entities: Vec<Box<dyn VisionSlot>>,
}

impl Content for VisionContent {
    fn clone_box(&self) -> Box<dyn Content> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VisionContent {
    pub(crate) fn new(action: VisionAction, entities: Vec<Box<dyn VisionSlot>>) -> Self {
        VisionContent {
            action,
            entities,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProcessResult {
    Success,
    Failure,
}
