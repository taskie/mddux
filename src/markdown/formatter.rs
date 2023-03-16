use std::cell::RefCell;

use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeCodeBlock, NodeValue};
use comrak::Arena;

use crate::config::{Config, FormatConfig};
use crate::executor::{Execution, ExecutionState};

use super::iter_nodes;

pub(crate) fn format<'a>(
    state: &mut ExecutionState,
    conf: &Config,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    root: &'a Node<'a, RefCell<Ast>>,
) {
    let mut code_block_index = 0;
    iter_nodes(root, &mut |node| {
        let mut ast = node.data.borrow_mut();
        match ast.value {
            NodeValue::FrontMatter(ref mut bs) => {
                let Some(front_matter) = state.front_matter.as_ref() else { return; };
                bs.clear();
                bs.extend_from_slice(front_matter.as_slice());
            }
            NodeValue::CodeBlock(ref mut code_block) => {
                let execution_index = state.code_block_to_execution.get(&code_block_index);
                let Some(execution_index) = execution_index else {
                    code_block_index += 1;
                    return;
                };
                let execution = &state.executions[*execution_index];
                let format_config = &state.format_configs[*execution_index];
                format_cmark(
                    code_block,
                    &execution.content,
                    conf,
                    format_config,
                    arena,
                    node,
                    execution,
                );
                code_block_index += 1;
            }
            _ => (),
        }
    });
}

fn format_cmark<'a>(
    code_block: &mut NodeCodeBlock,
    content: &String,
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
    let output = execution.output.as_ref().unwrap();
    if !output.stderr.is_empty() {
        let text = NodeCodeBlock {
            info: format_config
                .stderr_info
                .clone()
                .unwrap_or_else(|| "text".to_owned())
                .as_bytes()
                .to_vec(),
            literal: output.stderr.to_vec(),
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
    if !output.stdout.is_empty() {
        let text = NodeCodeBlock {
            info: format_config
                .stdout_info
                .clone()
                .unwrap_or_else(|| "text".to_owned())
                .as_bytes()
                .to_vec(),
            literal: output.stdout.to_vec(),
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
