use crate::{token::{Token, *}, OptionData};
mod helper;
use tree_sitter::{Node, Tree};

/// node を与えたときにテキストを返すための関数
fn node_to_text(node: &Node, text: &str) -> String {
    let range = node.byte_range();
    text[range.start..range.end].trim().to_string()
}

#[derive(Debug, Clone)]
struct Formatter<'a> {
    // 入力
    input: &'a str,
    // 出力
    output: String,
    // 内部のフォーマット後のテキスト
    inner: String,
    // 現在のインデントの深さ
    depth: usize,
    config: OptionData,
    tree: &'a Tree,
}

impl<'a> Formatter<'a> {
    fn indent(&self) -> String {
        use self::helper::indent_space;
        let result = indent_space(self.depth * self.config.indent_space);
        result
    }
}

/// format するための関数
/// inner にフォーマットされたテキストを入れていく
pub fn format<'a>(input: &'a str, tree: &Tree, config: OptionData) -> String {
    let root_node = tree.root_node();
    let output = String::new();
    let inner = String::new();
    let depth = 0;
    assert_eq!(root_node.kind(), "source_file");
    let mut data = Formatter {
        input,
        output,
        inner,
        depth,
        config,
        tree,
    };
    format_source_file(&mut data, &root_node);
    data.output
}

fn format_source_file<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::program_saty => format_program_saty(data, &child),
            Token::program_satyh => format_program_satyh(data, &child),
            Token::whitespace => format_whitespace(data, &child),
            _ => unreachable!(),
        };
        output += &data.inner;
    }
    data.output = output.trim().to_string();
}

fn format_comment<'a>(data: &mut Formatter<'a>, node: &Node) {
    let text = node_to_text(node, data.input) + "\n";
    data.inner = text;
}

fn format_program_saty<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::headers => {
                format_headers(data, &child);
                data.inner += (data.indent() + "\n").as_str();
            }
            Token::preamble => {
                format_preamble(data, &child);
            }
            token if LIST_EXPR.contains(&token) || LIST_UNARY.contains(&token) => {
                format_expr(data, &child);
            }
            Token::application => {
                format_expr(data, &child);
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
        output += &data.inner;
    };
    data.inner = output;
}

fn format_program_satyh<'a>(data: &mut Formatter<'a>, node: &Node) {
    println!("format file program_satyh");
    todo!()
}

fn format_headers<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::header_import | Token::header_require => {
                format_header_inner(data, &child);
            }
            Token::whitespace => {
                // println!("whitespace: {:?}", child.range());
                format_whitespace(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

#[inline]
fn format_header_inner<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::header_require => format_header_require(data, node),
        Token::header_import => format_header_import(data, node),
        // Token::header_stage => format_header_stage(data, node),
        _ => {
            unreachable!()
        }
    }
}

fn format_header_import<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::other(token) => {
                data.output += &token;
                if token == "@import:" {
                    data.inner = token;
                } else if token == "pkgname" {
                    let text = node_to_text(&child, data.input);
                    data.inner = text;
                } else {
                    unimplemented!()
                }
            }
            _ => {
                unreachable!()
            }
        }
        output += &data.inner;
    }
    output += "\n";
    data.inner = output;
}

fn format_header_require<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(token) => {
                if token == "@require:" {
                    data.inner = token;
                } else if token == "pkgname" {
                    // format_pkg_name(data, &child);
                    let text = node_to_text(&child, data.input);
                    // output += &(text + "\n");
                    data.inner = text;
                } else {
                    unimplemented!()
                }
            }
            Token::whitespace => {
                // format_whitespace(data, &child);
                data.inner = "".to_string();
            }
            _ => {
                unreachable!()
            }
        }
        output += &data.inner;
    }
    output += "\n";
    data.inner = output;
}

fn format_preamble<'a>(data: &mut Formatter<'a>, node: &Node) {
    unimplemented!()
}

fn format_expr<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::match_expr => {
            todo!()
        }
        Token::bind_stmt => {
            todo!()
        }
        Token::ctrl_while => {
            todo!()
        }
        Token::ctrl_if => {
            todo!()
        }
        Token::lambda => {
            todo!()
        }
        Token::assignment => {
            todo!()
        }
        Token::binary_expr => {
            todo!()
        }
        Token::application => {
            format_application(data, node);
        }
        Token::unary_operator_expr => {
            todo!()
        }
        Token::command_application => {
            todo!()
        }
        Token::variant_constructor => {
            todo!()
        }
        Token::record_member => {
            todo!()
        }
        Token::_unary => {
            todo!()
        }
        token if LIST_UNARY.contains(&token) => {
            format_unary(data, node);
        }
        _ => {
            unreachable!()
        }
    }
}

fn format_application<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::application => {
                format_application(data, &child);
            }
            Token::identifier => {
                format_identifier(data, &child);
            }
            token if LIST_UNARY.contains(&token) => {
                format_unary(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_identifier<'a>(data: &mut Formatter<'a>, node: &Node) {
    let output = node_to_text(node, data.input);
    data.inner = output;
}

fn format_unary<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::block_text => {
            format_block_text(data, node);
        }
        Token::inline_text => {
            format_inline_text(data, node);
        }
        Token::inline_text_list => {
            todo!();
        }
        Token::inline_text_bullet_list => {
            todo!();
        }
        Token::math_text => {
            todo!();
        }
        Token::math_list => {
            todo!();
        }
        Token::record => {
            format_record(data, node);
        }
        Token::list => {
            todo!();
        }
        Token::tuple => {
            todo!();
        }
        Token::binary_operator => {
            todo!();
        }
        Token::_expr => {
            data.output += "(";
            format_expr(data, node);
            data.output += ")";
        }
        Token::expr_with_mod => {
            todo!();
        }
        Token::modvar => {
            todo!();
        }
        // Token::_literal => {
        //     todo!();
        // }
        token if LIST_LITERAL.contains(&token) => {
            format_literal(data, node);
        }
        Token::identifier => {
            todo!();
        }
        _ => {
            unreachable!();
        }
    }
}

