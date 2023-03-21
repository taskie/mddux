use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use log::debug;
use runner::MdduxState;

use crate::config::Config;

mod config;
mod console;
mod executor;
mod formatter;
mod parser;
mod runner;
mod util;

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
    #[clap(short, long)]
    all: bool,
    #[clap(short, long)]
    state: Option<PathBuf>,
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
        let mut br = BufReader::new(f);
        let conf = Config {
            caption: Some(subargs.caption),
            ..Config::system_default()
        };
        let stdout = stdout().lock();
        let mut bw = BufWriter::new(stdout);
        if subargs.state.is_none() {
            runner::run(br, bw, &conf)?;
        } else {
            let opts = runner::make_comrak_options();
            let mut buf = String::new();
            br.read_to_string(&mut buf)?;
            let mut state = runner::load(&buf, &opts, &conf)?;
            let old_state: Option<MdduxState> = if let Some(path) = &subargs.state {
                if path.exists() {
                    let state_f = File::open(path)?;
                    let state_br = BufReader::new(state_f);
                    Some(serde_json::from_reader(state_br)?)
                } else {
                    None
                }
            } else {
                None
            };
            if subargs.all {
                state.execute_all()?;
            } else {
                state.execute_if_needed(&old_state)?;
            }
            if let Some(path) = &subargs.state {
                let state_f = File::create(path)?;
                let mut state_bw = BufWriter::new(state_f);
                serde_json::to_writer(&mut state_bw, &state)?;
                bw.write_all(b"\n")?;
            }
            runner::dump(&mut bw, &state, &buf, &opts, &conf)?;
        }
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
