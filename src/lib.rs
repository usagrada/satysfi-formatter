mod printer;
#[cfg(test)]
mod tests;
mod visualize;

use lspower::lsp::{FormattingOptions, TextEdit};
use satysfi_parser::{grammar, CstText};
pub use visualize::*;

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
