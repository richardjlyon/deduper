use std::env;
use std::path::PathBuf;

use deduper::duplicates_rayon::create_similarity_index_rayon;
use deduper::indexer::index_images_in_folder;
use deduper::setup_logger;
use log::LevelFilter;

fn main() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "debug".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    setup_logger(log_level).expect("Failed to initialize logger");
    let image_paths = get_test_images();

    let start = std::time::Instant::now();

    let similarity_index = create_similarity_index_rayon(image_paths);
    let duration = start.elapsed();
    println!("->> Time elapsed is: {:?}", duration);

    println!("{:#?}", similarity_index);
}

fn get_test_images() -> Vec<PathBuf> {
    let test_dir = PathBuf::from("test-data/02");
    index_images_in_folder(test_dir)
}
