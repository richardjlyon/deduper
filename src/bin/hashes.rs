//! Compute the image hashes and store to fjall keystore

use deduper::{error::AppError, indexer::index_images_in_folder};
use fjall::{Config, PartitionCreateOptions, PartitionHandle};
use std::{collections::HashMap, path::PathBuf};

const _MYLIO_VAULT_ROOT: &str = "/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures";
const TEST_DATA_ROOT: &str = "/Users/richardlyon/dev/deduper/test-data";

fn main() -> Result<(), AppError> {
    let keyspace = Config::new("./fjall").open()?;
    let image_hashes =
        keyspace.open_partition("image_hashes", PartitionCreateOptions::default())?;

    // Insert hashes of the images into the fjall keystore
    let image_paths = index_images_in_folder(&PathBuf::from(TEST_DATA_ROOT));
    let _ = insert_hashes(&image_paths, &image_hashes, false)?;

    // Find duplicate hashes in keystore values
    let duplicates = get_duplicates(&image_hashes)?;

    println!("{:#?}", duplicates);

    Ok(())
}

/// Insert the hashes of the images into the fjall keystore
fn insert_hashes(
    image_paths: &[PathBuf],
    partition: &PartitionHandle,
    do_clear: bool,
) -> Result<i32, AppError> {
    if do_clear {
        clear_partition(partition)?;
    }

    let mut insert_count = 0;

    for image_path in image_paths {
        let serialised_image_path = image_path.to_str().unwrap().as_bytes();

        // check if the hash already exists
        if partition.get(serialised_image_path).unwrap().is_some() {
            continue;
        }

        let image = deduper::image::Image::from_path(image_path)?;
        let hash = image.hash()?;

        // Convert the u64 hash to a byte array
        let serialised_hash = bincode::serialize(&hash).unwrap();

        partition.insert(serialised_image_path, serialised_hash)?;
        insert_count += 1;
    }

    Ok(insert_count)
}

// Find duplicate hashes in keystore values
fn get_duplicates(partition: &PartitionHandle) -> Result<HashMap<u64, Vec<String>>, AppError> {
    let kv = partition
        .iter()
        .filter_map(|kv| {
            kv.ok().and_then(|(key, value)| {
                let key = String::from_utf8(key.to_vec()).ok()?;
                let value: u64 = bincode::deserialize(&value).ok()?;
                Some((key, value))
            })
        })
        .collect::<Vec<_>>();

    let mut map: HashMap<u64, Vec<String>> = HashMap::new();

    for (string, number) in kv {
        map.entry(number).or_default().push(string);
    }
    map.retain(|_, v| v.len() > 1);

    Ok(map)
}

// Delete all keys
fn clear_partition(partition: &PartitionHandle) -> Result<(), AppError> {
    for kv in partition.iter() {
        let (key, _) = kv?;
        partition.remove(&key)?;
    }

    Ok(())
}
