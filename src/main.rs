use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::{ffi::CStr, io};

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

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git")?;
            fs::create_dir(".git/objects")?;
            fs::create_dir(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
        }
        Command::CatFile { object_hash, .. } => {
            let f = fs::File::open(format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]))?;

            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();
            z.read_until(0, &mut buf)?;
            let header = CStr::from_bytes_with_nul(&buf)?.to_str()?;
            
            if let Some(size_str) = header.strip_prefix("blob ") {
                let size: usize = size_str.parse()?;
                buf.clear();
                buf.resize(size, 0);
                z.read_exact(&mut buf)?;
                
                io::stdout().write_all(&buf)?;
                io::stdout().flush()?; // Make sure all output is written to stdout
            } else {
                return Err(anyhow::anyhow!("The header does not start with 'blob '."));
            }
        }
    };

    Ok(())
}
