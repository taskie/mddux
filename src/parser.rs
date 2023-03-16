use std::cell::RefCell;

use anyhow::{Context, Result};
use bstr::ByteVec;
use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeCodeBlock, NodeValue};
use comrak::Arena;
use linked_hash_map::LinkedHashMap;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::config::{Config, FormatConfig, RunnerConfig};
use crate::executor::{Execution, ExecutionCommand, ExecutionInput};
use crate::runner::{iter_nodes, MdduxState};

pub(crate) fn parse<'a>(
    state: &mut MdduxState,
    conf: &Config,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    root: &'a Node<'a, RefCell<Ast>>,
) {
    let mut code_block_index = 0;
    iter_nodes(root, &mut |node| {
        let mut ast = node.data.borrow_mut();
        match ast.value {
            NodeValue::FrontMatter(ref mut bs) => {
                let new_bs = parse_front_matter(state, bs);
                state.front_matter = Some(new_bs);
            }
            NodeValue::CodeBlock(ref mut code_block) => {
                parse_code_block(state, conf, arena, node, code_block_index, code_block);
                code_block_index += 1;
            }
            _ => (),
        }
    });
}

fn parse_code_block<'a>(
    state: &mut MdduxState,
    _conf: &Config,
    _arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    _node: &'a Node<'a, RefCell<Ast>>,
    code_block_index: usize,
    code_block: &mut NodeCodeBlock,
) {
    let info = String::from_utf8(code_block.info.clone()).unwrap();
    let info_vec: Vec<_> = info.splitn(2, ':').collect();
    let code_type = info_vec[0];
    // let file_name = info_vec.get(1);
    let runner = state.environment.runners.get(code_type).cloned();
    if let Some(runner) = runner {
        let stdin = code_block.literal.as_slice();
        let stdin_str = String::from_utf8_lossy(stdin).into_owned();
        let execution_index = state.executions.len();
        let execution_count = execution_index as i32 + 1;
        let input = make_execution_input(&runner, stdin);
        let (content, format_config) = parse_content(runner, stdin_str);
        let execution = Execution {
            execution_count,
            type_: code_type.to_owned(),
            input,
            output: None,
        };
        state.contents.push(content);
        state.executions.push(execution);
        state.format_configs.push(format_config);
        state
            .code_block_to_execution
            .insert(code_block_index, execution_index);
    }
}

fn parse_content(runner: RunnerConfig, stdin_str: String) -> (String, FormatConfig) {
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
    for raw_line in stdin_str.lines() {
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

fn parse_front_matter(state: &mut MdduxState, bs: &[u8]) -> Vec<u8> {
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

fn make_execution_input(runner: &RunnerConfig, stdin: &[u8]) -> ExecutionInput {
    let program = &runner.command[0];
    let args = runner.command[1..].to_vec();
    let command = ExecutionCommand {
        program: program.to_owned(),
        args,
    };

    ExecutionInput {
        command,
        stdin: stdin.to_vec().into(),
    }
}
