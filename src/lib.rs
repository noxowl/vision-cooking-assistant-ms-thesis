extern crate tokio_core;

#[derive(Debug, Clone)]
pub enum CommonMessage {
    HumanDetected,
    ObjectRecognizeOrder,
    DetectedObject(String),
    OrderDetected,
    OrderCancelled,
    RequestVA(String),
}
