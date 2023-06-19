#[cfg(test)]
mod camera_actor_tests {
    use std::sync::mpsc;
    use std::thread;
    use opencv::prelude::*;
    use opencv::core::Mat;
    use crate::smart_speaker::actors::camera_actor::CameraActor;
    use crate::smart_speaker::models::vision_model::{CameraCaptureSource, Capture};
    use crate::utils::camera_util::Camera;
    use crate::utils::message_util::{RequestCameraFrame, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};
    use crate::utils::vision_util::set_camera_capture;

    #[test]
    fn camera_actor_test() {
        let (core_tx, core_rx) = mpsc::channel::<SmartSpeakerMessage>();
        let (actor_tx, actor_rx) = mpsc::channel::<SmartSpeakerMessage>();
        let mut capture_source = Capture::new();
        set_camera_capture(&mut capture_source).expect("TODO: panic message");
        let mut camera_actor = CameraActor::new(capture_source, actor_rx, core_tx.clone());
        thread::spawn(move || {
            camera_actor.run();
        });
        thread::sleep(std::time::Duration::from_millis(180)); // cold-start duration at development mac mini(2014)
        actor_tx.send(SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::CameraActor,
            frame_data_bytes: vec![],
            height: 0,
        })).expect("TODO: panic message");
        thread::sleep(std::time::Duration::from_millis(33));
        let message = core_rx.try_recv().expect("TODO: panic message");
        match message {
            SmartSpeakerMessage::RequestCameraFrame(RequestCameraFrame { send_from, send_to,
                                                        frame_data_bytes, height }) => {
                assert_eq!(send_from, SmartSpeakerActors::CameraActor);
                assert_eq!(send_to, SmartSpeakerActors::CoreActor);
                let mat = Mat::from_slice(&frame_data_bytes).unwrap();
                let mat = mat.reshape(3, height).unwrap();
                assert_eq!(mat.size().unwrap().width >= 640, true);
                assert_eq!(mat.size().unwrap().height >= 480, true);
            },
            _ => {
                panic!("unexpected message");
            }
        }
        actor_tx.send(SmartSpeakerMessage::RequestShutdown(RequestShutdown {})).expect("TODO: panic message");
        actor_tx.send(SmartSpeakerMessage::RequestShutdown(RequestShutdown {})).expect("TODO: panic message");
        assert!(core_rx.try_recv().is_err());
    }
}
