use crate::FormatConfig;

mod comment;
mod comment_position;
mod common;
mod config;
mod ctrl_stmt;
mod directive;
mod exprs;
mod horizontal_single;
mod let_block;
#[cfg(feature = "lsp")]
mod lsp_api;
mod math;
mod math_mode;
mod module;
mod options;
mod path_expr;
mod patterns;
mod printer;
mod space;
mod stmts;
mod syntax;
mod text_mode;
mod types;

/// 行幅を指定した設定
pub(crate) fn config_with_width(width: usize) -> FormatConfig {
    FormatConfig {
        line_width: width,
        ..Default::default()
    }
}

/// 整形する。構文エラーのときは入力をそのまま返す。
pub(crate) fn format(input: &str, config: &FormatConfig) -> String {
    crate::format(input, config).unwrap_or_else(|_| input.to_string())
}

/// 行幅を指定して整形し、期待する出力と一致し、冪等であることを確認する。
fn test_width(input: &str, width: usize, expect: &str) {
    let config = config_with_width(width);
    let output = format(input, &config);
    assert_eq!(output, expect, "width = {width}");
    assert_eq!(
        format(&output, &config),
        output,
        "not idempotent (width = {width})"
    );
}

fn test_tmpl(input: &str, expect: &str) {
    let config = FormatConfig::default();
    let output = format(input, &config);
    assert_eq!(output, expect);
    // 冪等性: 整形結果をもう一度整形しても変わらない
    assert_eq!(format(&output, &config), output, "not idempotent");
}

/// `%` から行末までのコメントの開始位置 (文字列やエスケープの中の `%` は除く)
fn comment_start(line: &str) -> Option<usize> {
    let mut escaped = false;
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '`' {
            in_string = !in_string;
        } else if c == '%' && !in_string {
            return Some(i);
        }
    }
    None
}

/// コメント・空白・`;` を除いた文字列。整形で並びが変わっていないことの確認に使う。
fn tokens(text: &str) -> String {
    let text: String = text
        .lines()
        .map(|line| &line[..comment_start(line).unwrap_or(line.len())])
        .flat_map(|line| line.chars())
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect();
    // 型定義の先頭の `|` は省略できる
    text.replace("=|", "=")
}

/// `%` から行末までのコメントの一覧 (空白を除く)
fn comments(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| comment_start(line).map(|i| &line[i..]))
        .map(|comment| comment.chars().filter(|c| !c.is_whitespace()).collect())
        .collect()
}

/// どんな行幅で整形しても成り立つ性質を確認する。
/// * 整形結果をもう一度整形しても変わらない
/// * 空白と `;` を除いたトークン列が変わらない
/// * コメントが増減しない
pub(crate) fn check_invariants(name: &str, input: &str) {
    for width in [20, 40, 80, 200] {
        let config = config_with_width(width);
        let output = format(input, &config);
        assert_eq!(
            output,
            format(&output, &config),
            "{name}@{width}: not idempotent"
        );
        assert_eq!(
            tokens(&output),
            tokens(input),
            "{name}@{width}: tokens changed"
        );
        assert_eq!(
            comments(&output),
            comments(input),
            "{name}@{width}: comments changed"
        );
    }
}

