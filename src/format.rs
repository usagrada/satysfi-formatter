use crate::{
    token::{Token, *},
    OptionData,
};
mod helper;
use tree_sitter::{Node, Tree};

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

    /// depth + 1 したインデント用
    fn indent_start(&self) -> String {
        use self::helper::indent_space;
        let result = indent_space((self.depth + 1) * self.config.indent_space);
        result
    }
    /// node を与えたときにテキストを返すための関数
    fn node_to_text_trim(&self, node: &Node) -> String {
        let range = node.byte_range();
        self.input[range.start..range.end].trim().to_string()
    }
    fn node_to_text(&self, node: &Node) -> String {
        let range = node.byte_range();
        self.input[range.start..range.end].to_string()
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
    data.output = output.trim().to_string() + "\n";
}

fn format_comment<'a>(data: &mut Formatter<'a>, node: &Node) {
    let text = data.node_to_text_trim(node) + "\n";
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
                format_ignore(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
        output += &data.inner;
    }
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
                format_ignore(data, &child);
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
                format_ignore(data, &child);
            }
            Token::other(token) => {
                data.output += &token;
                if token == "@import:" {
                    data.inner = token;
                } else if token == "pkgname" {
                    let text = data.node_to_text_trim(&child);
                    output += " ";
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
                    let text = data.node_to_text_trim(&child);
                    output += " ";
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
    let output = data.node_to_text_trim(node);
    data.inner = output;
}

#[inline]
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
    let mut output = String::new();
    let mut records_vec = vec![];
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            token if LIST_UNARY.contains(&token) => {
                format_unary(data, &child);
            }
            token if LIST_RECORD_INNER.contains(&token) => {
                format_record_inner(data, &child);
                records_vec.push(data.inner.clone());
                data.inner = "".to_string();
            }
            Token::other(s) => match s.as_str() {
                "(|" => {
                    data.inner = s;
                    data.depth += 1;
                }
                "|)" => {
                    let mut records = String::new();
                    if records_vec.len() > 1 {
                        let sep = "\n".to_string() + &data.indent();
                        records = records_vec.join(&sep);
                        records += "\n";
                        data.depth -= 1;
                        records += &data.indent();
                        data.depth += 1;
                        output += "\n";
                        output += &data.indent();
                    } else if records_vec.len() == 1 {
                        for record in records_vec.iter() {
                            records += record;
                        }
                    }
                    data.inner = records;
                    data.inner += &s;
                    data.depth -= 1;
                }
                "with" => {
                    data.inner = s;
                }
                ";" => {
                    let len = records_vec.len();
                    records_vec[len - 1] += &s;
                    format_ignore(data, &child);
                }
                _ => {
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_ignore(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
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
                // format_whitespace(data, &child);
                format_ignore(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "=" => {
                    data.inner = " = ".to_string();
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
                "{" | "}" => {
                    data.inner = "".to_string();
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
    if output.contains("\n") {
        let indent_start = "\n".to_string() + &data.indent_start();
        let indent_end = "\n".to_string() + &data.indent();
        output = output.replace(
            "\n",
            &("\n".to_string() + &" ".repeat(data.config.indent_space)),
        );
        data.inner = format!("{{{}{}{}}}", indent_start, output, indent_end);
    } else {
        data.inner = format!("{{ {} }}", output);
    }
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
                format_inline_cmd(data, &child);
            }
            Token::inline_token => {
                let self_text = data.node_to_text(&child);
                format_inline_token(data, &child);
                if self_text.starts_with("\n") {
                    output += "\n";
                    output += &data.indent();
                } else if self_text.starts_with(char::is_whitespace) {
                    output += " "
                }
            }
            Token::other(_) => {
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
    data.inner = output.trim().to_string();
}

fn format_inline_token<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut text = data.node_to_text(node);
    if text.contains("\n") {
        let sep = "\n".to_string() + &data.indent();
        text = text
            .split("\n")
            .map(|line| {
                line.trim().to_string()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<String>>()
            .join(&sep);
    } else {
        text = text.trim().to_string();
    }
    data.inner = text;
}

fn format_inline_cmd<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::inline_cmd_name => {
                format_inline_cmd_name(data, &child);
            }
            Token::cmd_text_arg => {
                if data.config.command_args_space {
                    output += " ";
                }
                format_cmd_text_arg(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "{" => {
                    data.inner = s;
                }
                "}" => {
                    data.inner = s;
                }
                ";" => {
                    // end of command
                    data.inner = s;
                }
                _ => {
                    // println!("inline_cmd: {:?} {}", child.kind(), s);
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_ignore(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

#[inline]
fn format_inline_cmd_name<'a>(data: &mut Formatter<'a>, node: &Node) {
    data.inner = data.node_to_text_trim(node);
}

fn format_literal<'a>(data: &mut Formatter<'a>, node: &Node) {
    match node.kind().into() {
        Token::literal_unit => {
            todo!();
        }
        Token::literal_bool => {
            data.inner = data.node_to_text_trim(node);
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
    let mut vertical_inner_vec = vec![];
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::vertical => {
                format_vertical(data, &child);
                vertical_inner_vec.push(data.inner.clone());
                format_ignore(data, &child)
            }
            Token::other(s) => match s.as_str() {
                "'<" => {
                    data.inner = s;
                    data.depth += 1;
                }
                "<" => {
                    data.inner = s;
                    data.depth += 1;
                }
                ">" => {
                    if vertical_inner_vec.is_empty() {
                        data.inner = s;
                    } else {
                        output += "\n";
                        let sep = "\n".to_string() + &data.indent();
                        let vertical_inner = vertical_inner_vec.join(&sep);
                        data.inner = vertical_inner;
                        data.inner += &s;
                    }
                    data.depth -= 1;
                }
                _ => {
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_ignore(data, &child);
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
    for child in node.children(&mut node.walk()) {
        // println!("inline_text_list: {:?}", child.kind());
    }
    // todo!();
}

fn format_inline_text_bullet_list<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(s) => {
                data.inner = s.clone();
                if s == "{" {
                    data.depth += 1;
                    data.inner += "\n";
                } else if s == "}" {
                    data.depth -= 1;
                    output += &data.indent();
                }
            }
            Token::inline_text_bullet_item => {
                format_inline_text_bullet_item(data, &child);
                data.inner = data.indent() + &data.inner + "\n";
            }
            Token::whitespace => {
                format_ignore(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
}

fn format_inline_text_bullet_item<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::other(s) => {
                data.inner = s;
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::inline_text_bullet_star => {
                // format_inline_text_bullet_star(data, &child);
                data.inner = data.node_to_text_trim(&child);
                data.inner += " ";
            }
            Token::horizontal => {
                format_horizontal(data, &child);
            }
            _ => {
                unreachable!();
            }
        }
        output += &data.inner;
    }
    data.inner = output;
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
    // data.depth += 1;
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::block_cmd => {
                format_block_cmd(data, &child);
                data.inner = data.indent() + &data.inner + "\n";
            }
            Token::block_text_embedding => {
                todo!();
            }
            Token::whitespace => {
                format_ignore(data, &child);
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
    // data.depth -= 1;
    data.inner = output;
}

fn format_block_cmd<'a>(data: &mut Formatter<'a>, node: &Node) {
    let mut output = String::new();
    for child in node.children(&mut node.walk()) {
        match child.kind().into() {
            Token::block_cmd_name => {
                data.inner = data.node_to_text_trim(&child);
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
                if data.config.command_args_space {
                    output += " ";
                }
                format_cmd_text_arg(data, &child);
            }
            Token::whitespace => format_ignore(data, &child),
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
                format_inline_text(data, &child);
            }
            Token::inline_text_list => {
                format_inline_text_list(data, &child);
            }
            Token::inline_text_bullet_list => {
                format_inline_text_bullet_list(data, &child);
            }
            Token::vertical => {
                format_vertical(data, &child);
            }
            Token::other(s) => match s.as_str() {
                "<" => {
                    data.inner = s + "\n";
                    data.depth += 1;
                }
                ">" => {
                    data.inner = s;
                    data.depth -= 1;
                    output += &data.indent();
                }
                _ => {
                    unreachable!();
                }
            },
            Token::whitespace => {
                format_ignore(data, &child);
            }
            _ => {
                println!("token: {}", child.kind());
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
                data.inner = data.node_to_text_trim(&child);
            }
            Token::math_unary => {
                data.inner = data.node_to_text_trim(&child);
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
    let text = data.node_to_text_trim(node);
    data.inner = " ".to_string();
}

fn format_ignore<'a>(data: &mut Formatter<'a>, node: &Node) {
    data.inner = "".to_string();
}
