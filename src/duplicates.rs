//! Duplicate image detection.

use std::collections::HashMap;

pub fn find_duplicates<T: Eq + std::hash::Hash + Clone>(items: &[T]) -> HashMap<T, Vec<usize>> {
    let mut index_map: HashMap<T, Vec<usize>> = HashMap::new();

    for (index, item) in items.iter().enumerate() {
        index_map
            .entry(item.clone())
            .or_insert_with(Vec::new)
            .push(index);
    }

    index_map.retain(|_, indices| indices.len() > 1);
    index_map
}

// tests ------------------------------------------------------

#[cfg(test)]
mod tests {

    use std::{path::PathBuf, time::Instant};

    use crate::indexer::index_images_in_folder;

    use super::*;

    #[test]
    fn test_duplicates() {
        let image_paths = get_test_images();

        let start = Instant::now();

        find_duplicates();

        let duration = start.elapsed();
        println!("->> Time elapsed is: {:?}", duration);
    }

    fn get_test_images() -> Vec<PathBuf> {
        let test_dir = PathBuf::from("test-data");
        index_images_in_folder(test_dir)
    }
}
