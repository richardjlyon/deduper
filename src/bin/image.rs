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

    let path1 = PathBuf::from("test-data/02/face-right-1.jpg");
    let path2 = PathBuf::from("test-data/02/face-right-2.jpg");

    let img1 = Image::from_path(&path1).unwrap();
    let img2 = Image::from_path(&path2).unwrap();

    println!("Image 1 orientation: {:?}", img1.orientation());
    println!("Image 2 orientation: {:?}", img2.orientation());

    img1.image().unwrap().save("image1.jpg").unwrap();
    img2.image().unwrap().save("image2.jpg").unwrap();
}
