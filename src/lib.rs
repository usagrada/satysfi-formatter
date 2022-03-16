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
    row_length: usize,
    indent_space: usize,
}

#[allow(non_upper_case_globals)]
// format 設定のオプション
static default_option: OptionData = OptionData {
    row_length: 80,
    indent_space: 4,
};

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード  
/// * `output` - format された文字列
pub fn format(input: &str) -> String {
    /*
    CstText {
        text: string,
        lines: Vec<usize>, // start
        cst: Cst,
    }
    */
    let csttext = CstText::parse(input, grammar::program).expect("parse error");
    let csttext = csttext_insert_comments(csttext);
    let formatter = Formatter::new(&csttext);

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);
    // #[cfg(debug_assertions)]
    // dbg!(&csttext);

    let depth = 0;
    let mut output = formatter.to_string_cst(input, &csttext.cst, depth);

    // 末尾に改行がない場合、改行を挿入して終了
    if !output.ends_with('\n') {
        output += "\n";
    }

    output
}
