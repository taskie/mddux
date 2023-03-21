use std::collections::HashMap;
use std::io::{BufRead, Write};

use anyhow::Result;
use comrak::nodes::AstNode;
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};
use itertools::{EitherOrBoth, Itertools};
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

    pub(crate) fn execute_if_needed(&mut self, old_state: &Option<MdduxState>) -> Result<()> {
        if let Some(old_state) = old_state {
            self.execute_if_needed_with_old_state(old_state)
        } else {
            self.execute_if_needed_fast_path()
        }
    }

    pub(crate) fn execute_if_needed_fast_path(&mut self) -> Result<()> {
        for exe in self.executions.iter_mut() {
            if exe.output.is_none() {
                exe.execute()?;
            }
        }
        Ok(())
    }

    pub(crate) fn execute_if_needed_with_old_state(
        &mut self,
        old_state: &MdduxState,
    ) -> Result<()> {
        let mut old_state_usable = true;
        for zipped in self
            .executions
            .iter_mut()
            .zip_longest(old_state.executions.iter())
        {
            match zipped {
                EitherOrBoth::Both(l, r) => {
                    if old_state_usable && l.input == r.input && r.output.is_some() {
                        l.output = r.output.clone();
                    } else {
                        l.execute()?;
                        old_state_usable = false;
                    }
                }
                EitherOrBoth::Left(l) => {
                    l.execute()?;
                }
                EitherOrBoth::Right(_) => {
                    break;
                }
            };
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

pub(crate) fn load(buf: &str, opts: &ComrakOptions, config: &Config) -> Result<MdduxState> {
    let mut state = MdduxState::from_config(config);
    let arena = Arena::new();
    let root = parse_document(&arena, buf, opts);
    parser::parse(&mut state, config, &arena, root);
    Ok(state)
}

pub(crate) fn dump<W: Write>(
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

pub(crate) fn make_comrak_options() -> ComrakOptions {
    let mut opts = ComrakOptions::default();
    opts.extension.front_matter_delimiter = Some("---".to_owned());
    opts
}
