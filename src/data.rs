
use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

pub struct Object {
    pub mode: String,
    pub name: String,
    pub hash: [u8; 20]
}

impl Object {   
    // Serialize
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.mode.as_bytes()); // Add field1 as bytes
        data.extend_from_slice(self.name.as_bytes());    // Add field2 as bytes
        data.extend_from_slice(&self.hash);    // Add field2 as bytes
        data
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
    WriteTree 
}