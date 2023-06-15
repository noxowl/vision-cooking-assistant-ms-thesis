use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::smart_speaker::models::mic_model::AudioListener;
use crate::smart_speaker::controllers::mic_controller;
use crate::utils::message_util::{audio_stream_message, RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage, StringMessage};

pub(crate) struct AudioActor {
    alive: bool,
    core: AudioListener,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    stream: Vec<i16>,
}

impl AudioActor {
    pub(crate) fn new(core: AudioListener, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
            stream: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        println!("AudioActor started");
        self.core.info();
        let _ = self.core.start();
        while self.alive {
            match mic_controller::listen_mic(&mut self.core) {
                Ok(stream) => {
                    self.stream = stream;
                    if let Ok(message) = self.receiver.try_recv() {
                        self.handle_message(message);
                    }
                },
                _ => {}
            }
            thread::sleep(Duration::from_millis(33));
        }
        let _ = self.core.stop();
    }

     fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(RequestShutdown {}) => {
                self.alive = false;
            },
            SmartSpeakerMessage::StringMessage(StringMessage { send_from, send_to: _, message: _ }) => {
                self.sender.send(SmartSpeakerMessage::StringMessage(StringMessage {
                    send_from: SmartSpeakerActors::AudioActor,
                    send_to: send_from,
                    message: "pong".to_string(),
                })).expect("TODO: panic message");
            },
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from, send_to: _, stream: _ }) => {
                audio_stream_message(&self.sender, SmartSpeakerActors::AudioActor, send_from, self.stream.clone());
            }
            _ => {
                dbg!("unhandled message");
            }
        }
    }
}

//  fn listen_mic(tx: Arc<Mutex<Sender<CommonMessage>>>, vad: &mut Cobra, sti: &mut Rhino,
//                     recorder: &mut AudioRecorder, listening: &Cell<bool>, timeout: &Cell<i32>, halt: &Cell<bool>) {
//     while !halt.get() {
//         let record = recorder.listen();
//         match record {
//             Ok(pcm) => {
//                 if listening.get() {
//                     match auditio::process_sti(&pcm, sti) {
//                         Ok(true) => {
//                             listening.set(false);
//                             if let Ok(inference) = auditio::get_inference(sti) {
//                                 if inference.is_understood {
//                                     dbg!(&inference.intent);
//                                     let intent = IntentType::from_str(inference.intent.unwrap().as_str()).unwrap();
//                                     match tx.lock().unwrap().send(CommonMessage::OrderDetected(intent)) {
//                                         Ok(_) => {
//                                         }
//                                         Err(_) => {
//                                             dbg!("recv true. send message failed.");
//                                         }
//                                     }
//                                 } else {
//                                     dbg!("cannot understood order.");
//                                 }
//                             }
//                         },
//                         Ok(false) => {
//                             // let mut t = timeout.get();
//                             // timeout.set(t + 1);
//                             dbg!("not finished yet. hearing...");
//                             // if t > 20 {
//                             //     sti.
//                             //     timeout.set(0);
//                             //     listening.set(false);
//                             // }
//                         },
//                         Err(_) => {}
//                     }
//                 } else {
//                     match auditio::is_human_voice(&pcm, vad) {
//                         Ok(detected) => {
//                             if detected {
//                                 auditio::process_sti(&pcm, sti).expect("failed to process sti!");
//                                 listening.set(true);
//                                 match tx.lock().unwrap().send(CommonMessage::HumanDetected) {
//                                     Ok(_) => {
//                                     }
//                                     Err(_) => {
//                                         dbg!("recv true. send message failed.");
//                                     }
//                                 }
//                             }
//                         }
//                         Err(_) => {}
//                     }
//
//                 }
//
//             },
//             _ => {}
//         }
//         tokio::time::sleep(Duration::from_micros(1));
//     }
// }
//
//  fn audio_message(rx: Arc<Mutex<Receiver<CommonMessage>>>) {
//
// }

