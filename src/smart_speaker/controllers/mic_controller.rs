use anyhow::{Result};
use crate::smart_speaker::models::mic_model::{AudioListener, SpeechToIntent};

pub(crate) fn listen_mic(listener: &mut AudioListener) -> Result<Vec<i16>> {
    let record = listener.update().expect("failed to update from listener");
    Ok(record)
}

pub(crate)  fn speech_to_intent_feed(speech_to_intent: &mut SpeechToIntent, record: &Vec<i16>) -> Result<bool>  {
    return if let Ok(finalized) = speech_to_intent.app.process(record) {
        Ok(finalized)
    } else {
        Ok(false)
    }
}
