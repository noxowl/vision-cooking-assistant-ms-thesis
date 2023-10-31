#[cfg(test)]
mod core_actor_tests {
    use std::collections::HashMap;
    use std::sync::mpsc;
    use crate::smart_speaker::models::core_model::SmartSpeakerState;
    use crate::smart_speaker::models::debug_model::DebugData;
    use crate::utils::message_util::{ReportTerminated, RequestStateUpdate, RequestAudioStream, RequestShutdown, SmartSpeakerActors, SmartSpeakerMessage};
    use super::super::core_actor::*;

    #[test]
    fn core_message_handler_shutdown_test() {
        let (tx, rx) = mpsc::channel();
        let mut senders = HashMap::new();
        senders.insert(SmartSpeakerActors::CoreActor, tx);
        let mut handler = CoreActorMessageHandler {
            debug: DebugData::new(false),
            };
        let message = SmartSpeakerMessage::RequestShutdown(RequestShutdown {});
        let state = handler.handle_message(senders, message);
        assert_eq!(state, CoreActorState::ShutdownRequested {});
        assert_eq!(rx.recv().unwrap(), SmartSpeakerMessage::RequestShutdown(RequestShutdown {}));
    }

    #[test]
    fn core_message_handler_terminate_test() {
        let (tx, _) = mpsc::channel();
        let mut senders = HashMap::new();
        senders.insert(SmartSpeakerActors::CoreActor, tx);
        let mut handler = CoreActorMessageHandler {
            debug: DebugData::new(false),
        };
        let message = SmartSpeakerMessage::ReportTerminated(ReportTerminated {
            send_from: SmartSpeakerActors::CoreActor, send_to: SmartSpeakerActors::CoreActor});
        let state = handler.handle_message(senders, message);
        assert_eq!(state, CoreActorState::ActorTerminated { actor: SmartSpeakerActors::CoreActor });
    }

    #[test]
    fn core_message_handler_audio_stream_test() {
        let (tx, rx) = mpsc::channel();
        let mut senders = HashMap::new();
        senders.insert(SmartSpeakerActors::CoreActor, tx);
        let mut handler = CoreActorMessageHandler {
            debug: DebugData::new(false),
        };
        let message = SmartSpeakerMessage::RequestAudioStream(RequestAudioStream {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::CoreActor,
            stream: vec![],
        });
        let state = handler.handle_message(senders, message);
        assert_eq!(state, CoreActorState::WaitForNextMessage {});
        assert_eq!(rx.recv().unwrap(), SmartSpeakerMessage::RequestAudioStream(RequestAudioStream {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::CoreActor,
            stream: vec![],
        }));

    }

    #[test]
    fn core_message_handler_attention_test() {
        let (tx, _) = mpsc::channel();
        let mut senders = HashMap::new();
        senders.insert(SmartSpeakerActors::CoreActor, tx);
        let mut handler = CoreActorMessageHandler {
            debug: DebugData::new(false),
        };
        let message = SmartSpeakerMessage::RequestStateUpdate(RequestStateUpdate {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::CoreActor,
            state: SmartSpeakerState::Attention,
        });
        let state = handler.handle_message(senders, message);
        assert_eq!(state, CoreActorState::NewActorRequested { actor: SmartSpeakerActors::SpeechToIntentActor, custom_args: None });
    }

    #[test]
    fn core_message_handler_attention_finished_test() {
        let (tx, _) = mpsc::channel();
        let mut senders = HashMap::new();
        senders.insert(SmartSpeakerActors::CoreActor, tx);
        let mut handler = CoreActorMessageHandler {
            debug: DebugData::new(false),
        };
        let message = SmartSpeakerMessage::RequestStateUpdate(RequestStateUpdate {
            send_from: SmartSpeakerActors::CoreActor,
            send_to: SmartSpeakerActors::CoreActor,
            state: SmartSpeakerState::Attention,
        });
        let state = handler.handle_message(senders, message);
        assert_eq!(state, CoreActorState::NewActorRequested { actor: SmartSpeakerActors::WakeWordActor, custom_args: None });
    }
}
