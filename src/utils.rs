use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use sha1::{Digest, Sha1};
use base64;



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
pub fn compute_file_hash(path: &PathBuf) -> anyhow::Result<String> {
    let content = fs::read_to_string(path).unwrap();
    
    let mut hasher = Sha1::new();

    let hash_input = format!("100644 {}", content);

    hasher.update(hash_input.as_bytes());

    let object_hash = hasher.finalize();

    let base64_result = base64::encode(object_hash);
    
    // Return the hash as a hexadecimal string
    Ok(base64_result)
}

// Function to process a directory
pub fn process_directory(dir: &PathBuf) -> anyhow::Result<String>  {
    let mut input = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            // If it's a directory, process it recursively
            input.push(process_directory(&path).unwrap());
        } else if metadata.is_file() {
            // If it's a file, add its hash to the input for the final hash
            input.push(format!("100644 {} {}",entry.file_name().into_string().unwrap(), compute_file_hash(&path).unwrap()));
        }
        
    }
    let mut hasher = Sha1::new();

    hasher.update(input.concat().as_bytes());

    let object_hash = hasher.finalize();

    let hex_result = hex::encode(object_hash);

    let dir_out = format!("40000 {} {}", dir.file_name().unwrap().to_os_string().into_string().unwrap(), hex_result);

    Ok(dir_out)
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