fn format_record<'a>(data: &mut Formatter<'a>, node: &Node) {
    data.depth += 1;
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_UNARY.contains(&token) => {
                format_unary(data, &child);
            }
            token if LIST_RECORD_INNER.contains(&token) => {
                format_record_inner(data, &child);
                data.inner = data.indent() + &data.inner;
            }
            Token::other(s) => match s.as_str() {
                "(|" => {
                    data.inner = s + "\n";
                }
                "|)" => {
                    data.inner = s;
                }
                "with" => {
                    data.inner = s;
                }
                ";" => {
                    // format_record_inner(data, &child);
                    data.inner = s + "\n";
                }
                _ => {
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
    data.depth -= 1;
}

#[inline]
fn format_record_inner<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::record_unit => {
            format_record_unit(data, node);
        }
        Token::other(s) => match s.as_str() {
            _ => {
                unreachable!();
            }
        },
        _ => {
            unreachable!();
        }
    }
}

fn format_record_unit<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::identifier => {
                format_identifier(data, &child);
            }
            token if LIST_EXPR.contains(&token) || LIST_UNARY.contains(&token) => {
                format_expr(data, &child);
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "=" => {
                    data.inner = s;
                }
                _ => {
                    unreachable!();
                }
            },
            _ => {
                println!("record_unit: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_inline_text<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::inline_text => {
                format_inline_text(data, &child);
            }
            Token::horizontal => {
                format_horizontal(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "{" => {
                    data.inner = s;
                }
                "}" => {
                    data.inner = s;
                }
                _ => {
                    unreachable!();
                }
            },
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_horizontal<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::inline_literal_escaped => {
                todo!();
            }
            Token::inline_text_embedding => {
                todo!();
            }
            Token::math_text => {
                format_math_text(data, &child);
            }
            Token::literal_string => {
                // todo!();
            }
            Token::inline_cmd => {
                todo!();
            }
            Token::inline_token => {
                format_inline_token(data, &child);
                // todo!()
            }
            Token::other(token) => {
                // data.output += &token;
                println!("horizontal-other: {:?}", child.kind());
            }
            _ => {
                println!("horizontal: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}


fn format_inline_token<'a>(data: &mut Formatter<'a>, node: &Node) {
    data.inner = node_to_text(node, data.input);
}

fn format_literal<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::literal_unit => {
            todo!();
        }
        Token::literal_bool => {
            data.inner = node_to_text(node, data.input);
        }
        Token::literal_length => {
            todo!();
        }
        Token::literal_int => {
            todo!();
        }
        Token::literal_string => {
            todo!();
        }
        Token::literal_float => {
            todo!();
        }
        _ => {
            unreachable!();
        }
    }
}

fn format_block_text<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::vertical => {
                format_vertical(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "'<" => {
                    data.inner = s;
                }
                ">" => {
                    data.inner = s;
                }
                _ => {
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::comment => {
                format_comment(data, &child);
            }
            _ => {
                println!("block_text: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_inline_text_list<'a>(data: &mut Formatter<'a>, node: &Node) {
    for child in node.children(&mut node.walk()) {}
    // todo!();
}

fn format_inline_text_bullet_list<'a>(data: &mut Formatter<'a>, node: &Node) {
    todo!();
}

fn format_math_text<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::math => {
                format_math(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "${" => {
                    data.inner = s;
                }
                "}" => {
                    data.inner = s;
                }
                _ => {
                    unreachable!();
                }
            },
            _ => {
                println!("math_text: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_list<'a>(data: &mut Formatter<'a>, node: &Node) {
    todo!();
}

fn format_tuple<'a>(data: &mut Formatter<'a>, node: &Node) {
    todo!();
}

fn format_vertical<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    data.depth += 1;
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::block_cmd => {
                format_block_cmd(data, &child);
                output += &data.indent();
            }
            Token::block_text_embedding => {
                todo!();
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::comment => {
                format_comment(data, &child);
            }
            _ => {
                println!("vertical: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.depth -= 1;
    output += "\n";
    data.inner = output;
}

fn format_block_cmd<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::block_cmd_name => {
                data.inner = node_to_text(&child, data.input);
                // todo!();
                // format_block_cmd_name(data, &child);
            }
            Token::cmd_expr_arg => {
                todo!()
            }
            Token::cmd_expr_option => {
                todo!()
            }
            Token::cmd_text_arg => {
                format_cmd_text_arg(data, &child);
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::comment => {
                format_comment(data, &child);
            }
            _ => {
                println!("vertical: {:?}", child.kind());
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_cmd_text_arg<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::inline_text => {
                format_inline_text(data, &node);
            }
            Token::inline_text_list => {
                format_inline_text_list(data, &node);
            }
            Token::inline_text_bullet_list => {
                format_inline_text_bullet_list(data, &node);
            }
            Token::vertical => {
                // output += "<\n";
                format_block_text(data, &node);
                // output += ">\n"
            }
            Token::other(s) => match s.as_str() {
                "<" | ">" => {
                    // ignore
                    data.inner = s + "\n";
                }
                _ => {
                    unreachable!();
                }
            },
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_math<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::math_token => {
                data.inner = node_to_text(&child, data.input);
            }
            Token::math_unary => {
                data.inner = node_to_text(&child, data.input);
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_whitespace<'a>(data: &mut Formatter<'a>, node: &Node) {
    // println!("format whitespace");
    // let range = node.range();
    // println!("{:?}", range);
    data.inner = "".to_string();
}
