#[cfg(test)]
mod tests;
mod visualize;
pub use visualize::*;

use satysfi_parser::{grammar, Cst, CstText};

type ReservedText = &'static str;

#[allow(dead_code)]
struct ReservedWord {
    let_block: ReservedText,
    let_math: ReservedText,
    let_mutable: ReservedText,
    type_stmt: ReservedText,
    let_inline: ReservedText,
    constraint: ReservedText,
    inline_command: ReservedText,
    block_command: ReservedText,
    math_command: ReservedText,
    let_rec: ReservedText,
}

const RESERVED_WORD: ReservedWord = ReservedWord {
    constraint: "constraint",
    inline_command: "inline-cmd",
    block_command: "block-cmd",
    math_command: "math-cmd",
    let_mutable: "let-mutable",
    let_inline: "let-inline",
    let_block: "let-block",
    let_math: "let-math",
    type_stmt: "type",
    let_rec: "let-rec", // / "controls" / "command" / "before" / "module" / "direct" / "struct"
                        // / "cycle" / "match" / "while" / "false"
                        // / "else" / "open" / "then" / "true" / "type" / "when" / "with"
                        // / "and" / "end" / "fun" / "let" / "mod" / "not" / "sig" / "val"
                        // / "as" / "do" / "if" / "in" / "of")
};

pub struct OptionData {
    row_length: usize,
    indent_space: usize,
}

#[allow(non_upper_case_globals)]
// format 設定のオプション
static default_option: OptionData = OptionData {
    row_length: 40,
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

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);

    let depth = 0;
    let mut output = to_string_cst_inner(input, &csttext.cst, depth);

    // 末尾に改行がない場合、改行を挿入して終了
    if output.chars().nth_back(0) != Some('\n') {
        output += "\n";
    }

    output
}

/// cst の inner の要素を結合して文字列に変換する関数
fn to_string_cst_inner(text: &str, cst: &Cst, depth: usize) -> String {
    /*
    Cst {
      rule: Rule,
      span: Span { start: number, end: number },
      inner: [Cst] }
    */
    use satysfi_parser::Rule;
    let csts = cst.inner.clone();
    let sep = &match cst.rule {
        Rule::block_cmd | Rule::horizontal_single => " ".to_string(),
        Rule::vertical | Rule::horizontal_bullet_list => format!("\n{}", indent_space(depth)),
        Rule::record => format!(";\n{}", indent_space(depth)),
        _ => "".to_string(),
    };

    let output = match cst.rule {
        Rule::let_block_stmt_ctx
        | Rule::let_block_stmt_noctx
        | Rule::let_inline_stmt_ctx
        | Rule::let_inline_stmt_noctx
        | Rule::let_math_stmt
        => {
            csts.iter().fold(String::new(), |current, now_cst| {
                match now_cst.rule {
                    Rule::var => current + " " + &to_string_cst(text, now_cst, depth),
                    Rule::block_cmd_name => current + " " + &to_string_cst(text, now_cst, depth),
                    Rule::expr => current + " = " + &to_string_cst(text, now_cst, depth),
                    _ => String::new(),
                }
                // current + &to_string_cst_inner(text, cst, depth)
            })
        }
        _ => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, &now_cst, depth);
            if current.is_empty() {
                s
            } else if s.is_empty() {
                current
            } else {
                current + sep + &s
            }
        }),
    };

    output
}

