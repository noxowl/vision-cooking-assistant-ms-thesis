#![feature(exclusive_range_pattern)]
mod auditio;
mod lib;
mod visio;

use std::borrow::BorrowMut;
use std::cell::Cell;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::auditio::{AudioRecorder, create_vad, create_stt};
use crate::visio::{VisioCapture, generate_aruco, detect_aruco, draw_aruco, find_nearest_aruco};
use anyhow::{anyhow, Context, Error, Result};
use cobra::Cobra;
use opencv::{self as cv, imgproc, prelude::*, videoio};
use opencv::core::{Point, Point2f, Vector};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use VAs_got_vision::CommonMessage;

async fn draw_debug_frame(frame: Arc<Mutex<Mat>>, halt: &Cell<bool>) {
    while !halt.get() {
        {
            let mut frame_lock = frame.lock().unwrap();
            cv::highgui::imshow("visio - debug", &*frame_lock).unwrap();
        }
        cv::highgui::wait_key(10).unwrap();
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}

async fn capture_frame(capture: &mut VisioCapture, frame: Arc<Mutex<Mat>>, halt: &Cell<bool>) {
    while !halt.get() {
        match capture.capture().await {
            Ok(m) => {
                {
                    let mut frame_lock = frame.lock().unwrap();
                    *frame_lock = m;
                }
            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}

async fn listen_mic(tx: Arc<Mutex<Sender<CommonMessage>>>, vad: &mut Cobra, recorder: &mut AudioRecorder, halt: &Cell<bool>) {
    while !halt.get() {
        let result = recorder.listen(vad).await;
        match result {
            Ok(true) => {
                match tx.lock().unwrap().send(CommonMessage::HumanDetected).await {
                    Ok(_) => {}
                    Err(_) => {
                        dbg!("recv true. send message failed.");
                    }
                }
            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn message_centre(v_tx: Arc<Mutex<Sender<CommonMessage>>>, v_rx: Arc<Mutex<Receiver<CommonMessage>>>, frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>,
                        a_tx: Arc<Mutex<Sender<CommonMessage>>>, a_rx: Arc<Mutex<Receiver<CommonMessage>>>, halt: &Cell<bool>) -> Result<()> {
    while !halt.get() {
        match v_rx.lock().unwrap().try_recv() {
            Ok(msg) => {
                match msg {
                    CommonMessage::ObjectRecognizeOrder => {
                        {
                            dbg!("recv recognize order!");
                            let mut frame_lock = frame.lock().unwrap();
                            let aruco_result = detect_aruco(&*frame_lock).await.unwrap();
                            if aruco_result.1.len() > 0 {
                                let mut gaze_lock = gaze.lock().unwrap();
                                let nearest = find_nearest_aruco(&*gaze_lock, &aruco_result.0, &aruco_result.1).await?;
                                let mut nearest_corner = Vector::new();
                                nearest_corner.push(nearest.corner.clone());
                                let mut nearest_id = Vector::new();
                                nearest_id.push(nearest.id.clone());
                                draw_aruco(&mut *frame_lock,&nearest_corner, &nearest_id).await;
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
                    _ => {}
                }
            },
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    Ok(())
}

async fn check_pre_require() -> Result<()> {
    Ok(())
}

async fn run_va() -> Result<()> {
    match check_pre_require().await {
        Ok(_) => {
            va_main().await
        },
        _ => {
            Ok(())
        },
    }
}

async fn va_main() -> Result<()> {
    let halt = Cell::new(false);
    let mut frame = Arc::new(Mutex::new(Mat::default()));
    let mut gaze = Arc::new(Mutex::new(Point2f::default()));
    let (mut visio_tx, mut visio_rx) = mpsc::channel::<CommonMessage>(32);
    let mut v_tx = Arc::new(Mutex::new(visio_tx));
    let mut v_rx = Arc::new(Mutex::new(visio_rx));
    let (mut auditio_tx, mut auditio_rx) = mpsc::channel::<CommonMessage>(32);
    let mut a_tx = Arc::new(Mutex::new(auditio_tx));
    let mut a_rx = Arc::new(Mutex::new(auditio_rx));

    let mut visio_capture = VisioCapture::new();
    visio_capture.init();
    let mut recorder = AudioRecorder::new();
    let mut vad = create_vad().unwrap();
    let mut stt = create_stt().unwrap();
    recorder.start_listen();

    tokio::join!(
        capture_frame(&mut visio_capture, frame.clone(), &halt),
        listen_mic(a_tx.clone(), &mut vad, &mut recorder, &halt),
        message_centre(v_tx.clone(), v_rx.clone(), frame.clone(), gaze.clone(),
            a_tx.clone(), a_rx.clone(), &halt),
        draw_debug_frame(frame.clone(), &halt)
    );
    Ok(())
}

async fn init_marker() -> Result<()> {
    generate_aruco()
}

struct Cli {
    command: String,
    option: Option<String>
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = env::args().nth(1).expect("no query given");
    let option = env::args().nth(2);
    let args = Cli {
        command,
        option,
    };
    match &args.command[..] {
        "run" => {
            run_va().await?
        },
        "init" => {
            init_marker().await?
        },
        _ => {
            println!("no matched query found. type help for available commands.");
        }
    }
    Ok(())
}
