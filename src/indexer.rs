//! Recursively index a directory and generate a list of image paths.

use std::path::PathBuf;
use walkdir::WalkDir;

use crate::image::Image;

pub fn index_images_in_folder(folder: PathBuf) -> Vec<PathBuf> {
    let mut image_files = Vec::new();
    let valid_extensions = Image::valid_extensions();

    for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let extension = path.extension().unwrap_or_default().to_str().unwrap();
        if path.is_file() && valid_extensions.contains(&extension) {
            image_files.push(path.to_path_buf());
        }
    }

    image_files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_images_in_folder() {
        let test_dir = PathBuf::from("test-data");
        let image_files = index_images_in_folder(test_dir);

        assert_eq!(image_files.len(), 12);
    }
}
