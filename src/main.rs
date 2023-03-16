use std::fs::File;
use std::io::{stdin, stdout, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use log::debug;

use crate::config::Config;

mod cmark;
mod config;
mod console;
mod executor;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Subcommand {
    Run(RunArgs),
    RunConsole(RunConsoleArgs),
}

#[derive(Debug, Clone, clap::Args)]
struct RunArgs {
    #[clap(long = "no-caption", action = clap::ArgAction::SetFalse)]
    caption: bool,
    #[clap(long = "caption", overrides_with = "caption")]
    _no_caption: bool,
    #[clap(name = "FILE")]
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
#[clap(author, version, about, long_about = None)]
struct RunConsoleArgs {
    #[clap(short, long)]
    timeout: Option<u64>,
    #[clap(name = "FILE")]
    file: Option<PathBuf>,
}

fn run(_args: &Args, subargs: &RunArgs) -> Result<()> {
    for file in subargs.files.iter() {
        let f = File::open(file)?;
        let br = BufReader::new(f);
        let conf = Config {
            caption: Some(subargs.caption),
            ..Config::system_default()
        };
        cmark::process(br, &conf)?;
    }
    Ok(())
}

fn console(_args: &Args, subargs: &RunConsoleArgs) -> Result<()> {
    let stdout = stdout().lock();
    let file = subargs.file.clone().unwrap_or_else(|| "-".into());
    let timeout_ms = subargs.timeout.map(|v| v * 1000);
    if file.as_path() == Path::new("-") {
        let stdin = stdin().lock();
        let br = BufReader::new(stdin);
        console::run_console(br, stdout, timeout_ms)?;
    } else {
        let f = File::open(file)?;
        let br = BufReader::new(f);
        console::run_console(br, stdout, timeout_ms)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    debug!("{:?}", args);
    match &args.subcommand {
        Subcommand::Run(subargs) => run(&args, subargs),
        Subcommand::RunConsole(subargs) => console(&args, subargs),
    }
}
