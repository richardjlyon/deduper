//! Duplicate image detection.
//!
//! Employs Rayon to parallelize the comparison of images.
//! NOTE: This is a hack. The parallelising breaks the algorithm for detecting if duplicates
//! are already in the index. This is because the index is mutable and shared between threads.
//! So we have to do some extra work to clean up the index afterwards.
//!
//! However, in tests it reduced processing time by about 50%.

use crate::image::Image;
use crate::similarity::ssim_index2;
use image::DynamicImage;
use indicatif::ProgressStyle;
use log::debug;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use twox_hash::XxHash64;

pub fn create_similarity_index_rayon(image_paths: Vec<PathBuf>) -> HashMap<String, Vec<String>> {
    debug!("Processing {} images", image_paths.len());

    let n = image_paths.len();
    let max_iterations = (n * (n - 1)) / 2; // Calculate the maximum number of comparisons
    let pb = indicatif::ProgressBar::new(max_iterations as u64);
    pb.set_style(
        ProgressStyle::with_template("[{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let similarity_index = image_paths
        .par_iter()
        .enumerate()
        .fold(
            || (HashMap::new(), HashMap::new(), HashMap::new()),
            |(mut similarity_index, mut comparison_cache, mut image_cache), (i, path1)| {
                debug!("Processing image: {}", path1.display());
                pb.set_message(path1.parent().unwrap().display().to_string());

                // Skip if the image has already been added to the similarity index
                if contains_value(&similarity_index, &path1.to_string_lossy()) {
                    debug!("Skipping image: {}", path1.display());
                    return (similarity_index, comparison_cache, image_cache);
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

                    let similarity = get_or_calculate_similarity(
                        &mut comparison_cache,
                        &mut image_cache,
                        path1,
                        path2,
                    );

                    if similarity > 0.95 {
                        similarity_index
                            .entry(hash1.clone())
                            .or_default()
                            .push(path2.to_string_lossy().into_owned());
                        debug!("Similarity: {}", similarity);
                    }

                    pb.inc(1);
                }

                (similarity_index, comparison_cache, image_cache)
            },
        )
        .reduce(
            || (HashMap::new(), HashMap::new(), HashMap::new()),
            |(mut acc_index, mut acc_cache, mut acc_image_cache), (index, cache, image_cache)| {
                for (key, value) in index {
                    acc_index.entry(key).or_default().extend(value);
                }
                acc_cache.extend(cache);
                acc_image_cache.extend(image_cache);
                (acc_index, acc_cache, acc_image_cache)
            },
        )
        .0;

    pb.finish_with_message("Duplicate search complete");

    // Discard entries with no duplicates
    let result = remove_subset_entries(similarity_index);

    let mut total_count = 0;
    for values in result.values() {
        total_count += values.len();
    }
    pb.finish_with_message(format!("Found {} duplicates", total_count));

    result
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

fn remove_subset_entries(
    mut similarity_index: HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let keys: Vec<String> = similarity_index.keys().cloned().collect();
    let mut to_remove = HashSet::new();

    for i in 0..keys.len() {
        for j in 0..keys.len() {
            if i != j {
                let vec_i = &similarity_index[&keys[i]];
                let vec_j = &similarity_index[&keys[j]];

                if is_subset(vec_i, vec_j) {
                    to_remove.insert(keys[i].clone());
                }
            }
        }
    }

    for key in to_remove {
        similarity_index.remove(&key);
    }
    similarity_index.retain(|_, vec| vec.len() > 1);

    similarity_index
}

fn is_subset(vec1: &[String], vec2: &[String]) -> bool {
    let set1: HashSet<_> = vec1.iter().collect();
    let set2: HashSet<_> = vec2.iter().collect();
    set1.is_subset(&set2)
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

        let similarity_index = create_similarity_index_rayon(image_paths);

        let duration = start.elapsed();
        println!("->> Time elapsed is: {:?}", duration);

        println!("{:#?}", similarity_index);
    }

    fn get_test_images() -> Vec<PathBuf> {
        let test_dir = PathBuf::from("test-data");
        index_images_in_folder(&test_dir)
    }
}
