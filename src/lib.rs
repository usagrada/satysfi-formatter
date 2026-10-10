mod comment;
mod formatter;
mod helper;
mod printer;
mod reserved_words;
#[cfg(test)]
mod tests;
mod visualize;

use comment::*;
use formatter::Formatter;
use lspower::lsp::{FormattingOptions, TextEdit};
use satysfi_parser::{grammar, CstText};
pub use visualize::*;

/// 旧実装の format。移行が完了したら削除する。
/// * `input` - satysfi のコード
/// * `output` - format された文字列
pub fn format_legacy(input: &str, option: FormattingOptions) -> String {
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
    

    formatter.format(input, &csttext.cst, depth)
}

/// satysfi のコードを format する。
///
/// 行幅は `option.properties["lineWidth"]` (既定は 80) で指定できる。
/// コメントによる整形の制御 (`% satysfi-fmt: skip` など) に対応している。
pub fn format(input: &str, option: FormattingOptions) -> String {
    let csttext = match CstText::parse(input, grammar::program) {
        Ok(csttext) => csttext,
        Err(err) => {
            eprintln!(
                "disable to format\n[parse error] line: {}, column: {}",
                err.0.line, err.0.column
            );
            return input.to_string();
        }
    };
    printer::Printer::new(&csttext, option).print()
}

pub fn formatting(input: &str, option: FormattingOptions) -> Vec<TextEdit> {
    let csttext = match CstText::parse(input, grammar::program) {
        Ok(csttext) => csttext,
        Err(err) => {
            eprintln!(
                "disable to format\n[parse error] line: {}, column: {}",
                err.0.line, err.0.column
            );
            return Vec::new();
        }
    };
    let output = printer::Printer::new(&csttext, option).print();
    vec![TextEdit {
        range: lspower::lsp::Range {
            start: lspower::lsp::Position {
                line: 0,
                character: 0,
            },
            end: lspower::lsp::Position {
                line: csttext.lines.len() as u32,
                character: csttext.text.split('\n').last().unwrap().len() as u32,
            },
        },
        new_text: output,
    }]
}
