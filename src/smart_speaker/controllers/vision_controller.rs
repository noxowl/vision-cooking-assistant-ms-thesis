use anyhow::{anyhow, Result};
use opencv::{objdetect, imgproc, imgcodecs};
use opencv::prelude::*;
use opencv::core::{Point, Point2f, Scalar, Vector, Size};
use opencv::types::{VectorOfi32, VectorOfPoint2f, VectorOfVectorOfPoint2f};


pub(crate) fn data_bytes_to_mat(bytes: Vec<u8>, height: i32) -> Result<Mat> {
    match Mat::from_slice(&bytes) {
        Ok(mat) => {
            match mat.reshape(3, height) {
                Ok(mat) => {
                    Ok(mat)
                }
                Err(_) => {
                    Err(anyhow!("failed to reshape Mat"))
                }
            }
        }
        Err(_) => {
            Err(anyhow!("failed to convert bytes to Mat"))
        }
    }
}

pub(crate) fn resize_frame(frame: Mat) -> Mat {
    let mut resized_frame = Mat::default();
    imgproc::resize(
        &frame,
        &mut resized_frame,
        Size {
            width: frame.cols() / 2,
            height: frame.rows() / 2,
        },
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    )
    .unwrap();
    resized_frame
}

pub(crate) struct DetectedMarker {
    pub corner: VectorOfPoint2f,
    pub id: i32,
    pub centroid: Point2f,
}

impl DetectedMarker {
    fn new(corner: Vector<Point2f>, id: i32, centroid: Point2f) -> Self {
        Self {
            corner,
            id,
            centroid,
        }
    }

    fn update(mut self, corner: Vector<Point2f>, id: i32, centroid: Point2f) {
        self.corner = corner;
        self.id = id;
        self.centroid = centroid;
    }

    fn default() -> Self {
        Self {
            corner: Default::default(),
            id: 0,
            centroid: Default::default(),
        }
    }
}

// pub  fn find_nearest_aruco(gaze: &(f32, f32), corners: &VectorOfVectorOfPoint2f, ids: &Vector<i32>) -> Result<DetectedMarker> {
//     let mut nearest_index = 0;
//     if ids.len() > 0 {
//         let mut nearest_distance: f32 = 0.0;
//         for i in 0..ids.len() {
//             let square = corners.get(i).unwrap();
//             let square_mid = vision_util::midpoint(
//                 &square.get(0).unwrap(),
//                 &square.get(2).unwrap());
//             let dist = vision_util::distance(
//                 &gaze.0, &gaze.1,
//                 &);
//             if i == 0 || &dist < &nearest_distance {
//                 nearest_distance = dist;
//                 *&nearest_index.clone_from(&i);
//             }
//         }
//     }
//     Ok(DetectedMarker::new(
//         corners.get(nearest_index).unwrap(),
//         ids.get(nearest_index).unwrap(),
//         vision_util::midpoint(&corners.get(nearest_index).unwrap().get(0).unwrap(),&corners.get(nearest_index).unwrap().get(2).unwrap())
//     ))
// }

pub(crate) fn detect_aruco(frame: &Mat, max_markers: usize) -> Result<(VectorOfVectorOfPoint2f, Vector<i32>)> {
    let parameters = opencv::objdetect::DetectorParameters::default()?;
    let dictionary = opencv::objdetect::get_predefined_dictionary(objdetect::PredefinedDictionaryType::DICT_4X4_50)?;
    let mut corners: VectorOfVectorOfPoint2f = Default::default();
    let mut rejected: VectorOfVectorOfPoint2f = Default::default();
    let mut ids = VectorOfi32::default();
    let detector = objdetect::ArucoDetector::new(&dictionary, &parameters,
                                                 objdetect::RefineParameters::new(10., 3., true).unwrap())?;
    detector.detect_markers(frame, &mut corners, &mut ids, &mut rejected).expect("TODO: panic message");
    Ok((corners, ids))
}

pub(crate) fn debug_draw_aruco(frame: &mut Mat, corners: &VectorOfVectorOfPoint2f, ids: &VectorOfi32) {
    objdetect::draw_detected_markers(frame, corners, ids, Scalar::new(255., 0., 0., 255.)).unwrap();
}

pub(crate) fn debug_put_text(frame: &mut Mat, text: &str, pt: [i32; 2]) {
    imgproc::put_text(frame, text, Point::new(pt[0], pt[1]), imgproc::FONT_ITALIC, 0.5, Scalar::new(0., 255., 0., 255.),
                      2, 0, false).unwrap();
}

pub(crate) fn debug_draw_marker(frame: &mut Mat, pt: [i32; 2], colour: [f64; 3]) {
    imgproc::draw_marker(frame, Point::new(pt[0], pt[1]),
                         Scalar::new(colour[0],
                                     colour[1],
                                     colour[2], 255.), 0, 0, 3, 0).unwrap();
}
