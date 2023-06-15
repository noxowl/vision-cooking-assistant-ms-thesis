use opencv::prelude::*;
// use opencv::{objdetect, imgproc, imgcodecs};
use opencv::core::{CV_8UC1, hconcat2, Mat, Rect, Scalar, vconcat};
// use opencv::types::{VectorOfi32, VectorOfMat};
use anyhow::Result;

// pub fn generate_aruco(marker_amount: u32) -> Result<Vec<u32>> {
//     let mut index_generated = vec![];
//     let pixel = 200;
//     let mut img: Mat = Default::default();
//     let mut imwrite_params = VectorOfi32::new();
//     imwrite_params.push(imgcodecs::IMWRITE_PNG_COMPRESSION);
//     imwrite_params.push(1);
//     let mut concat_temp_1: VectorOfMat = Default::default();
//     let mut concat_temp_2: VectorOfMat = Default::default();
//     let mut v_concat_1: Mat = Default::default();
//     let mut v_concat_2: Mat = Default::default();
//
//     let dictionary = objdetect::get_predefined_dictionary(objdetect::PredefinedDictionaryType::DICT_4X4_50)?;
//     for i in 0..marker_amount {
//         let mut ar_marker: Mat = Default::default();
//         imgproc::draw_marker(&mut img, Point::new(pixel / 2, pixel / 2),
//                              Scalar::new(0., 0., 0., 255.), dictionary .get(i as i32).unwrap(), pixel, 1, 1).unwrap();
//         let mut padding_img = Mat::new_rows_cols_with_default(
//             ar_marker.rows() + 100, ar_marker.cols() + 100,
//             CV_8UC1, Scalar::new(255., 255., 255., 255.,))?;
//         let mut padding_roi = Mat::roi(&mut padding_img, Rect::new(50, 50, ar_marker.cols(), ar_marker.rows()))?;
//         ar_marker.copy_to(&mut padding_roi).unwrap();
//         if i % 2 == 0 {
//             concat_temp_1.push(padding_img.clone());
//         } else {
//             concat_temp_2.push(padding_img.clone());
//         }
//         imgcodecs::imwrite(format!("ar_{i}.png").as_str(), &ar_marker, &imwrite_params)?;
//         index_generated.push(i);
//     }
//     match concat_temp_1.len() > concat_temp_2.len() {
//         true => {
//             concat_temp_2.push(generate_padding_img(&concat_temp_1.get(0)?)?.clone());
//         },
//         false => {
//             concat_temp_1.push(generate_padding_img(&concat_temp_1.get(0)?)?.clone());
//         },
//     }
//     vconcat(&concat_temp_1, &mut v_concat_1)?;
//     vconcat(&concat_temp_2, &mut v_concat_2)?;
//     hconcat2(&v_concat_1, &v_concat_2, &mut img).unwrap();
//     imgcodecs::imwrite("ar_all.png", &img, &imwrite_params).unwrap();
//     Ok(index_generated)
// }

fn generate_padding_img(mat: &Mat) -> Result<Mat> {
    Ok(Mat::new_rows_cols_with_default(
        mat.rows(), mat.cols(),
        CV_8UC1, Scalar::new(255., 255., 255., 255.,))?)
}
