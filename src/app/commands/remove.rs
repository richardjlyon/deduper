//! Duplicate removal command.

use deduper::{error::AppError, get_kv_partition, HASH_PARTITION};
use fjall::PartitionHandle;
use std::collections::HashMap;

/// Analyse duplicates, identify the best image, and remove the rest,
/// preserving GPS info and tags.
pub fn remove_command() -> Result<(), AppError> {
    println!("Removing duplicates");

    let image_hash_partition = get_kv_partition(HASH_PARTITION)?;
    let duplicates = get_duplicates(&image_hash_partition)?;

    duplicates.iter().take(2).for_each(|duplicate| {
        println!("Duplicate: {:#?}", duplicate);
    });

    Ok(())
}

/// Get a map of hashes to duplicate image paths from the kv store.
pub fn get_duplicates(partition: &PartitionHandle) -> Result<HashMap<u64, Vec<String>>, AppError> {
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
