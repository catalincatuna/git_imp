// Removed the unused import 'anyhow::Context'
#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::{ffi::CStr, io::BufRead};

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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git")?;
            fs::create_dir(".git/objects")?;
            fs::create_dir(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
        }
        Command::CatFile { object_hash, .. } => {
            let path = format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]);
            let f = File::open(path)?;
            // Removed 'mut' from the 'z' variable
            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();
            z.read_until(b'\0', &mut buf)?;

            let header = CStr::from_bytes_with_nul(&buf).unwrap();
            let header_str = header.to_str()?;

            let size = if let Some(s) = header_str.strip_prefix("blob ") {
                s.parse::<usize>()?
            } else {
                anyhow::bail!("Object is not a blob");
            };

            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf)?;

            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(&buf)?;
        }
    }

    Ok(())
}
