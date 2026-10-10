//! 旧 API `formatting` (satysfi-language-server が移行するまでの互換)。シグネチャと挙動を保つ。
#![allow(deprecated)]
use crate::formatting;
use lspower::lsp::{FormattingOptions, FormattingProperty, Position, Range, TextEdit};

fn option(tab_size: u32, insert_spaces: bool) -> FormattingOptions {
    FormattingOptions {
        tab_size,
        insert_spaces,
        ..Default::default()
    }
}

fn new_text(edits: Vec<TextEdit>) -> String {
    assert_eq!(edits.len(), 1, "{edits:?}");
    edits.into_iter().next().unwrap().new_text
}

#[test]
fn formatting_replaces_whole_document() {
    let input = "let x  =  1\nlet y = `あ`";
    assert_eq!(
        formatting(input, option(4, true)),
        vec![TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(1, 11),
            },
            new_text: "let x = 1\nlet y = `あ`\n".to_string(),
        }]
    );
}

#[test]
fn formatting_returns_no_edit_on_parse_error() {
    assert_eq!(formatting("let x = = 1\n", option(4, true)), vec![]);
}

#[test]
fn formatting_options_are_honored() {
    let input = "let x = [aaaaaaaa; bbbbbbbb]\n";
    let mut narrow = option(2, true);
    narrow
        .properties
        .insert("lineWidth".to_string(), FormattingProperty::Number(16));
    assert_eq!(
        new_text(formatting(input, narrow.clone())),
        "let x = [\n  aaaaaaaa;\n  bbbbbbbb;\n]\n"
    );
    narrow.insert_spaces = false;
    assert_eq!(
        new_text(formatting(input, narrow)),
        "let x = [\n\taaaaaaaa;\n\tbbbbbbbb;\n]\n"
    );
}
