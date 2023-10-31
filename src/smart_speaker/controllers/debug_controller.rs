use opencv::{prelude::*, imgproc, objdetect, core::Scalar, core::Point, types::VectorOfVectorOfPoint2f, types::VectorOfi32};
use opencv::core::{Point2f, RotatedRect};
use opencv::types::VectorOfPoint2f;

pub(crate) fn write_text_to_mat(mat: &mut Mat, text: &str, x: i32, y: i32) {
    imgproc::put_text(mat, text, Point::new(x, y), 1, 0.8, Scalar::new(0., 255., 0., 255.), 1, 0, false).unwrap();
}

pub(crate) fn draw_circle_to_mat(mat: &mut Mat, x: i32, y: i32) {
    imgproc::circle(mat, Point::new(x, y), 10, Scalar::new(0., 255., 0., 255.), 1, 0, 0).unwrap();
}

pub(crate) fn draw_aruco(frame: &mut Mat, corners: &VectorOfVectorOfPoint2f, ids: &VectorOfi32) {
    objdetect::draw_detected_markers(frame, corners, ids, Scalar::new(255., 0., 0., 255.)).unwrap();
}

pub(crate) fn draw_rotated_rect_to_mat(mat: &mut Mat, rect: &RotatedRect) {
    let mut points = [Point2f::default(); 4];
    rect.points(&mut points).unwrap();
    for i in 0..4 {
        imgproc::line(
            mat,
            Point::new (points[i].x.round() as i32, points[i].y.round() as i32),
            Point::new (points[(i + 1) % 4].x.round() as i32, points[(i + 1) % 4].y.round() as i32),
            Scalar::new(0., 255., 0., 255.),
            1,
            0,
            0).unwrap();
    }
}

pub(crate) fn draw_approx_poly_to_mat(mat: &mut Mat, approx: &VectorOfPoint2f) {
    for i in 0..approx.len() {
        imgproc::line(
            mat,
            Point::new (approx.get(i).unwrap().x.round() as i32, approx.get(i).unwrap().y.round() as i32),
            Point::new (approx.get((i + 1) % approx.len()).unwrap().x.round() as i32, approx.get((i + 1) % approx.len()).unwrap().y.round() as i32),
            Scalar::new(0., 255., 0., 255.),
            1,
            0,
            0).unwrap();
    }
}
