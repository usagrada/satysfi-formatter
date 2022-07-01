mod comment;
mod formatter;
mod reserved_words;
#[cfg(test)]
mod tests;
mod visualize;

use comment::*;
use formatter::Formatter;
use satysfi_parser::{grammar, CstText};
pub use visualize::*;

pub struct OptionData {
    pub row_length: usize,
    pub indent_space: usize,
    pub command_args_space: bool,
}

impl Default for OptionData {
    fn default() -> Self {
        Self {
            row_length: 80,
            indent_space: 4,
            command_args_space: true,
        }
    }
}

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード
/// * `output` - format された文字列
pub fn format(input: &str, option: OptionData) -> String {
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
        eprintln!("disable to format\n[parse error] line: {}, column: {}", line, col);
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
