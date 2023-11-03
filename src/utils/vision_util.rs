use std::str::FromStr;
use anyhow::{anyhow, Result};
use opencv::core::{Point2f, in_range, Size, Point, bitwise_and, BORDER_DEFAULT, Vector, RotatedRect, Point_};
use opencv::imgproc;
use opencv::prelude::*;
use opencv::types::{VectorOfPoint, VectorOfPoint2f, VectorOfVectorOfPoint, VectorOfVectorOfPoint2f};
use crate::smart_speaker::models::vision_model::{CameraCaptureSource, Capture, DetectableObject, PupilCaptureSource};
use crate::utils::camera_util::Camera;
use crate::utils::pupil_util::{Pupil, PupilRemote};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VisionType {
    None,
    Pupil,
    BuiltInCamera,
}

impl FromStr for VisionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(VisionType::None),
            "false" => Ok(VisionType::None),
            "pupil" => Ok(VisionType::Pupil),
            "built-in-camera" => Ok(VisionType::BuiltInCamera),
            "camera" => Ok(VisionType::BuiltInCamera),
            "cam" => Ok(VisionType::BuiltInCamera),
            _ => Err(anyhow!("invalid vision type")),
        }
    }
}

pub(crate) enum MarkerPerimeter {
    Small, // edge length: 50mm
    Medium, // edge length: 100mm
    Large, // edge length: 150mm
    A4_12, // edge length: 38mm
}

impl MarkerPerimeter {
    pub(crate) fn get_perimeter(&self) -> f32 {
        match self {
            MarkerPerimeter::Small => 20.,
            MarkerPerimeter::Medium => 40.,
            MarkerPerimeter::Large => 60.,
            MarkerPerimeter::A4_12 => 15.2,
        }
    }
}

pub(crate) fn set_pupil_capture(capture: &mut Capture, zmq_endpoint: String) -> Result<()> {
    capture.source = Some(Box::new(
        PupilCaptureSource::new(
            Pupil::new(PupilRemote::new(zmq_endpoint, "frame")))));
    Ok(())
}

pub(crate) fn set_camera_capture(capture: &mut Capture) -> Result<()> {
    capture.source = Some(Box::new(
        CameraCaptureSource::new(
            Camera::new()?)));
    Ok(())
}


pub(crate) fn centroid_of_frame(x: u32, y: u32) -> (f32, f32) {
    if x > 0 && x > 0 {
        ((x / 2) as f32, (y / 2) as f32)
    } else {
        (0., 0.)
    }
}

pub(crate) fn midpoint(px_x: &f32, px_y: &f32, py_x: &f32, py_y: &f32) -> (f32, f32) {
    ((px_x + py_x) / 2., (px_y + py_y) / 2.)
}

pub(crate) fn distance(px_x: &f32, px_y: &f32, py_x: &f32, py_y: &f32) -> f32 {
    ((py_x - px_x).abs().powi(2) + (py_y - px_y).abs().powi(2)).sqrt()
}

// pub(crate) fn get_size_of_rotated_rect_from_midpoint

pub(crate) fn aruco_perimeter_ratio(size: &[Point_<f32>; 4]) -> f32 {
    let aruco_perimeter = distance(&size[0].x, &size[0].y, &size[1].x, &size[1].y)
        + distance(&size[1].x, &size[1].y, &size[2].x, &size[2].y)
        + distance(&size[2].x, &size[2].y, &size[3].x, &size[3].y)
        + distance(&size[3].x, &size[3].y, &size[0].x, &size[0].y);
    return aruco_perimeter / MarkerPerimeter::A4_12.get_perimeter();
}

pub(crate) fn aruco_side_ratio(side: f32) -> f32 {
    let criteria = (MarkerPerimeter::A4_12.get_perimeter() / 4.) as f32;
    side / criteria
}

/// get actual size of object by aruco marker
pub(crate) fn pixel_to_metric(target: f32, ratio: &f32) -> f32 {
    target / ratio
}

pub(crate) fn approx_to_arch_length_metric(contour: &VectorOfPoint2f, ratio: &f32) -> f32 {
    imgproc::contour_area(&contour, false).unwrap() as f32 / ratio
}

