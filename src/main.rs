use fjall::{Config, Keyspace, PartitionCreateOptions, PersistMode};
use serde::de::Error;
use std::{path::Path, time::Instant};

use deduper::{image::Image, process_images};

fn main() -> Result<(), fjall::Error> {
    let path = Path::new("test-data/01/house.jpg");
    process_images(&path);
    let image = Image::from_path(&path.to_owned()).unwrap();
    let time = Instant::now();
    let hash = image.hash(0.5).unwrap();
    let total = Instant::now();
    println!(
        "hashed {} in {}",
        hash,
        total.duration_since(time).as_millis()
    );

    let keyspace = Config::new("./fjall").open()?;

    // Each partition is its own physical LSM-tree
    let items = keyspace.open_partition("byte_hashes", PartitionCreateOptions::default())?;

    // Write some data
    let data = serde_json::to_string(&(0u64, "./test-data/01/house.jpg")).unwrap();
    items.insert("a", data)?;

    // And retrieve it
    let bytes = items.get("a")?;
    println!("bytes {:?}", bytes);

    // Or remove it again
    // items.remove("a")?;

    // Search by prefix
    for kv in items.prefix("prefix") {
        // ...
    }

    // Search by range
    for kv in items.range("a"..="z") {
        // ...
    }

    // Iterators implement DoubleEndedIterator, so you can search backwards, too!
    for kv in items.prefix("prefix").rev() {
        // ...
    }

    // Sync the journal to disk to make sure data is definitely durable
    // When the keyspace is dropped, it will try to persist
    keyspace.persist(PersistMode::SyncAll)?;

    Ok(())
}
