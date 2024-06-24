use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{Read, Write};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git")?;
            fs::create_dir(".git/objects")?;
            fs::create_dir(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
        }
        Command::CatFile {
            pretty_print: _,
            object_hash,
        } => {
            let file_path = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);
            let f = fs::File::open(file_path)?;
            let mut z = ZlibDecoder::new(f);

            let mut header_buf = Vec::new();
            z.read_until(0, &mut header_buf).context("read header from .git/objects")?;
            // Remove the null byte and parse the header
            let header_str = String::from_utf8(header_buf).context("invalid UTF-8 sequence")?;
            let header_parts: Vec<&str> = header_str.splitn(2, '\0').collect();
            if header_parts.len() != 2 || !header_parts[0].starts_with("blob ") {
                return Err("invalid blob object".into());
            }

            let content_buf = header_parts[1].as_bytes();

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(content_buf)?;
        }
    };

    Ok(())
}
