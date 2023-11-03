#[cfg(test)]
mod audio_actor_tests {
    use std::sync::mpsc;
    use std::thread;
    use crate::smart_speaker::models::mic_model::AudioListener;
    use crate::utils::message_util::{RequestAudioStream, ShutdownMessage, SmartSpeakerActors, SmartSpeakerMessage};
    use super::super::audio_actor::*;

    #[test]
    fn audio_actor_test() {
        let (core_tx, core_rx) = mpsc::channel::<SmartSpeakerMessage>();
        let (actor_tx, actor_rx) = mpsc::channel::<SmartSpeakerMessage>();
        let mut audio_actor = AudioActor::new(AudioListener::new(0), actor_rx, core_tx.clone());
        thread::spawn(move || {
            audio_actor.run();
        });
        thread::sleep(std::time::Duration::from_millis(100)); // cold-start duration at development mac mini(2014)
        actor_tx.send(SmartSpeakerMessage::RequestAudioStream(RequestAudioStream {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::AudioActor,
            stream: vec![],
        })).expect("TODO: panic message");
        thread::sleep(std::time::Duration::from_millis(33)); // wait for first audio stream
        let message = core_rx.try_recv().expect("TODO: panic message");
        match message {
            SmartSpeakerMessage::RequestAudioStream(RequestAudioStream { send_from, send_to, stream }) => {
                assert_eq!(send_from, SmartSpeakerActors::AudioActor);
                assert_eq!(send_to, SmartSpeakerActors::CoreActor);
                assert_eq!(stream.len() >= 0, true);
            },
            _ => {
                panic!("unexpected message");
            }
        }
        actor_tx.send(SmartSpeakerMessage::RequestShutdown(ShutdownMessage {})).expect("TODO: panic message");
        actor_tx.send(SmartSpeakerMessage::RequestShutdown(ShutdownMessage {})).expect("TODO: panic message");
        assert!(core_rx.try_recv().is_err());
    }
}
