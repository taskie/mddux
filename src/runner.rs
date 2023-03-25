use std::collections::HashMap;
use std::io::{BufRead, Write};

use anyhow::Result;
use comrak::nodes::AstNode;
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};
use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};

use crate::config::{Config, ExecutionConfig, FormatConfig};
use crate::executor::{Execution, ExecutionEnvironment};
use crate::{formatter, parser};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MdduxState {
    pub environment: ExecutionEnvironment,
    pub contents: Vec<String>,
    pub executions: Vec<Execution>,
    pub code_block_to_execution: HashMap<usize, usize>,
    pub front_matter: Option<Vec<u8>>,
    pub execution_configs: Vec<ExecutionConfig>,
    pub format_configs: Vec<FormatConfig>,
}

impl MdduxState {
    pub(crate) fn from_config(config: &Config) -> MdduxState {
        MdduxState {
            environment: ExecutionEnvironment {
                execution: config.execution.clone().unwrap_or_default(),
                format: config.format.clone().unwrap_or_default(),
                runners: config.runners.clone().unwrap_or_default(),
            },
            contents: vec![],
            executions: vec![],
            code_block_to_execution: HashMap::new(),
            front_matter: None,
            execution_configs: vec![],
            format_configs: vec![],
        }
    }

    pub(crate) fn execute_all(&mut self) -> Result<()> {
        let configs = self.compose_execution_configs();
        for (i, exe) in self.executions.iter_mut().enumerate() {
            exe.execute(&configs[i])?;
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
        let configs = self.compose_execution_configs();
        for (i, exe) in self.executions.iter_mut().enumerate() {
            if exe.output.is_none() {
                exe.execute(&configs[i])?;
            }
        }
        Ok(())
    }

    pub(crate) fn execute_if_needed_with_old_state(
        &mut self,
        old_state: &MdduxState,
    ) -> Result<()> {
        let configs = self.compose_execution_configs();
        let mut old_state_usable = true;
        for (i, zipped) in self
            .executions
            .iter_mut()
            .zip_longest(old_state.executions.iter())
            .enumerate()
        {
            match zipped {
                EitherOrBoth::Both(l, r) => {
                    if old_state_usable && l.input == r.input && r.output.is_some() {
                        l.output = r.output.clone();
                    } else {
                        l.execute(&configs[i])?;
                        old_state_usable = false;
                    }
                }
                EitherOrBoth::Left(l) => {
                    l.execute(&configs[i])?;
                }
                EitherOrBoth::Right(_) => {
                    break;
                }
            };
        }
        Ok(())
    }

    fn compose_execution_configs(&self) -> Vec<ExecutionConfig> {
        self.executions
            .iter()
            .enumerate()
            .map(|(i, exe)| self.compose_execution_config(i, exe))
            .collect()
    }

    pub(crate) fn compose_execution_config(
        &self,
        i: usize,
        execution: &Execution,
    ) -> ExecutionConfig {
        let env = &self.environment;
        let mut execution_config = env.execution.clone();
        if let Some(execution) = env
            .runners
            .get(&execution.type_)
            .and_then(|r| r.exection.as_ref())
        {
            execution_config.apply(execution);
        }
        execution_config.apply(&self.execution_configs[i]);
        execution_config
    }

    pub(crate) fn compose_format_config(&self, i: usize, execution: &Execution) -> FormatConfig {
        let env = &self.environment;
        let mut format_config = env.format.clone();
        if let Some(format) = env
            .runners
            .get(&execution.type_)
            .and_then(|r| r.format.as_ref())
        {
            format_config.apply(format);
        }
        format_config.apply(&self.format_configs[i]);
        format_config
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
