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
    // 関数内で改行するときはこれを使用する
    let indent = indent_space(depth);
    let newline = format!("\n{indent}");
    let sep = &match cst.rule {
        Rule::block_cmd | Rule::inline_cmd => " ".to_string(),
        Rule::type_application => " ".to_string(),
        Rule::type_prod => " * ".to_string(),
        Rule::dyadic_expr | Rule::match_expr | Rule::unary_operator_expr => " ".to_string(),
        Rule::vertical | Rule::horizontal_bullet_list => newline.clone(),
        Rule::horizontal_list => format!("|{newline}"),
        Rule::unary => "#".to_string(),
        Rule::type_optional => " ?-> ".to_string(),
        Rule::record | Rule::type_record | Rule::list => format!(";{newline}"),
        Rule::type_block_cmd | Rule::type_inline_cmd | Rule::type_math_cmd => format!(";{newline}"),
        Rule::horizontal_single => "".to_string(),
        Rule::variant_constructor => " ".to_string(),
        _ => "".to_string(),
    };

    let output = match cst.rule {
        Rule::variant_constructor => {
            let mut output = String::new();
            for cst in csts {
                let s = to_string_cst(text, &cst, depth);
                if output.is_empty() {
                    output = s;
                } else if !(cst.rule == Rule::unary && s.starts_with("(")) {
                    output += &sep;
                    output += &s;
                } else {
                    output += &s;
                }
            }
            output
        }
        Rule::let_mutable_stmt => {
            csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    return s;
                }
                match now_cst.rule {
                    Rule::var => current + " " + &s,
                    Rule::expr => {
                        if s.contains("\n") {
                            // 1つインデントを深くする
                            let s = to_string_cst(text, now_cst, depth + 1);
                            current + " <-" + &newline + &indent_space(1) + s.trim_start()
                        } else {
                            current + " <- " + &s
                        }
                    }
                    Rule::comments => current + &s,
                    _ => unreachable!(),
                }
            })
        }
        Rule::let_block_stmt_ctx
        | Rule::let_block_stmt_noctx
        | Rule::let_inline_stmt_ctx
        | Rule::let_inline_stmt_noctx
        | Rule::let_stmt
        | Rule::let_rec_stmt
        | Rule::sig_val_stmt
        | Rule::sig_direct_stmt
        | Rule::type_stmt
        | Rule::let_math_stmt => {
            csts.iter()
                .enumerate()
                .fold(String::new(), |current, (index, now_cst)| {
                    let s = to_string_cst(text, now_cst, depth);
                    let s = if cst.rule == Rule::sig_val_stmt && now_cst.rule == Rule::bin_operator
                    {
                        format!("({s})")
                    } else {
                        s
                    };
                    if current.is_empty() {
                        return s;
                    }
                    match now_cst.rule {
                        Rule::var => current + " " + &s,
                        Rule::block_cmd_name => current + " " + &s,
                        Rule::type_expr => current + ": " + &s,
                        Rule::constraint => {
                            // 1つインデントを深くする
                            let s = to_string_cst(text, now_cst, depth + 1);
                            current + &newline + &indent_space(1) + &s
                        }
                        Rule::expr => {
                            // 直前がコメント
                            if index > 0 && csts[index - 1].rule == Rule::comments {
                                // 1つインデントを深くする
                                let s = to_string_cst(text, now_cst, depth + 1);
                                current + &s
                            }
                            // ブロック定義は例外
                            else if s.starts_with("let")
                                || (!s.starts_with("'<") && !s.starts_with("{")) && s.contains("\n")
                            {
                                // 1つインデントを深くする
                                let s = to_string_cst(text, now_cst, depth + 1);
                                current + " =" + &newline + &indent_space(1) + s.trim_start()
                            } else {
                                current + " = " + &s
                            }
                        }
                        Rule::comments => {
                            if index + 1 < csts.len() && csts[index + 1].rule == Rule::expr {
                                // 1つインデントを深くする
                                let s = to_string_cst(text, now_cst, depth + 1);
                                current + " =" + &newline + &indent_space(1) + &s
                            } else {
                                current + &s
                            }
                        }
                        _ => current + " " + &s,
                    }
                })
        }
        Rule::math_cmd_expr_arg | Rule::math_cmd_expr_option => {
            // 高々1つの要素
            csts.iter().fold(String::new(), |_, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::math_list | Rule::math_single => format!("{{ {s} }}"),
                    Rule::horizontal_list
                    | Rule::horizontal_bullet_list
                    | Rule::horizontal_single => format!("!{{ {s} }}"),
                    Rule::vertical => format!("!{s}"),
                    Rule::expr => format!("!({s})"),
                    Rule::record | Rule::list => format!("!{s}"),
                    _ => unreachable!(),
                }
            })
        }
        Rule::cmd_expr_option => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            let s = if now_cst.rule == Rule::expr {
                format!("({s})")
            } else {
                s
            };
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::expr => current + &s,
                _ => current + &sep + &s,
            }
        }),
        Rule::pat_cons => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::pattern => current + " " + &s,
                Rule::pat_variant => current + " " + &s,
                Rule::pat_as => current + " :: " + &s,
                _ => unreachable!(),
            }
        }),
        Rule::constraint => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::type_param => current + " " + &s,
                Rule::type_record => current + " :: " + &s,
                _ => unreachable!(),
            }
        }),
        Rule::record => {
            let mut iter = csts.into_iter().peekable();
            let mut output = String::new();
            while iter.peek() != None {
                let now_cst = &iter.next().unwrap();
                let s = to_string_cst(text, now_cst, depth);
                let s = if now_cst.rule == Rule::unary {
                    format!("{s} {} ", RESERVED_WORD.with)
                } else {
                    s
                };
                match now_cst.rule {
                    Rule::unary => {
                        output += &s;
                        continue;
                    }
                    Rule::record_unit => {
                        output += &s;
                    }
                    _ => unreachable!(),
                };
                // 次の要素が存在すれば結合
                let next = iter.peek();
                if next != None && next.unwrap().rule == Rule::record_unit {
                    output += &sep;
                };
            }
            output
        }
        Rule::type_inner => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            let s = if now_cst.rule == Rule::type_name {
                s + " = "
            } else {
                s
            };
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::type_param => current + &s,
                // not end cst
                Rule::type_name => current + " " + &s,
                Rule::type_variant => current + " | " + &s,
                Rule::type_expr => current + &s,
                _ => current + &s,
            }
        }),
        Rule::type_variant => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    return s;
                }
                match now_cst.rule {
                    Rule::variant_name => current + &s,
                    Rule::type_expr => current + " of " + &s,
                    Rule::comments => current + &s,
                    _ => unreachable!(),
                }
            });
            output
        }
        Rule::let_rec_inner => {
            // for rule let_rec_stmt_argument()
            let mut type_expr = false;
            csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    return s;
                }

                match now_cst.rule {
                    Rule::pattern => current + &s,
                    Rule::let_rec_matcharm => current + &newline + "| " + s.trim(),
                    Rule::type_expr => {
                        type_expr = true;
                        current + ": " + &s
                    }
                    Rule::arg => {
                        if type_expr {
                            // 一度だけマッチ
                            type_expr = false;
                            current + " | " + &s
                        } else {
                            current + " " + &s
                        }
                    }
                    Rule::expr => current + " = " + &s,
                    _ => current + &s,
                }
            })
        }
        Rule::let_rec_matcharm => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::arg => current + " " + &s,
                Rule::expr => current + " = " + &s,
                _ => unreachable!(),
            }
        }),
        Rule::match_arm => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            // ptn:pat_as() _ guard:match_guard()? _ "->" _ expr:(!match_expr() e:expr() {e})
            match now_cst.rule {
                Rule::pat_as => current + " " + &s,
                Rule::expr => current + " -> " + &s,
                _ => current + &s,
            }
        }),
        Rule::match_expr => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                if now_cst.rule == Rule::expr {
                    return format!("{} {s} {}", RESERVED_WORD.match_stmt, RESERVED_WORD.with);
                }
                return s;
            }
            match now_cst.rule {
                Rule::expr => {
                    current
                        + " "
                        + &format!("{} {s} {}", RESERVED_WORD.match_stmt, RESERVED_WORD.with)
                }
                Rule::match_arm => current + &newline + "| " + &s,
                _ => current + &s,
            }
        }),
        Rule::ctrl_if => {
            // s:p() kwd("if") _ cond:expr() _ kwd("then") _ et:expr() _ kwd("else") _ ee:expr() e:p()
            if csts.len() < 3 {
                panic!("ctrl_if: csts.len() < 3");
            }
            let mut cnt = 0;
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::expr => {
                        cnt += 1;
                        match cnt {
                            1 => current + "if " + &s,
                            2 => current + " then " + &s,
                            3 => current + " else " + &s,
                            _ => unreachable!(),
                        }
                    }
                    Rule::comments => current + &s,
                    _ => unreachable!(),
                }
            });

            output
        }
        Rule::unary => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            let s = if now_cst.rule == Rule::bin_operator || now_cst.rule == Rule::expr {
                format!("({s})")
            } else {
                s
            };
            if current.is_empty() {
                return s;
            }
            match &*current {
                "!" | "&" | "~" => {
                    return current + &s;
                }
                _ => {}
            }
            match now_cst.rule {
                Rule::bin_operator | Rule::expr => current + &s,
                _ => current + &sep + &s,
            }
        }),
        Rule::lambda => csts
            .iter()
            .fold(RESERVED_WORD.fun.to_string(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                if current.is_empty() {
                    return s;
                }
                match now_cst.rule {
                    Rule::pattern => current + " " + &s,
                    Rule::comments => current + &s,
                    _ => current + " -> " + &s,
                }
            }),
        Rule::record_unit => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::var_ptn => current + " " + &s,
                Rule::expr => current + " = " + &s,
                Rule::comments => current + &s,
                _ => unreachable!(),
            }
        }),
        Rule::type_record_unit => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::var => current + " " + &s,
                Rule::type_expr => current + ": " + &s,
                Rule::comments => current + &s,
                _ => unreachable!(),
            }
        }),
        Rule::type_application => {
            let mut output = String::new();
            for cst in &csts {
                let s = to_string_cst(text, cst, depth);
                if !output.is_empty() {
                    output += &sep;
                }
                if cst.rule == Rule::type_expr {
                    output += &format!("({s})");
                } else {
                    output += &s;
                }
            }
            output
        }
        Rule::assignment => csts.iter().fold(String::new(), |current, now_cst| {
            let s = to_string_cst(text, now_cst, depth);
            if current.is_empty() {
                return s;
            }
            match now_cst.rule {
                Rule::var => current + " " + &s,
                Rule::dyadic_expr
                | Rule::unary_operator_expr
                | Rule::application
                | Rule::unary
                | Rule::variant_constructor => current + " <- " + &s,
                Rule::comments => current + &s,
                _ => unreachable!(),
            }
        }),
        Rule::application => {
            if csts.is_empty() {
                return "".to_string();
            }
            let first_text = to_string_cst(text, &csts[0], depth);
            let insert_space = first_text != "document";
            let mut output = first_text;
            for cst in csts.iter().skip(1) {
                let s = to_string_cst(text, cst, depth);
                if insert_space {
                    output += " ";
                }
                output += &s;
            }
            output
        }
        Rule::bind_stmt => {
            // let* ~ in のとき用
            let output =
                csts.iter()
                    .enumerate()
                    .fold(String::new(), |current, (index, now_cst)| {
                        let s = to_string_cst(text, now_cst, depth);
                        match now_cst.rule {
                            Rule::let_stmt
                            | Rule::let_rec_stmt
                            | Rule::let_math_stmt
                            | Rule::let_mutable_stmt
                            | Rule::open_stmt => {
                                if index == 0 {
                                    current + &s + " " + RESERVED_WORD.in_stmt
                                } else {
                                    current + &newline + &s + " " + RESERVED_WORD.in_stmt
                                }
                            }
                            Rule::expr => {
                                let current = if index > 0 && csts[index - 1].rule == Rule::comments
                                {
                                    current
                                } else if s.starts_with("let") || s.contains("\n") {
                                    current + &newline
                                } else {
                                    current + " "
                                };
                                if s.starts_with("let") {
                                    current + s.trim_start()
                                } else {
                                    if s.contains("\n") {
                                        let s = to_string_cst(text, now_cst, depth + 1);
                                        // 1つ深くする
                                        current + &indent_space(1) + s.trim_start()
                                    } else {
                                        current + s.trim_start()
                                    }
                                }
                            }
                            Rule::comments => {
                                if current.ends_with(RESERVED_WORD.in_stmt) {
                                    current + &newline + &s
                                } else {
                                    current + &s
                                }
                            }
                            _ => current + &s,
                        }
                    });
            output
        }
        Rule::type_expr => {
            let mut iter = csts.into_iter().peekable();
            let mut now_cst = iter.next().unwrap();
            let mut output = to_string_cst(text, &now_cst, depth);
            while iter.peek() != None {
                // 次の要素が存在すれば結合
                if now_cst.rule == Rule::type_optional {
                    output += " ?-> ";
                } else {
                    output += " -> ";
                }
                now_cst = iter.next().unwrap();

                let s = to_string_cst(text, &now_cst, depth);
                match now_cst.rule {
                    Rule::type_prod | Rule::type_optional => {
                        output += &s;
                    }
                    Rule::comments => {
                        output += &s;
                    }
                    _ => unreachable!(),
                }
            }
            output
        }
        Rule::module_stmt => {
            let mut iter = csts.into_iter().peekable();
            let first = iter.next().unwrap();
            let mut output = to_string_cst(text, &first, depth);
            while iter.peek() != None {
                let now_cst = &iter.next().unwrap();
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::module_stmt => {}
                    Rule::sig_stmt => {
                        output += ": ";
                    }
                    Rule::struct_stmt => {
                        output += " = ";
                    }
                    Rule::comments => {}
                    _ => unreachable!(),
                }
                output += &s;
            }
            output
        }
        Rule::struct_stmt => {
            let check = &format!("\n{newline}");
            let output =
                csts.iter()
                    .enumerate()
                    .fold(String::new(), |current, (index, now_cst)| {
                        let s = to_string_cst(text, now_cst, depth);

                        // 改行の制御
                        let current = if current.is_empty() || current.ends_with(check) {
                            current
                        } else if index > 0 && csts[index - 1].rule == Rule::comments {
                            current
                        } else if index > 0 && csts[index - 1].rule != now_cst.rule {
                            // ルールの切り替わり位置
                            current + "\n" + &newline
                        } else if !s.contains("\n") {
                            current + &newline
                        } else if csts[index - 1].rule == Rule::let_stmt
                            || csts[index - 1].rule == Rule::let_rec_stmt
                        {
                            current + "\n" + &newline
                        } else {
                            match now_cst.rule {
                                Rule::let_stmt | Rule::let_rec_stmt => current + "\n" + &newline,
                                Rule::comments => current,
                                _ => {
                                    // 基本的に改行する
                                    current + &newline
                                }
                            }
                        };
                        match now_cst.rule {
                            Rule::let_stmt | Rule::let_rec_stmt => current + &s,
                            Rule::comments => current + &s,
                            Rule::bind_stmt => current + &s,
                            _ => current + &s,
                        }
                    });
            output
        }
        Rule::sig_stmt => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, now_cst, depth);
                match now_cst.rule {
                    Rule::module_name => current + " " + &s,
                    Rule::struct_stmt => current + "= " + RESERVED_WORD.struct_stmt + &s,
                    Rule::comments => current + &s,
                    _ => current + &s,
                }
            });
            output
        }
        Rule::horizontal_single => {
            let output = csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, &now_cst, depth);
                if current.is_empty() {
                    s
                } else if now_cst.rule == Rule::regular_text && s.trim().is_empty() {
                    // 空行・スペースの処理
                    if current.ends_with(char::is_whitespace) {
                        // 既に空白がある場合には何もしない
                        current
                    } else {
                        current + &s
                    }
                } else if now_cst.rule == Rule::regular_text {
                    if current.ends_with(char::is_whitespace) {
                        // 既に空白がある場合には何もしない
                        current + &s.trim_start()
                    } else {
                        current + &s
                    }
                } else if now_cst.rule == Rule::comments {
                    format!("{}{newline}{s}", current.trim_end())
                } else {
                    current + &s
                }
            });

            // コメントが末尾にあるとき余計な改行が残ってしまうので削除
            output.trim().to_string()
        }
        Rule::preamble => csts.iter().fold(String::new(), |current, now_cst| {
            // 例外処理
            let s = to_string_cst(text, &now_cst, depth).trim().to_string();
            if current.is_empty() {
                s
            } else if s.is_empty() {
                current
            } else {
                match now_cst.rule {
                    Rule::module_stmt => current + "\n\n" + &s,
                    _ => current + "\n" + &s,
                }
            }
        }),
        Rule::list => csts
            .iter()
            .fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, &now_cst, depth);
                let flag = now_cst.rule == Rule::comments;
                if flag {
                    current + &s
                } else if current.is_empty() {
                    s + &sep
                } else if s.is_empty() {
                    current
                } else if now_cst.rule == Rule::comments {
                    current + &s
                } else {
                    current + &s + sep
                }
            })
            .trim_end()
            .to_string(),
        _ => {
            csts.iter().fold(String::new(), |current, now_cst| {
                let s = to_string_cst(text, &now_cst, depth);
                let flag = now_cst.rule == Rule::comments && false;
                let output = if flag {
                    current + &s
                } else if current.is_empty() {
                    s
                } else if s.is_empty() {
                    current
                } else if current.ends_with(&newline) {
                    current + &s
                } else {
                    current + sep + &s
                };

                if cst.rule == Rule::program_saty && now_cst.rule == Rule::preamble {
                    // program saty だった場合、in を入れる
                    output + "\n" + RESERVED_WORD.in_stmt + "\n\n"
                } else {
                    output
                }
            })
        }
    };

    output
}

