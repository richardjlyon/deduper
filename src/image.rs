//! Represents an image. Computes the hash of an image for detecting duplicates, and
//! checks for sidecar files.

use image::DynamicImage;
use std::io::{self};
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

use crate::error::AppError;

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
    pub image: DynamicImage,
}

impl Image {
    /// Creates a new Image from a file path.
    pub fn from_path(path: PathBuf) -> Result<Image, AppError> {
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

        let image = image::open(&path)?;

        Ok(Image { path, image })
    }

    /// Returns true if the image has a sidecar file.
    pub fn has_sidecar(&self) -> bool {
        let sidecar = self.path.with_extension("xmp");
        sidecar.exists()
    }

    /// Returns the hash of the image.
    /// To reduce compute time, the hash is computed from a chunk of the image, which is
    ///  defined by the `chunksize` parameter.
    pub fn hash(&self, chunksize: f32) -> io::Result<u64> {
        if let Some(extension) = self.path.extension() {
            match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                "heic" => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "HEIC images are not supported",
                    ))
                }
                _ => (),
            }
        }

        if chunksize <= 0.0 || chunksize > 1.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "chunksize must be between 0.0 and 1.0",
            ));
        }

        // Convert the image to RGB8 format
        let rgb_image = self.image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // Define the chunk size (e.g., 10% of the image)
        let chunk_width = ((width as f32) * chunksize) as u32;
        let chunk_height = ((height as f32) * chunksize) as u32;

        // Collect the pixel data from the chunk
        let mut chunk_data = Vec::with_capacity((chunk_width * chunk_height * 3) as usize);
        for y in 0..chunk_height {
            for x in 0..chunk_width {
                let pixel = rgb_image.get_pixel(x, y);
                chunk_data.extend_from_slice(&pixel.0);
            }
        }

        // Create a hash of the chunk data using xxHash
        let hash = xxh3_64(&chunk_data);

        Ok(hash)
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
    use std::time::Instant;

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
        let img = Image::from_path(img_path).unwrap();

        assert_eq!(img.image.dimensions(), (4032, 3024));
    }

    #[test]
    // #[ignore = "slow"]
    fn test_hash() {
        let img = get_img("img.jpg").unwrap();
        let img_duplicate = get_img("img-duplicate.jpg").unwrap();
        let img_different = get_img("img-different.jpeg").unwrap();

        let start = Instant::now();
        assert!(img == img_duplicate);
        assert!(img != img_different);

        let duration = start.elapsed();
        println!("Time taken to hash image: {:?}", duration);
    }

    fn get_img(img_name: &str) -> Result<Image, AppError> {
        let test_dir = PathBuf::from("test-data/case-1/");
        let img_path = test_dir.join(img_name);

        Image::from_path(img_path)
    }
}
