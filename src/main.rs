use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::{self, Write};

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
            let f = fs::File::open(&file_path).context("Could not open object file")?;
            let mut z = ZlibDecoder::new(f);
            
            let mut decompressed_data = Vec::new();
            z.read_to_end(&mut decompressed_data).context("Failed to read zlib decompressed data")?;

            let mut split = decompressed_data.splitn(2, |b| *b == 0);
            let header = split.next().ok_or("Invalid blob object: Missing header")?;
            let content = split.next().ok_or("Invalid blob object: Missing content")?;

            // Header is skipped, not needed unless we want to validate size or type of object

            let stdout = io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(content)?;
        }
    };

    Ok(())
}
