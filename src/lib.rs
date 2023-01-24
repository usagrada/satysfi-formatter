mod comment;
mod format_lsp;
mod formatter;
mod helper;
mod reserved_words;
#[cfg(test)]
mod tests;
mod visualize;

use comment::*;
use formatter::Formatter;
use lspower::lsp::{FormattingOptions, TextEdit};
use satysfi_parser::{grammar, CstText};
pub use visualize::*;

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード
/// * `output` - format された文字列
pub fn format(input: &str, option: FormattingOptions) -> String {
    /*
    CstText {
        text: string,
        lines: Vec<usize>, // start
        cst: Cst,
    }
    */
    let csttext = CstText::parse(input, grammar::program);
    if csttext.is_err() {
        let err = csttext.unwrap_err();
        let line = err.0.line;
        let col = err.0.column;
        eprintln!(
            "disable to format\n[parse error] line: {}, column: {}",
            line, col
        );
        return input.to_string();
    }
    let csttext = csttext.unwrap();
    let csttext = csttext_insert_comments(csttext);
    let formatter = Formatter::new(&csttext, option);

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);

    let depth = 0;
    let output = formatter.format(input, &csttext.cst, depth);

    output
}

pub fn format_lsp(input: &str, option: FormattingOptions) -> Vec<TextEdit> {
    let csttext = CstText::parse(input, grammar::program);
    if csttext.is_err() {
        let err = csttext.unwrap_err();
        let line = err.0.line;
        let col = err.0.column;
        eprintln!(
            "disable to format\n[parse error] line: {}, column: {}",
            line, col
        );
        return Vec::new();
    }
    let csttext = csttext.unwrap();
    let csttext = csttext_insert_comments(csttext);
    let formatter = format_lsp::Formatter::new(&csttext, option);
    let output = formatter.format(input, &csttext.cst, 0);
    let mut edits = Vec::new();
    edits.push(TextEdit {
        range: lspower::lsp::Range {
            start: lspower::lsp::Position {
                line: 0,
                character: 0,
            },
            end: lspower::lsp::Position {
                line: csttext.lines.len() as u32,
                character: csttext.text.split("\n").last().unwrap().len() as u32,
            },
        },
        new_text: output,
    });
    edits
}