/// cst を文字列にするための関数
fn to_string_cst(text: &str, cst: &Cst, depth: usize) -> String {
    // インデントを制御するための変数
    let new_depth = match cst.rule {
        Rule::block_text | Rule::cmd_text_arg | Rule::record => depth + 1,
        _ => depth,
    };
    let start_indent = "\n".to_string() + &indent_space(new_depth);
    let end_indent = "\n".to_string() + &indent_space(depth);

    let output = to_string_cst_inner(text, cst, new_depth);
    let self_text = text.get(cst.span.start..cst.span.end).unwrap().to_string();

    use satysfi_parser::Rule;
    // 中身をそのまま返すものは output をそのまま返す
    match cst.rule {
        // header
        Rule::stage => output,
        Rule::headers => output + "\n",
        Rule::header_require => "@require: ".to_string() + &output + "\n",
        Rule::header_import => "@import: ".to_string() + &output + "\n",
        Rule::pkgname => self_text,

        // statement
        Rule::let_stmt => output,
        Rule::let_rec_stmt => format!("{}{}", RESERVED_WORD.let_rec, output),
        Rule::let_rec_inner => output,
        Rule::let_rec_matcharm => output,
        Rule::let_inline_stmt_ctx => format!("{}{}", RESERVED_WORD.let_inline, output),
        Rule::let_inline_stmt_noctx => format!("{}{}", RESERVED_WORD.let_inline, output),
        Rule::let_block_stmt_ctx => format!("{}{}", RESERVED_WORD.let_block, output),

        Rule::let_block_stmt_noctx => format!("{}{}", RESERVED_WORD.let_block, output),
        Rule::let_math_stmt => format!("{}{}", RESERVED_WORD.let_math, output),
        Rule::let_mutable_stmt => format!("{}{}", RESERVED_WORD.let_mutable, output),
        Rule::type_stmt => format!("{}{}", RESERVED_WORD.type_stmt, output),
        Rule::type_inner => output,
        Rule::type_variant => output,
        Rule::module_stmt => output,
        Rule::open_stmt => output,
        Rule::arg => output,

        // unary
        Rule::unary => output,
        Rule::unary_prefix => output,
        Rule::block_text => {
            if self_text.chars().nth(0) == Some('\'') {
                if output.len() > 0 {
                    format!("'<{start_indent}{output}{end_indent}>")
                } else {
                    format!("'<{output}>")
                }
            } else {
                if output.len() > 0 {
                    format!("<{start_indent}{output}{end_indent}>")
                } else {
                    format!("<{output}>")
                }
            }
        }
        Rule::horizontal_text => self_text,
        Rule::math_text => self_text,
        Rule::list => self_text,
        Rule::record => {
            // TODO: consider
            if cst.inner.len() > 1 {
                // 2 つ以上のときは改行
                format!("(|{start_indent}{output};{end_indent}|)")
            } else {
                // 1つだけの時は、改行しない
                format!("(|{output}|)")
            }
        }
        Rule::record_unit => self_text, // TODO
        Rule::tuple => self_text,
        Rule::bin_operator => self_text,
        Rule::expr_with_mod => self_text,
        Rule::var => self_text,
        Rule::var_ptn => self_text,
        Rule::modvar => self_text,
        Rule::mod_cmd_name => self_text,
        Rule::module_name => self_text,
        Rule::variant_name => self_text,

        // command
        Rule::cmd_name_ptn => self_text,
        Rule::cmd_expr_arg => self_text,
        Rule::cmd_expr_option => self_text,
        Rule::cmd_text_arg => {
            // 括弧の種類を取得
            let start_arg = self_text.chars().nth(0).unwrap();
            let end_arg = self_text.chars().nth_back(0).unwrap();
            // 改行を含んでいたら、改行を入れる
            let include_kaigyou = output.find("\n") != None || start_arg == '<';
            match output.trim().len() {
                0 => format!("{start_arg}{end_arg}"),
                num if include_kaigyou || num > default_option.row_length => {
                    format!("{start_arg}{start_indent}{output}{end_indent}{end_arg}")
                }
                _ => format!("{start_arg} {output} {end_arg}"),
            }
        }
        Rule::inline_cmd => {
            if self_text.chars().nth_back(0) == Some(';') {
                format!("{output};")
            } else {
                output
            }
        }
        Rule::inline_cmd_name => self_text,
        Rule::block_cmd => {
            if self_text.chars().nth_back(0) == Some(';') {
                format!("{output};")
            } else {
                output
            }
        }

        Rule::block_cmd_name => self_text,
        Rule::math_cmd => self_text,
        Rule::math_cmd_name => self_text,
        Rule::math_cmd_expr_arg => self_text,
        Rule::math_cmd_expr_option => self_text,

        // expr
        Rule::expr => output,
        Rule::match_expr => output,          // TODO
        Rule::match_arm => output,           // TODO
        Rule::match_guard => output,         // TODO
        Rule::bind_stmt => output,           // TODO
        Rule::ctrl_while => output,          // TODO
        Rule::ctrl_if => output,             // TODO
        Rule::lambda => output,              // TODO
        Rule::assignment => output,          // TODO
        Rule::dyadic_expr => output,         // TODO
        Rule::unary_operator_expr => output, // TODO
        Rule::unary_operator => output,      // TODO
        // application
        Rule::application => output,
        Rule::application_args_normal => output,
        Rule::application_args_optional => output,
        Rule::command_application => output,
        Rule::variant_constructor => output,

        // horizontal
        Rule::horizontal_single => output,
        Rule::horizontal_list => output,                  // TODO
        Rule::horizontal_bullet_list => output,           // TODO
        Rule::horizontal_bullet => output,                // TODO
        Rule::horizontal_bullet_star => "* ".to_string(), // TODO
        Rule::regular_text => {
            let sep = format!("\n{}", indent_space(depth));
            let output = self_text
                .split("\n")
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<String>>()
                .join(&sep);
            // remove space of start and end
            // self_text.trim().to_string()
            output
        }
        Rule::horizontal_escaped_char => output, // TODO
        Rule::inline_text_embedding => output,   // TODO

        // vertical
        Rule::vertical => output,             // TODO
        Rule::block_text_embedding => output, // TODO

        // constants
        Rule::const_unit => self_text,
        Rule::const_bool => self_text,
        Rule::const_int => self_text,
        Rule::const_float => self_text,
        Rule::const_length => self_text,
        Rule::const_string => self_text,

        // TODO other things
        Rule::misc => " ".to_string(),
        Rule::program_saty => output,
        Rule::program_satyh => output,
        Rule::preamble => {
            if output.len() > 0 {
                format!("{output}\nin\n\n")
            } else {
                output
            }
        }
        // TODO
        // _ => self_text,
        _ => "".to_string(),
    }
}

#[inline]
fn indent_space(depth: usize) -> String {
    let mut output = String::new();
    for _ in 0..default_option.indent_space * depth {
        output += " "
    }
    output
}
