use ascii85::encode;
use base64;
use clap::Error;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::prelude::*;
use std::mem;
use std::path::PathBuf;
use std::{fs, usize};

use crate::data::Object;

pub fn extract_after_numeric(input: String, patterns: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    let mut start_index = 0;

    while let Some(pos) = patterns
        .iter()
        .filter_map(|pattern| input[start_index..].find(pattern).map(|i| (i, *pattern)))
        .min_by_key(|(i, _)| *i)
    // Find the closest match
    {
        let pattern_pos = start_index + pos.0;
        let pattern = pos.1;
        start_index = pattern_pos + pattern.len();

        let next_part = input[start_index + 1..]
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

    let array: [u8; 20] = object_hash
        .as_slice()
        .try_into()
        .expect("SHA-1 hash should be 20 bytes");

    // let encoded = encode(object_hash.as_slice());

    // let encoded_string = encoded.to_string();

    Ok(array)
}

// Function to process a directory
pub fn process_directory(dir: &PathBuf) -> anyhow::Result<[u8; 20]> {
    let mut tree = vec![];
    let mut size: usize = 100;

    //tree.push("tree ".as_bytes());
    tree.extend_from_slice(b"tree ");

    let usize_position = tree.len();

    tree.extend_from_slice(&size.to_be_bytes());

    tree.push(0);

    // let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|res| res.ok()).collect();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for entry in entries {
        let path = entry.path();
        // Safely attempt to retrieve metadata
        match entry.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    let obj = Object {
                        mode: String::from("040000 "),
                        name: entry.file_name().into_string().unwrap(),
                        hash: process_directory(&path).unwrap(),
                    };
                    // If it's a directory, process it recursively
                    tree.extend(obj.serialize());
                    size = size + calculate_total_size(&obj);
                } else if metadata.is_file() {
                    let obj = Object {
                        mode: String::from("100644 "),
                        name: entry.file_name().into_string().unwrap(),
                        hash: compute_file_hash(&path).unwrap(),
                    };
                    tree.extend(obj.serialize());
                    size = size + calculate_total_size(&obj);
                    // If it's a file, add its hash to the input for the final hash
                }
            }
            Err(e) => {
                eprintln!("Failed to get metadata for {:?}: {}", path, e);
            }
        }
    }

    // replace initial size with actual sizeF
    let size_bytes = size.to_be_bytes();

    // tree[usize_position..usize_position + std::mem::size_of::<usize>()]
    //     .copy_from_slice(&size_bytes);

    //    // Convert the relevant part of the byte array to a string
    //  if let Ok(ascii_str) = String::from_utf8(tree.clone()) {
    //  println!("ASCII interpretation: {}", ascii_str);
    // } else {
    //     println!("The bytes are not valid UTF-8!");
    // }

 
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(&tree)?;

    let compressed = e.finish()?;

    //input.sort();
    let mut hasher = Sha1::new();

    hasher.update(tree);

    let object_hash = hasher.finalize();

    let array: [u8; 20] = object_hash
        .as_slice()
        .try_into()
        .expect("SHA-1 hash should be 20 bytes");

    let hex_result = hex::encode(array);

    let path = format!(".git/objects/{}", &hex_result[..2]);

    let tree_path = format!("{}/{}", path, &hex_result[2..]);

    if !fs::metadata(&path).is_ok() {
        // Create the directory if it doesn't exist
        fs::create_dir(&path)?;
    } else {
        // do nothing
    }

    fs::write(tree_path, compressed)?;

    // let encoded = encode(object_hash.as_slice());

    // let encoded_string = encoded.to_string();

    //println!("{:?}", obj);

    Ok(array)
}

pub fn extract_filename(input: String) -> String {
    let filename = input
        .chars()
        .rev()
        .take_while(|&c| c != '\\')
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    filename
}

pub fn calculate_total_size(obj: &Object) -> usize {
    // Size of the object on the stack
    let size_on_stack = mem::size_of_val(obj);

    // Size of the heap-allocated data for `String`
    let heap_size = obj.mode.len() + obj.name.len();

    // Total size (stack + heap)
    size_on_stack + heap_size
}
