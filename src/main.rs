use anyhow::anyhow;
use anyhow::Context;
#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex::ToHex;
use reqwest::header;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::io::Stdout;
use std::path::PathBuf;
use std::{ffi::CStr, io::BufReader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write_object: bool,

        object_file: String,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_sha: String,
    },
}

fn extract_filenames(input: &str) -> Vec<String> {
    input
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts
                .get(1)
                .map(|s| s.split('_').next().unwrap().to_string())
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear hrrr!");

    let args = Args::parse();
    //println!("Args: {:?}", args);

    match args.command {
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

            //println!("{:?}", blob);

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

            // dbg!(compressed);

            let file_content = &hex_result[2..];

            //println!("{:?}", path);

            if !fs::metadata(&path).is_ok() {
                // Create the directory if it doesn't exist
                fs::create_dir(&path)?;
            } else {
                // do nothing
            }
            //fs::create_dir(format!(".git/objects")).unwrap();

            fs::write(&file_path, &compressed).unwrap();
        }

        Command::LsTree {
            name_only,
            tree_sha,
        } => {
            let path = format!("../.git/objects/{}/{}", &tree_sha[..2], &tree_sha[2..]);

            println!("{}", path);
            let f = std::fs::File::open(path).unwrap();

            let z = ZlibDecoder::new(f);

            let mut z = BufReader::new(z);

            let mut buf = Vec::new();

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

            z.read_exact(&mut buf[..]).context("read tree")?;

            let string_data = String::from_utf8_lossy(&buf);

            let file_names = extract_filenames(&string_data);

            // for filename in file_names {
            //     println!("{}", filename);
            // }

            

            let stdout = std::io::stdout();

            let mut stdout = stdout.lock();

            for f in file_names {
                stdout.write_all(f.as_bytes())
                .context("write all to stdout")?;
            }
        }
        _ => {
            println!("unknown command: {:?}", args.command);
        }
    };

    Ok(())
}
