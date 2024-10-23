use deduper::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct HashMapWrapper {
    pub duplicates: HashMap<u64, Vec<String>>,
}

pub fn serialize_to_json_file(
    data: HashMap<u64, Vec<String>>,
    output_path: &PathBuf,
) -> Result<(), AppError> {
    let wrapper = HashMapWrapper { duplicates: data };
    let file = File::create(output_path)?;
    to_writer_pretty(file, &wrapper)?;
    Ok(())
}

pub fn _deserialize_from_json_file(file_path: &PathBuf) -> Result<HashMapWrapper, AppError> {
    let json_str = fs::read_to_string(file_path)?;
    let duplicates = serde_json::from_str(&json_str)?;
    Ok(duplicates)
}
