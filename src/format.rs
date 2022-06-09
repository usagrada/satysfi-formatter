use crate::token::Token;
use tree_sitter::{Node, Tree};

#[derive(Debug, Clone)]
struct Formatter<'a> {
    input: &'a str,
    output: &'a str,
    tree: &'a Tree,
}

pub fn format<'a>(input: &'a str, tree: &Tree) -> String {
    let root_node = tree.root_node();
    assert_eq!(root_node.kind(), "source_file");
    // let output = if root_node.child(0).unwrap().kind() == Token::program_saty.value() {
    //     println!("saty: {}", root_node.child(0).unwrap().kind());
    //     format_program_saty(input, tree)
    // } else if root_node.child(0).unwrap().kind() == Token::program_satyh.value() {
    //     println!("satyh: {}", root_node.child(0).unwrap().kind());
    //     format_program_satyh(input, tree)
    // } else {
    //     unreachable!()
    // };
    let child = root_node.child(0).unwrap();
    let output = String::new();
    let data = Formatter {
        input,
        output: &output,
        tree,
    };
    match child.kind().into() {
        Token::program_saty => format_program_saty(data, &child),
        Token::program_satyh => format_program_satyh(data, &child),
        _ => unreachable!(),
    };
    output
}

fn format_program_saty<'a>(data: Formatter<'a>, node: &Node) -> &'a str {
    println!("format file program_saty");
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::headers => {
                format_headers(data.clone(), &child);
                // output
            }
            Token::application => {
                let mut output = String::new();
                // let mut output = String::new();
                // for whitespace in child.children(&mut child.walk()) {
                //     output.push_str(&format_whitespace(input, tree, &whitespace));
                // }
                // output
            }
            Token::other(token) => {
                println!("others: {}", token);
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}

fn format_program_satyh<'a>(data: Formatter<'a>, node: &Node) -> &'a str {
    println!("format file program_satyh");
    data.output
}

fn format_headers<'a>(data: Formatter<'a>, node: &Node) -> &'a str {
    println!("format headers");
    for child in node.children(&mut node.walk()) {
        println!("{}", child.kind());
        match child.kind().into() {
            Token::header_import | Token::header_require | Token::header_stage => {
                // println!("header")
            }
            Token::other(token) => {
                println!("others: {}", token);
            }
            _ => {
                unreachable!()
            }
        }
    }
    data.output
}
