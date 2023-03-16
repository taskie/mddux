use std::cell::RefCell;
use std::io::{stdout, BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;

use anyhow::{Context, Result};
use bstr::{BString, ByteVec};
use comrak::arena_tree::Node;
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};
use linked_hash_map::LinkedHashMap;
use log::debug;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::config::{Config, ExecutionConfig, FormatConfig, RunnerConfig};
use crate::executor::{
    Execution, ExecutionCommand, ExecutionEnvironment, ExecutionResult, ExecutionState,
};

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub(crate) fn process<R: BufRead>(mut r: R, conf: &Config) -> Result<()> {
    let arena = Arena::new();
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let mut opts = ComrakOptions::default();
    opts.extension.front_matter_delimiter = Some("---".to_owned());
    let root = parse_document(&arena, &buf, &opts);
    let state = Arc::new(RefCell::new(ExecutionState {
        environment: ExecutionEnvironment {
            runners: conf.runners.clone(),
        },
        executions: vec![],
        execution_count: 0,
    }));
    iter_nodes(root, &|node| {
        let mut state = state.borrow_mut();
        match node.data.borrow_mut().value {
            NodeValue::FrontMatter(ref mut bs) => {
                let mut new_bs = parse_front_matter(&mut state, bs);
                bs.clear();
                bs.append(&mut new_bs);
            }
            NodeValue::CodeBlock(ref mut code_block) => {
                parse_code_block(&mut state, conf, &arena, node, code_block);
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

fn parse_code_block<'a>(
    mut state: &mut ExecutionState,
    conf: &Config,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    node: &'a Node<'a, RefCell<Ast>>,
    code_block: &mut NodeCodeBlock,
) {
    let info = String::from_utf8(code_block.info.clone()).unwrap();
    let info_vec: Vec<_> = info.splitn(2, ':').collect();
    let code_type = info_vec[0];
    // let file_name = info_vec.get(1);
    let runner = state.environment.runners.get(code_type).cloned();
    if let Some(runner) = runner {
        let raw_content = String::from_utf8_lossy(&code_block.literal).into_owned();
        state.execution_count += 1;
        let execution_count = state.execution_count;
        let (command, result) = execute_command(&runner, code_block.literal.as_slice());
        let (content, format_config) = parse_content(runner, raw_content);
        let execution = Execution {
            execution_count,
            type_: code_type.to_owned(),
            content: content.clone(),
            config: ExecutionConfig {},
            command,
            result,
        };
        state.executions.push(execution.clone());
        format_cmark(
            code_block,
            content,
            conf,
            &format_config,
            arena,
            node,
            &execution,
        );
    }
}

fn format_cmark<'a>(
    code_block: &mut NodeCodeBlock,
    content: String,
    conf: &Config,
    format_config: &FormatConfig,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    node: &'a Node<'a, RefCell<Ast>>,
    execution: &Execution,
) {
    code_block.literal = content.as_bytes().to_vec();
    let execution_count = execution.execution_count;
    if conf.caption.unwrap_or_default() {
        let caption = format!("In [{}]:", execution_count).as_bytes().to_vec();
        let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
        let new_node = arena.alloc(new_node);
        node.insert_before(new_node);
    }
    let result = execution.result.as_ref().unwrap();
    if !result.stderr.is_empty() {
        let text = NodeCodeBlock {
            info: format_config
                .stderr_info
                .clone()
                .unwrap_or_else(|| "text".to_owned())
                .as_bytes()
                .to_vec(),
            literal: result.stderr.to_vec(),
            ..code_block.clone()
        };
        let new_node = Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
        let new_node = arena.alloc(new_node);
        node.insert_after(new_node);
        if conf.caption.unwrap_or_default() {
            let caption = format!("Err [{}]:", execution_count).as_bytes().to_vec();
            let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
            let new_node = arena.alloc(new_node);
            node.insert_after(new_node);
        }
    }
    if !result.stdout.is_empty() {
        let text = NodeCodeBlock {
            info: format_config
                .stdout_info
                .clone()
                .unwrap_or_else(|| "text".to_owned())
                .as_bytes()
                .to_vec(),
            literal: result.stdout.to_vec(),
            ..code_block.clone()
        };
        let new_node = Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
        let new_node = arena.alloc(new_node);
        node.insert_after(new_node);
        if conf.caption.unwrap_or_default() {
            let caption = format!("Out [{}]:", execution_count).as_bytes().to_vec();
            let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(caption))));
            let new_node = arena.alloc(new_node);
            node.insert_after(new_node);
        }
    }
}

