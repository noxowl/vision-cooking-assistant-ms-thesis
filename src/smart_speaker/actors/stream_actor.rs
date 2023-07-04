// use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use gstreamer;
use crate::smart_speaker::controllers::vision_controller;
use crate::utils::message_util::{self, RequestCameraFrame, RequestAudioStream, SmartSpeakerActors, SmartSpeakerMessage};

pub(crate) struct StreamActor {
    alive: bool,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
    // tcp_listener: TcpListener,
    stream_before: Vec<i16>,
}

impl StreamActor {
    pub(crate) fn new(receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            receiver,
            sender,
            // tcp_listener: TcpListener::bind("0.0.0.0:3333").unwrap(),
            stream_before: vec![],
        }
    }

    pub(crate) fn run(&mut self) {
        println!("StreamActor started");
        let _ = gstreamer::init().unwrap();
        let buf = gstreamer::Buffer::new();
        while self.alive {
            if let Ok(message) = self.receiver.try_recv() {
                let mut pending = true;
                while pending {
                    if let Ok(message) = self.receiver.try_recv() {
                        self.handle_message(message);
                    } else {
                        pending = false;
                    }
                }
                // for result in self.tcp_listener.incoming() {
                //     if let Ok(stream) = result {
                //         let mut stream = stream;
                //         let mut buf = [0; 1024];
                //         let _ = stream.read(&mut buf);
                //         let _ = stream.write(&self.stream_before);
                //     }
                // }
            }
            if self.alive {
                self.request_camera_frame();
                self.request_audio_stream();
            }
            thread::sleep(Duration::from_millis(33));
        }
        println!("StreamActor terminated");
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(_) => {
                self.alive = false;
            },
            SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame { send_from: _,
                                                        send_to: _,
                                                        frame_data_bytes,
                                                        height,}) => {
                self.handle_frame_data_bytes(frame_data_bytes, height);
            },
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream{
                                                        send_from: _,
                                                        send_to: _,
                                                        stream, }) => {
                self.handle_audio_stream(stream);
            }
            _ => {}
        }
    }

    fn handle_frame_data_bytes(&mut self, frame_data_bytes: Vec<u8>, height: i32) {
        match vision_controller::data_bytes_to_mat(frame_data_bytes, height) {
            Ok(frame) => {
            }
            Err(_) => {}
        };
    }

    fn handle_audio_stream(&mut self, stream: Vec<i16>) {
        if self.stream_before != stream {
            self.stream_before = stream.clone();
        }
    }

    fn request_camera_frame(&self) {
        message_util::camera_frame_message(
            &self.sender,
            SmartSpeakerActors::StreamActor,
            SmartSpeakerActors::CameraActor,
            vec![],
            0);
    }

    fn request_audio_stream(&self) {
        message_util::audio_stream_message(
            &self.sender,
            SmartSpeakerActors::StreamActor,
            SmartSpeakerActors::AudioActor,
            vec![],
        )
    }
}