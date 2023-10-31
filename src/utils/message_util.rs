use std::sync::mpsc;
use opencv::{core::Vector, types::VectorOfVectorOfPoint2f};
use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentSlot};
use crate::smart_speaker::models::vision_model::{VisionAction, VisionSlot};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) enum SmartSpeakerActors {
    CoreActor,
    CameraActor,
    VisionActor,
    GazeActor,
    AudioActor,
    WakeWordActor,
    SpeechToIntentActor,
    MachineSpeechActor,
    StreamActor,
    QueryActor,
    ContextActor,
    VoiceActivityDetectActor,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerMessage {
    StringMessage(StringMessage),
    // AttentionFinished(AttentionFinished),
    IntentFinalized(IntentFinalized),
    VisionFinalized(VisionFinalized),
    ReportTerminated(ReportTerminated),
    RequestShutdown(RequestShutdown),
    RequestAudioStream(RequestAudioStream),
    RequestCameraFrame(RequestCameraFrame),
    RequestGazeInfo(RequestGazeInfo),
    RequestStateUpdate(RequestStateUpdate),
    RequestMarkerInfo(RequestMarkerInfo),
    RequestQuery(QueryMessage),
    RequestActorGenerate(RequestActorGenerate),
    RequestVisionAction(RequestVisionAction),
    RequestTextToSpeech(StringMessage),
    ForceActivate(ForceActivate),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ForceActivate {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestStateUpdate {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub state: SmartSpeakerState,
}

// #[derive(Debug, Clone, PartialEq)]
// pub(crate) struct AttentionFinished {
//     pub send_from: SmartSpeakerActors,
//     pub send_to: SmartSpeakerActors,
//     pub result: AttentionResult,
// }

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReportTerminated {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestShutdown {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestAudioStream {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub stream: Vec<i16>
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestCameraFrame {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub frame_data_bytes: Vec<u8>,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestGazeInfo {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub gaze_info: (f32, f32)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestMarkerInfo {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub marker_info: (Vec<Vec<(f32, f32)>>, Vec<i32>)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StringMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QueryMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestActorGenerate {
    pub send_from: SmartSpeakerActors,
    pub request: SmartSpeakerActors
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestVisionAction {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub actions: Vec<VisionAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IntentFinalized {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub result: ProcessResult,
    pub content: IntentContent,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VisionFinalized {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub result: ProcessResult,
    pub contents: Vec<VisionContent>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IntentContent {
    pub intent: IntentAction,
    pub entities: Vec<Box<dyn IntentSlot>>,
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

impl VisionContent {
    pub(crate) fn new(action: VisionAction, entities: Vec<Box<dyn VisionSlot>>) -> Self {
        VisionContent {
            action,
            entities,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContentType {
    None,
    Intent,
    Vision,
}

pub(crate) trait Content {
    fn clone_box(&self) -> Box<dyn Content>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Content for IntentContent {
    fn clone_box(&self) -> Box<dyn Content> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Content for VisionContent {
    fn clone_box(&self) -> Box<dyn Content> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProcessResult {
    Success,
    Failure,
}

pub(crate) fn audio_stream_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                   send_from: SmartSpeakerActors,
                                   send_to: SmartSpeakerActors,
                                   stream: Vec<i16>) {
    match sender.send(SmartSpeakerMessage::RequestAudioStream(RequestAudioStream {
        send_from,
        send_to,
        stream,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn camera_frame_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                   send_from: SmartSpeakerActors,
                                   send_to: SmartSpeakerActors,
                                   frame_data_bytes: Vec<u8>,
                                   height: i32) {
    match sender.send(SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame {
        send_from,
        send_to,
        frame_data_bytes,
        height,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn generate_actor_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                     send_from: SmartSpeakerActors,
                                     request: SmartSpeakerActors) {
    match sender.send(SmartSpeakerMessage::RequestActorGenerate(RequestActorGenerate {
        send_from,
        request,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn gaze_info_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                send_from: SmartSpeakerActors,
                                send_to: SmartSpeakerActors,
                                gaze_info: (f32, f32)) {
    match sender.send(SmartSpeakerMessage::RequestGazeInfo(RequestGazeInfo {
        send_from,
        send_to,
        gaze_info,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn marker_info_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                  send_from: SmartSpeakerActors,
                                  send_to: SmartSpeakerActors,
                                  marker_info: (VectorOfVectorOfPoint2f, Vector<i32>)) {
    match sender.send(SmartSpeakerMessage::RequestMarkerInfo(RequestMarkerInfo {
        send_from,
        send_to,
        marker_info: (
            marker_info.0.to_vec().into_iter()
                .map(|x| x.to_vec().into_iter()
                    .map(|point| (point.x, point.y)).collect()
                ).collect(),
            marker_info.1.to_vec()),
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn state_update_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                   send_from: SmartSpeakerActors,
                                   send_to: SmartSpeakerActors,
                                   state: SmartSpeakerState) {
    match sender.send(SmartSpeakerMessage::RequestStateUpdate(RequestStateUpdate {
        send_from,
        send_to,
        state
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

// pub(crate) fn attention_finished_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
//                                          send_from: SmartSpeakerActors,
//                                          send_to: SmartSpeakerActors,
//                                          result: AttentionResult) {
//     match sender.send(SmartSpeakerMessage::AttentionFinished(AttentionFinished {
//         send_from,
//         send_to,
//         result,
//     })) {
//         Ok(_) => {}
//         Err(e) => {
//             println!("Error: {}", e);
//         }
//     }
// }

pub(crate) fn intent_finalized_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                         send_from: SmartSpeakerActors,
                                         send_to: SmartSpeakerActors,
                                         result: ProcessResult, content: IntentContent) {
    match sender.send(SmartSpeakerMessage::IntentFinalized(IntentFinalized {
        send_from,
        send_to,
        result,
        content
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn vision_action_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                    send_from: SmartSpeakerActors,
                                    send_to: SmartSpeakerActors,
                                    actions: Vec<VisionAction>) {
    match sender.send(SmartSpeakerMessage::RequestVisionAction(RequestVisionAction {
        send_from,
        send_to,
        actions
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn vision_finalized_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                       send_from: SmartSpeakerActors,
                                       send_to: SmartSpeakerActors,
                                       result: ProcessResult, contents: Vec<VisionContent>) {
    match sender.send(SmartSpeakerMessage::VisionFinalized(VisionFinalized {
        send_from,
        send_to,
        result,
        contents
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn text_to_speech_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                     send_from: SmartSpeakerActors,
                                     send_to: SmartSpeakerActors,
                                     message: String) {
    match sender.send(SmartSpeakerMessage::RequestTextToSpeech(StringMessage {
        send_from,
        send_to,
        message,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn terminate_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                send_from: SmartSpeakerActors) {
    match sender.send(SmartSpeakerMessage::ReportTerminated(ReportTerminated {
        send_from,
        send_to: SmartSpeakerActors::CoreActor,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
