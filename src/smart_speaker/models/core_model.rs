use std::fmt::{Display, Formatter};
use crate::smart_speaker::models::vision_model::VisionAction;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerState {
    Idle,
    Attention,
    Pending(PendingType),
}

impl Display for SmartSpeakerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SmartSpeakerState::Idle => write!(f, "Idle"),
            SmartSpeakerState::Attention => write!(f, "Attention"),
            SmartSpeakerState::Pending(pending_type) => write!(f, "Pending({})", pending_type),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PendingType {
    Speak,
    Vision(Vec<VisionAction>),
}

impl Display for PendingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PendingType::Speak => write!(f, "Speak"),
            PendingType::Vision(VisionAction) => write!(f, "Vision"),
        }
    }
}
