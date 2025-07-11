use anyhow::{anyhow, Result};
use opencv::{objdetect, imgproc};
use opencv::prelude::*;
use opencv::core::{Vector, Size, Point2f, Point};
use opencv::types::{VectorOfi32, VectorOfVectorOfPoint2f};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionObjectShape, VisionObjectSize};
use crate::utils::vision_util;


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

pub(crate) fn detect_target_objects(frame: &Mat, target: &DetectableObject) -> Result<Vector<Vector<Point2f>>> {
    let mut detected_objects = Vector::new();
    let mut object_mask = Mat::default();
    match target {
        DetectableObject::Carrot => {
            object_mask = vision_util::mask_object(&frame, DetectableObject::Carrot).unwrap();
        }
        DetectableObject::Potato => {
            object_mask = vision_util::mask_object(&frame, DetectableObject::Potato).unwrap();
        }
        DetectableObject::HumanSkin => {
            object_mask = vision_util::mask_object(&frame, DetectableObject::HumanSkin).unwrap();
        }
    }
    let object_contours = vision_util::get_object_contours(&object_mask).unwrap();
    for contour in object_contours {
        detected_objects.push(vision_util::get_approx_poly_dp(&contour, false));
    }
    Ok(detected_objects)
}

pub(crate) fn detect_object_shape(object_contours: &VectorOfVectorOfPoint2f) -> Result<Vec<VisionObjectShape>>{
    let mut shapes = vec![];
    for contour in object_contours {
        let approx = vision_util::get_approx_poly_dp(&contour.iter().map(|c| Point::new(c.x as i32, c.y as i32)).collect(), true);
        if approx.len() == 3 {
            shapes.push(VisionObjectShape::Triangle);
        } else if approx.len() == 4 {
            let rect = vision_util::get_min_rect2f(&contour);
            let mut points = [Point2f::default(); 4];
            rect.points(&mut points).unwrap();
            let width = vision_util::distance(&points[1].x, &points[1].y, &points[2].x, &points[2].y);
            let height = vision_util::distance(&points[0].x, &points[0].y, &points[1].x, &points[1].y);
            let ratio = width / height;
            if ratio >= 0.95 && ratio <= 1.05 {
                shapes.push(VisionObjectShape::Square);
            } else {
                shapes.push(VisionObjectShape::Rectangle);
            }
        } else {
            let rect = vision_util::get_min_rect2f(&contour);
            let mut points = [Point2f::default(); 4];
            rect.points(&mut points).unwrap();
            let width = vision_util::distance(&points[1].x, &points[1].y, &points[2].x, &points[2].y);
            let height = vision_util::distance(&points[0].x, &points[0].y, &points[1].x, &points[1].y);
            let ratio = width / height;
            if ratio >= 0.95 && ratio <= 1.05 {
                shapes.push(VisionObjectShape::Circle);
            } else {
                shapes.push(VisionObjectShape::SemiCircle);
            }
        }
    }
    Ok(shapes)
}

/// measure object size by aruco marker
pub(crate) fn measure_object_size_by_aruco(aruco_corners: &VectorOfVectorOfPoint2f, object_contours: &VectorOfVectorOfPoint2f) -> Result<Vec<VisionObjectSize>> {
    let mut results: Vec<VisionObjectSize> = Vec::new();
    let (width_ratios, height_ratios) = vision_util::get_measure_criteria_from_aruco(&aruco_corners)?;
    let ratios = width_ratios.iter().zip(height_ratios.iter()).map(|(a, b)| a * b).collect::<Vec<f32>>();

    for contour in object_contours {
        let rect = vision_util::get_min_rect2f(&contour);
        let mut points = [Point2f::default(); 4];
        rect.points(&mut points).unwrap();
        let width = vision_util::distance(&points[1].x, &points[1].y, &points[2].x, &points[2].y);
        let height = vision_util::distance(&points[0].x, &points[0].y, &points[1].x, &points[1].y);
        let mut width_candidates: Vec<f32> = Vec::new();
        let mut height_candidates: Vec<f32> = Vec::new();
        let mut perimeter_candidates: Vec<f32> = Vec::new();
        for i in 0..ratios.len() {
            let object_width = vision_util::pixel_to_metric(
                width,
                width_ratios.get(i).unwrap());
            let object_height = vision_util::pixel_to_metric(
                height,
                height_ratios.get(i).unwrap());
            perimeter_candidates.push(vision_util::approx_to_arch_length_metric(&contour, ratios.get(i).unwrap()));
            width_candidates.push(object_width);
            height_candidates.push(object_height);
        }
        let object_width = width_candidates.iter().sum::<f32>() / width_candidates.len() as f32;
        let object_height = height_candidates.iter().sum::<f32>() / height_candidates.len() as f32;
        let object_perimeter = perimeter_candidates.iter().sum::<f32>() / perimeter_candidates.len() as f32;
        results.push(VisionObjectSize::new(
            object_perimeter,
            object_width,
            object_height,
        ));
    }

    Ok(results)
}

pub(crate) fn detect_aruco(frame: &Mat) -> Result<(VectorOfVectorOfPoint2f, Vector<i32>)> {
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

pub(crate) fn find_nearest_object_from_gaze(gaze_as_px: &(f32, f32), objects: &Vector<Vector<Point2f>>) -> Result<(usize, f32)> {
    let mut nearest_distances: Vec<f32> = vec![];
    for i in 0..objects.len() {
        let object = objects.get(i).unwrap();
        let object_mid = vision_util::get_min_rect2f(&object).center;
        let dist = vision_util::distance(
            &gaze_as_px.0, &gaze_as_px.1,
            &object_mid.x, &object_mid.y);
        nearest_distances.push(dist);
    }
    let mut min = 0.0;
    if nearest_distances.len() > 1 {
        min = nearest_distances.clone().into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    } else {
        min = nearest_distances.get(0).unwrap().clone();
    }
    let nearest_index = nearest_distances.iter().position(|&x| x == min).unwrap();
    let nearest_distance = nearest_distances.get(nearest_index).unwrap().clone();
    Ok((nearest_index, nearest_distance))
}
