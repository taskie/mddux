use std::cell::RefCell;
use std::collections::HashMap;

use comrak::arena_tree::Node;
use comrak::nodes::{Ast, NodeCodeBlock, NodeValue};
use comrak::Arena;
use strfmt::strfmt;

use crate::config::{Config, ExecutionConfig, FormatConfig};
use crate::executor::Execution;
use crate::runner::{iter_nodes, MdduxState};

pub(crate) fn format<'a>(
    state: &'a MdduxState,
    conf: &'a Config,
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
                let content = &state.contents[*execution_index];
                let execution = &state.executions[*execution_index];
                let execution_config = state.compose_execution_config(*execution_index, execution);
                let format_config = state.compose_format_config(*execution_index, execution);
                let options = FormatCmarkOptions {
                    conf,
                    execution_config: &execution_config,
                    format_config: &format_config,
                };
                format_cmark(code_block, content, arena, node, execution, &options);
                code_block_index += 1;
            }
            _ => (),
        }
    });
}

struct FormatCmarkOptions<'a, 'b> {
    conf: &'a Config,
    execution_config: &'b ExecutionConfig,
    format_config: &'b FormatConfig,
}

fn format_cmark<'a>(
    code_block: &mut NodeCodeBlock,
    content: &'a str,
    arena: &'a Arena<Node<'a, RefCell<Ast>>>,
    node: &'a Node<'a, RefCell<Ast>>,
    execution: &Execution,
    options: &FormatCmarkOptions<'a, '_>,
) {
    let &FormatCmarkOptions {
        conf,
        execution_config,
        format_config,
    } = options;
    if execution_config.skipped.unwrap_or_default() {
        code_block.literal = content.as_bytes().to_vec();
        return;
    }
    if !format_config.stdin_hidden.unwrap_or_default() {
        code_block.literal = content.as_bytes().to_vec();
        if conf.caption.unwrap_or_default()
            && !format_config.stdin_caption_hidden.unwrap_or_default()
        {
            let caption = make_caption(&format_config.stdin_caption_format, execution, "In");
            let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(
                caption.as_bytes().to_vec(),
            ))));
            let new_node = arena.alloc(new_node);
            node.insert_before(new_node);
        }
    }
    if let Some(output) = execution.output.as_ref() {
        if !format_config.stderr_hidden.unwrap_or_default() && !output.stderr.as_ref().is_empty() {
            let text = NodeCodeBlock {
                info: format_config
                    .stderr_info
                    .clone()
                    .unwrap_or_else(|| "text".to_owned())
                    .as_bytes()
                    .to_vec(),
                literal: output.stderr.as_ref().to_vec(),
                ..code_block.clone()
            };
            let new_node = Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
            let new_node = arena.alloc(new_node);
            node.insert_after(new_node);
            if conf.caption.unwrap_or_default()
                && !format_config.stderr_caption_hidden.unwrap_or_default()
            {
                let caption = make_caption(&format_config.stderr_caption_format, execution, "Err");
                let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(
                    caption.as_bytes().to_vec(),
                ))));
                let new_node = arena.alloc(new_node);
                node.insert_after(new_node);
            }
        }
        if !format_config.stdout_hidden.unwrap_or_default() && !output.stdout.as_ref().is_empty() {
            let text = NodeCodeBlock {
                info: format_config
                    .stdout_info
                    .clone()
                    .unwrap_or_else(|| "text".to_owned())
                    .as_bytes()
                    .to_vec(),
                literal: output.stdout.as_ref().to_vec(),
                ..code_block.clone()
            };
            let new_node = Node::new(RefCell::new(Ast::new(NodeValue::CodeBlock(text))));
            let new_node = arena.alloc(new_node);
            node.insert_after(new_node);
            if conf.caption.unwrap_or_default()
                && !format_config.stdout_caption_hidden.unwrap_or_default()
            {
                let caption = make_caption(&format_config.stdout_caption_format, execution, "Out");
                let new_node = Node::new(RefCell::new(Ast::new(NodeValue::Text(
                    caption.as_bytes().to_vec(),
                ))));
                let new_node = arena.alloc(new_node);
                node.insert_after(new_node);
            }
        }
    }
    if format_config.stdin_hidden.unwrap_or_default() {
        node.detach();
    }
}

fn make_caption(
    caption_format: &Option<String>,
    execution: &Execution,
    code_block_type: &str,
) -> String {
    if let Some(caption) = caption_format {
        let mut vars = HashMap::new();
        let execution_count = execution.execution_count.to_string();
        vars.insert("execution_count".to_owned(), execution_count.as_str());
        vars.insert("type".to_owned(), execution.type_.as_str());
        vars.insert("code_block_type".to_owned(), code_block_type);
        strfmt(caption, &vars).unwrap()
    } else {
        format!("{} [{}]:", code_block_type, execution.execution_count)
    }
}
