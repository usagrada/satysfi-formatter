use tree_sitter::Node;

use crate::token::Token;

use super::{Formatter, format_ignore, format_literal};

pub(crate) fn format_headers<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output_vec = vec![];
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::whitespace => {
                // println!("whitespace: {:?}", child.range());
                format_ignore(data, &child);
            }
            _ => {
                let is_inner = format_header_inner(data, &child);
                if !is_inner {
                    unreachable!("headers: {}", child.kind())
                }
            }
        }
        output_vec.push(data.inner.clone());
    }
    eprintln!("output_vec: {:?}", output_vec);

    let output = {
        output_vec.sort_by(|line1, line2|{
            // use package open pkgname
            // use package pkgname
            match (line1.contains(" of "), line2.contains(" of ")) {
                (true, true) | (false, false) => {
                    match (line1.contains(" open "), line2.contains(" open ")) {
                        (true, true) | (false, false) => {
                            line1.cmp(line2)
                        }
                        (true, false) => {
                            std::cmp::Ordering::Greater
                        }
                        (false, true) => {
                            std::cmp::Ordering::Less
                        }
                    }
                }
                (true, false) => {
                    std::cmp::Ordering::Greater
                }
                (false, true) => {
                    std::cmp::Ordering::Less
                }
            }
        });
        output_vec.join("")
    };
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
            return false;
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
