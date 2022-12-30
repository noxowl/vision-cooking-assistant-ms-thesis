use std::borrow::Borrow;
use crate::lib::CommonMessage;
use anyhow::{anyhow, Result};
use opencv::core::{CV_8UC1, hconcat, hconcat2, Point, Point2f, Rect, Scalar, Size, vconcat, Vector, VectorExtern};
use opencv::videoio::VideoCapture;
use opencv::{self as cv, imgproc, prelude::*, videoio, aruco};
use std::future::Future;
use std::time::Duration;
use opencv::types::{VectorOfi32, VectorOfMat, VectorOfPoint2f, VectorOfVectorOfPoint, VectorOfVectorOfPoint2f};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};


enum CaptureSource {
    Pupil,
    FallbackCam,
}

pub struct VisioCapture {
    capture_source: CaptureSource,
    current_frame: Mat,
    fallback_video_capture: VideoCapture,
}

impl VisioCapture {
    pub fn new() -> Self {
        Self {
            capture_source: CaptureSource::FallbackCam,
            current_frame: Mat::default(),
            fallback_video_capture: VideoCapture::default().unwrap(),
        }
    }

    pub fn init(&mut self) {
        self.init_capture_source();
    }

    fn init_capture_source(&mut self) {
        match self.setup_from_pupil() {
            Ok(result) => {
                println!("pupil ok");
                self.capture_source = CaptureSource::Pupil
            }
            Err(e) => {
                println!("pupil error");
                match self.setup_from_cam() {
                    Ok(result) => {
                        println!("fallback setup ok");
                        dbg!(&self.fallback_video_capture.get_backend_name());
                        self.capture_source = CaptureSource::FallbackCam
                    }
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    }

    fn setup_from_pupil(&mut self) -> Result<()> {
        Err(anyhow!("no pupil connection found!"))
    }

    fn capture_from_pupil(&mut self) -> Result<()> {
        Err(anyhow!("no pupil connection found!"))
    }

    fn setup_from_cam(&mut self) -> Result<()> {
        self.fallback_video_capture = VideoCapture::new(0, videoio::CAP_ANY)?;
        println!("cam setup");
        Ok(())
    }

    fn capture_from_cam(&mut self) -> Result<()> {
        match self.fallback_video_capture.is_opened() {
            Ok(_) => {
                self.fallback_video_capture.grab()?;
                match self.fallback_video_capture
                    .retrieve(&mut self.current_frame, videoio::CAP_ANY)
                {
                    Ok(result) => {
                        Ok(())
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        Err(anyhow!("capture failed!"))
                    }
                }
            }
            Err(e) => Err(anyhow!("camera not opened!")),
        }
    }

    pub async fn capture(&mut self) -> Result<Mat> {
        match self.capture_source {
            CaptureSource::Pupil => {
                self.capture_from_pupil()?;
            }
            CaptureSource::FallbackCam => {
                self.capture_from_cam()?;
            }
        }
        Ok(self.current_frame.clone())
    }
}

pub struct DetectedMarker {
    pub corner: VectorOfPoint2f,
    pub id: i32,
}

impl DetectedMarker {
    fn new(corner: Vector<Point2f>, id: i32) -> Self {
        Self {
            corner,
            id,
        }
    }

    fn update(mut self, corner: Vector<Point2f>, id: i32) {
        self.corner = corner;
        self.id = id;
    }
    
    fn default() -> Self {
        Self {
            corner: Default::default(),
            id: 0,
        }
    }
}

fn centroid(corner: &Point) -> Point {
    Point::new(corner.x / 2, corner.y / 2)
}

fn midpoint(px: &Point2f, py: &Point2f) -> Point2f {
    Point2f::new((px.x + py.x) / 2., (px.y + py.y) / 2.)
}

pub async fn find_nearest_aruco(gaze: &Point2f, corners: &VectorOfVectorOfPoint2f, ids: &Vector<i32>) -> Result<DetectedMarker> {
    let mut nearest_index = 0;
    if ids.len() > 0 {
        let mut nearest_mid = Point2f::default();
        for i in 0..ids.len() {
            let square = corners.get(i).unwrap();
            let mid = midpoint(gaze, &midpoint(&square.get(0).unwrap(),&square.get(2).unwrap()));
            if i == 0 || &mid < &nearest_mid {
                dbg!(&mid);
                &nearest_mid.clone_from(&mid);
                *&nearest_index.clone_from(&i);
            }
        }
    }
    Ok(DetectedMarker::new(corners.get(nearest_index).unwrap(),ids.get(nearest_index).unwrap()))
}

pub async fn detect_aruco(frame: &Mat) -> Result<(VectorOfVectorOfPoint2f, Vector<i32>)> {
    let parameters = aruco::DetectorParameters::create()?;
    let dictionary = aruco::get_predefined_dictionary(aruco::PREDEFINED_DICTIONARY_NAME::DICT_4X4_50)?;
    let mut corners: VectorOfVectorOfPoint2f = Default::default();
    let mut rejected: VectorOfVectorOfPoint2f = Default::default();
    let mut ids = VectorOfi32::default();
    aruco::detect_markers(frame, &dictionary, &mut corners, &mut ids, &parameters, &mut rejected)?;
    Ok((corners, ids))
}

pub async fn draw_aruco(frame: &mut Mat, corners: &VectorOfVectorOfPoint2f, ids: &VectorOfi32) {
    aruco::draw_detected_markers(frame, corners, ids, Scalar::new(255., 0., 0., 255.)).unwrap();
}

fn generate_padding_img(mat: &Mat) -> Result<Mat> {
    Ok(Mat::new_rows_cols_with_default(
        mat.rows(), mat.cols(),
        CV_8UC1, Scalar::new(255., 255., 255., 255.,))?)
}

pub fn generate_aruco() -> Result<()> {
    let generate_amount = 10;
    let pixel = 200;
    let mut img: Mat = Default::default();
    let mut imwrite_params = VectorOfi32::new();
    imwrite_params.push(cv::imgcodecs::IMWRITE_PNG_COMPRESSION);
    imwrite_params.push(1);
    let mut concat_temp_1: VectorOfMat = Default::default();
    let mut concat_temp_2: VectorOfMat = Default::default();
    let mut v_concat_1: Mat = Default::default();
    let mut v_concat_2: Mat = Default::default();

    let dictionary = aruco::get_predefined_dictionary(aruco::PREDEFINED_DICTIONARY_NAME::DICT_4X4_50)?;
    for i in 0..generate_amount {
        let mut ar_marker: Mat = Default::default();
        aruco::draw_marker(&dictionary, i, *&pixel, &mut ar_marker, 1);
        let mut padding_img = Mat::new_rows_cols_with_default(
            ar_marker.rows() + 100, ar_marker.cols() + 100,
            CV_8UC1, Scalar::new(255., 255., 255., 255.,))?;
        let mut padding_roi = Mat::roi(&mut padding_img, Rect::new(50, 50, ar_marker.cols(), ar_marker.rows()))?;
        ar_marker.copy_to(&mut padding_roi).unwrap();
        if i % 2 == 0 {
            concat_temp_1.push(padding_img.clone());
        } else {
            concat_temp_2.push(padding_img.clone());
        }
        cv::imgcodecs::imwrite(format!("ar_{i}.png").as_str(), &ar_marker, &imwrite_params)?;
    }
    match concat_temp_1.len() as i16 - concat_temp_2.len() as i16 {
        1 => {
            concat_temp_2.push(generate_padding_img(&concat_temp_1.get(0)?)?.clone());
        },
        -1 => {
            concat_temp_1.push(generate_padding_img(&concat_temp_1.get(0)?)?.clone());
        },
        _ => {}
    }
    vconcat(&concat_temp_1, &mut v_concat_1)?;
    vconcat(&concat_temp_2, &mut v_concat_2)?;
    hconcat2(&v_concat_1, &v_concat_2, &mut img).unwrap();
    cv::imgcodecs::imwrite("ar_all.png", &img, &imwrite_params).unwrap();
    Ok(())
}

// pub struct VisioActor {
//     capture_source: CaptureSource,
//     pub current_frame: Mat,
//     last_frame_buffer: Vec<Mat>,
//     captured_roi: Mat,
//     gaze: Point,
//     fallback_video_capture: VideoCapture,
// }
//
// impl VisioActor {
//     pub fn new(
//         tx: mpsc::UnboundedSender<CommonMessage>,
//         rx: mpsc::UnboundedReceiver<CommonMessage>,
//     ) -> Self {
//         Self {
//             capture_source: CaptureSource::FallbackCam,
//             current_frame: Mat::default(),
//             last_frame_buffer: vec![],
//             captured_roi: Mat::default(),
//             gaze: Default::default(),
//             fallback_video_capture: VideoCapture::default().unwrap(),
//         }
//     }
//
//     // fn raw_frame_to_mat(&mut self) -> Mat {
//     //
//     // }
//
//     fn raw_gaze_to_point(&mut self) {}
//
//     fn fetch_gaze(&mut self) -> () {
//         match self.capture_source {
//             CaptureSource::Pupil => match self.fetch_gaze_from_pupil() {
//                 Ok(gaze) => self.raw_gaze_to_point(),
//                 Err(e) => match self.gaze_fallback() {
//                     Ok(gaze) => self.gaze = gaze,
//                     Err(e) => self.gaze = Point::new(0, 0),
//                 },
//             },
//             CaptureSource::FallbackCam => match self.gaze_fallback() {
//                 Ok(gaze) => self.gaze = gaze,
//                 Err(e) => self.gaze = Point::new(0, 0),
//             },
//         }
//     }
//
//     fn fetch_gaze_from_pupil(&mut self) -> Result<()> {
//         Err(anyhow!("can't fetch from pupil!"))
//     }
//
//     fn gaze_fallback(&mut self) -> Result<Point> {
//         if self.current_frame.cols() > 0 {
//             Ok(Point::new(
//                 self.current_frame.cols() / 2,
//                 self.current_frame.rows() / 2,
//             ))
//         } else {
//             Err(anyhow!("no frame found!"))
//         }
//     }
// }
