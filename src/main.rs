mod image;
use std::path::PathBuf;

use image::Image;

fn main() {
    let mut project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_dir = project_root.join("test-data/case-1");
    println!("{:#?}", test_dir);

    let mut test_dir = PathBuf::from("/Users/richardlyon/dev/deduper/test-data/case-1");
    let img1_name = test_dir.join("2017-03-12-iPhone 7.jpg");
    let img2_name = test_dir.join("IMG_0026-1.JPG");

    let img1 = Image { path: img1_name };

    let img2 = Image { path: img2_name };

    println!("image1: {:#?}", img1);
}
