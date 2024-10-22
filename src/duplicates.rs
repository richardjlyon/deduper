//! Duplicate image detection.

use image::DynamicImage;
use log::debug;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use twox_hash::XxHash64;

use crate::image::Image;
use crate::similarity::ssim_index2;

pub fn create_similarity_index(image_paths: Vec<PathBuf>) -> HashMap<String, Vec<String>> {
    // Key is the hash of the image path and the value is a vector of similar image paths
    let mut similarity_index: HashMap<String, Vec<String>> = HashMap::new();

    // Similarity scores of image pairs that have already been compared
    let mut comparison_cache: HashMap<(String, String), f32> = HashMap::new();

    // Avoid repeated disk I/O for the same image
    let mut image_cache: HashMap<String, DynamicImage> = HashMap::new();

    debug!("Processing {} images", image_paths.len());

    for (i, path1) in image_paths.iter().enumerate() {
        debug!("Processing image: {}", path1.display());

        // Skip if the image has already been added to the similarity index
        if contains_value(&similarity_index, &path1.to_string_lossy()) {
            debug!("Skipping image: {}", path1.display());
            continue;
        }

        // Add the image path to the cache
        let hash1 = calculate_hash(path1);
        similarity_index
            .entry(hash1.clone())
            .or_default()
            .push(path1.to_string_lossy().into_owned());

        // Compare similarity with all other images
        for path2 in image_paths.iter().skip(i + 1) {
            debug!("Comparing image: {}", path2.display());

            let similarity =
                get_or_calculate_similarity(&mut comparison_cache, &mut image_cache, path1, path2);

            if similarity > 0.95 {
                similarity_index
                    .entry(hash1.clone())
                    .or_default()
                    .push(path2.to_string_lossy().into_owned());

                debug!(
                    "Found similar: similarity_index now {:#?}",
                    similarity_index
                );
            }
        }
    }

    // Discard entries with no duplicates
    similarity_index.retain(|_, vec| vec.len() > 1);

    similarity_index
}

fn contains_value(map: &HashMap<String, Vec<String>>, value: &str) -> bool {
    for vec in map.values() {
        if vec.contains(&value.to_string()) {
            return true;
        }
    }
    false
}

fn calculate_hash(path: &PathBuf) -> String {
    let mut hasher = XxHash64::default();
    path.hash(&mut hasher);
    hasher.finish().to_string()
}

fn get_or_calculate_similarity(
    cache: &mut HashMap<(String, String), f32>,
    image_cache: &mut HashMap<String, DynamicImage>,
    path1: &PathBuf,
    path2: &PathBuf,
) -> f32 {
    let key = if path1 < path2 {
        (
            path1.to_string_lossy().into_owned(),
            path2.to_string_lossy().into_owned(),
        )
    } else {
        (
            path2.to_string_lossy().into_owned(),
            path1.to_string_lossy().into_owned(),
        )
    };

    *cache.entry(key).or_insert_with(|| {
        let img1 = get_or_load_image(image_cache, path1);
        let img2 = get_or_load_image(image_cache, path2);

        ssim_index2(&img1, &img2).unwrap()
    })
}

fn get_or_load_image(cache: &mut HashMap<String, DynamicImage>, path: &PathBuf) -> DynamicImage {
    let img = Image::from_path(path).unwrap();
    cache
        .entry(path.to_string_lossy().into_owned())
        .or_insert_with(|| img.image().expect("Failed to open image"))
        .clone()
}

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;
    use crate::indexer::index_images_in_folder;
    use std::{path::PathBuf, time::Instant};

    #[test]
    #[ignore = "slow"]
    fn test_duplicates() {
        let image_paths = get_test_images();

        let start = Instant::now();

        let similarity_index = create_similarity_index(image_paths);

        let duration = start.elapsed();
        println!("->> Time elapsed is: {:?}", duration);

        println!("{:#?}", similarity_index);
    }

    fn get_test_images() -> Vec<PathBuf> {
        let test_dir = PathBuf::from("test-data");
        index_images_in_folder(test_dir)
    }
}
