//! Compute the image hashes and store to fjall keystore

use deduper::{error::AppError, indexer::index_images_in_folder};
use fjall::{Config, PartitionCreateOptions, PartitionHandle};
use indicatif::ProgressStyle;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

const MYLIO_VAULT_ROOT: &str = "/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures";
const TEST_DATA_ROOT: &str = "/Users/richardlyon/dev/deduper/test-data";

fn main() -> Result<(), AppError> {
    let keyspace = Config::new("./fjall").open()?;
    let image_hashes =
        keyspace.open_partition("image_hashes", PartitionCreateOptions::default())?;

    let image_paths = index_images_in_folder(&PathBuf::from(MYLIO_VAULT_ROOT));

    let _ = insert_hashes(&image_paths, &image_hashes, false)?;

    let duplicates = get_duplicates(&image_hashes)?;

    println!("Duplicates:");
    println!("{:#?}", duplicates);

    Ok(())
}

/// Insert the hashes of the images into the fjall keystore
fn insert_hashes(
    image_paths: &[PathBuf],
    partition: &PartitionHandle,
    do_clear: bool,
) -> Result<(), AppError> {
    if do_clear {
        clear_partition(partition)?;
    }

    let pb = indicatif::ProgressBar::new(image_paths.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let partition = Arc::new(Mutex::new(partition));
    let pb = Arc::new(Mutex::new(pb));

    image_paths.par_iter().for_each(|image_path| {
        let serialised_image_path = image_path.to_str().unwrap().as_bytes();

        // Check if the hash already exists
        let exists = {
            let partition = partition.lock().unwrap();
            partition.get(serialised_image_path).unwrap().is_some()
        };

        if exists {
            return;
        }

        let image = deduper::image::Image::from_path(image_path).unwrap();

        let hash = match image.hash() {
            Ok(hash) => hash,
            Err(_) => {
                println!("Failed to get hash from {:?}", image_path);
                return;
            }
        };

        // Convert the u64 hash to a byte array
        let serialised_hash = bincode::serialize(&hash).unwrap();

        {
            let partition = partition.lock().unwrap();
            partition
                .insert(serialised_image_path, serialised_hash)
                .unwrap();
        }

        {
            let pb = pb.lock().unwrap();
            pb.inc(1);
        }
    });

    // pb.finish_with_message("Duplicate search complete");

    Ok(())
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
