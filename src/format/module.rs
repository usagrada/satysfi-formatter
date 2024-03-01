use tree_sitter::Node;

use super::Formatter;

pub(crate) fn format_module_path<'a>(data: &mut Formatter<'a>, node: &Node) {
    // todo!("format_module_path: {}", data.node_to_text(node));
    data.inner = data.node_to_text(node);
}
