use std::io::{BufRead, Write};

use anyhow::{Context as _, Result};
use log::{debug, warn};
use rexpect::{reader::Regex, spawn_bash};

#[derive(Clone, Debug)]
struct Document {
    entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
struct Entry {
    command: String,
    expected: Option<String>,
}

fn parse_lines_as_document<I: Iterator<Item = String>>(iter: I) -> Document {
    let mut entries = vec![];
    let mut command_lines = vec![];
    let mut output_lines = vec![];
    let mut push_entry = |command_lines: &mut Vec<String>, output_lines: &mut Vec<String>| {
        if !command_lines.is_empty() {
            let expected = if output_lines.is_empty() {
                None
            } else {
                Some(output_lines.join("\n") + "\n")
            };
            let entry = Entry {
                command: command_lines.join("\n"),
                expected,
            };
            entries.push(entry);
        }
        command_lines.clear();
        output_lines.clear();
    };
    let mut cmd_continues = false;
    for l in iter {
        let cmd_starts = !cmd_continues && l.starts_with('$');
        let is_cmd = cmd_starts || cmd_continues;
        if is_cmd {
            if !cmd_continues {
                push_entry(&mut command_lines, &mut output_lines);
            }
            let start: usize = usize::from(cmd_starts);
            cmd_continues = is_cmd && l.ends_with('\\');
            let end: usize = if cmd_continues { l.len() - 1 } else { l.len() };
            let command_line = l[start..end].to_owned();
            command_lines.push(command_line);
        } else {
            output_lines.push(l);
        }
    }
    push_entry(&mut command_lines, &mut output_lines);
    Document { entries }
}

fn parse_document<R: BufRead>(r: R) -> Result<Document> {
    itertools::process_results(r.lines(), |iter| parse_lines_as_document(iter))
        .context("can't read input")
}

pub(crate) fn run_console<R: BufRead, W: Write>(
    r: R,
    mut w: W,
    timeout: Option<u64>,
) -> Result<()> {
    let ansi_color_regex = Regex::new("\u{1b}\\[[0-9;]*m").unwrap();
    let document = parse_document(r)?;
    debug!("{:?}", document);
    let mut p = spawn_bash(timeout)?;
    // workaround
    p.execute("bind 'set enable-bracketed-paste off'", "")?;
    p.wait_for_prompt()?;
    for entry in document.entries.iter() {
        w.write_all(b"$")?;
        w.write_all(entry.command.as_bytes())?;
        w.write_all(b"\n")?;
        p.execute(&entry.command, "")?;
        let actual = p.wait_for_prompt()?;
        let actual = ansi_color_regex.replace_all(&actual, "");
        let actual = actual.replace('\r', "");
        if let Some(expected) = entry.expected.as_ref() {
            if expected != &actual {
                warn!("expected: \n{}", expected.trim_end());
            }
        }
        w.write_all(actual.as_bytes())?;
    }
    p.execute("exit", "")?;
    Ok(())
}
