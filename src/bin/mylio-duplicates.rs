//! A binary for finding duplicate files in a Mylio vault.
//!
//! To ease pressure on memory, images are processed in batches corresponding to
//! the subfolders of the selected vault subfolder.

const MYLIO_VAULT_ROOT: &str = "/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures";

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

use deduper::setup_logger;
use deduper::{duplicates_rayon::create_similarity_index_rayon, error::AppError};
use log::{info, LevelFilter};
use serde::Serialize;
use serde_json::to_writer_pretty;

#[derive(Serialize, Default)]
struct HashMapWrapper {
    groups: HashMap<String, Vec<String>>,
}

fn main() {
    init_logging();

    let output_dir = PathBuf::from("results");

    let vault_root = PathBuf::from(MYLIO_VAULT_ROOT);
    let folders = get_folders(&vault_root);

    for folder in &folders {
        let subfolders = get_folders(folder);
        for subfolder in subfolders {
            // Create the output filename
            let folder_name = clean_path(folder);
            let subfolder_name = clean_path(&subfolder);

            let result_file = output_dir.join(format!("{}__{}.json", folder_name, subfolder_name));

            // Sip if the file already exists
            if result_file.exists() {
                info!("File exists: skipping {}", result_file.display());
                continue;
            }

            // Index the images in the folder and create a similarity index
            let image_paths = deduper::indexer::index_images_in_folder(&subfolder);

            if image_paths.is_empty() {
                info!("No images found in {}", subfolder.display());
                continue;
            }

            info!(
                "Found {} images in {}",
                image_paths.len(),
                subfolder.display()
            );

            let similarity_index = create_similarity_index_rayon(image_paths);

            let _ = serialize_to_json_file(similarity_index, &result_file);
        }
    }
}

fn init_logging() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    setup_logger(log_level).expect("Failed to initialize logger");
}

fn clean_path(path: &Path) -> String {
    path.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .replace(" ", "_")
}

fn get_folders(vault_root: &PathBuf) -> Vec<PathBuf> {
    let mut folders = Vec::new();

    for entry in std::fs::read_dir(vault_root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            folders.push(path);
        }
    }

    folders.sort();

    folders
}

fn serialize_to_json_file(
    data: HashMap<String, Vec<String>>,
    output_path: &PathBuf,
) -> Result<(), AppError> {
    // Wrap the data in a struct to make it serializable

    let wrapper = HashMapWrapper { groups: data };

    // Create or open the output file
    let file = File::create(output_path)?;

    // Serialize the data to the file in pretty JSON format
    to_writer_pretty(file, &wrapper)?;

    Ok(())
}
