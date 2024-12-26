use anyhow::Context;
use ascii85::encode;
use base64;
use clap::Error;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::{prelude::*, Cursor};
use std::mem;
use std::path::PathBuf;
use std::{fs, usize};
use std::cmp::Ordering;


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

    let contents = std::fs::read_to_string(&path).unwrap_or(' '.to_string());
    let mut blob = vec![];
    
    write!(&mut blob, "blob {}\0{:?}", contents.len(), contents.as_bytes())?;

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(&blob)?;

    let compressed = e.finish()?;

    let mut hasher = Sha1::new();

    hasher.update(compressed);

    let object_hash = hasher.finalize();

    Ok(object_hash.into())
}

// Function to process a directory
pub fn process_directory(dir: &PathBuf) -> anyhow::Result<Option<[u8; 20]>> {
    let mut input = vec![];
    let mut tree = vec![];
    let mut entries = vec![];
    
    let mut dir =
    fs::read_dir(dir).with_context(|| format!("open directory {}", dir.display()))?;

    // let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    // let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|res| res.ok()).collect();

    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| "bad directory entry")?;
        let name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry")?;
        entries.push((entry, name, meta));
    }

    // entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    entries.sort_unstable_by(|a, b| {
        // git has very specific rules for how to compare names
        // https://github.com/git/git/blob/e09f1254c54329773904fe25d7c545a1fb4fa920/tree.c#L99
        let afn = &a.1;
        let afn = afn.as_encoded_bytes();
        let bfn = &b.1;
        let bfn = bfn.as_encoded_bytes();
        let common_len = std::cmp::min(afn.len(), bfn.len());
        match afn[..common_len].cmp(&bfn[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }
        if afn.len() == bfn.len() {
            return Ordering::Equal;
        }
        let c1 = if let Some(c) = afn.get(common_len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        c1.cmp(&c2)
    });


    for (entry, filename, meta) in entries {
        if filename == ".git" {
            continue;
        }
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        // } else if (meta.permissions(). & 0o111) != 0 {
        //     "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = process_directory(&path)? else {
                // empty directory, so don't include in parent
                continue;
            };
            hash
        } else {
            let hash = compute_file_hash(&path)
                .context("open blob input file")?;
                // .write(std::fs::File::create(tmp).context("construct temporary file for blob")?)
                // .context("stream file into blob")?;
 
            hash
        };

        // println!("{}", mode);
        // println!("{:?}", filename);
        // println!("{:?}", hash);

        input.extend(mode.as_bytes());
        input.push(b' ');
        input.extend(filename.as_encoded_bytes());
        input.push(0);
        input.extend(hash);

    }


    // for entry in entries {
    //     if entry.file_name().to_str().unwrap() == ".git" {
    //         continue;
    //     }
    //     let path = entry.path();
    //     // Safely attempt to retrieve metadata
    //     match entry.metadata() {
    //         Ok(metadata) => {
    //             if metadata.is_dir() {
    //                 let obj = Object {
    //                     mode: String::from("40000 "),
    //                     name: entry.file_name().into_string().unwrap(),
    //                     hash: process_directory(&path).unwrap(),
    //                 };
    //                 // If it's a directory, process it recursively
    //                 // input.extend(obj.serialize());
    //                 input.push(obj);
    //                 //size = size + calculate_total_size(&obj);
    //             } else if metadata.is_file() {
    //                 let obj = Object {
    //                     mode: String::from("100644 "),
    //                     name: entry.file_name().into_string().unwrap(),
    //                     hash: compute_file_hash(&path).unwrap(),
    //                 };
    //                 // input.extend(obj.serialize());
    //                 input.push(obj);
    //                 //size = size + calculate_total_size(&obj);
    //                 // If it's a file, add its hash to the input for the final hash
    //             }
    //         }
    //         Err(e) => {
    //             eprintln!("Failed to get metadata for {:?}: {}", path, e);
    //         }
    //     }
    // }


    // for i in &input {
    //     write!(&mut aux, "{:?}", i.serialize())?;
    //     //i.display();
    // }

    let mut reader = Cursor::new(&input);
    write!(&mut tree, "tree {}\0", input.len() as u64)?;
    std::io::copy(&mut reader, &mut tree).context("stream file into tree")?;
    println!("{:?}", &tree.to_ascii_lowercase());
    //tree.push("tree ".as_bytes());
    // tree.extend_from_slice(b"tree ");
    // write!(&mut tree, "tree ")?;
    // write!(&mut tree, "{}", aux.len())?;
    // tree.push(0);

    // for i in &input {
    //     write!(&mut tree, "{} ", i.mode)?;
    //     write!(&mut tree, "{}", i.name)?;
    //     tree.push(0);
    //     // write!(&mut tree, "{}", i.hash)?;
    //     tree.extend_from_slice(&i.hash);
    // }
    // // replace initial size with actual sizeF
    // let size_bytes = size.to_be_bytes();

    // tree[usize_position..usize_position + std::mem::size_of::<usize>()]
    //     .copy_from_slice(&size_bytes);

    //    // Convert the relevant part of the byte array to a string
    //  if let Ok(ascii_str) = String::from_utf8(tree.clone()) {
    //  println!("ASCII interpretation: {}", ascii_str);
    // } else {
    //     println!("The bytes are not valid UTF-8!");
    // }

    
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    
    //let s = String::from("salut");
    //let s = String::from_utf8_lossy(&compressed).to_string();


    e.write_all(&tree)?;

    let compressed = e.finish()?;



    //input.sort();
    let mut hasher = Sha1::new();

    hasher.update(&compressed);

    let object_hash = hasher.finalize();

    let hex_result = hex::encode(object_hash);

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

    Ok(Some(object_hash.into()))
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
