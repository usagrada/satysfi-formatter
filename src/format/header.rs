use tree_sitter::Node;

use crate::token::Token;

use super::{format_ignore, Formatter};

pub(crate) fn format_headers<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output_vec = vec![];
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::whitespace => {
                // println!("whitespace: {:?}", child.range());
                format_ignore(data, &child);
            }
            Token::header => {
                format_header(data, &child);
            }
            _ => {
                unreachable!("headers: {}", child.kind())
            }
        }
        output_vec.push(data.inner.clone());
    }

    let output = {
    //     output_vec.sort_by(|line1, line2| {
    //         // use package open pkgname
    //         // use package pkgname
    //         match (line1.contains(" of "), line2.contains(" of ")) {
    //             (true, true) | (false, false) => {
    //                 match (line1.contains(" open "), line2.contains(" open ")) {
    //                     (true, true) | (false, false) => line1.cmp(line2),
    //                     (true, false) => std::cmp::Ordering::Greater,
    //                     (false, true) => std::cmp::Ordering::Less,
    //                 }
    //             }
    //             (true, false) => std::cmp::Ordering::Greater,
    //             (false, true) => std::cmp::Ordering::Less,
    //         }
    //     });
        output_vec.join("")
    };
    data.inner = output;
}

#[inline]
fn format_header<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(token) => match token.as_str() {
                "use" | "package" | "open" | "package_name" | "of" | "package_path" => {
                    let text = data.node_to_text_trim(&child);
                    output += &text;
                    output += " ";
                }
                "\n" => {
                    output += "\n";
                }
                _ => {
                    unreachable!("header: {}", token)
                }
            },
            _ => {
                eprintln!("header: {:?}", node.kind());
            }
        }
    }
    data.inner = output;
}
