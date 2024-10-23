use std::path::PathBuf;

use deduper::indexer::index_images_in_folder;

fn main() {
    let test_dir = PathBuf::from("/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures/2001/(07) July");
    let image_files = index_images_in_folder(&test_dir);

    println!(
        "Found {} images in {}",
        image_files.len(),
        test_dir.display()
    );
}
