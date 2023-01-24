mod reserved_words;
#[cfg(test)]
mod tests;
mod token;
mod visualize;
mod format;

use lsp_types::{FormattingOptions, TextEdit};
pub use visualize::*;

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード
/// * `output` - format された文字列
pub fn format(input: &str, option: FormattingOptions) -> String {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_satysfi::language())
        .expect("Error loading SATySFi language");

    let tree = parser.parse(input, Option::None).unwrap();
    #[cfg(debug_assertions)]
    println!("{:#?}", tree.root_node().to_sexp());
    #[cfg(debug_assertions)]
    visualize::visualize_csttext_tree(input, &tree);

    // "format".to_string()
    format::format(input, &tree, option)
}


pub fn format_lsp(input: &str, option: FormattingOptions) -> Vec<TextEdit> {
    let s = format(input, option);
    let mut edits = Vec::new();
    edits.push(TextEdit::new(lsp_types::Range::new(
        lsp_types::Position::new(0, 0),
        lsp_types::Position::new(input.split('\n').collect::<Vec<_>>().len() as u32, 0),
    ), s));
    edits
}

/// tree-sitter でどのように parse されるかの確認用
#[test]
fn test_tree_sitter() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_satysfi::language())
        .expect("Error loading SATySFi language");
    // let text = input[296..300].to_string();
    let text = r#"@require: stdja
@require: itemize

document(|
    author = {author};
    show-title = false;
    show-toc = true;
    title = {title};
|)'<
    +section{section}<
        +p {
            \listing{
                * item1
                * item2
                * item3
            }
        }
    >
>
"#;
    // println!("{}", text);
    let tree = parser.parse(text, Option::None).unwrap();
    dbg!(&tree.root_node().child(0).unwrap().kind());
    dbg!(&tree.root_node().child(0).unwrap().child(0).unwrap());
    dbg!(&tree.root_node().child(0).unwrap().child(1).unwrap());
    dbg!(&tree.root_node().child(0).unwrap().child(2).unwrap());
    dbg!(&tree.root_node().child(1).unwrap());
    let root_node = tree.root_node();
    dbg!(root_node);
    assert_eq!(root_node.kind(), "source_file");
    assert_eq!(tree.root_node().child(0).unwrap().kind(), "program_saty");
    assert_eq!(
        tree.root_node().child(0).unwrap().child(0).unwrap().kind(),
        "headers"
    );
    assert_eq!(
        tree.root_node().child(0).unwrap().child(1).unwrap().kind(),
        "whitespace"
    );
    assert_eq!(
        tree.root_node().child(0).unwrap().child(2).unwrap().kind(),
        "application"
    );
    assert_eq!(tree.root_node().child(1).unwrap().kind(), "whitespace");
}
