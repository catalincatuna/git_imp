use flate2::read::ZlibDecoder;
use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use core::hash;
#[allow(unused_imports)]
use std::env;
use std::fmt::format;
#[allow(unused_imports)]
use std::fs;
use std::fs::Metadata;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{ffi::CStr, io::BufReader};
use walkdir::WalkDir;

#[path = "utils.rs"]
mod utils;

use crate::data::Command;

pub fn execute_git_function(cmd: Command) -> anyhow::Result<()>{

    match cmd {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            let f = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .unwrap();

            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;
            let header = CStr::from_bytes_with_nul(&buf).expect("one null at the end");

            let header = header.to_str().unwrap();

            let mut size: usize = 0;

            if let Some(s) = header.strip_prefix("blob ") {
                // println!("Blob size: {:?}", s);
                size = s.parse::<usize>().unwrap();
                // println!("Blob size: {}", size);
            } else {
                // println!("The header does not start with 'blob '.");
            }

            buf.clear();
            buf.resize(size, 0);

            z.read_exact(&mut buf[..]).context("read blob")?;

            let stdout = std::io::stdout();

            let mut stdout = stdout.lock();

            stdout.write_all(&buf).context("write all to stdout")?;
        }

        Command::HashObject {
            write_object,
            object_file,
        } => {
            let file_path = PathBuf::from(format!("{}", object_file));

            let contents = std::fs::read_to_string(&file_path)?;

            let blob = format!("blob {}\0{}", contents.len(), contents);

            let mut hasher = Sha1::new();
            hasher.update(blob.as_bytes());

            let object_hash = hasher.finalize();

            let hex_result = hex::encode(object_hash);

            println!("{}", hex_result);

            let path = format!(".git/objects/{}", &hex_result[..2]);

            let file_path = format!("{}/{}", path, &hex_result[2..]);

            let file_content = &hex_result[2..];

            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

            e.write_all(&blob.as_bytes())?;

            let compressed = e.finish()?;

            let file_content = &hex_result[2..];

            if !fs::metadata(&path).is_ok() {
                // Create the directory if it doesn't exist
                fs::create_dir(&path)?;
            } else {
                // do nothing
            }

            fs::write(&file_path, &compressed).unwrap();
        }

        Command::LsTree {
            name_only,
            tree_sha,
        } => {
            let path = format!(".git/objects/{}/{}", &tree_sha[..2], &tree_sha[2..]);

            //println!("{}", path);
            let f = std::fs::File::open(path).unwrap();

            let z = ZlibDecoder::new(f);

            let mut z = BufReader::new(z);

            let mut buf = Vec::new();

            let mut buf_copy = buf.clone();

            z.read_until(0, &mut buf)
                .context("read header from .git/objects"); 

            
            let header = CStr::from_bytes_with_nul(&buf).unwrap();

            let mut header = header.to_str().unwrap();

            let mut size: usize = 0;

            if let Some(s) = header.strip_prefix("tree ") {
                size = s.parse::<usize>().unwrap();
            } else {
                println!("not a tree");
            }

            buf.clear();

            buf.resize(size, 0);

            z.read_to_end(&mut buf).context("read tree")?;

            //println!("{:?}", buf);

                // let string_data = match String::from_utf8(buf) {
                //     Ok(valid_data) => valid_data,
                //     Err(_) => "/".to_string()//String::from_utf8_lossy(&buf.clo).into_owned()
                // };
                //let string_data = String::from_utf8_lossy(&buf).into_owned();

                let string_data = String::from_utf8_lossy(&buf)
                                 .chars()
                                 .map(|c| {if c.is_ascii() || c.is_alphanumeric() {c} else {'/'}})
                                 .collect::<String>();
                

                let patterns = &["100644", "0100755", "40000"];

                let extracted = utils::extract_after_numeric(string_data, patterns);
                    
                    
                    let stdout = std::io::stdout();
                    
                    let mut stdout = stdout.lock();
                    
                    for f in extracted {
                        stdout.write_all(f.as_bytes())
                        .context("write all to stdout")?;
                    stdout
                    .write_all(b"\n")
                    .context("write newline to stdout")?;
                }
            
        }
        Command::WriteTree => {

            let mut current_dir = std::env::current_dir()?;
            
            //let parent_dir = current_dir.parent().unwrap();
            
            println!("current directory is {:?}", current_dir);
            let mut entries = vec![];


            // for entry in WalkDir::new(current_dir) {
            //     match entry {

            //         Ok(entry) => {
            //             let filename = utils::extract_filename(entry.path().to_string_lossy().to_string());
            //             //=println!("{:?}", &filename);
            //             entries.push(filename);
            //         }
            //         Err(e) => {
            //             println!("error {}", e);
            //         }
                    
            //     };
            // }

            for entry in fs::read_dir(current_dir).unwrap() {
                match entry {

                    Ok(entry) => {

                        let metadata = fs::metadata(entry.path()).unwrap();

                        let filename = utils::extract_filename(entry.path().to_string_lossy().to_string());
                        
                        if(metadata.is_file()) {

                        let content = fs::read_to_string(entry.path()).unwrap();

                        let hash_input = format!("100644 {}", content);

                        let mut hasher = Sha1::new();
                        hasher.update(hash_input.as_bytes());
            
                        let object_hash = hasher.finalize();
            
                        let hex_result = hex::encode(object_hash);
                        

                        entries.push(format!("100644 {}\0{}", filename, hex_result));

                        }
                        else if (metadata.is_dir()) {
                            entries.push(format!("40000 {}\0 asd", filename));
                        }

                    }
                    Err(e) => {
                        println!("error {}", e);
                    }
                }
            }

            let tree = format!("tree {}\0{}", entries.concat().len(), entries.concat());

            println!("{:?}", tree);
            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            
            e.write_all(&tree.as_bytes())?;

            let compressed = e.finish()?;

            let mut hasher = Sha1::new();
            hasher.update(&tree.as_bytes());

            let object_hash = hasher.finalize();

            let hex_result = hex::encode(object_hash);

            println!("{}", hex_result);

            let path = format!(".git/objects/{}", &hex_result[..2]);

            let tree_path = format!("{}/{}", path, &hex_result[2..]);

            if !fs::metadata(&path).is_ok() {
                // Create the directory if it doesn't exist
                fs::create_dir(&path)?;
            } else {
                // do nothing
            }

            fs::write(tree_path, compressed)?;


        }
        _ => {
            println!("unknown command: {:?}", cmd);
        }
    };

Ok(())
}