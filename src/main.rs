use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{Read, Write};
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
            let mut z = ZlibDecoder::new(BufReader::new(f));

            let mut buf = Vec::new();
            z.read_until(0, &mut buf).context("read header from .git/objects")?;
            let header = CStr::from_bytes_with_nul(&buf)?.to_str()?;

            let size = if let Some(s) = header.strip_prefix("blob ") {
                s.parse::<usize>().context("invalid blob size")?
            } else {
                return Err("The header does not start with 'blob '".into());
            };

            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf[..]).context("read blob")?;

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(&buf)?;
        }
    };

    Ok(())
}
