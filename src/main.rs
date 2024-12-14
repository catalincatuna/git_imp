use anyhow::Context;
use clap::Parser;
#[allow(unused_imports)]
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{ffi::CStr, io::BufReader};

//
mod git_functions;
mod utils;
mod data;



use data::Command;
use data::Args;


fn main() -> anyhow::Result<()> {

  
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear hrrr!");

    let args = Args::try_parse().unwrap();

    git_functions::execute_git_function(args.command)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // Bring the `add` function from the parent module into scope
    use super::*;

    #[test]
    fn test1() {
        let result = git_functions::execute_git_function(Command::Init).unwrap_or(()); // Now `add` is accessible
        assert_eq!(result, ());
    }
}
