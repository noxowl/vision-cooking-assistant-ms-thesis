use std::sync::mpsc;
use crate::smart_speaker::models::vision_model::Capture;
use crate::utils::message_util::SmartSpeakerMessage;

pub(crate) struct VisionActor {
    alive: bool,
    core: Capture,
    receiver: mpsc::Receiver<SmartSpeakerMessage>,
    sender: mpsc::Sender<SmartSpeakerMessage>,
}

impl VisionActor {
    pub(crate) fn new(core: Capture, receiver: mpsc::Receiver<SmartSpeakerMessage>, sender: mpsc::Sender<SmartSpeakerMessage>) -> Self {
        Self {
            alive: true,
            core,
            receiver,
            sender,
        }
    }

    pub(crate) fn run(&mut self) {
        println!("VisionActor started");
        self.core.info();
        while self.alive {
            if let Ok(message) = self.receiver.try_recv() {
                self.handle_message(message);
            }
        }
    }

    fn handle_message(&mut self, message: SmartSpeakerMessage) {
        match message {
            SmartSpeakerMessage::RequestShutdown(_) => {
                self.alive = false;
            },
            _ => {}
        }
    }
}

//  fn capture_frame(capture: &mut visio::VisioCapture, frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>, halt: &Cell<bool>) {
//     while !halt.get() {
//         match capture.capture() {
//             Ok(m) => {
//                 {
//                     let mut frame_lock = frame.lock().unwrap();
//                     *frame_lock = m;
//                 }
//                 match capture.capture_gaze() {
//                     Ok(g) => {
//                         let mut gaze_lock = gaze.lock().unwrap();
//                         *gaze_lock = g;
//                     }
//                     _ => {}
//                 }
//             },
//             _ => {}
//         }
//         tokio::time::sleep(Duration::from_millis(33));
//     }
// }