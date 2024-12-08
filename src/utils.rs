use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use clap::Error;
use sha1::{Digest, Sha1};
use base64;
use ascii85::encode;

use crate::data::Object;


pub fn extract_after_numeric(input: String, patterns: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    let mut start_index = 0;

    while let Some(pos) = patterns
        .iter()
        .filter_map(|pattern| input[start_index..].find(pattern).map(|i| (i, *pattern)))
        .min_by_key(|(i, _)| *i) // Find the closest match
    {
        let pattern_pos = start_index + pos.0;
        let pattern = pos.1;
        start_index = pattern_pos + pattern.len();

        let next_part = input[start_index+1..]
            .chars()
            .take_while(|&c| c != '\0')
            .collect::<String>();
        if !next_part.is_empty() {
            results.push(next_part);
        }
    }

    results
}
// Function to compute the hash of a file
pub fn compute_file_hash(path: &PathBuf) -> anyhow::Result<[u8; 20], Error> {

    let content = fs::read(path)?; // Read raw bytes
    let string_content = String::from_utf8_lossy(&content).to_string(); // Convert to String, replacing invalid bytes

    
    let mut hasher = Sha1::new();

    let hash_input = format!("100644 {}", string_content);

    hasher.update(hash_input.as_bytes());

    let object_hash = hasher.finalize();

    let array: [u8; 20] = object_hash.as_slice().try_into().expect("SHA-1 hash should be 20 bytes");

    // let encoded = encode(object_hash.as_slice());

    // let encoded_string = encoded.to_string();

    Ok(array)
}

// Function to process a directory
pub fn process_directory(dir: &PathBuf) -> anyhow::Result<Object>  {
    let mut input = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            // If it's a directory, process it recursively
            input.extend(process_directory(&path).unwrap().serialize());
        } else if metadata.is_file() {

            let obj = Object {
                mode: String::from("100644"),
                name: entry.file_name().into_string().unwrap(),
                hash: compute_file_hash(&path).unwrap()
            };
            // If it's a file, add its hash to the input for the final hash
            input.extend(obj.serialize());
            
        }
        
    }
    //input.sort();
    let mut hasher = Sha1::new();

    hasher.update(input);

    let object_hash = hasher.finalize();

    // let encoded = encode(object_hash.as_slice());

    // let encoded_string = encoded.to_string();
    let array: [u8; 20] = object_hash.as_slice().try_into().expect("SHA-1 hash should be 20 bytes");

    let obj = Object {
        mode: String::from("40000"),
        name: dir.file_name().unwrap().to_os_string().into_string().unwrap(),
        hash: array
    };

    //let dir_out = format!("40000 {} {}", dir.file_name().unwrap().to_os_string().into_string().unwrap(), array);

    //println!("{:?}", obj);

    Ok(obj)
}

pub fn extract_filename(input: String) -> String {
    
    let filename = input.chars()
                        .rev()
                        .take_while(|&c| c != '\\')
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>();
    filename
}