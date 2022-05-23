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
        return input.to_string();
    }
    let csttext = csttext.unwrap();
    let csttext = csttext_insert_comments(csttext);
    let formatter = Formatter::new(&csttext, option);

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);

    let depth = 0;
    let mut output = formatter.to_string_cst(input, &csttext.cst, depth);

    // 末尾に改行がない場合、改行を挿入して終了
    if !output.ends_with('\n') {
        output += "\n";
    }

    output
}
