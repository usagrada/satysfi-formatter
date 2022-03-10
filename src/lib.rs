mod comment;
#[cfg(test)]
mod tests;
mod visualize;

use comment::*;
use satysfi_parser::{grammar, Cst, CstText};
pub use visualize::*;

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
    controls: ReservedText,
    command: ReservedText,
    before: ReservedText,
    module: ReservedText,
    direct: ReservedText,
    struct_stmt: ReservedText,
    cycle: ReservedText,
    match_stmt: ReservedText,
    while_stmt: ReservedText,
    if_stmt: ReservedText,
    else_stmt: ReservedText,
    true_stmt: ReservedText,
    false_stmt: ReservedText,
    open: ReservedText,
    then: ReservedText,
    when: ReservedText,
    with: ReservedText,
    and: ReservedText,
    end: ReservedText,
    fun: ReservedText,
    let_stmt: ReservedText,
    mod_stmt: ReservedText,
    not: ReservedText,
    sig: ReservedText,
    val: ReservedText,
    as_stmt: ReservedText,
    do_stmt: ReservedText,
    in_stmt: ReservedText,
    of: ReservedText,
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
    let_rec: "let-rec",
    controls: "controls",
    command: "command",
    before: "before",
    module: "module",
    direct: "direct",
    struct_stmt: "struct",
    cycle: "cycle",
    match_stmt: "match",
    while_stmt: "while",
    if_stmt: "if",
    else_stmt: "else",
    true_stmt: "true",
    false_stmt: "false",
    open: "open",
    then: "then",
    when: "when",
    with: "with",
    and: "and",
    end: "end",
    fun: "fun",
    let_stmt: "let",
    mod_stmt: "mod",
    not: "not",
    sig: "sig",
    val: "val",
    as_stmt: "as",
    do_stmt: "do",
    in_stmt: "in",
    of: "of",
};

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

    #[cfg(debug_assertions)]
    visualize_csttext_tree(&csttext);
    // #[cfg(debug_assertions)]
    // dbg!(&csttext);

    let depth = 0;
    let mut output = to_string_cst_inner(input, &csttext.cst, depth);

    // 末尾に改行がない場合、改行を挿入して終了
    if !output.ends_with("\n") {
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
        inner: [Cst]
    }
    */
    use satysfi_parser::Rule;
    let csts = cst.inner.clone();
    let sep = &match cst.rule {
        Rule::block_cmd | Rule::inline_cmd => " ".to_string(),
        Rule::vertical | Rule::horizontal_bullet_list => {
            format!("\n{}", indent_space(depth))
        }
        Rule::record | Rule::list => format!(";\n{}", indent_space(depth)),
        Rule::horizontal_single => "".to_string(),
        _ => "".to_string(),
    };

    let output = match cst.rule {
        Rule::let_block_stmt_ctx
        | Rule::let_block_stmt_noctx
        | Rule::let_inline_stmt_ctx
        | Rule::let_inline_stmt_noctx
        | Rule::let_math_stmt => {
            csts.iter().fold(String::new(), |current, now_cst| {
                let text = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::var => current + " " + &text,
                    Rule::block_cmd_name => current + " " + &text,
                    // Rule::arg => current + " " +text,
                    Rule::expr => current + " = " + &text,
                    Rule::comments => (current + &text),
                    _ => current + " " + &text,
                }
            }) + "\n"
        }
        Rule::horizontal_single => {
            let output = csts
                .iter()
                .fold((String::new(), 0), |current, now_cst| {
                    let s = to_string_cst(text, &now_cst, depth);
                    let start_newline =
                        text[now_cst.span.start..now_cst.span.end].starts_with("\n");
                    if current.0.is_empty() {
                        if now_cst.rule == Rule::comments {
                            (s, 0)
                        } else {
                            (s.clone(), s.chars().count())
                        }
                    } else if s.trim().is_empty() {
                        // 空行の処理
                        // 文字がない場合は何もしない
                        current
                    } else if now_cst.rule == Rule::comments {
                        let indent = indent_space(depth);
                        (format!("{}\n{indent}{s}", current.0.trim_end()), 0)
                    } else if start_newline {
                        let indent = indent_space(depth);
                        (format!("{}\n{indent}{s}", current.0.trim_end()), 0)
                    } else {
                        (current.0 + &s, current.1 + s.chars().count())
                    }
                    // } else if current.1 == 0 {
                    //     // 既に改行されている
                    //     // コメントに対応させるための例外処理
                    //     (current.0 + &s, s.chars().count())
                    // } else {
                    // if current.1 == 0 {
                    //     // コメントで終了している
                    //     (current.0 + &s, s.chars().count())
                    // 改行を勝手に入れると出力結果が変わってしまうのでコメントアウト
                    // } else if current.1 > default_option.row_length {
                    //     if current.1 + s.chars().count() > default_option.row_length {
                    //         // 一定以上の長さの時は改行を挿入
                    //         (
                    //             current.0 + "\n" + &indent_space(depth) + &s,
                    //             s.chars().count(),
                    //         )
                    //     } else {
                    //         (current.0 + &s, s.chars().count())
                    //     }
                    // } else {
                    //     (current.0 + &s, current.1 + s.chars().count())
                    // }
                    // (current.0 + &s, current.1 + s.chars().count())
                    // }
                })
                .0;
            // コメントが末尾にあるとき余計な改行が残ってしまうので削除
            output.trim().to_string()
        }
        _ => {
            let output = csts
                .iter()
                .fold((String::new(), false), |current, now_cst| {
                    let s = to_string_cst(text, &now_cst, depth);
                    let flag = now_cst.rule == Rule::comments;
                    let output = if current.1 {
                        (current.0.clone() + &s, flag)
                    } else if current.0.is_empty() {
                        (s, flag)
                    } else if s.is_empty() {
                        current
                    } else {
                        (current.0 + sep + &s, flag)
                    };

                    if cst.rule == Rule::program_saty && now_cst.rule == Rule::preamble {
                        // program saty だった場合、in を入れる
                        (output.0 + RESERVED_WORD.in_stmt + "\n\n", output.1)
                    // } else if cst.rule == Rule::list && now_cst.rule == Rule::expr {
                    //     // list だった場合、expr の後に ; を入れる
                    //     (output.0 + ";", output.1)
                    } else {
                        output
                    }
                })
                .0;
            output
        }
    };

    output
}