/// ソース中のあらゆる空白の位置にコメントを挿入して整形しても、
/// コメントが失われず、整形結果が冪等で、トークン列が変わらないことを確認する。
/// 挿入によってパースできなくなる位置は飛ばす。確認した位置の数を返す。
pub(crate) fn check_comment_insertion(name: &str, src: &str) -> usize {
    use satysfi_parser::{CstText, grammar};
    let config = FormatConfig::default();
    let mut checked = 0;
    let mut failures = vec![];
    let mut in_string = false;
    for (i, ch) in src.char_indices() {
        if ch == '`' {
            in_string = !in_string;
        }
        if in_string || !(ch == ' ' || ch == '\n') {
            continue;
        }
        // 既存のコメントの中には挿入しない
        let line_start = src[..i].rfind('\n').map_or(0, |k| k + 1);
        if src[line_start..i].contains('%') {
            continue;
        }
        let modified = format!("{}%cmt{}\n{}", &src[..i], checked, &src[i..]);
        if CstText::parse(&modified, grammar::program).is_err() {
            continue;
        }
        checked += 1;
        let output = format(&modified, &config);
        let mut expected_comments = comments(&modified);
        expected_comments.sort();
        let mut actual_comments = comments(&output);
        actual_comments.sort();
        let problem = if actual_comments != expected_comments {
            Some("comments changed")
        } else if format(&output, &config) != output {
            Some("not idempotent")
        } else if tokens(&output) != tokens(&modified) {
            Some("tokens changed")
        } else {
            None
        };
        if let Some(problem) = problem {
            let context: String = src[..i]
                .chars()
                .rev()
                .take(24)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            failures.push(format!(
                "{name}: {problem} after {context:?}\n--- input\n{modified}\n--- output\n{output}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed.\n{}\nfirst:\n{}",
        failures.len(),
        checked,
        failures
            .iter()
            .map(|f| f.lines().next().unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        failures.first().unwrap()
    );
    checked
}

/// コメントを挿入できる位置 (文字列とコメントの外にある空白の位置)。
fn insertion_points(src: &str) -> Vec<usize> {
    let mut points = vec![];
    let mut in_string = false;
    for (i, ch) in src.char_indices() {
        if ch == '`' {
            in_string = !in_string;
        }
        if in_string || !(ch == ' ' || ch == '\n') {
            continue;
        }
        let line_start = src[..i].rfind('\n').map_or(0, |k| k + 1);
        if !src[line_start..i].contains('%') {
            points.push(i);
        }
    }
    points
}

/// 行頭の位置 (文字列の外)。
fn line_starts(src: &str) -> Vec<usize> {
    let mut points = vec![0];
    let mut in_string = false;
    for (i, ch) in src.char_indices() {
        if ch == '`' {
            in_string = !in_string;
        }
        if ch == '\n' && !in_string && i + 1 < src.len() {
            points.push(i + 1);
        }
    }
    points
}

/// `check_off_on_insertion` で off / on を置く位置。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    /// 行頭に単独行のコメントとして置く (普通の書き方)。必ず整形できること
    OwnLine,
    /// 空白のあるあらゆる位置に行末のコメントとして置く。
    /// 構文の途中では整形でコメントの位置が変わるので、範囲を広げて元のまま戻す
    Anywhere,
    /// 既存の行の末尾に置く (普通の書き方)。必ず整形できること
    LineEnd,
}

/// `off` / `on` のコメントの前にあるコード (コメント・空白・`;` を除く) が、整形の前後で同じか。
/// 整形でコメントがコードに対して動いていないことの確認に使う。
pub(crate) fn directives_keep_their_position(input: &str, output: &str) -> bool {
    fn directive_contexts(text: &str) -> Vec<String> {
        let mut result = vec![];
        let mut code = String::new();
        for line in text.lines() {
            match comment_start(line) {
                Some(i) => {
                    code.push_str(&line[..i]);
                    let comment = &line[i..];
                    if comment.contains("satysfi-fmt ") {
                        result.push(tokens(&code));
                    }
                }
                None => code.push_str(line),
            }
        }
        result
    }
    directive_contexts(input) == directive_contexts(output)
}

/// `% satysfi-fmt off` / `on` の組をいろいろな位置に挿入して整形し、次を確認する。
/// * off と on の間の文字列が元のまま残る
/// * `off` / `on` のコメントが元の位置から動かない
/// * 整形結果が冪等で、トークン列とコメントが変わらない
/// * どの位置でもエラーにならない (範囲を戻せないときは、ノードの並びを丸ごと元のまま出力する)
///
/// 挿入によってパースできなくなる組は飛ばす。確認した組の数を返す。
pub(crate) fn check_off_on_insertion(name: &str, src: &str, placement: Placement) -> usize {
    use satysfi_parser::{CstText, grammar};
    let config = FormatConfig::default();
    let (points, off_text, on_text) = match placement {
        Placement::OwnLine => (line_starts(src), "% satysfi-fmt off", "% satysfi-fmt on"),
        Placement::Anywhere => (
            insertion_points(src),
            "%satysfi-fmt off",
            "% satysfi-fmt on",
        ),
        Placement::LineEnd => (
            insertion_points(src)
                .into_iter()
                .filter(|&i| src[i..].starts_with('\n'))
                .collect(),
            " % satysfi-fmt off",
            " % satysfi-fmt on",
        ),
    };
    let mut checked = 0;
    let mut failures = vec![];
    for (n, &i) in points.iter().enumerate() {
        // `Anywhere` は位置が多いので、off は 1 つおきに置く
        if placement == Placement::Anywhere && n % 2 == 1 {
            continue;
        }
        // on は off の 1・4・16 個先の位置に置く。最後の組は on を置かない (ファイルの最後まで)。
        // `Anywhere` は位置が多いので 1・16 個先だけ
        let distances: &[usize] = if placement == Placement::Anywhere {
            &[1, 16]
        } else {
            &[1, 4, 16]
        };
        let ons = distances
            .iter()
            .map(|k| points.get(n + k).copied())
            .collect::<Vec<_>>();
        for on in ons {
            let modified = match on {
                Some(j) => format!(
                    "{}{off_text}\n{}{on_text}\n{}",
                    &src[..i],
                    &src[i..j],
                    &src[j..]
                ),
                None => format!("{}{off_text}\n{}", &src[..i], &src[i..]),
            };
            let csttext = match CstText::parse(&modified, grammar::program) {
                Ok(csttext) => csttext,
                Err(_) => continue,
            };
            // ヘッダのパッケージ名の直後などでは、コメントにならない
            let starts: Vec<usize> = crate::printer::directive::comment_spans(&csttext)
                .iter()
                .map(|span| span.start)
                .collect();
            let indent = |text: &str| text.len() - text.trim_start().len();
            let off_start = i + indent(off_text);
            let on_start = on.map(|j| j + off_text.len() + 1 + indent(on_text));
            if !starts.contains(&off_start) || on_start.is_some_and(|k| !starts.contains(&k)) {
                continue;
            }
            checked += 1;
            // 元のまま残るべき範囲: off コメントの直後から、on コメントの前の空白の手前まで
            let region_start = i + off_text.len();
            let region = match on_start {
                Some(k) => modified[region_start..k].trim_end_matches([' ', '\t']),
                None => &modified[region_start..],
            };
            let problem = match crate::format(&modified, &config) {
                Err(err) => Some(err.to_string()),
                Ok(output) => {
                    let mut expected_comments = comments(&modified);
                    expected_comments.sort();
                    let mut actual_comments = comments(&output);
                    actual_comments.sort();
                    // 構文の途中に置いたときは、ノード単位で戻すので、範囲の前後の空白や
                    // ノードの外にある記号 (`:` `=` など) までは元のまま保てない。
                    let kept = region.trim_end_matches('\n');
                    let again = |text: &str| crate::format(text, &config).unwrap_or_default();
                    if placement != Placement::Anywhere && !output.contains(kept) {
                        Some("region changed".to_string())
                    } else if actual_comments != expected_comments {
                        Some("comments changed".to_string())
                    } else if !directives_keep_their_position(&modified, &output) {
                        Some("off / on moved".to_string())
                    } else if again(&output) != output {
                        Some("not idempotent".to_string())
                    } else if tokens(&output) != tokens(&modified) {
                        Some("tokens changed".to_string())
                    } else {
                        None
                    }
                }
            };
            if let Some(problem) = problem {
                let around = |k: usize| {
                    let from = modified[..k]
                        .char_indices()
                        .rev()
                        .nth(16)
                        .map_or(0, |(p, _)| p);
                    let to = modified[k..]
                        .char_indices()
                        .nth(40)
                        .map_or(modified.len(), |(p, _)| k + p);
                    format!("{:?}", &modified[from..to])
                };
                let on_context = on_start.map_or("(none)".to_string(), around);
                failures.push(format!(
                    "{name}: {problem}\n  off: {}\n  on:  {on_context}\n--- input\n{modified}",
                    around(i)
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed.\n{}\nfirst:\n{}",
        failures.len(),
        checked,
        failures
            .iter()
            .map(|f| f.lines().take(3).collect::<Vec<_>>().join("\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        failures.first().unwrap()
    );
    checked
}

#[test]
fn test_unicode() {
    let text = r#"

document(||)'<
+section{ section }<
+p{日本語}
>>"#;

    let expect = r#"document(||)'<
    +section{ section }<
        +p{ 日本語 }
    >
>
"#;
    test_tmpl(text, expect)
}
