//! Represents an image.

use image::DynamicImage;
use std::io::{self};
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
    pub image: DynamicImage,
}

impl Image {
    pub fn from_path(path: PathBuf) -> Image {
        let image = image::open(&path).unwrap();

        Image { path, image }
    }

    pub fn hash_image(&self) -> io::Result<u64> {
        // Convert the image to RGB8 format
        let rgb_image = self.image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // Define the chunk size (e.g., 10% of the image)
        let chunk_width = width / 10;
        let chunk_height = height / 10;

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

    /// Returns true if the image has a sidecar file.
    pub fn has_sidecar(&self) -> bool {
        let sidecar = self.path.with_extension("xmp");
        sidecar.exists()
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        match (self.hash_image(), other.hash_image()) {
            (Ok(self_hash), Ok(other_hash)) => self_hash == other_hash,
            _ => false,
        }
    }
}

// tests

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use image::GenericImageView;

    use super::*;

    #[test]
    fn test_has_sidecar() {
        let img_with = get_img("img.jpg");
        let img_without = get_img("img-duplicate.jpg");

        assert!(img_with.has_sidecar());
        assert!(!img_without.has_sidecar());
    }

    #[test]
    fn test_from() {
        let test_dir = get_test_dir();
        let img_path = test_dir.join("img.jpg");
        let img = Image::from_path(img_path);

        assert_eq!(img.image.dimensions(), (4032, 3024));
    }

    #[test]
    // #[ignore = "slow"]
    fn test_hash_image() {
        let img = get_img("img.jpg");
        let img_duplicate = get_img("img-duplicate.jpg");
        let img_different = get_img("img-different.jpeg");

        let start = Instant::now();
        assert!(img == img_duplicate);
        assert!(img != img_different);

        let duration = start.elapsed();
        println!("Time taken to hash image: {:?}", duration);
    }

    fn get_img(img_name: &str) -> Image {
        let test_dir = get_test_dir();
        let img_path = test_dir.join(img_name);

        Image::from_path(img_path)
    }

    fn get_test_dir() -> PathBuf {
        let test_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data");

        test_root.join("case-1")
    }
}
