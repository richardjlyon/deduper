//! Represents an image.
//! Computes the hash of an image for detecting duplicates, and checks for sidecar files.

use std::{fs::File, hash::Hasher, io::BufReader, path::PathBuf};

use exif::{Exif, In, Reader, Tag};
use image::{DynamicImage, ImageReader};
use itertools::Itertools;
use log::debug;
use xxhash_rust::xxh3::Xxh3;

use crate::error::AppError;

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum Orientation {
    Normal,
    MirrorHorizontal,
    Rotated180,
    MirrorVertical,
    MirroredHorizontallyAndRotated270,
    Rotated90,
    MirroredHorizontallyAndRotated90,
    Rotated270,
    Unknown,
}

impl From<u32> for Orientation {
    fn from(value: u32) -> Self {
        match value {
            1 => Orientation::Normal,
            2 => Orientation::MirrorHorizontal,
            3 => Orientation::Rotated180,
            4 => Orientation::MirrorVertical,
            5 => Orientation::MirroredHorizontallyAndRotated270,
            6 => Orientation::Rotated90,
            7 => Orientation::MirroredHorizontallyAndRotated90,
            8 => Orientation::Rotated270,
            _ => Orientation::Unknown,
        }
    }
}

impl Image {
    /// Create an image from a file path.
    pub fn from_path(path: &PathBuf) -> Result<Image, AppError> {
        Ok(Image {
            path: path.to_owned(),
        })
    }

    pub fn valid_extensions() -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"]
    }

    /// Returns true if the image can be processed.
    pub fn is_valid(&self) -> bool {
        if let Some(extension) = self.path.extension() {
            let extension_str = extension.to_str().unwrap_or("").to_lowercase();
            Self::valid_extensions().contains(&extension_str.as_str())
        } else {
            false
        }
    }

    /// Returns true if the image has a sidecar file.
    pub fn has_sidecar(&self) -> bool {
        let sidecar = self.path.with_extension("xmp");
        sidecar.exists()
    }

    /// Returns the hash of the image.
    pub fn metadata(&self) -> Result<Exif, AppError> {
        let file = File::open(&self.path).unwrap();
        let exif = Reader::new().read_from_container(&mut BufReader::new(&file))?;

        Ok(exif)
    }

    /// Get the orientation of the image.
    pub fn orientation(&self) -> Orientation {
        match self.metadata() {
            Ok(exif) => {
                if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
                    if let Some(orientation_value) = field.value.get_uint(0) {
                        let orientation: Orientation = orientation_value.into();
                        orientation
                    } else {
                        Orientation::Unknown
                    }
                } else {
                    Orientation::Unknown
                }
            }
            Err(_) => Orientation::Unknown,
        }
    }

    /// Returns the unrotated image.
    pub fn image(&self) -> Result<DynamicImage, AppError> {
        let mut image = image::open(&self.path)?;

        match self.orientation() {
            Orientation::Rotated180 => {
                debug!("rotated 180 degrees");
                image = image.rotate180();
            }
            Orientation::Rotated90 => {
                debug!("rotated 90 degrees clockwise");
                image = image.rotate90();
            }
            Orientation::Rotated270 => {
                debug!("rotated 90 degrees counterclockwise");
                image = image.rotate270();
            }
            // todo: complete other rotation variants
            _ => {}
        }

        Ok(image)
    }

    /// Returns the resolution of the image.
    pub fn resolution(&self) -> Result<(u32, u32), AppError> {
        let reader = ImageReader::open(&self.path)?;
        let (w, h) = reader.into_dimensions()?;

        match self.orientation() {
            Orientation::Rotated90 | Orientation::Rotated270 => Ok((h, w)),
            _ => Ok((w, h)),
        }
    }

    /// Returns the aspect ratio of the image.
    pub fn aspect_ratio(&self) -> Result<f32, AppError> {
        let (width, height) = self.resolution()?;
        Ok(width as f32 / height as f32)
    }

    /// Return the hash of the image.
    pub fn hash(&self) -> Result<u64, AppError> {
        // Convert the image to RGB8 format
        let rgb_image = self.image()?.to_rgb8();

        let mut hasher = Xxh3::new();
        for chunk in &rgb_image.pixels().flat_map(|rgb| rgb.0).chunks(131072) {
            let data: arrayvec::ArrayVec<u8, 131072> = chunk.collect();
            hasher.update(&data);
        }
        Ok(hasher.finish())
    }
}

impl PartialEq for Image {
    /// Returns true if the image hahes are equal.
    fn eq(&self, other: &Self) -> bool {
        match (self.hash(), other.hash()) {
            (Ok(self_hash), Ok(other_hash)) => self_hash == other_hash,
            _ => false,
        }
    }
}

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {

    use exif::{In, Tag};

    use super::*;

    #[test]
    fn test_from_path() {
        let path = PathBuf::from("test-data/01/house.jpg");
        let img = Image::from_path(&path).unwrap();
        assert_eq!(img.path, path);
    }

    #[test]
    fn test_is_valid() {
        let img = get_img("01/house.jpg").unwrap();
        assert!(img.is_valid());
    }

    #[test]
    fn test_has_sidecar() {
        let img = get_img("01/house.jpg").unwrap();
        assert!(img.has_sidecar());
    }

    #[test]
    fn test_metadata() {
        let img = get_img("02/face-right-2.jpg").unwrap();
        let exif = img.metadata().unwrap();
        if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
            println!("{}: {}", field.tag, field.display_value());
        }
    }

    #[test]
    fn test_orientation() {
        let img_normal = get_img("01/house.jpg").unwrap();
        assert!(matches!(img_normal.orientation(), Orientation::Normal));

        let img_90deg = get_img("02/face-right-2.jpg").unwrap();
        assert!(matches!(img_90deg.orientation(), Orientation::Rotated90));
    }

    #[test]
    #[ignore = "slow"]
    fn test_resolution() {
        let img = get_img("01/house.jpg").unwrap();
        assert_eq!(img.resolution().unwrap(), (4032, 3024));
    }

    #[test]
    #[ignore = "slow"]
    fn test_aspect_ratio() {
        let img = get_img("01/house.jpg").unwrap();
        assert!((img.aspect_ratio().unwrap() - 1.3333).abs() < 0.0001);
    }

    #[test]
    #[ignore = "slow"]
    fn test_hash() {
        let img1 = get_img("01/house.jpg").unwrap();
        let img2 = get_img("01/house-duplicate.jpg").unwrap();
        let img3 = get_img("01/coffee.jpeg").unwrap();

        assert_eq!(img1.hash().unwrap(), img2.hash().unwrap());
        assert_ne!(img1.hash().unwrap(), img3.hash().unwrap())
    }

    #[test]
    #[ignore = "slow"]
    fn test_equal() {
        let img1 = get_img("01/house.jpg").unwrap();
        let img2 = get_img("01/house-duplicate.jpg").unwrap();
        let img3 = get_img("01/coffee.jpeg").unwrap();

        assert_eq!(img1, img2);
        assert_ne!(img1, img3);
    }

    fn get_img(img_name: &str) -> Result<Image, AppError> {
        let test_dir = PathBuf::from("test-data");
        let img_path = test_dir.join(img_name);

        Image::from_path(&img_path)
    }
}