pub(crate) fn get_min_rect(contour: &VectorOfPoint) -> RotatedRect {
    imgproc::min_area_rect(&contour).expect("TODO: panic message")
}

pub(crate) fn get_min_rect2f(contour: &VectorOfPoint2f) -> RotatedRect {
    imgproc::min_area_rect(&contour).expect("TODO: panic message")
}

pub(crate) fn get_approx_poly_dp(contour: &VectorOfPoint) -> VectorOfPoint2f {
    let mut approx = VectorOfPoint2f::new();
    imgproc::approx_poly_dp(
        &contour,
        &mut approx,
        0.02, // * imgproc::arc_length(&contour, true).unwrap(),
        true,
    ).unwrap();
    approx
}

pub(crate) fn mask_object(frame: &Mat, target: DetectableObject) -> Result<Mat> {
    let mut hsv = Mat::default();
    let mut mask = Mat::default();
    let mut dst = Mat::default();
    imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0).unwrap();
    match target {
        DetectableObject::Carrot => {
            in_range(
                &hsv,
                &Vector::from_slice(&[10., 100., 20.]),
                &Vector::from_slice(&[25., 255., 255.]),
                &mut mask,
            )?;
        },
        DetectableObject::HumanSkin => {
            in_range(
                &hsv,
                &Vector::from_slice(&[0., 48., 80.]),
                &Vector::from_slice(&[20., 255., 255.]),
                &mut mask,
            )?;
        }
    }
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        Size::new(3, 3), Default::default()).unwrap();
    imgproc::erode(
        &mask,
        &mut dst,
        &kernel,
        Point::new(-1, -1),
        2, 0, Default::default()).unwrap();
    imgproc::dilate(
        &dst,
        &mut mask,
        &kernel,
        Point::new(-1, -1),
        2, 0, Default::default()).unwrap();

    imgproc::gaussian_blur(&mask, &mut dst, Size::new(3, 3), 0.,0., BORDER_DEFAULT).unwrap();
    bitwise_and(frame, frame, &mut mask, &dst).unwrap();
    imgproc::cvt_color(&mask, &mut dst, imgproc::COLOR_HSV2BGR, 0).unwrap();
    imgproc::threshold(&dst, &mut mask, 60., 255., imgproc::THRESH_BINARY).unwrap();
    imgproc::cvt_color(&mask, &mut dst, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    Ok(dst)
}

pub(crate) fn get_object_contours(frame: &Mat) -> Result<VectorOfVectorOfPoint> {
    let mut contours = VectorOfVectorOfPoint::new();
    imgproc::find_contours(
        &frame,
        &mut contours,
        imgproc::RETR_TREE,
        imgproc::CHAIN_APPROX_SIMPLE,
        Default::default(),
    ).unwrap();
    let mut selected_contours = VectorOfVectorOfPoint::new();
    for i in 0..contours.len() {
        let area = imgproc::contour_area(&contours.get(i).unwrap(), false).unwrap();
        if area > 1000. {
            selected_contours.push(contours.get(i).unwrap());
        }
    }
    Ok(selected_contours)
}

pub(crate) fn get_measure_criteria_from_aruco(aruco_corners: &VectorOfVectorOfPoint2f) -> Result<(Vec<f32>, Vec<f32>)> {
    let mut width_ratios: Vec<f32> = Vec::new();
    let mut height_ratios: Vec<f32> = Vec::new();
    for i in 0..aruco_corners.len() {
        let square = get_min_rect2f(&aruco_corners.get(i).unwrap());
        let mut points = [Point2f::default(); 4];
        square.points(&mut points).unwrap();
        let width = distance(&points[1].x, &points[1].y, &points[2].x, &points[2].y);
        let height = distance(&points[0].x, &points[0].y, &points[1].x, &points[1].y);
        let width_ratio = aruco_side_ratio(width.clone());
        let height_ratio = aruco_side_ratio(height.clone());
        width_ratios.push(width_ratio);
        height_ratios.push(height_ratio);
    }
    Ok((width_ratios, height_ratios))
}



