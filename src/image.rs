//! Represents an image.

use std::io::{self};
use std::{fs::File, path::PathBuf};
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug)]
pub struct Image {
    pub path: PathBuf,
}

impl Image {
    /// Hash the image file.
    ///
    /// We hash the decoded image to avoid any exif
    /// data altering the hash.
    pub fn hash_image(&self) -> io::Result<u64> {
        // Open the image file
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);

        // Decode the image from the file, ignore EXIF metadata
        let image = image::ImageReader::new(reader)
            .with_guessed_format()?
            .decode()
            .unwrap();

        // Convert the image into raw pixel data
        let raw_pixels = image.to_rgb8();
        let pixel_data = raw_pixels.as_raw();

        let hash = xxh3_64(&pixel_data);

        Ok(hash)
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

    use super::*;

    #[test]
    fn test_hash_image() {
        let img = get_img("img.jpg");
        let img_duplicate = get_img("img-duplicate.jpg");
        let img_different = get_img("img-different.jpg");

        assert!(img == img_duplicate);
        assert!(img != img_different);
    }

    fn get_img(img_name: &str) -> Image {
        let test_dir = get_test_dir();
        let img = test_dir.join(img_name);

        Image { path: img }
    }

    fn get_test_dir() -> PathBuf {
        let test_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data");

        test_root.join("case-1")
    }
}
