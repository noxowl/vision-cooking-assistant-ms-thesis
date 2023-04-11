use std::any::Any;
use std::cell::Cell;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::{anyhow, Context, Error, Result};
use cobra::Cobra;
use opencv::core::{Mat, Point, Point2f, Vector};
use opencv::highgui;
use rhino::{Rhino, RhinoInference};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::auditio::AudioRecorder;
use crate::lib::{ActionType, AgentAction, CommonMessage, Config, IntentType};
use crate::auditio;
use crate::visio;
use crate::debug;

async fn capture_frame(capture: &mut visio::VisioCapture, frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>, halt: &Cell<bool>) {
    while !halt.get() {
        match capture.capture().await {
            Ok(m) => {
                {
                    let mut frame_lock = frame.lock().unwrap();
                    *frame_lock = m;
                }
                match capture.capture_gaze().await {
                    Ok(g) => {
                        let mut gaze_lock = gaze.lock().unwrap();
                        *gaze_lock = g;
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}

async fn listen_mic(tx: Arc<Mutex<Sender<CommonMessage>>>, vad: &mut Cobra, sti: &mut Rhino,
                    recorder: &mut AudioRecorder, listening: &Cell<bool>, timeout: &Cell<i32>, halt: &Cell<bool>) {
    while !halt.get() {
        let record = recorder.listen().await;
        match record {
            Ok(pcm) => {
                if listening.get() {
                    match auditio::process_sti(&pcm, sti) {
                        Ok(true) => {
                            listening.set(false);
                            if let Ok(inference) = auditio::get_inference(sti) {
                                if inference.is_understood {
                                    dbg!(&inference.intent);
                                    let intent = IntentType::from_str(inference.intent.unwrap().as_str()).unwrap();
                                    match tx.lock().unwrap().send(CommonMessage::OrderDetected(intent)).await {
                                        Ok(_) => {
                                        }
                                        Err(_) => {
                                            dbg!("recv true. send message failed.");
                                        }
                                    }
                                } else {
                                    dbg!("cannot understood order.");
                                }
                            }
                        },
                        Ok(false) => {
                            // let mut t = timeout.get();
                            // timeout.set(t + 1);
                            dbg!("not finished yet. hearing...");
                            // if t > 20 {
                            //     sti.
                            //     timeout.set(0);
                            //     listening.set(false);
                            // }
                        },
                        Err(_) => {}
                    }
                } else {
                    match auditio::is_human_voice(&pcm, vad) {
                        Ok(detected) => {
                            if detected {
                                auditio::process_sti(&pcm, sti).expect("failed to process sti!");
                                listening.set(true);
                                match tx.lock().unwrap().send(CommonMessage::HumanDetected).await {
                                    Ok(_) => {
                                    }
                                    Err(_) => {
                                        dbg!("recv true. send message failed.");
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }

                }

            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_micros(1)).await;
    }
}

async fn audio_message(rx: Arc<Mutex<Receiver<CommonMessage>>>) {

}

async fn message_centre(v_tx: Arc<Mutex<Sender<CommonMessage>>>, v_rx: Arc<Mutex<Receiver<CommonMessage>>>, frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>, thing: Arc<Mutex<Option<usize>>>,
                        a_tx: Arc<Mutex<Sender<CommonMessage>>>, a_rx: Arc<Mutex<Receiver<CommonMessage>>>, config: Arc<Mutex<Config>>, halt: &Cell<bool>) -> Result<()> {
    while !halt.get() {
        match v_rx.lock().unwrap().try_recv() {
            Ok(msg) => {
                match msg {
                    CommonMessage::ObjectRecognizeOrder => {
                        {
                            dbg!("recv recognize order!");
                            let mut frame_lock = frame.lock().unwrap();
                            let mut config_lock = config.lock().unwrap();
                            let aruco_result = visio::detect_aruco(&*frame_lock, config_lock.actions.len()).await.unwrap();
                            if aruco_result.1.len() > 0 {
                                let mut gaze_lock = gaze.lock().unwrap();
                                let nearest = visio::find_nearest_aruco(&*gaze_lock, &aruco_result.0, &aruco_result.1).await?;
                                if nearest.id <= config_lock.actions.len() as i32 {
                                    let mut thing_lock = thing.lock().unwrap();
                                    *thing_lock = Some(nearest.id.clone() as usize);
                                    let mut nearest_corner = Vector::new();
                                    nearest_corner.push(nearest.corner.clone());
                                    let mut nearest_id = Vector::new();
                                    nearest_id.push(nearest.id.clone());
                                    visio::debug_put_text(&mut *frame_lock, config_lock.actions.get(nearest.id as usize).unwrap().to_string().as_str(),
                                                          [50, 100]).await;
                                    visio::debug_draw_aruco(&mut *frame_lock, &nearest_corner, &nearest_id).await;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        match a_rx.lock().unwrap().try_recv() {
            Ok(msg) => {
                match msg {
                    CommonMessage::HumanDetected => {
                        v_tx.lock().unwrap().send(CommonMessage::ObjectRecognizeOrder).await.unwrap();
                    },
                    CommonMessage::OrderDetected(intent_type) => {
                        let mut thing_lock = thing.lock().unwrap();
                        dbg!(&thing_lock);
                        dbg!(&intent_type);
                        match intent_type {
                            IntentType::TurnOnSomething => {}
                            IntentType::TurnOffSomething => {}
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
    Ok(())
}

fn no_target() {

}

async fn va_main_without_vision(config: Config) -> Result<()> {
    let halt = Cell::new(false);
    let listening = Cell::new(false);
    let timeout = Cell::new(0);
    let (mut auditio_tx, mut auditio_rx) = mpsc::channel::<CommonMessage>(32);
    let mut a_tx = Arc::new(Mutex::new(auditio_tx));
    let mut a_rx = Arc::new(Mutex::new(auditio_rx));
    let mut recorder = AudioRecorder::new(config.mic_index);
    let mut vad = auditio::create_vad(config.pv_api_key.as_str()).unwrap();
    let mut sti = auditio::create_sti(config.pv_api_key.as_str(), config.pv_model_name.as_str()).unwrap();
    // let mut stt = auditio::create_stt(config.pv_api_key.as_str()).unwrap();
    tokio::join!(
        listen_mic(a_tx.clone(), &mut vad, &mut sti, &mut recorder, &listening, &timeout, &halt),
        // message_centre(a_tx.clone(), a_rx.clone(), share_config.clone(),&halt),
    );
    Ok(())
}

async fn va_main(config: Config) -> Result<()> {
    let halt = Cell::new(false);
    let listening = Cell::new(false);
    let timeout = Cell::new(0);
    let mut frame = Arc::new(Mutex::new(Mat::default()));
    let mut gaze = Arc::new(Mutex::new(Point2f::default()));
    let mut current_thing_id: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));
    let (mut visio_tx, mut visio_rx) = mpsc::channel::<CommonMessage>(32);
    let mut v_tx = Arc::new(Mutex::new(visio_tx));
    let mut v_rx = Arc::new(Mutex::new(visio_rx));
    let (mut auditio_tx, mut auditio_rx) = mpsc::channel::<CommonMessage>(32);
    let mut a_tx = Arc::new(Mutex::new(auditio_tx));
    let mut a_rx = Arc::new(Mutex::new(auditio_rx));

    let mut visio_capture = visio::VisioCapture::new();
    visio_capture.init();
    let mut recorder = AudioRecorder::new(config.mic_index);
    let mut vad = auditio::create_vad(config.pv_api_key.as_str()).unwrap();
    let mut sti = auditio::create_sti(config.pv_api_key.as_str(), config.pv_model_name.as_str()).unwrap();
    // let mut stt = auditio::create_stt(config.pv_api_key.as_str()).unwrap();
    recorder.start_listen();
    let share_config = Arc::new(Mutex::new(config));
    tokio::join!(
        capture_frame(&mut visio_capture, frame.clone(), gaze.clone(), &halt),
        listen_mic(a_tx.clone(), &mut vad, &mut sti, &mut recorder, &listening, &timeout, &halt),
        message_centre(v_tx.clone(), v_rx.clone(), frame.clone(), gaze.clone(), current_thing_id,
            a_tx.clone(), a_rx.clone(), share_config.clone(),&halt),
        debug::draw_debug_frame(frame.clone(), gaze.clone(), &halt)
    );
    Ok(())
}

fn read_config() -> Result<Config> {
    match std::fs::File::open("config.yml") {
        Ok(f) => {
            match serde_yaml::from_reader(f) {
                Ok(c) => {
                    Ok(c)
                },
                Err(_) => {
                    Err(anyhow!("can't parse config file! check config.yml or run init again."))
                }
            }
        }
        Err(_) => {
            Err(anyhow!("can't open config file! check config.yml is exist or run init first."))
        }
    }
}

fn write_config(markers: Vec<i32>) -> Result<()> {
    let mut config: Config = Default::default();
    for marker in markers {
        config.actions.push(
            AgentAction {
                id: marker,
                name: format!("marker_{}", marker),
                action_type: ActionType::Beep,
                command: "".to_string(),
            }
        )
    }

    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("config.yml")
        .expect("can't open file");
    serde_yaml::to_writer(f, &config).expect("failed to write yaml!");
    Ok(())
}


pub(crate) async fn init_marker() -> Result<()> {
    write_config(visio::generate_aruco(10).unwrap_or(vec![]))
}

async fn check_pre_require() -> Result<Config> {
    read_config()
}

pub(crate) async fn run_va() -> Result<()> {
    match check_pre_require().await {
        Ok(config) => {
            if config.with_vision {
                va_main(config).await
            } else {
                va_main_without_vision(config).await
            }
        },
        Err(err) => {
            println!("{}", err);
            Ok(())
        },
    }
}

