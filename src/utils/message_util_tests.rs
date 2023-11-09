#[cfg(test)]
mod message_util_tests {
    use std::sync::mpsc;
    use crate::smart_speaker::models::core_model::SmartSpeakerState;
    use crate::smart_speaker::models::message_model::*;
    use super::super::message_util::*;

    #[test]
    fn audio_stream_message_test() {
        let (sender, receiver) = mpsc::channel::<SmartSpeakerMessage>();
        audio_stream_message(
            &sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::CoreActor,
            vec![],
        );
        let message = receiver.recv().unwrap();
        match message {
            SmartSpeakerMessage::RequestAudioStream(AudioStreamMessage { send_from, send_to, stream }) => {
                assert_eq!(send_from, SmartSpeakerActors::WakeWordActor);
                assert_eq!(send_to, SmartSpeakerActors::CoreActor);
                assert_eq!(stream, Vec::<i16>::new());
            },
            _ => {
                panic!("unexpected message");
            }
        }
    }

    #[test]
    fn attention_message_test() {
        let (sender, receiver) = mpsc::channel::<SmartSpeakerMessage>();
        state_update_message(
            &sender,
            SmartSpeakerActors::WakeWordActor,
            SmartSpeakerActors::CoreActor,
            SmartSpeakerState::Attention,
        );
        let message = receiver.recv().unwrap();
        match message {
            SmartSpeakerMessage::RequestStateUpdate(
                StateUpdateMessage { send_from, send_to, state }) => {
                assert_eq!(send_from, SmartSpeakerActors::WakeWordActor);
                assert_eq!(send_to, SmartSpeakerActors::CoreActor);
            },
            _ => {
                panic!("unexpected message");
            }
        }
    }

    #[test]
    fn terminate_message_test() {
        let (sender, receiver) = mpsc::channel::<SmartSpeakerMessage>();
        terminate_message(
            &sender,
            SmartSpeakerActors::WakeWordActor,
        );
        let message = receiver.recv().unwrap();
        match message {
            SmartSpeakerMessage::ReportTerminated(ReportTerminated { send_from, send_to }) => {
                assert_eq!(send_from, SmartSpeakerActors::WakeWordActor);
                assert_eq!(send_to, SmartSpeakerActors::CoreActor);
            },
            _ => {
                panic!("unexpected message");
            }
        }
    }
}