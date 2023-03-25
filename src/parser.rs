use std::cell::RefCell;

use anyhow::{Context, Result};
use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeCodeBlock, NodeValue};
use comrak::Arena;
use linked_hash_map::LinkedHashMap;
use log::warn;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::config::{Config, ExecutionConfig, FormatConfig, RunnerConfig};
use crate::executor::{Execution, ExecutionCommand, ExecutionEnvironment, ExecutionInput};
use crate::runner::{iter_nodes, MdduxState};
use crate::util::{calc_fast_digest, Content};

pub(crate) fn parse<'a>(
    state: &mut MdduxState,
    conf: &Config,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    root: &'a Node<'a, RefCell<Ast>>,
) {
    let mut code_block_index = 0;
    let mut execution_count = 1;
    iter_nodes(root, &mut |node| {
        let mut ast = node.data.borrow_mut();
        match ast.value {
            NodeValue::FrontMatter(ref mut bs) => {
                let new_bs = parse_front_matter(state, bs);
                state.front_matter = Some(new_bs);
            }
            NodeValue::CodeBlock(ref mut code_block) => {
                parse_code_block(
                    state,
                    conf,
                    arena,
                    node,
                    code_block_index,
                    code_block,
                    &mut execution_count,
                );
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
    execution_count: &mut i32,
) {
    let info = String::from_utf8(code_block.info.clone()).unwrap();
    let info_vec: Vec<_> = info.splitn(2, ':').collect();
    let code_type = info_vec[0];
    // let file_name = info_vec.get(1);
    let runner = state.environment.runners.get(code_type).cloned();
    if let Some(runner) = runner {
        let raw_code = code_block.literal.as_slice();
        let raw_code_str = String::from_utf8_lossy(raw_code).into_owned();
        let execution_index = state.executions.len();
        let (content, execution_config, format_config) =
            parse_content(&state.environment, &runner, raw_code_str);
        let skipped = execution_config.skipped.unwrap_or_default();
        let stdin = content.as_bytes();
        let input = make_execution_input(&runner, stdin).unwrap();
        let execution = Execution {
            execution_count: *execution_count,
            type_: code_type.to_owned(),
            input,
            output: None,
        };
        state.contents.push(content);
        state.executions.push(execution);
        state.execution_configs.push(execution_config);
        state.format_configs.push(format_config);
        state
            .code_block_to_execution
            .insert(code_block_index, execution_index);
        if !skipped {
            *execution_count += 1;
        }
    }
}

fn parse_content(
    _environment: &ExecutionEnvironment,
    runner: &RunnerConfig,
    stdin_str: String,
) -> (String, ExecutionConfig, FormatConfig) {
    let special_comment_prefix = runner
        .special_comment_prefix
        .clone()
        .unwrap_or_else(|| "#".to_owned());
    let special_comment_suffix = runner
        .special_comment_suffix
        .clone()
        .unwrap_or_else(|| "".to_owned());
    let mut content = String::new();
    let mut execution_config = ExecutionConfig::default();
    let mut format_config = FormatConfig::default();
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
        let prefix = "mddux-";
        if let Some(key) = parts[0].trim().to_ascii_lowercase().strip_prefix(prefix) {
            let value = parts[1].trim();
            let execution_ok = execution_config.insert_with_str(key, value);
            if !execution_ok {
                let format_ok = format_config.insert_with_str(key, value);
                if !format_ok {
                    warn!("no such config key: {}{}", prefix, key);
                }
            }
        }
    }
    (content, execution_config, format_config)
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
            bs.extend(new_docs_str.as_bytes());
            bs.extend(b"\n---\n\n");
        }
    }
    let mddux = &doc["mddux"];
    let mut mddux_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut mddux_str);
        let Ok(_) = emitter.dump(mddux) else { return bs.clone(); };
    }
    let Ok(mddux): Result<Config> = serde_yaml::from_str(&mddux_str).context("can't parse front matter") else { return bs.clone(); };
    if let Some(runners) = mddux.runners {
        for (k, v) in runners {
            state.environment.runners.insert(k, v);
        }
    }
    if let Some(execution) = mddux.execution {
        state.environment.execution.apply(&execution)
    }
    if let Some(format) = mddux.format {
        state.environment.format.apply(&format)
    }
    bs
}

fn make_execution_input(runner: &RunnerConfig, stdin: &[u8]) -> Result<ExecutionInput> {
    let program = &runner.command[0];
    let args = runner.command[1..].to_vec();
    let command = ExecutionCommand {
        program: program.to_owned(),
        args,
    };
    let stdin_hash = calc_fast_digest(stdin)?;
    Ok(ExecutionInput {
        command,
        stdin_hash,
        stdin: Content::Binary(stdin.to_vec()),
    })
}