/// cst を文字列にするための関数
fn to_string_cst(text: &str, cst: &Cst, depth: usize) -> String {
    // インデントを制御するための変数
    let new_depth = match cst.rule {
        Rule::block_text | Rule::cmd_text_arg | Rule::record | Rule::type_record => depth + 1,
        Rule::horizontal_list | Rule::list => depth + 1,
        Rule::type_block_cmd | Rule::type_inline_cmd | Rule::math_cmd => depth + 1,
        Rule::match_expr | Rule::let_rec_matcharm => depth + 1,
        Rule::let_rec_inner => depth + 1,
        Rule::sig_stmt | Rule::struct_stmt => depth + 1,
        _ => depth,
    };
    let start_indent = "\n".to_string() + &indent_space(new_depth);
    let end_indent = "\n".to_string() + &indent_space(depth);

    let output = to_string_cst_inner(text, cst, new_depth);
    let self_text = text.get(cst.span.start..cst.span.end).unwrap().to_string();

    use satysfi_parser::Rule;
    // 中身をそのまま返すものは output をそのまま返す
    // self_text は元の文字列をそのまま返したいときに使用
    match cst.rule {
        Rule::comments => to_comment_string(self_text) + &end_indent,
        // header
        // stage の次は必ず改行する
        Rule::stage => "@stage: ".to_string() + self_text.trim() + "\n\n",
        // headers があれば必ず改行する
        Rule::headers => {
            if output.len() > 0 {
                output + "\n"
            } else {
                output
            }
        }
        Rule::header_require => "@require: ".to_string() + &output + "\n",
        Rule::header_import => "@import: ".to_string() + &output + "\n",
        // 末尾のスペースなどは削除 (スペースで終わるpkgnameが導入されるとバグるけれど無いでしょう……)
        Rule::pkgname => self_text.trim().to_string(),

        // statement
        Rule::let_stmt => format!("{} {output}", RESERVED_WORD.let_stmt),
        Rule::let_rec_stmt => format!("{} {output}", RESERVED_WORD.let_rec),
        Rule::let_rec_inner => output,
        Rule::let_rec_matcharm => output,
        Rule::let_inline_stmt_ctx => {
            format!("{} {output}", RESERVED_WORD.let_inline)
        }
        Rule::let_inline_stmt_noctx => {
            format!("{} {output}", RESERVED_WORD.let_inline)
        }
        Rule::let_block_stmt_ctx => format!("{} {output}", RESERVED_WORD.let_block),
        Rule::let_block_stmt_noctx => {
            format!("{} {output}", RESERVED_WORD.let_block)
        }
        Rule::let_math_stmt => format!("{} {}", RESERVED_WORD.let_math, output),
        Rule::let_mutable_stmt => format!("{} {}", RESERVED_WORD.let_mutable, output),
        Rule::type_stmt => format!("{} {}", RESERVED_WORD.type_stmt, output),
        Rule::type_inner => output,
        Rule::type_variant => output,
        Rule::module_stmt => format!("{start_indent}{} {}", RESERVED_WORD.module, output),
        Rule::open_stmt => format!("{} {output}", RESERVED_WORD.open),
        Rule::arg => self_text,

        // struct
        Rule::sig_stmt => format!(
            "{}{output}{end_indent}{}",
            RESERVED_WORD.sig, RESERVED_WORD.end
        ), // TODO
        Rule::struct_stmt => format!(
            "{}{start_indent}{output}{end_indent}{}",
            RESERVED_WORD.struct_stmt, RESERVED_WORD.end
        ), // TODO
        Rule::sig_type_stmt => format!("{start_indent}{} {output}", RESERVED_WORD.type_stmt),
        Rule::sig_val_stmt => format!("{start_indent}{} {output}", RESERVED_WORD.val),
        Rule::sig_direct_stmt => format!("{start_indent}{} {output}", RESERVED_WORD.direct),

        // types
        Rule::type_expr => output,
        Rule::type_optional => output,
        Rule::type_prod => output,
        Rule::type_inline_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.inline_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.inline_command)
            }
        }
        Rule::type_block_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.block_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.block_command)
            }
        }
        Rule::type_math_cmd => {
            if output.contains("\n") {
                format!(
                    "[{start_indent}{output};{end_indent}] {}",
                    RESERVED_WORD.math_command
                )
            } else {
                format!("[{output}] {}", RESERVED_WORD.math_command)
            }
        }
        Rule::type_list_unit_optional => output + "?",
        Rule::type_application => output,
        Rule::type_name => self_text,
        // Rule::type_record => output,
        Rule::type_record_unit => output,
        Rule::type_param => format!("'{output}"),
        Rule::constraint => format!("{} {output}", RESERVED_WORD.constraint),

        // unary
        Rule::unary => output,
        Rule::unary_prefix => self_text,
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
        // Rule::horizontal_text => output,
        Rule::math_text => self_text,
        Rule::list => {
            if output.len() > 0 {
                format!("[{start_indent}{output}{end_indent}]")
            } else {
                format!("[]")
            }
        }
        Rule::record | Rule::type_record => {
            if cst.inner.len() > 1 {
                // 2 つ以上のときは改行
                format!("(|{start_indent}{output};{end_indent}|)")
            } else {
                // 1つだけの時は、改行しない
                format!("(|{output}|)")
            }
        }
        Rule::record_unit => output,
        Rule::tuple => self_text,
        Rule::bin_operator => {
            if self_text == "|>" {
                // 1つ深くする
                format!("{start_indent}{}{self_text}", indent_space(1))
            } else {
                self_text
            }
        }
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
        Rule::cmd_expr_option => {
            if self_text == ("?*") {
                "?*".to_string()
            } else {
                format!("?:{output}")
            }
        }
        Rule::cmd_text_arg | Rule::horizontal_text => {
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
        Rule::math_cmd_expr_arg => output,
        Rule::math_cmd_expr_option => format!(":?{output}"),

        // pattern
        Rule::pat_as => output,
        Rule::pat_cons => output,
        Rule::pattern => self_text, // TODO どのパターンでも中身をそのまま出力
        Rule::pat_variant => output,
        Rule::pat_list => output,
        Rule::pat_tuple => output,

        // expr
        Rule::expr => {
            if self_text.ends_with(";") {
                output + ";"
            } else {
                output
            }
        }
        Rule::match_expr => output, // TODO
        Rule::match_arm => output,  // TODO
        Rule::match_guard => format!("{} {output}", RESERVED_WORD.when), // TODO
        Rule::bind_stmt => output,  // TODO
        Rule::ctrl_while => output, // TODO
        Rule::ctrl_if => output,    // TODO
        Rule::lambda => output,     // TODO
        Rule::assignment => output, // TODO
        Rule::dyadic_expr => output, // TODO
        Rule::unary_operator_expr => output, // TODO
        Rule::unary_operator => output, // TODO
        // application
        Rule::application => output,
        Rule::application_args_normal => output,
        Rule::application_args_optional => {
            if self_text == ("?*") {
                "?*".to_string()
            } else {
                format!("?:{output}")
            }
        }
        Rule::command_application => format!("{} {output}", RESERVED_WORD.command),
        Rule::variant_constructor => output,

        // horizontal
        Rule::horizontal_single => output,
        Rule::horizontal_list => output,        // TODO
        Rule::horizontal_bullet_list => output, // TODO
        Rule::horizontal_bullet => output,      // TODO
        Rule::horizontal_bullet_star => "  ".repeat(self_text.len() - 1) + &self_text + " ",
        Rule::regular_text => {
            let sep = format!("\n{}", indent_space(depth));
            let output = self_text
                .split("\n")
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<String>>()
                .join(&sep);
            let start_space = if self_text.starts_with(char::is_whitespace) {
                " "
            } else {
                ""
            };
            let end_space = if self_text.ends_with(char::is_whitespace) {
                " "
            } else {
                ""
            };
            if output.trim().is_empty() {
                if self_text.starts_with("\n") {
                    start_indent
                } else {
                    start_space.to_string()
                }
            } else {
                let end_newline = self_text.trim_end_matches(&['\t', ' ']) != self_text.trim_end();
                match (self_text.starts_with("\n"), end_newline) {
                    (true, true) => format!("{start_indent}{output}{end_indent}"),
                    (true, false) => format!("{start_indent}{output}{end_space}"),
                    (false, true) => format!("{start_space}{output}{end_indent}"),
                    (false, false) => format!("{start_space}{output}{end_space}"),
                }
            }
        }
        Rule::horizontal_escaped_char => self_text, // TODO
        Rule::inline_text_embedding => format!("#{output};"), // TODO

        // vertical
        Rule::vertical => output,             // TODO
        Rule::block_text_embedding => self_text, // TODO

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
        Rule::preamble => output.trim_start().to_string(),
        // TODO
        // dummy
        Rule::dummy_header => self_text,
        Rule::dummy_sig_stmt => self_text,
        Rule::dummy_stmt => self_text,
        Rule::dummy_inline_cmd_incomplete => self_text,
        Rule::dummy_block_cmd_incomplete => self_text,
        Rule::dummy_modvar_incomplete => self_text,
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
