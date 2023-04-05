use tree_sitter::Node;

use crate::token::{Token, LIST_EXPR, LIST_RECORD_INNER, LIST_LITERAL};

use super::{
    format_block_text, format_ignore, format_inline_text, format_math_text, format_record_unit,
    Formatter, format_comment, format_literal,
};

pub(crate) fn format_expr<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::expr_parened => format_expr_parened(data, node),
        Token::expr_constructor => {
            // todo!()
            // data.inner = data.node_to_text(node);
            data.inner = "todo!()".to_string();
        }
        Token::expr_application => {
            format_expr_application(data, node);
        }
        Token::expr_var_path => {
            format_expr_var_path(data, &node);
        }
        Token::expr_lambda => {
            todo!()
        }
        Token::expr_bind => {
            todo!()
        }
        Token::expr_open => {
            todo!()
        }
        Token::expr_match => {
            todo!()
        }
        Token::expr_if => {
            todo!()
        }
        Token::expr_assignment => {
            todo!()
        }
        Token::expr_binary_operation => {
            todo!()
        }
        Token::expr_binary_operator => {
            todo!()
        }
        Token::expr_unary_operation => {
            todo!()
        }
        Token::inline_text => format_inline_text(data, node),
        Token::block_text => format_block_text(data, node),
        Token::math_text => format_math_text(data, node),
        Token::expr_record => {
            format_expr_record(data, node);
        }
        Token::expr_list => {
            format_expr_list(data, node)
        }
        Token::expr_tuple => {
            todo!()
        }
        Token::expr_record_member => {
            todo!()
        }
        Token::expr_command => {
            todo!()
        }
        token if LIST_LITERAL.contains(&token) => {
            format_literal(data, node)
        }
        Token::comment => {
            format_ignore(data, node);
        }
        _ => {
            unreachable!("expr: {}", node.kind());
        }
    }
}

fn format_expr_parened<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    data.depth += 1;
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_EXPR.contains(&token) => {
                format_expr(data, &child);
            }
            _ => {
                unreachable!("expr application: {}", child.kind());
            }
        }
        output += &data.inner;
    }
    data.depth -= 1;
    match output.contains("\n") {
        true => {
            let start_indent = data.indent_start();
            let end_indent = data.indent();
            output = format!("(\n{start_indent}{}\n{end_indent})", output);
        }
        false => {
            output = format!("({})", output);
        }
    }
    data.inner = output;
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
    let mut output = String::new();
    let record_unit_count = {
        let mut binding = node.walk();
        node.children(&mut binding)
            .filter(|child| {
                let token: Token = child.kind().into();
                LIST_EXPR.contains(&token) || LIST_RECORD_INNER.contains(&token)
            })
            .count()
    };

    let mut new_line_flag = false;
    if record_unit_count >= 2 {
        data.depth += 1;
        output += data.indent().as_str();
        new_line_flag = true;
    }

    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_EXPR.contains(&token) => {
                format_expr(data, &child);
            }
            token if LIST_RECORD_INNER.contains(&token) => {
                format_expr_record_inner(data, &child);
                if new_line_flag {
                    output += "\n";
                    output += data.indent().as_str();
                }
            }
            Token::other(token) if token.as_str() == "," => {
                data.inner = token;
            }
            Token::other(token) if token == "(|" || token == "|)" => {
                format_ignore(data, &child)
            }
            Token::other(token) => {
                data.inner = token;
            }
            Token::comment => {
                format_comment(data, &child);
            }
            _ => {
                unreachable!("expr record: {}", child.kind());
            }
        }
        output += &data.inner;
    }
    if record_unit_count >= 2 {
        data.depth -= 1;
    }
    match new_line_flag {
        true => {
            let start_indent = data.indent_start();
            let end_indent = data.indent();
            data.inner = format!("(|\n{start_indent}{}\n{end_indent}|)", output.trim());
        }
        false => {
            data.inner = format!("(|{}|)", output);
        }
    }
}

#[inline]
fn format_expr_record_inner<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::record_unit => format_record_unit(data, node),
        _ => {
            unreachable!("expr recordinner: {}", node.kind());
        }
    }
}


fn format_expr_list<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();

    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_EXPR.contains(&token) => {
                format_expr(data, &child);
            }
            Token::other(token) if token.as_str() == "," => {
                data.inner = token;
            }
            Token::other(token) => {
                data.inner = token;
            }
            Token::comment => {
                format_comment(data, &child);
            }
            _ => {
                unreachable!("expr record: {}", child.kind());
            }
        }
        output += &data.inner;
    }
    data.inner = output
}
