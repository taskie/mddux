use std::collections::HashMap;
use std::io::{BufRead, Write};

use anyhow::Result;
use comrak::nodes::AstNode;
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};
use serde::{Deserialize, Serialize};

use crate::config::{Config, FormatConfig};
use crate::executor::{Execution, ExecutionEnvironment};
use crate::{formatter, parser};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MdduxState {
    pub environment: ExecutionEnvironment,
    pub contents: Vec<String>,
    pub executions: Vec<Execution>,
    pub code_block_to_execution: HashMap<usize, usize>,
    pub front_matter: Option<Vec<u8>>,
    pub format_configs: Vec<FormatConfig>,
}

impl MdduxState {
    pub(crate) fn from_config(config: &Config) -> MdduxState {
        MdduxState {
            environment: ExecutionEnvironment {
                runners: config.runners.clone(),
            },
            contents: vec![],
            executions: vec![],
            code_block_to_execution: HashMap::new(),
            front_matter: None,
            format_configs: vec![],
        }
    }

    pub(crate) fn execute_all(&mut self) -> Result<()> {
        for exe in self.executions.iter_mut() {
            exe.execute()?;
        }
        Ok(())
    }
}

pub(crate) fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F)
where
    F: FnMut(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub(crate) fn run<R: BufRead, W: Write>(mut r: R, w: W, config: &Config) -> Result<()> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let opts = make_comrak_options();
    let mut state = load(&buf, &opts, config)?;
    state.execute_all()?;
    dump(w, &state, &buf, &opts, config)?;
    Ok(())
}

fn load(buf: &str, opts: &ComrakOptions, config: &Config) -> Result<MdduxState> {
    let mut state = MdduxState::from_config(config);
    let arena = Arena::new();
    let root = parse_document(&arena, buf, opts);
    parser::parse(&mut state, config, &arena, root);
    Ok(state)
}

fn dump<W: Write>(
    mut w: W,
    state: &MdduxState,
    buf: &str,
    opts: &ComrakOptions,
    config: &Config,
) -> Result<()> {
    let arena = Arena::new();
    let root = parse_document(&arena, buf, opts);
    formatter::format(state, config, &arena, root);
    format_commonmark(root, opts, &mut w)?;
    Ok(())
}

fn make_comrak_options() -> ComrakOptions {
    let mut opts = ComrakOptions::default();
    opts.extension.front_matter_delimiter = Some("---".to_owned());
    opts
}
