//! Represents an image. Computes the hash of an image for detecting duplicates, and
//! checks for sidecar files.

use image::{DynamicImage, GenericImageView};
use rexif::{ExifTag, TagValue};
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
    pub image: DynamicImage,
}

impl Image {
    /// Valid images
    pub fn is_image_file(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Creates a new Image from a file path.
    pub fn from_path(path: &PathBuf) -> Result<Image, AppError> {
        // return an error if the extension is '.heic'
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                "heic" => {
                    return Err(AppError::UnsupportedType(
                        extension.to_string_lossy().into_owned(),
                    ));
                }
                _ => (),
            }
        }

        let mut image = image::open(&path)?;

        // Ensure the image is not rotated by checking the EXIf orientation tag
        let exif = rexif::parse_file(&path).expect("Failed to parse EXIF data");
        if let Some(orientation) = exif
            .entries
            .iter()
            .find(|entry| entry.tag == ExifTag::Orientation)
        {
            match &orientation.value {
                TagValue::U16(val) if !val.is_empty() => match val[0] {
                    3 => image = image.rotate180(), // rotated 180 degrees
                    6 => image = image.rotate90(),  // rotated 90 degrees counterclockwise
                    8 => image = image.rotate270(), // rotated 90 degrees clockwise
                    _ => (),
                },
                _ => println!("The orientation tag is not in the expected format."),
            }
        } else {
            println!("No orientation information found in EXIF data.");
        }

        Ok(Image {
            path: path.to_owned(),
            image,
        })
    }

    /// Returns true if the image has a sidecar file.
    pub fn has_sidecar(&self) -> bool {
        let sidecar = self.path.with_extension("xmp");
        sidecar.exists()
    }

    /// Returns the aspect ratio of the image.
    pub fn aspect_ratio(&self) -> f32 {
        let (width, height) = self.image.dimensions();
        width as f32 / height as f32
    }
}

impl PartialEq for Image {
    /// Returns true if the image hahes are equal.
    fn eq(&self, other: &Self) -> bool {
        const CHUNKSIZE: f32 = 0.1;
        match (self.hash(CHUNKSIZE), other.hash(CHUNKSIZE)) {
            (Ok(self_hash), Ok(other_hash)) => self_hash == other_hash,
            _ => false,
        }
    }
}

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;
    use image::GenericImageView;

    #[test]
    fn test_aspect_ratio() {
        let img = get_img("img.jpg").unwrap();
        assert!((img.aspect_ratio() - 1.333).abs() < 0.01);
    }

    #[test]
    fn test_is_image_file() {
        let image_extensions = vec!["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"];
        let non_image_extensions = vec!["txt", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"];

        for ext in image_extensions {
            let file_name = format!("file.{}", ext);
            let path = Path::new(&file_name);
            assert!(Image::is_image_file(path));
        }

        for ext in non_image_extensions {
            let file_name = format!("file.{}", ext);
            let path = Path::new(&file_name);
            assert!(!Image::is_image_file(path));
        }
    }

    #[test]
    #[ignore = "slow"]
    fn test_has_sidecar() {
        let img_with = get_img("img.jpg").unwrap();
        let img_without = get_img("img-duplicate.jpg").unwrap();

        assert!(img_with.has_sidecar());
        assert!(!img_without.has_sidecar());
    }

    #[test]
    #[ignore = "slow"]
    fn test_constructor() {
        let img_path = PathBuf::from("test-data/case-1/img.jpg");
        let img = Image::from_path(&img_path).unwrap();

        assert_eq!(img.image.dimensions(), (4032, 3024));
    }

    fn get_img(img_name: &str) -> Result<Image, AppError> {
        let test_dir = PathBuf::from("test-data/case-1/");
        let img_path = test_dir.join(img_name);

        Image::from_path(&img_path)
    }
}
