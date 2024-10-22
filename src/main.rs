mod duplicates;
mod error;
mod image;
mod indexer;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use error::AppError;
use image::ssim_score;
use indexer::index_images_in_folder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::to_writer_pretty;

#[derive(Serialize)]
struct PathBufWrapper {
    paths: Vec<Vec<PathBuf>>,
}

fn main() -> Result<(), AppError> {
    let library_root = Path::new("/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures");
    let directories = directories_in_dir(library_root);

    for dir in directories {
        let folder_name = dir.file_name().unwrap().to_str().unwrap();
        let data_file = Path::new("results").join(format!("{}.json", folder_name));

        if data_file.exists() {
            continue;
        }

        // let images = load_images(dir.to_str().unwrap());
        // let threshold = 0.95;
        // let similar_groups = find_duplicate_groups(&images, threshold);

        // serialize_to_json_file(similar_groups, &data_file)?;
    }
    Ok(())
}

fn directories_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut directories = Vec::new();

    // Read the directory entries
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                // Check if the entry is a directory
                if path.is_dir() {
                    directories.push(path);
                }
            }
        }
    }

    directories
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
            .template("[{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    pb
}

fn serialize_to_json_file(data: Vec<Vec<PathBuf>>, output_path: &PathBuf) -> Result<(), AppError> {
    // Wrap the data in a struct to make it serializable
    let wrapper = PathBufWrapper { paths: data };

    // Create or open the output file
    let file = File::create(output_path)?;

    // Serialize the data to the file in pretty JSON format
    to_writer_pretty(file, &wrapper)?;

    Ok(())
}
