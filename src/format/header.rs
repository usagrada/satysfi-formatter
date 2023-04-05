use tree_sitter::Node;

use crate::token::Token;

use super::{Formatter, format_ignore, format_literal};

pub(crate) fn format_headers<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::whitespace => {
                // println!("whitespace: {:?}", child.range());
                format_ignore(data, &child);
            }
            token => {
                let is_inner = format_header_inner(data, &child);
                if !is_inner {
                    unreachable!("{}", token)
                }
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

#[inline]
fn format_header_inner<'a>(data: &mut Formatter<'a>, node: &Node) -> bool {
    match node.kind().into() {
        Token::other(token) => match token.as_str() {
            "pkgname" => {
                let text = data.node_to_text_trim(&node);
                data.inner = text;
            }
            "\n" => {
                data.inner = "\n".to_string();
            }
            _ => {
                unreachable!("header inner: {}", token)
            }
        },
        Token::header_use_package => format_header_use_package(data, node),
        // Token::header_stage => format_header_stage(data, node),
        _ => {
            println!("header: {:?}", node.kind());
            unreachable!()
        }
    }
    return true;
}

fn format_header_use_package<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::whitespace => {
                format_ignore(data, &child);
            }
            Token::other(token) => {
                data.inner = token;
            }
            Token::pkgname => {
                let text = data.node_to_text_trim(&child);
                data.inner = text;
            }
            Token::literal_string => {
                format_literal(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
        if output != "" {
            output += " ";
        }
        output += &data.inner;
    }
    data.inner = output;
}
