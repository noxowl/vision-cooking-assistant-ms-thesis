use std::sync::mpsc;

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
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerMessage {
    StringMessage(StringMessage),
    // WakeWordDetected(WakeWordDetected),
    AttentionFinished(AttentionFinished),
    ReportTerminated(ReportTerminated),
    RequestShutdown(RequestShutdown),
    RequestAudioStream(RequestAudioStream),
    RequestCameraFrame(RequestCameraFrame),
    RequestGazeInfo(RequestGazeInfo),
    RequestAttention(RequestAttention),
    RequestQuery(QueryMessage),
    RequestActorGenerate(RequestActorGenerate),
    ForceActivate(ForceActivate),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ForceActivate {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RequestAttention {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AttentionFinished {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

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

pub(crate) fn attention_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                send_from: SmartSpeakerActors,
                                send_to: SmartSpeakerActors) {
    match sender.send(SmartSpeakerMessage::RequestAttention(RequestAttention {
        send_from,
        send_to,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn attention_finished_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                         send_from: SmartSpeakerActors,
                                         send_to: SmartSpeakerActors) {
    match sender.send(SmartSpeakerMessage::AttentionFinished(AttentionFinished {
        send_from,
        send_to,
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
