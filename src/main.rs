mod error;
mod image;
mod indexer;

use std::path::PathBuf;

use image::ssim_score;
use indexer::index_images_in_folder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

fn main() {
    let images = load_images("/Users/richardlyon/dev/deduper/test-data");
    let threshold = 0.95;
    let similar_groups = find_duplicate_groups(&images, threshold);

    println!("{:#?}", similar_groups);
}

fn load_images(root_dir: &str) -> Vec<PathBuf> {
    let test_dir = PathBuf::from(root_dir);

    index_images_in_folder(test_dir)
}

fn find_duplicate_groups(image_paths: &[PathBuf], ssim_threshold: f64) -> Vec<Vec<PathBuf>> {
    let n = image_paths.len();
    let max_iterations = (n * (n - 1)) / 2; // Calculate the maximum number of comparisons
    let pb = create_progress_bar(max_iterations);

    println!("Finding duplicates...");

    let duplicate_groups: Vec<Vec<PathBuf>> = image_paths
        .par_iter()
        .enumerate()
        .map(|(i, path_i)| {
            let mut current_duplicates: Vec<PathBuf> = vec![path_i.clone()];

            for j in (i + 1)..image_paths.len() {
                pb.inc(1); // Increment the progress bar for each comparison
                if let Ok(ssim_value) = ssim_score(path_i, &image_paths[j]) {
                    if ssim_value >= ssim_threshold {
                        current_duplicates.push(image_paths[j].clone());
                    }
                } else {
                    // Skip this iteration if ssim_score returns an error
                    continue;
                }
            }

            if current_duplicates.len() > 1 {
                Some(current_duplicates)
            } else {
                None
            }
        })
        .filter_map(|group| group)
        .collect();

    pb.finish_with_message("Duplicate search complete");

    duplicate_groups
}

fn create_progress_bar(max_iterations: usize) -> ProgressBar {
    let pb = ProgressBar::new(max_iterations as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    pb
}
