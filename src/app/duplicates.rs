//! Compute the image hashes, store to fjall keystore and save duplicates to a JSON file.

use deduper::{error::AppError, indexer::index_images_in_folder, setup_logger};
use fjall::{Config, PartitionCreateOptions, PartitionHandle};
use indicatif::ProgressStyle;
use log::{error, LevelFilter};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::to_writer_pretty;
use std::{
    collections::HashMap,
    env,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

const _TEST_DATA_ROOT: &str = "/Users/richardlyon/dev/deduper/test-data";
const RESULT_FILE: &str = "results/duplicates.json";

#[derive(Serialize, Default)]
struct HashMapWrapper {
    duplicates: HashMap<u64, Vec<String>>,
}

pub fn duplicates_command(root: &str, do_reset: bool) -> Result<i32, AppError> {
    // initialise logging
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);
    setup_logger(log_level).expect("Failed to initialize logger");

    // compute the hashes for the images
    let image_paths = index_images_in_folder(&PathBuf::from(root));
    let keyspace = Config::new("./fjall").open()?;
    let image_hashes =
        keyspace.open_partition("image_hashes", PartitionCreateOptions::default())?;

    // insert the hashes in the kv store (set to true to rebuild the partition)
    let fail_count = insert_hashes(&image_paths, &image_hashes, do_reset)?;

    // compute duplicates and save to a JSON file
    let duplicates = get_duplicates(&image_hashes)?;
    match serialize_to_json_file(duplicates, &PathBuf::from(RESULT_FILE)) {
        Ok(_) => Ok(fail_count),
        Err(e) => Err(e),
    }
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

    let pb = indicatif::ProgressBar::new(image_paths.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("[{eta_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let partition = Arc::new(Mutex::new(partition));
    let pb = Arc::new(Mutex::new(pb));
    let fail_count = Arc::new(Mutex::new(0));

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
                error!("Failed to get hash from {:?}", image_path);
                let mut fail_count = fail_count.lock().unwrap();
                *fail_count += 1;
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

    let pb = pb.lock().unwrap();
    pb.finish_with_message("Duplicate search complete");

    let fail_count = *fail_count.lock().unwrap();

    Ok(fail_count)
}

/// Find duplicate hashes in keystore values
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

/// Delete all keys
fn clear_partition(partition: &PartitionHandle) -> Result<(), AppError> {
    for kv in partition.iter() {
        let (key, _) = kv?;
        partition.remove(&key)?;
    }

    Ok(())
}

fn serialize_to_json_file(
    data: HashMap<u64, Vec<String>>,
    output_path: &PathBuf,
) -> Result<(), AppError> {
    // Wrap the data in a struct to make it serializable

    let wrapper = HashMapWrapper { duplicates: data };

    // Create or open the output file
    let file = File::create(output_path)?;

    // Serialize the data to the file in pretty JSON format
    to_writer_pretty(file, &wrapper)?;

    Ok(())
}
