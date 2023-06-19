use opencv::core::Mat;
use anyhow::Result;
use crate::smart_speaker::models::vision_model::CaptureSource;

pub(crate) fn capture_frame(source: &mut Box<dyn CaptureSource + Send + 'static>) -> Result<Mat> {
    source.get_frame()
}