/// cst を文字列にするための関数
fn to_string_cst(text: &str, cst: &Cst, depth: usize) -> String {
    // インデントを制御するための変数
    let new_depth = match cst.rule {
        Rule::block_text | Rule::cmd_text_arg | Rule::record | Rule::list => depth + 1,
        _ => depth,
    };
    let start_indent = "\n".to_string() + &indent_space(new_depth);
    let end_indent = "\n".to_string() + &indent_space(depth);

    let output = to_string_cst_inner(text, cst, new_depth);
    let self_text = text.get(cst.span.start..cst.span.end).unwrap().to_string();

    use satysfi_parser::Rule;
    // 中身をそのまま返すものは output をそのまま返す
    match cst.rule {
        Rule::comments => to_comment_string(self_text) + &end_indent,
        // header
        Rule::stage => output,
        Rule::headers => {
            if output.len() > 0 {
                output + "\n"
            } else {
                output
            }
        }
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
        Rule::module_stmt => self_text, // TODO
        Rule::open_stmt => output,
        Rule::arg => output,

        // unary
        Rule::unary => output,
        Rule::unary_prefix => output,
        Rule::block_text => {
            if self_text.starts_with("'") {
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
        Rule::list => {
            let output = if output.len() > 0 {
                format!("[{start_indent}{output}{end_indent}]")
            } else {
                format!("[{output}]")
            };
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
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
        Rule::cmd_expr_arg => {
            if self_text.starts_with("(") {
                format!("({output})",)
            } else {
                output
            }
        }
        Rule::cmd_expr_option => self_text,
        Rule::cmd_text_arg => {
            // 括弧の種類を取得
            let start_arg = self_text.chars().nth(0).unwrap();
            let end_arg = self_text.chars().nth_back(0).unwrap();
            // コメントで開始 or 改行を含んでいたら、改行を入れる
            let include_comment = output.starts_with("%");
            let include_kaigyou = output.find("\n") != None || start_arg == '<' || include_comment;
            match output.trim().len() {
                0 => format!("{start_arg}{end_arg}"),
                num if include_kaigyou || num > default_option.row_length => {
                    format!("{start_arg}{start_indent}{output}{end_indent}{end_arg}")
                }
                _ => format!("{start_arg} {output} {end_arg}"),
            }
        }
        Rule::inline_cmd => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
        Rule::inline_cmd_name => self_text,
        Rule::block_cmd => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }

        Rule::block_cmd_name => self_text,
        Rule::math_cmd => self_text,
        Rule::math_cmd_name => self_text,
        Rule::math_cmd_expr_arg => self_text,
        Rule::math_cmd_expr_option => self_text,

        // pattern
        Rule::pat_as => output,
        Rule::pat_cons => output,
        Rule::pattern => output,
        Rule::pat_variant => output,
        Rule::pat_list => output,
        Rule::pat_tuple => output,

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
            output
        }
        Rule::horizontal_escaped_char => self_text, // TODO
        Rule::inline_text_embedding => output,      // TODO

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
        Rule::program_saty => output.trim_start().to_string(),
        Rule::program_satyh => output.trim_start().to_string(),
        Rule::preamble => output,
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
