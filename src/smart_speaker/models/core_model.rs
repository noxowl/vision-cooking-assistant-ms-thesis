use std::fmt::{Display, Formatter};
use crate::smart_speaker::models::vision_model::VisionAction;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerState {
    Idle,
    Attention,
    WaitingForInteraction(WaitingInteraction),
}

impl Display for SmartSpeakerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SmartSpeakerState::Idle => write!(f, "Idle"),
            SmartSpeakerState::Attention => write!(f, "Attention"),
            SmartSpeakerState::WaitingForInteraction(waiting) => write!(f, "Pending({})", &waiting),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WaitingInteraction {
    Speak,
    Vision(Vec<VisionAction>),
}

impl Display for WaitingInteraction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitingInteraction::Speak => write!(f, "Speak"),
            WaitingInteraction::Vision(actions) => write!(f, "Vision({:?})", &actions),
        }
    }
}
