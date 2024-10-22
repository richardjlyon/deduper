use std::env;
use std::path::PathBuf;

use deduper::image::Image;
use deduper::setup_logger;
use log::LevelFilter;

fn main() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    setup_logger(log_level).expect("Failed to initialize logger");

    let path = PathBuf::from("test-data/02/face-right-2.jpg");
    let img = Image::from_path(&path).unwrap();

    println!("->> {:?}", img.orientation());

    println!("{:#?}", img);
}
