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
        let rgb_image = self.image.to_rgb8();
        let bytes = rgb_image.as_raw();
        let hash = xxh3_64(&bytes);

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

    use image::GenericImageView;

    use super::*;

    #[test]
    #[ignore = "slow"]
    fn test_hash_image() {
        let img = get_img("img.jpg");
        let img_duplicate = get_img("img-duplicate.jpg");
        let img_different = get_img("img-different.jpg");

        assert!(img == img_duplicate);
        assert!(img != img_different);
    }

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
