use tree_sitter::Node;

use crate::token::Token;

use super::{Formatter, format_ignore};

pub(crate) fn format_config<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::config_record => {
                format_config_record(data, &child);
            }
            Token::other(s) => {
                // s matches #[, config, ]
                output += &s;
            }
            Token::whitespace => {
                data.inner = " ".to_string();
            }
            _ => {
                eprintln!("{:?}", child.kind());
                unreachable!()
            }
        }
        output += &data.inner;
    }
    data.inner = output;
    data.inner += "\n";
}

fn format_config_record<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(s) => {
                output += &s;
            }
            _ => {
                // todo!()
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}