fn parse_content(runner: RunnerConfig, raw_content: String) -> (String, FormatConfig) {
    let special_comment_prefix = runner
        .special_comment_prefix
        .clone()
        .unwrap_or_else(|| "#".to_owned());
    let special_comment_suffix = runner
        .special_comment_suffix
        .clone()
        .unwrap_or_else(|| "".to_owned());
    let mut content = String::new();
    let mut format_config = FormatConfig::default();
    format_config.apply_runner_config(&runner);
    for raw_line in raw_content.lines() {
        let line = raw_line.trim();
        let Some(line) = line.strip_prefix(&special_comment_prefix) else {
            content.push_str(raw_line);
            content.push('\n');
            continue;
        };
        let Some(line) = line.strip_suffix(&special_comment_suffix) else {
            content.push_str(raw_line);
            content.push('\n');
            continue;
        };
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() < 2 {
            content.push_str(raw_line);
            content.push('\n');
            continue;
        }
        let key = parts[0].trim().to_ascii_lowercase();
        let value = parts[1].trim();
        match key.as_str() {
            "mddux-stdout-info" => {
                format_config.stdout_info = Some(value.to_owned());
            }
            "mddux-stderr-info" => {
                format_config.stderr_info = Some(value.to_owned());
            }
            _ => (),
        }
    }
    (content, format_config)
}

fn execute_command(
    runner: &RunnerConfig,
    input: &[u8],
) -> (ExecutionCommand, Option<ExecutionResult>) {
    let program = &runner.command[0];
    let args = runner.command[1..].to_vec();
    let mut child = Command::new(program)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin.write_all(input).unwrap();
    };
    let output = child.wait_with_output().unwrap();
    let command = ExecutionCommand {
        program: program.to_owned(),
        args,
    };
    let result = ExecutionResult {
        status_code: output.status.code(),
        stdout: BString::new(output.stdout.clone()),
        stderr: BString::new(output.stderr),
    };
    (command, Some(result))
}

fn parse_front_matter(state: &mut ExecutionState, bs: &[u8]) -> Vec<u8> {
    let s = String::from_utf8(bs.to_owned()).unwrap();
    let docs = YamlLoader::load_from_str(&s).unwrap();
    if docs.is_empty() {
        return bs.to_owned();
    }
    let mut bs = bs.to_owned();
    let doc = &docs[0];
    let hash = doc.as_hash();
    if let Some(hash) = hash {
        let mut new_hash = LinkedHashMap::new();
        for (k, v) in hash {
            if k.as_str() == Some("mddux") {
                continue;
            }
            new_hash.insert(k.to_owned(), v.to_owned());
        }
        bs.clear();
        if !new_hash.is_empty() {
            let new_doc = Yaml::Hash(new_hash);
            let mut new_docs_str = String::new();
            YamlEmitter::new(&mut new_docs_str).dump(&new_doc).unwrap();
            bs.push_str(new_docs_str);
            bs.push_str("\n---\n\n");
        }
    }
    let mddux = &doc["mddux"];
    let mut mddux_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut mddux_str);
        let Ok(_) = emitter.dump(mddux) else { return bs.clone(); };
    }
    let Ok(mddux): Result<Config> = serde_yaml::from_str(&mddux_str).context("can't parse front matter") else { return bs.clone(); };
    for (k, v) in mddux.runners {
        state.environment.runners.insert(k, v);
    }
    bs
}
