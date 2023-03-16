use std::io::{BufRead, Write};

use anyhow::Result;
use comrak::nodes::AstNode;
use comrak::{format_commonmark, parse_document, Arena, ComrakOptions};

use crate::config::Config;
use crate::executor::ExecutionState;

mod formatter;
mod parser;

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F)
where
    F: FnMut(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub(crate) fn process<R: BufRead, W: Write>(mut r: R, w: W, config: &Config) -> Result<()> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let opts = make_comrak_options();
    let mut state = ExecutionState::from_config(config);
    load(&buf, &opts, &mut state, config)?;
    state.execute_all()?;
    dump(w, &buf, &opts, &mut state, config)?;
    Ok(())
}

fn load(
    buf: &str,
    opts: &ComrakOptions,
    state: &mut ExecutionState,
    config: &Config,
) -> Result<()> {
    let arena = Arena::new();
    let root = parse_document(&arena, buf, opts);
    parser::parse(state, config, &arena, root);
    Ok(())
}

fn dump<W: Write>(
    mut w: W,
    buf: &str,
    opts: &ComrakOptions,
    state: &mut ExecutionState,
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
