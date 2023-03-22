use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use log::debug;
use runner::MdduxState;
use tempfile::NamedTempFile;

use crate::config::Config;

mod config;
mod console;
mod executor;
mod formatter;
mod parser;
mod runner;
mod util;

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Subcommand {
    /// Execute code blocks within a specified Markdown file
    Run(RunArgs),
    /// Execute a console code block content
    RunConsole(RunConsoleArgs),
}

#[derive(Debug, Clone, clap::Args)]
struct RunArgs {
    /// Execute all code blocks forcedly
    #[clap(short, long)]
    all: bool,
    /// Output the result as a Markdown file (*.md)
    #[clap(short = 'O', long)]
    output: bool,
    /// Specify a file to save the result
    #[clap(short = 'o', long)]
    output_file: Option<PathBuf>,
    /// Save the execution state as a JSON file (*.state.json)
    #[clap(short = 'S', long)]
    save_state: bool,
    /// A file to save or load the execution state
    #[clap(long)]
    state_file: Option<PathBuf>,
    /// Execute a file without asking for confirmation
    #[clap(short = 'y', long)]
    no_confirm: bool,
    /// Enable displaying captions for code blocks
    #[clap(long = "caption", overrides_with = "caption")]
    _no_caption: bool,
    /// Disable displaying captions for code blocks
    #[clap(long = "no-caption", action = clap::ArgAction::SetFalse)]
    caption: bool,
    /// An input Markdown file (*.spec.md) to execute
    #[clap(name = "FILE")]
    file: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
struct RunConsoleArgs {
    /// A timeout for the execution
    #[clap(short, long)]
    timeout: Option<u64>,
    /// A console code block content file to execute
    #[clap(name = "FILE")]
    file: Option<PathBuf>,
}

fn make_path(
    original: &Path,
    target: &Option<PathBuf>,
    auto: bool,
    extension: &str,
    default_name: &str,
) -> Option<PathBuf> {
    if target.is_some() {
        return target.clone();
    }
    if !auto {
        return None;
    }
    if original.as_os_str().is_empty() || original == Path::new("-") {
        return Some(default_name.into());
    }
    let parent = original.parent().unwrap_or_else(|| Path::new(""));
    if let Some(stem) = original.file_stem().and_then(|s| Path::new(s).file_stem()) {
        Some(parent.join(stem).with_extension(extension))
    } else {
        Some(parent.join(default_name))
    }
}

fn make_state_path<P: AsRef<Path>>(
    file: P,
    state_file: &Option<PathBuf>,
    save_state: bool,
) -> Option<PathBuf> {
    make_path(
        file.as_ref(),
        state_file,
        save_state,
        "state.json",
        "state.json",
    )
}

fn make_output_path<P: AsRef<Path>>(
    file: P,
    output_file: &Option<PathBuf>,
    output: bool,
) -> Option<PathBuf> {
    make_path(file.as_ref(), output_file, output, "md", "output.md")
}

fn prompt_yes_no(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    let _ = stdout().flush();
    let mut input = String::new();
    let _ = stdin().read_line(&mut input);
    let input = input.trim();
    input.to_lowercase() == "y" || input.to_lowercase() == "yes"
}

fn run_impl<R: BufRead, W: Write>(
    mut br: R,
    mut bw: W,
    _args: &Args,
    subargs: &RunArgs,
) -> Result<()> {
    let conf = Config {
        caption: Some(subargs.caption),
        ..Config::system_default()
    };
    if !subargs.save_state {
        runner::run(br, bw, &conf)?;
    } else {
        let file = &subargs.file;
        let opts = runner::make_comrak_options();
        let mut buf = String::new();
        br.read_to_string(&mut buf)?;
        let mut state = runner::load(&buf, &opts, &conf)?;
        let state_path = make_state_path(file, &subargs.state_file, subargs.save_state);
        let old_state: Option<MdduxState> = if let Some(path) = &state_path {
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
        if let Some(path) = &state_path {
            let swp = NamedTempFile::new_in(path.parent().unwrap())?;
            {
                let state_f = File::create(&swp)?;
                let mut state_bw = BufWriter::new(state_f);
                serde_json::to_writer(&mut state_bw, &state)?;
                bw.write_all(b"\n")?;
            }
            swp.persist(path)?;
        }
        runner::dump(&mut bw, &state, &buf, &opts, &conf)?;
    }
    Ok(())
}

fn run_with_read<R: BufRead>(br: R, args: &Args, subargs: &RunArgs) -> Result<()> {
    let file = &subargs.file;
    let output_path = make_output_path(file, &subargs.output_file, subargs.output);
    if let Some(output_path) = output_path {
        let swp = NamedTempFile::new_in(output_path.parent().unwrap())?;
        {
            let f = File::create(&swp)?;
            let bw = BufWriter::new(f);
            run_impl(br, bw, args, subargs)?;
        }
        swp.persist(output_path)?;
    } else {
        let stdout = stdout().lock();
        let bw = BufWriter::new(stdout);
        run_impl(br, bw, args, subargs)?;
    };
    Ok(())
}

fn run(args: &Args, subargs: &RunArgs) -> Result<()> {
    let file = &subargs.file;
    let file_name = file
        .file_name()
        .expect("file path is not normal")
        .to_string_lossy();
    if file_name == "-" {
        let r = stdin().lock();
        let br = BufReader::new(r);
        run_with_read(br, args, subargs)
    } else {
        if !file_name.ends_with(".spec.md") {
            let prompt = format!("{} seems not to be a *.spec.md file. Proceed?", file_name);
            if !subargs.no_confirm && !prompt_yes_no(&prompt) {
                exit(1);
            }
        }
        let f = File::open(file)?;
        let br = BufReader::new(f);
        run_with_read(br, args, subargs)
    }
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
