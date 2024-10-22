use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::PathBuf;

use deduper::duplicates_rayon::create_similarity_index_rayon;
use deduper::error::AppError;
use deduper::indexer::index_images_in_folder;
use deduper::setup_logger;
use log::LevelFilter;
use serde::Serialize;
use serde_json::to_writer_pretty;

#[derive(Serialize)]
struct HashMapWrapper {
    groups: HashMap<String, Vec<String>>,
}

fn main() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    setup_logger(log_level).expect("Failed to initialize logger");
    let image_paths = get_test_images();

    let start = std::time::Instant::now();

    let similarity_index = create_similarity_index_rayon(image_paths);
    let duration = start.elapsed();
    println!("->> Time elapsed is: {:?}", duration);

    serialize_to_json_file(similarity_index, "result.json");
}

fn get_test_images() -> Vec<PathBuf> {
    let test_dir = PathBuf::from("test-data/02");
    index_images_in_folder(test_dir)
}

fn serialize_to_json_file(
    data: HashMap<String, Vec<String>>,
    output_path: &str,
) -> Result<(), AppError> {
    // Wrap the data in a struct to make it serializable
    let wrapper = HashMapWrapper { groups: data };

    // Create or open the output file
    let file = File::create(output_path)?;

    // Serialize the data to the file in pretty JSON format
    to_writer_pretty(file, &wrapper)?;

    println!("Hashmap has been serialized and saved to '{}'", output_path);

    Ok(())
}
