use std::sync::mpsc;
use crate::smart_speaker::models::core_model::SmartSpeakerState;
use crate::smart_speaker::models::message_model::*;

pub(crate) fn audio_stream_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                   send_from: SmartSpeakerActors,
                                   send_to: SmartSpeakerActors,
                                   stream: Vec<i16>) {
    match sender.send(SmartSpeakerMessage::RequestAudioStream(AudioStreamMessage {
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
    match sender.send(SmartSpeakerMessage::RequestCameraFrame(CameraFrameMessage {
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
    match sender.send(SmartSpeakerMessage::RequestActorGenerate(ActorGenerateMessage {
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
    match sender.send(SmartSpeakerMessage::RequestGazeInfo(GazeInfoMessage {
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

pub(crate) fn state_update_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                   send_from: SmartSpeakerActors,
                                   send_to: SmartSpeakerActors,
                                   state: SmartSpeakerState) {
    match sender.send(SmartSpeakerMessage::RequestStateUpdate(StateUpdateMessage {
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

pub(crate) fn intent_finalized_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                         send_from: SmartSpeakerActors,
                                         send_to: SmartSpeakerActors,
                                         result: ProcessResult, content: IntentContent) {
    match sender.send(SmartSpeakerMessage::IntentFinalized(IntentFinalizedMessage {
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

// pub(crate) fn vision_action_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
//                                     send_from: SmartSpeakerActors,
//                                     send_to: SmartSpeakerActors,
//                                     actions: Vec<VisionAction>) {
//     match sender.send(SmartSpeakerMessage::RequestVisionAction(RequestVisionAction {
//         send_from,
//         send_to,
//         actions
//     })) {
//         Ok(_) => {}
//         Err(e) => {
//             println!("Error: {}", e);
//         }
//     }
// }

pub(crate) fn vision_finalized_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                       send_from: SmartSpeakerActors,
                                       send_to: SmartSpeakerActors,
                                       result: ProcessResult, contents: Vec<VisionContent>) {
    match sender.send(SmartSpeakerMessage::VisionFinalized(VisionFinalizedMessage {
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
                                     i18n: SmartSpeakerI18nText) {
    match sender.send(SmartSpeakerMessage::RequestTextToSpeech(TextToSpeechMessage {
        send_from,
        send_to,
        message: TextToSpeechMessageType::Normal(i18n),
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn text_to_speech_boilerplate_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                     send_from: SmartSpeakerActors,
                                     send_to: SmartSpeakerActors,
                                     index: usize) {
    match sender.send(SmartSpeakerMessage::RequestTextToSpeech(TextToSpeechMessage {
        send_from,
        send_to,
        message: TextToSpeechMessageType::Boilerplate(index),
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub(crate) fn text_to_speech_finished_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                              send_from: SmartSpeakerActors,
                                              send_to: SmartSpeakerActors,
                                              message: String) {
    match sender.send(SmartSpeakerMessage::TextToSpeechFinished(StringMessage {
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

pub(crate) fn write_log_message(sender: &mpsc::Sender<SmartSpeakerMessage>,
                                send_from: SmartSpeakerActors,
                                message: SmartSpeakerLogMessageType) {
    match sender.send(SmartSpeakerMessage::WriteLog(LogMessage {
        send_from,
        send_to: SmartSpeakerActors::LoggerActor,
        message,
    })) {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
