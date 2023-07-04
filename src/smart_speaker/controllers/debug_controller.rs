use opencv::{prelude::*, imgproc, core::Scalar, core::Point};

pub(crate) fn write_text_to_mat(mat: &mut Mat, text: &str, x: i32, y: i32) {
    imgproc::put_text(mat, text, Point::new(x, y), 1, 0.8, Scalar::new(0., 255., 0., 255.), 1, 0, false).unwrap();
}

pub(crate) fn draw_circle_to_mat(mat: &mut Mat, x: i32, y: i32) {
    imgproc::circle(mat, Point::new(x, y), 10, Scalar::new(0., 255., 0., 255.), 1, 0, 0).unwrap();
}
