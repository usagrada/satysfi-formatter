use crate::token::{token_to_string, Token};
use clap::Values;
use tree_sitter::{Node, Tree};

pub fn format<'a>(input: &'a str, tree: &Tree) -> &'a str {
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
    let output = match child.kind().into() {
        Token::program_saty => format_program_saty(input, tree, &child),
        Token::program_satyh => format_program_satyh(input, tree, &child),
        _ => unreachable!(),
    };
    output
}

fn format_program_saty<'a>(input: &'a str, tree: &Tree, node: &Node) -> &'a str {
    println!("format file program_saty");
    input
}
fn format_program_satyh<'a>(input: &'a str, tree: &Tree, node: &Node) -> &'a str {
    println!("format file program_satyh");
    input
}
