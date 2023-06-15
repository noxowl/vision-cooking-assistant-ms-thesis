use opencv::core::Mat;
use anyhow::Result;

pub(crate) struct Pupil {
    pub pupil_remote: PupilRemote,
}

impl Pupil {
    pub fn new(pupil_remote: PupilRemote) -> Self {
        Self { pupil_remote }
    }

    pub fn get_frame(&self) -> Result<Mat> {
        let mut frame = Mat::default();
        self.pupil_remote.get_frame(&mut frame)?;
        Ok(frame)
    }

    pub fn get_gaze(&self) -> Result<(f64, f64)> {
        let mut gaze = (0., 0.);
        self.pupil_remote.get_gaze(&mut gaze)?;
        Ok(gaze)
    }
}

pub(crate) struct PupilRemote {
}

impl PupilRemote {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_frame(&self, frame: &mut Mat) -> Result<()> {
        Ok(())
    }

    pub fn get_gaze(&self, gaze: &mut (f64, f64)) -> Result<()> {
        Ok(())
    }
}