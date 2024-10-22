use std::env;
use std::path::PathBuf;

use deduper::image::Image;
use deduper::indexer::index_images_in_folder;
use deduper::setup_logger;
use log::LevelFilter;

fn main() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    setup_logger(log_level).expect("Failed to initialize logger");

    let folder = PathBuf::from("test-data");
    let paths = index_images_in_folder(folder);

    for path in &paths {
        let img = Image::from_path(path).unwrap();
        println!("{:?}", path);
        println!("- {:?}", img.resolution());
        println!("- {:?}", img.orientation());
        println!();
    }
}
