use std::sync::mpsc;
use rgb::RGB8;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) enum SmartSpeakerActors {
    CoreActor,
    CameraActor,
    GazeActor,
    AudioActor,
    WakeWordActor,
    SpeechToIntentActor,
    MachineSpeechActor,
    QueryActor,
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub(crate) struct WakeWordDetected {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub wake_word: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RequestAttention {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone)]
pub(crate) struct AttentionFinished {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone)]
pub(crate) struct ReportTerminated {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
}

#[derive(Debug, Clone)]
pub(crate) struct RequestShutdown {}

#[derive(Debug, Clone)]
pub(crate) struct RequestAudioStream {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub stream: Vec<i16>
}

#[derive(Debug, Clone)]
pub(crate) struct RequestCameraFrame {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub frame: Vec<u8>
}

#[derive(Debug, Clone)]
pub(crate) struct RequestGazeInfo {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub gaze_info: Vec<f32>
}

#[derive(Debug, Clone)]
pub(crate) struct StringMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RgbMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: RGB8,
}

#[derive(Debug, Clone)]
pub(crate) struct QueryMessage {
    pub send_from: SmartSpeakerActors,
    pub send_to: SmartSpeakerActors,
    pub message: String,
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
                                   frame: Vec<u8>) {
    match sender.send(SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame {
        send_from,
        send_to,
        frame,
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
                                gaze_info: Vec<f32>) {
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
