use std::io::{Cursor, Read};
use opencv::prelude::*;
use anyhow::{anyhow, Result};
use zmq;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct PupilNotifySetFrameFormat {
    subject: String,
    format: String
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct PupilCameraPayload {
    format: String,
    // projection_matrix: String,
    // distortion_coeffs: String,
    topic: String,
    width: i32,
    height: i32,
    index: i32,
    timestamp: f64,
    raw_data: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Pupil3dGazePayload {
    // id: i32,
    topic: String,
    // method: String,
    norm_pos: Vec<f32>,
    // diameter: f32,
    timestamp: f64,
    confidence: f32,
    // ellipse: Pupil3dGazeEllipse,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct Pupil3dGazeEllipse {
    angle: f32,
    center: Vec<f32>,
    axes: Vec<f32>,
}


pub(crate) enum PupilTopic {
    WorldFrame,
    Gaze2D,
}

impl PupilTopic {
    pub fn from_str(topic: &str) -> Result<Self> {
        match topic {
            "frame" => {
                Ok(PupilTopic::WorldFrame)
            }
            "gaze" => {
                Ok(PupilTopic::Gaze2D)
            }
            "frame.world" => {
                Ok(PupilTopic::WorldFrame)
            }
            "gaze.2d.0." => {
                Ok(PupilTopic::Gaze2D)
            }
            _ => {
                Err(anyhow!("invalid topic"))
            }
        }
    }

    pub fn get_topic(&self) -> String {
        match self {
            PupilTopic::WorldFrame => {
                "frame.world".to_string()
            }
            PupilTopic::Gaze2D => {
                "gaze.2d.0.".to_string()
            }
        }
    }
}

pub(crate) struct Pupil {
    pub pupil_remote: PupilRemote,
}

impl Pupil {
    pub fn new(mut pupil_remote: PupilRemote) -> Self {
        pupil_remote.init();
        Self { pupil_remote }
    }

    pub fn get_frame(&self) -> Result<Mat> {
        let mut frame = Mat::default();
        match self.pupil_remote.get_frame(&mut frame) {
            Ok(_) => {
                Ok(frame)
            }
            Err(_) => {
                Err(anyhow!("failed to get frame from pupil core"))
            }
        }
    }

    pub fn get_gaze(&self) -> Result<(f32, f32)> {
        let mut gaze = (0., 0.);
        match self.pupil_remote.get_gaze(&mut gaze) {
            Ok(_) => {
                Ok(gaze)
            }
            Err(_) => {
                Err(anyhow!("failed to get gaze from pupil core"))
            }
        }
    }
}

pub(crate) struct PupilRemote {
    topic: PupilTopic,
    endpoint: String,
    ctx: zmq::Context,
    req: Option<zmq::Socket>,
    sub: Option<zmq::Socket>,
}

impl PupilRemote {
    pub fn new(endpoint: String, topic: &str) -> Self {
        Self {
            topic: PupilTopic::from_str(&topic).unwrap(),
            endpoint,
            ctx: zmq::Context::new(),
            req: None,
            sub: None,
        }
    }
    
    pub fn init(&mut self) {
        let mut sub_port: String = "".to_string();
        self.req = Some(self.ctx.socket(zmq::REQ).unwrap());
        match self.req.as_ref() {
            Some(socket) => {
                socket.connect(&format!("tcp://{}", self.endpoint)).unwrap();
                socket.send("SUB_PORT", 0).unwrap();
                sub_port = socket.recv_string(0).unwrap().unwrap();
            },
            None => {}
        }
        let sub_endpoint = format!("{}:{}", self.endpoint.split(":").collect::<Vec<&str>>()[0], sub_port);
        self.sub = Some(self.ctx.socket(zmq::SUB).unwrap());
        match self.sub.as_mut() {
            Some(socket) => {
                socket.connect(&format!("tcp://{}", sub_endpoint)).unwrap();
                match self.topic {
                    PupilTopic::WorldFrame => {
                        socket.set_subscribe(b"frame.world").unwrap();
                        self.set_frame_format("bgr");
                    }
                    PupilTopic::Gaze2D => {
                        socket.set_subscribe(b"gaze").unwrap();
                    }
                }
            },
            None => {}
        }
        dbg!(&sub_endpoint);
    }

    fn set_frame_format(&self, format: &str) {
        let message = PupilNotifySetFrameFormat {
            subject: "frame_publishing.set_format".to_string(),
            format: format.to_string(),
        };
        let topic = format!("notify.{}", message.subject);
        let mut payload = Serializer::new(Vec::new()).with_struct_map();
        message.serialize(&mut payload).unwrap();
        match self.req.as_ref() {
            None => {}
            Some(socket) => {
                socket.send(&topic, zmq::SNDMORE).unwrap();
                socket.send(&payload.into_inner(), 0).unwrap();
                let _ = socket.recv_string(0).unwrap();
            }
        }

    }

    pub fn get_frame(&self, frame: &mut Mat) -> Result<()> {
        match self.sub.as_ref() {
            None => {}
            Some(socket) => {
                let topic = socket.recv_string(0).unwrap().unwrap();
                let msg = socket.recv_bytes(0).unwrap();
                let mut de = Deserializer::new(Cursor::new(&msg[..]));
                let mut payload: PupilCameraPayload = Deserialize::deserialize(&mut de).unwrap();
                let mut additional_data: Vec<u8> = vec![];
                while socket.get_rcvmore().unwrap() {
                    additional_data.append(&mut socket.recv_bytes(0).unwrap());
                }
                payload.raw_data = Some(additional_data);
                *frame = Mat::from_slice(&payload.raw_data.unwrap()).unwrap().reshape(3, payload.height).unwrap();
                return Ok(())
            }
        }
        Err(anyhow!("failed to get frame from pupil core. it is not connected?"))
    }

    pub fn get_gaze(&self, gaze: &mut (f32, f32)) -> Result<()> {
        match self.sub.as_ref() {
            None => {}
            Some(socket) => {
                let topic = socket.recv_string(0).unwrap().unwrap();
                let msg = socket.recv_bytes(0).unwrap();
                let mut de = Deserializer::new(Cursor::new(&msg[..]));
                let payload: Pupil3dGazePayload = Deserialize::deserialize(&mut de).unwrap();
                if &payload.confidence > &0.8_f32 {
                    gaze.0 = payload.norm_pos[0];
                    gaze.1 = payload.norm_pos[1];
                    return Ok(())
                } else {
                    return Err(anyhow!("low confidence"))
                }
            }
        }
        Err(anyhow!("failed to get gaze from pupil core. it is not connected?"))
    }
}