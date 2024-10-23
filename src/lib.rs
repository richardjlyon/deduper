pub mod error;
pub mod image;
pub mod indexer;
pub mod similarity;
// use itertools::Itertools;

use error::AppError;
use fjall::{Config, PartitionCreateOptions, PartitionHandle};
use log::LevelFilter;
use std::time::SystemTime;

pub const DUPLICATES_FILE: &str = "results/duplicates.json";
pub const HASH_PARTITION: &str = "image_hashes";
const KV_STORE_PATH: &str = "./fjall";
const LOG_FILE: &str = "logs/output.log";

pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        // .chain(std::io::stdout())
        .chain(fern::log_file(LOG_FILE)?)
        .apply()?;

    Ok(())
}

pub fn get_kv_partition(partition_name: &str) -> Result<PartitionHandle, AppError> {
    let keyspace = Config::new(KV_STORE_PATH).open()?;
    let partition_handle =
        keyspace.open_partition(partition_name, PartitionCreateOptions::default())?;

    Ok(partition_handle)
}

// pub fn process_images(root: &Path) {
//     // hashes of the byte data
//     let byte_hashes = get_byte_hashes(root);

//     // perceptual hash of the content
//     // let perceptual_hashes: Vec<u64> = get_perceptual_hashes(root);

//     // 123 | g1
//     // 123 |
//     // 125 | g2
//     // abc | g3
//     // abc |
//     // fej | g4
//     // sfs | g5

//     let groups = byte_hashes.iter().sorted().chunk_by(|k| k.0);
//     let duplicates = groups.into_iter().filter_map(|(_group_id, mut chunk)| {
//         let first = chunk.next().unwrap();
//         let second = chunk.next()?;
//         Some([first, second].into_iter().chain(chunk))
//     });

//     let duplicates: Vec<_> = duplicates.flatten().collect();
//     println!("{:?}", duplicates);
// }

// fn get_byte_hashes(_root: &Path) -> Vec<(u64, &str)> {
//     vec![
//         (0, "a.png"),
//         (1, "b.png"),
//         (2, "c.png"),
//         (3, "d.png"),
//         (4, "e.png"),
//         (3, "f.png"),
//         (3, "g.png"),
//         (7, "h.png"),
//         (0, "i.png"),
//         (1, "j.png"),
//     ]
// }
