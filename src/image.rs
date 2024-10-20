//! Represents an image.

use image::{DynamicImage, GenericImageView};
use std::io::{self, Cursor, Read};
use std::{fs::File, path::PathBuf};

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
}

impl Image {
    pub fn hash_image(self) -> io::Result<String> {
        /// Hashes the image file using SHA-256.
        // Open the image file
        let file = File::open(self.path)?;
        let mut reader = io::BufReader::new(file);

        // Decode the image from the file, ignore EXIF metadata
        let image = image::load(reader, image::ImageFormat::Jpeg).unwrap();

        // Convert the image into raw pixel data
        let raw_pixels = image.to_rgb8();

        // Access the underlying raw buffer (Vec<u8>) of the ImageBuffer
        let pixel_data = raw_pixels.as_raw();

        // Create a hash of the pixel data using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&pixel_data);
        let result = hasher.finalize();

        // Return the hash as a hexadecimal string
        Ok(format!("{:x}", result))
    }
}

// tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image() {
        let img1 = get_img1();

        assert_eq!(
            img1.path,
            PathBuf::from("/Users/richardlyon/dev/deduper/test-data/case-1/img.jpg")
        );
    }

    #[test]
    fn test_hash_image() {
        let img1 = get_img1();
        let img2 = get_img2();

        let hash1 = img1.hash_image().unwrap();
        let hash2 = img2.hash_image().unwrap();

        assert_eq!(hash1, hash2);
    }

    fn get_img1() -> Image {
        let test_dir = get_test_dir();
        let img1_name = test_dir.join("img.jpg");

        Image { path: img1_name }
    }

    fn get_img2() -> Image {
        let test_dir = get_test_dir();
        let img2_name = test_dir.join("img-duplicate.JPG");

        Image { path: img2_name }
    }

    fn get_test_dir() -> PathBuf {
        let test_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data");

        test_root.join("case-1")
    }
}
