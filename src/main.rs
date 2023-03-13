use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{stdout, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};
use bstr::BString;
use clap::Parser;
use comrak::arena_tree::Node;
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};
use log::debug;
use serde::{Deserialize, Serialize};
use yaml_rust::{YamlEmitter, YamlLoader};

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

#[derive(Debug, Serialize)]
struct Execution {
    execution_count: i32,
    type_: String,
    content: String,
    settings: ExecutionSettings,
    command: ExecutionCommand,
    result: ExecutionResult,
}

#[derive(Debug, Serialize)]
struct ExecutionSettings {
    // nop
}

#[derive(Debug, Serialize)]
struct ExecutionCommand {
    program: String,
    args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExecutionResult {
    status_code: Option<i32>,
    stdout: BString,
    stderr: BString,
}

#[derive(Debug)]
struct State {
    runners: HashMap<String, Runner>,
    executions: Vec<Execution>,
    execution_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MadduxFrontMatter {
    runners: HashMap<String, Runner>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Runner {
    name: String,
    command: Vec<String>,
    special_comment_prefix: Option<String>,
    special_comment_suffix: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let mut runners = HashMap::<String, Runner>::new();
        runners.insert(
            "sh".to_owned(),
            Runner {
                name: "sh".to_owned(),
                command: vec!["/bin/sh".to_owned()],
                special_comment_prefix: Some("#".to_owned()),
                special_comment_suffix: None,
            },
        );
        runners.insert(
            "bash".to_owned(),
            Runner {
                name: "bash".to_owned(),
                command: vec!["/bin/bash".to_owned()],
                special_comment_prefix: Some("#".to_owned()),
                special_comment_suffix: None,
            },
        );
        Self {
            runners,
            executions: Default::default(),
            execution_count: 0,
        }
    }
}

fn process<R: BufRead>(mut r: R) -> Result<()> {
    let arena = Arena::new();
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let mut opts = ComrakOptions::default();
    opts.extension.front_matter_delimiter = Some("---".to_owned());
    let root = parse_document(&arena, &buf, &opts);
    let state = Arc::new(RefCell::new(State::default()));
    iter_nodes(root, &|node| {
        let mut state = state.borrow_mut();
        match node.data.borrow().value {
            NodeValue::FrontMatter(ref bs) => {
                let s = String::from_utf8(bs.clone()).unwrap();
                debug!("s: {:?}", s);
                let docs = YamlLoader::load_from_str(&s).unwrap();
                debug!("docs: {:?}", docs);
                if !docs.is_empty() {
                    let doc = &docs[0];
                    let mddux = &doc["mddux"];
                    let mut mddux_str = String::new();
                    {
                        let mut emitter = YamlEmitter::new(&mut mddux_str);
                        let Ok(_) = emitter.dump(mddux) else { return; };
                    }
                    let Ok(mddux): Result<MadduxFrontMatter> = serde_yaml::from_str(&mddux_str).context("can't parse front matter") else { return; };
                    for (k, v) in mddux.runners {
                        state.runners.insert(k, v);
                    }
                }
            }
            NodeValue::CodeBlock(ref code_block) => {
                let info = String::from_utf8(code_block.info.clone()).unwrap();
                let info_vec: Vec<_> = info.splitn(2, ':').collect();
                let code_type = info_vec[0];
                let file_name = info_vec.get(1);
                let file_name = file_name.map_or("tmp".to_owned(), |s| (*s).to_owned());
                create_dir_all("tmp").unwrap();
                let runner = state.runners.get(code_type).cloned();
                if let Some(runner) = runner {
                    let fpath = format!("{}/{}", "tmp", file_name);
                    {
                        let mut w = File::create(&fpath).unwrap();
                        w.write_all(code_block.literal.as_slice()).unwrap();
                        w.flush().unwrap();
                    }
                    state.execution_count += 1;
                    let execution_count = state.execution_count;
                    let program = &runner.command[0];
                    let mut args = runner.command[1..].to_vec();
                    args.push(fpath);
                    let output = Command::new(program).args(&args).output().unwrap();
                    let execution = Execution {
                        execution_count,
                        type_: code_type.to_owned(),
                        content: String::from_utf8_lossy(&code_block.literal).into_owned(),
                        settings: ExecutionSettings {},
                        command: ExecutionCommand {
                            program: program.to_owned(),
                            args: args.clone(),
                        },
                        result: ExecutionResult {
                            status_code: output.status.code(),
                            stdout: BString::new(output.stdout.clone()),
                            stderr: BString::new(output.stderr.clone()),
                        },
                    };
                    state.executions.push(execution);
                    debug!("status: {:?}", output.status);
                    let caption = format!("In [{}]:", execution_count).as_bytes().to_vec();
                    let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
                    let new_node = arena.alloc(new_node);
                    node.insert_before(new_node);
                    if !output.stderr.is_empty() {
                        let text = NodeCodeBlock {
                            info: b"text:stderr".to_vec(),
                            literal: output.stderr.clone(),
                            ..code_block.clone()
                        };
                        let new_node =
                            Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
                        let new_node = arena.alloc(new_node);
                        node.insert_after(new_node);
                        let caption = format!("Err [{}]:", execution_count).as_bytes().to_vec();
                        let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
                        let new_node = arena.alloc(new_node);
                        node.insert_after(new_node);
                    }
                    if !output.stdout.is_empty() {
                        let text = NodeCodeBlock {
                            info: b"text:stdout".to_vec(),
                            literal: output.stdout,
                            ..code_block.clone()
                        };
                        let new_node =
                            Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
                        let new_node = arena.alloc(new_node);
                        node.insert_after(new_node);
                        let caption = format!("Out [{}]:", execution_count).as_bytes().to_vec();
                        let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
                        let new_node = arena.alloc(new_node);
                        node.insert_after(new_node);
                    }
                }
            }
            _ => (),
        }
    });
    let mut stdout = stdout().lock();
    format_commonmark(root, &ComrakOptions::default(), &mut stdout)?;
    debug!(
        "{}",
        serde_json::to_string(&state.borrow().executions).unwrap()
    );
    Ok(())
}

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
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
        process(br)?;
    }
    Ok(())
}
