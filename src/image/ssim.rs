//! Structural Similarity Index (SSIM) implementation.

use super::Image;
use crate::error::AppError;
use std::path::PathBuf;

const ASPECT_RATIO_TOLERANCE: f32 = 0.01;

pub fn ssim_score(img1: PathBuf, img2: PathBuf) -> Result<f64, AppError> {
    let image1 = Image::from_path(&img1)?;
    let image2 = Image::from_path(&img2)?;

    if (image1.aspect_ratio() - image2.aspect_ratio()).abs() > ASPECT_RATIO_TOLERANCE {
        println!(
            "img1: {} img2: {}",
            image1.aspect_ratio(),
            image2.aspect_ratio(),
        );
        return Err(AppError::DifferentAspectRatio);
    }

    // Compute mean intensity of the greyscale images
    let gray1 = image1.image.to_luma8();
    let gray2 = image2.image.to_luma8();

    let mu1 = mean_intensity(&gray1);
    let mu2 = mean_intensity(&gray2);

    // Compute the variance and covariance of the greyscale images
    let (var1, var2, cov) = variance_covariance(&gray1, &gray2, mu1, mu2);

    let ssim_score = ssim(mu1, mu2, var1, var2, cov);

    Ok(ssim_score)
}

fn mean_intensity(img: &image::GrayImage) -> f64 {
    let sum: u32 = img.pixels().map(|&p| p.0[0] as u32).sum();
    sum as f64 / (img.width() * img.height()) as f64
}

fn variance_covariance(
    img1: &image::GrayImage,
    img2: &image::GrayImage,
    mu1: f64,
    mu2: f64,
) -> (f64, f64, f64) {
    let mut var1 = 0.0;
    let mut var2 = 0.0;
    let mut cov = 0.0;

    for (p1, p2) in img1.pixels().zip(img2.pixels()) {
        let x = p1.0[0] as f64 - mu1;
        let y = p2.0[0] as f64 - mu2;
        var1 += x * x;
        var2 += y * y;
        cov += x * y;
    }

    let n = img1.width() * img1.height();
    (var1 / n as f64, var2 / n as f64, cov / n as f64)
}

fn ssim(mu1: f64, mu2: f64, var1: f64, var2: f64, cov: f64) -> f64 {
    let c1: f64 = (0.01f64 * 255.0f64).powi(2);
    let c2: f64 = (0.03f64 * 255.0f64).powi(2);

    let numerator = (2.0 * mu1 * mu2 + c1) * (2.0 * cov + c2);
    let denominator = (mu1.powi(2) + mu2.powi(2) + c1) * (var1 + var2 + c2);

    numerator / denominator
}

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    // #[ignore = "slow"]
    fn test_ssim_similar() {
        // These images have the same dimensions and resolution
        let img1 = get_test_img_path("2009-02-25-LX3-10.jpg").unwrap();
        let img2 = get_test_img_path("090225-161708.jpg").unwrap();

        assert!((ssim_score(img1, img2).unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    // #[ignore = "slow"]
    fn test_ssim_similar_2() {
        // The second image is rotated
        let img1 = get_test_img_path("2009-02-25-LX3-10.jpg").unwrap();
        let img2 = get_test_img_path("2009-02-25-LX3-11.jpg").unwrap();

        assert!((ssim_score(img1, img2).unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    // #[ignore = "slow"]
    fn test_ssim_dissimilar() {
        // These images are dissimilar
        let img1 = get_test_img_path("2009-02-25-LX3-10.jpg").unwrap();
        let img2 = get_test_img_path("2009-02-25-LX3-13.jpg").unwrap();

        assert!((ssim_score(img1, img2).unwrap() - 1.0).abs() > 0.4);
    }

    #[test]
    // #[ignore = "slow"]
    fn test_ssim_different_aspect_ratio_error() {
        // These images have different aspect ratios
        let img1 = get_test_img_path("2009-02-25-LX3-10.jpg").unwrap();
        let img2 = get_test_img_path("img-different.jpeg").unwrap();

        let result = ssim_score(img1, img2);

        assert!(result.is_err());
    }

    fn get_test_img_path(path: &str) -> Result<PathBuf, AppError> {
        let project_root = std::path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let image_path = project_root.join(format!("test-data/case-2/{}", path));

        Ok(image_path)
    }
}
