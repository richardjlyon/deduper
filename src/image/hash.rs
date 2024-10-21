use std::io;

use super::Image;
use xxhash_rust::xxh3::xxh3_64;

impl Image {
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

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::{error::AppError, image::Image};
    use std::env;

    #[test]
    #[ignore = "slow"]
    fn test_hash() {
        let img = get_test_img("img.jpg").unwrap();
        let img_duplicate = get_test_img("img-duplicate.jpg").unwrap();
        let img_different = get_test_img("img-different.jpeg").unwrap();

        let start = Instant::now();
        assert!(img == img_duplicate);
        assert!(img != img_different);

        let duration = start.elapsed();
        println!("Time taken to hash image: {:?}", duration);
    }

    fn get_test_img(path: &str) -> Result<Image, AppError> {
        let project_root = std::path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let image_path = project_root.join(format!("test-data/case-1/{}", path));
        println!("Image path: {:?}", image_path);

        super::Image::from_path(&image_path)
    }
}
