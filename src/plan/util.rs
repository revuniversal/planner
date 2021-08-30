use comrak::nodes::{AstNode, NodeValue};

pub fn get_node_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text_bytes = Vec::new();
    collect_text(node, &mut text_bytes);
    String::from_utf8(text_bytes).unwrap().trim().to_string()
}

/// Collects text from `node` and all descendants, then appends it to `output`.
pub fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) => output.extend_from_slice(literal),
        NodeValue::Code(ref literal) => {
            output.push(b'`');
            output.extend_from_slice(literal);
            output.push(b'`');
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
