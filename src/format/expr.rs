use tree_sitter::Node;

use crate::token::{Token, LIST_EXPR};

use super::{Formatter, format_ignore};

pub(crate) fn format_expr<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::expr_application => {
            format_expr_application(data, node);
        }
        Token::expr_var_path => {
            // format_expr_application(data, &child);
            format_expr_var_path(data, &node);
        }
        Token::expr_record => {
            format_expr_record(data, node);
        }
        _ => {
            unreachable!("expr: {}", node.kind());
        }
    }
    data.inner = "todo: expr".to_string();
}

fn format_expr_application<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_EXPR.contains(&token) => {
                format_expr(data, &child);
            }
            // Token::expr_application => {
            //     format_expr_application(data, &child);
            // }
            // Token::identifier => {
            //     format_identifier(data, &child);
            // }
            // token if LIST_UNARY.contains(&token) => {
            //     format_unary(data, &child);
            // }
            _ => {
                unreachable!("expr application: {}", child.kind());
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_expr_var_path<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::module_name => {
                data.inner = data.node_to_text(&child);
            }
            Token::var_name => {
                data.inner = data.node_to_text(&child);
            }
            Token::other(token) if token == "." => {
                data.inner = token;
            }
            _ => {
                unreachable!("expr var path: {}", child.kind());
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_expr_record<'a>(data: &mut Formatter<'a>, node: &Node) {
    todo!()
}
