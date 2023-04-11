use std::cell::Cell;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use opencv::core::{Mat, Point2f};
use opencv::highgui;
use crate::visio;

pub async fn draw_debug_frame(frame: Arc<Mutex<Mat>>, gaze: Arc<Mutex<Point2f>>, halt: &Cell<bool>) {
    while !halt.get() {
        {
            let mut frame_lock = frame.lock().unwrap();
            let mut gaze_lock = gaze.lock().unwrap();
            visio::debug_draw_marker(&mut *frame_lock, [gaze_lock.x as i32, gaze_lock.y as i32], [255., 0., 0.]).await;
            highgui::imshow("visio - debug", &*frame_lock).unwrap();
        }
        highgui::wait_key(10).unwrap();
        tokio::time::sleep(Duration::from_millis(33)).await;
    }
}