use crate::token::Token;
use tree_sitter::{Node, Tree};

/// node を与えたときにテキストを返すための関数
fn node_to_text(node: &Node, text: &str) -> String {
    let range = node.byte_range();
    text[range.start..range.end].trim().to_string()
}

#[derive(Debug, Clone)]
struct Formatter<'a> {
    input: &'a str,
    output: &'a str,
    tree: &'a Tree,
}

pub fn format<'a>(input: &'a str, tree: &Tree) -> String {
    let root_node = tree.root_node();
    assert_eq!(root_node.kind(), "source_file");
    let child = root_node.child(0).unwrap();
    let output = String::new();
    let mut data = Formatter {
        input,
        output: &output,
        tree,
    };
    match child.kind().into() {
        Token::program_saty => format_program_saty(&mut data, &child),
        Token::program_satyh => format_program_satyh(&mut data, &child),
        _ => unreachable!(),
    };
    output
}

fn format_program_saty<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    println!("format file program_saty");
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::headers => {
                format_headers(data, &child);
                // output
            }
            Token::application => {
                let output = String::new();
                // let mut output = String::new();
                // for whitespace in child.children(&mut child.walk()) {
                //     output.push_str(&format_whitespace(input, tree, &whitespace));
                // }
                // output
            }
            Token::other(token) => {
                println!("others: {}", token);
            }
            Token::whitespace => {
                println!("whitespace: {:?}", child.range());
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}

fn format_program_satyh<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    println!("format file program_satyh");
    data.output
}

fn format_headers<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    println!("format headers");
    println!("{}", node.to_sexp());
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::header_import | Token::header_require | Token::header_stage => {
                format_header_inner(data, &child);
            }
            Token::other(token) => {
                // println!("others: {}", token);
                unreachable!();
            }
            Token::whitespace => {
                // println!("whitespace: {:?}", child.range());
                // format_whitespace(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}

#[inline]
fn format_header_inner<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    match node.kind().into() {
        Token::header_require => format_header_require(data, node),
        Token::header_import => format_header_import(data, node),
        _ => {
            unreachable!()
        }
    }
}

fn format_header_import<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            Token::other(token) => {
                if token == "@import:" {
                    print!("@import: ");
                } else if token == "pkgname" {
                    let text = node_to_text(&child, data.input);
                    println!("pkg: {}", text);
                } else {
                    unimplemented!()
                }
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}

fn format_header_require<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::header_require => {
                println!("{:?}", child);
            }
            Token::other(token) => {
                if token == "@require:" {
                    print!("@import: ");
                } else if token == "pkgname" {
                    // format_pkg_name(data, &child);
                    let text = node_to_text(&child, data.input);
                    println!("pkg: {}", text);
                } else {
                    unimplemented!()
                }
            }
            Token::whitespace => {
                format_whitespace(data, &child);
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}

fn format_whitespace<'a>(data: &mut Formatter<'a>, node: &Node) -> &'a str {
    println!("format whitespace");
    let range = node.range();
    println!("{:?}", range);
    data.output
}
