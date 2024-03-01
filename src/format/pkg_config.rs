use tree_sitter::Node;

use crate::{format::expr::format_expr, token::Token};

use super::{format_ignore, Formatter};

pub(crate) fn format_config<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::config_record => {
                format_config_record(data, &child);
            }
            Token::other(s) => {
                // s matches #[, config, ]
                data.inner = s;
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
            Token::other(s) => match s.as_str() {
                "|)" => {
                    output += "\n";
                    data.inner = s;
                }
                "]" => {
                    data.inner = s;
                }
                _ => {
                    data.inner = s;
                }
            },
            Token::config_registries => {
                format_config_registries(data, &child);
                output += "\n";
                output += data.indent_start().as_str();
            }
            Token::config_dependencies => {
                format_config_dependencies(data, &child);
                output += "\n";
                output += data.indent_start().as_str();
            }
            _ => {
                // todo!()
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_config_registries<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    data.depth += 1;

    let mut output_vec = vec![];
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(s) => {
                // data.inner = s;
                output_vec.push(s);
            }
            Token::expr_list => {
                format_expr(data, &child);
                output_vec.push(data.inner.clone());
            }
            _ => {
                todo!()
            }
        }
        output += &data.inner;
    }
    println!("registries: {:?}", output_vec);

    data.depth -= 1;
    data.inner = output;
}

fn format_config_dependencies<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    data.depth += 1;

    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(s) => {
                data.inner = s;
            }
            Token::expr_list => {
                format_expr(data, &child);
            }
            _ => {
                todo!()
            }
        }
        output += &data.inner;
    }

    data.depth -= 1;
    data.inner = output;
}
