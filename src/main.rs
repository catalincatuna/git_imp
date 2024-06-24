use anyhow::Context;
#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::Compression;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::io::Stdout;
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
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    //println!("Logs from your program will appear here!");

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
            let mut f = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .unwrap();

            let mut z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("read header from .git/objects");
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

            z.read_exact(&mut buf[..]).context("read blob");

            let stdout = std::io::stdout();

            let mut stdout = stdout.lock();

            stdout.write_all(&buf).context("write all to stdout");

            // let stdout = std::io::stdout();
            // let mut stdout = stdout.lock();
            // stdout.write_all(&f).context("write all to stdout")?;

            // let mut z = BufReader::new(z);
            // let mut buf = Vec::new();

            // z.read_until(0, &mut buf).context("read header from .git/objects");
            // let header = CStr::from_bytes_with_nul(&buf).expect("one null at the end");

            // let header = header.to_str().context("not valid UTF-8");

            // let Some(size) = header.strip_prefix(("blob")) else {
            //     anyhow::bail!("not a blob");
            // };

            // let size = size.parse::<usize>().context(".git/objects not a blob")?;
            // buf.clear();

            // buf.resize(size, 0);

            // z.read_exact(&mut buf[..]).context("read blob")?;

            // let n = z.read(&mut [0]).context("validate EOF")?;

            // anyhow::ensure!(n == 0, "invalid EOF, n has trailing bytes");

            // let stdout = std::io::stdout();

            // let mut stdout = stdout.lock();

            // stdout.write_all(&buf).context("write all to stdout")?;
        }
        _ => {
            println!("unknown command: {:?}", args.command)
        }
    };
}
