use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use log::debug;

use crate::config::Config;

mod cmark;
mod config;
mod executor;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long = "no-caption", action = clap::ArgAction::SetFalse)]
    caption: bool,
    #[clap(long = "caption", overrides_with = "caption")]
    _no_caption: bool,
    #[clap(name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    debug!("{:?}", args);
    for file in args.files.iter() {
        let f = File::open(file)?;
        let br = BufReader::new(f);
        let conf = Config {
            caption: Some(args.caption),
            ..Default::default()
        };
        cmark::process(br, &conf)?;
    }
    Ok(())
}
