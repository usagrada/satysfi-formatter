//! コメントによる整形の制御 (clang-format の `// clang-format off` / `on` と同じ考え方)。
//!
//! * `% satysfi-fmt: off` から `% satysfi-fmt: on` までの文字列は、整形せずに元のまま出力する。
//!   空白・改行・字下げ・間にあるコメントも元のまま残る。ノードや行の途中に書いてもよい。
//! * `off` / `on` のコメント自体は、通常どおり整形 (字下げ) される。
//! * `on` がなければ、ファイルの最後まで整形しない。
//! * `% satysfi-fmt: off: 理由` のように、`:` の後ろに説明を書いてよい。
//!
//! いったん全体を整形してから、`off` と `on` の間を整形前の文字列に戻す ([`restore_off_regions`])。

use satysfi_parser::{grammar, CstText, Item, Span};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Directive {
    Off,
    On,
}

/// コメント文字列 (`%` を含む) から directive を取り出す。
pub fn parse(comment: &str) -> Option<Directive> {
    let body = comment.trim_start_matches('%').trim();
    let rest = body.strip_prefix("satysfi-fmt:")?;
    // `off: 理由` のような説明は読み飛ばす
    match rest.split(':').next()?.trim() {
        "off" => Some(Directive::Off),
        "on" => Some(Directive::On),
        _ => None,
    }
}

/// ファイル中のすべてのコメントの位置 (先頭から順に)。
pub(crate) fn comment_spans(csttext: &CstText) -> Vec<Span> {
    fn collect(csttext: &CstText, node: &satysfi_parser::Cst, spans: &mut Vec<Span>) {
        for item in csttext.items(node) {
            match item {
                Item::Comment { span, .. } => spans.push(span),
                Item::Node(child) => collect(csttext, child, spans),
                Item::BlankLines(_) => {}
            }
        }
    }
    let mut spans = vec![];
    collect(csttext, &csttext.cst, &mut spans);
    spans.sort_by_key(|span| span.start);
    spans.dedup();
    spans
}

/// `off` と、それを閉じる `on` のコメントの番号の組。`on` がなければファイルの最後まで。
fn off_regions(text: &str, comments: &[Span]) -> Vec<(usize, Option<usize>)> {
    let mut regions = vec![];
    let mut open: Option<usize> = None;
    for (i, span) in comments.iter().enumerate() {
        match (parse(&text[span.start..span.end]), open) {
            (Some(Directive::Off), None) => open = Some(i),
            (Some(Directive::On), Some(off)) => {
                regions.push((off, Some(i)));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(off) = open {
        regions.push((off, None));
    }
    regions
}

/// `pos` の直前にある空白 (改行を除く) を読み飛ばした位置。
fn back_over_spaces(text: &str, pos: usize) -> usize {
    text[..pos].trim_end_matches([' ', '\t']).len()
}

/// 整形で変わってよい違い (空白、`;` の有無、型定義の先頭の `|`) を除いた文字列。
fn significant(text: &str) -> String {
    let text: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect();
    text.replace("=|", "=")
}

/// 整形結果 `output` の `% satysfi-fmt: off` から `on` までを、整形前 (`input`) の文字列に戻す。
///
/// 整形でコメントとコードの並びが変わり、範囲を正しく戻せないときは、
/// その `off` コメントの行 (0 始まり) を返す。
pub fn restore_off_regions(input: &CstText, output: &str) -> Result<String, usize> {
    let src = input.text.as_str();
    let in_comments = comment_spans(input);
    let regions = off_regions(src, &in_comments);
    if regions.is_empty() {
        return Ok(output.to_string());
    }
    let line_of = |comment: usize| src[..in_comments[comment].start].matches('\n').count();

    let formatted = CstText::parse(output, grammar::program).map_err(|_| line_of(regions[0].0))?;
    let out_comments = comment_spans(&formatted);
    if out_comments.len() != in_comments.len() {
        return Err(line_of(regions[0].0));
    }

    let mut result = String::with_capacity(output.len());
    let mut out_pos = 0;
    for (off, on) in regions {
        // off / on のコメントが、整形の前後で対応していること
        let same_comment = |i: usize| {
            significant(&src[in_comments[i].start..in_comments[i].end])
                == significant(&output[out_comments[i].start..out_comments[i].end])
        };
        if !same_comment(off) || !on.map_or(true, same_comment) {
            return Err(line_of(off));
        }
        let in_start = in_comments[off].end;
        let out_start = out_comments[off].end;
        // 範囲の終わり: `on` コメントの前の空白の手前 (なければファイルの最後)
        let (in_end, out_end) = match on {
            Some(on) => (
                back_over_spaces(src, in_comments[on].start),
                back_over_spaces(output, out_comments[on].start),
            ),
            None => (src.len(), output.len()),
        };
        let region = &src[in_start..in_end];
        // 戻す範囲のコードが、整形結果の同じ範囲と同じであること
        if significant(region) != significant(&output[out_start..out_end]) {
            return Err(line_of(off));
        }
        result.push_str(&output[out_pos..out_start]);
        result.push_str(region);
        out_pos = match on {
            Some(on) => {
                let on_start = out_comments[on].start;
                if region.ends_with('\n') {
                    // 単独行の `on` は、整形結果の字下げに合わせる
                    let line_start = output[..on_start].rfind('\n').map_or(0, |i| i + 1);
                    let indent = &output[line_start..on_start];
                    if indent.trim().is_empty() {
                        result.push_str(indent);
                    } else {
                        result.push_str(&src[in_end..in_comments[on].start]);
                    }
                } else {
                    // 行末の `on` は、コードとの間を空白 1 つにする
                    result.push(' ');
                }
                on_start
            }
            None => output.len(),
        };
    }
    result.push_str(&output[out_pos..]);

    // 戻した結果が正しい SATySFi のコードであること
    if CstText::parse(&result, grammar::program).is_err() {
        return Err(line_of(off_regions(src, &in_comments)[0].0));
    }
    Ok(result)
}

/// 元の文字列を壊さずに出力するための目印。
/// 行頭の目印の前にあるインデントは `strip_markers` で取り除かれる。
const FIRST: char = '\u{E002}';
const CONT: char = '\u{E000}';
const END: char = '\u{E001}';

/// 整形しない文字列を、後処理で元のまま復元できる形にする。
pub fn verbatim(text: &str) -> String {
    let text = text.trim_end_matches(['\n', '\r']);
    text.split('\n')
        .enumerate()
        .map(|(i, line)| {
            let line = line.trim_end_matches('\r');
            format!("{}{line}{END}", if i == 0 { FIRST } else { CONT })
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `verbatim` の 2 行目以降か。この行の目印より前のインデントは `strip_markers` で取り除かれる。
pub fn is_verbatim_continuation(line: &str) -> bool {
    line.contains(CONT)
}

/// `verbatim` の目印を取り除く。2 行目以降は、Doc が付けたインデントも取り除く。
pub fn strip_markers(output: &str) -> String {
    output
        .split('\n')
        .map(|line| match line.find(CONT) {
            Some(i) => line[i + CONT.len_utf8()..].to_string(),
            None => line.to_string(),
        })
        .map(|line| line.replace([FIRST, END], ""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_directive() {
        // skip は対応していない (普通のコメント)
        assert_eq!(parse("% satysfi-fmt: skip"), None);
        assert_eq!(parse("%satysfi-fmt:off"), Some(Directive::Off));
        assert_eq!(parse("%%  satysfi-fmt:  on  "), Some(Directive::On));
        assert_eq!(parse("% satysfi-fmt: unknown"), None);
        // `:` の後ろの説明は読み飛ばす
        assert_eq!(
            parse("% satysfi-fmt: off: aligned table"),
            Some(Directive::Off)
        );
        assert_eq!(parse("% satysfi-fmt: offset"), None);
        assert_eq!(parse("% other: skip"), None);
    }

    #[test]
    fn region_whose_code_changed_cannot_be_restored() {
        let input = CstText::parse(
            "let a = 1\n% satysfi-fmt: off\nlet b = 2\n",
            grammar::program,
        )
        .unwrap();
        // 範囲内のコードが入力と違う整形結果 (コメントが別の位置に動いた場合など)
        let output = "let a = 1\n% satysfi-fmt: off\nlet c = 2\n";
        assert_eq!(restore_off_regions(&input, output), Err(1));
        // コードが同じなら、空白だけ入力に戻す
        let output = "let a = 1\n% satysfi-fmt: off\nlet b    =    2\n";
        assert_eq!(
            restore_off_regions(&input, output).as_deref(),
            Ok("let a = 1\n% satysfi-fmt: off\nlet b = 2\n")
        );
    }

    #[test]
    fn verbatim_roundtrip() {
        let v = verbatim("  a  \n    b  ");
        // Doc が付けるインデントを想定
        let indented = v.replace('\n', "\n        ");
        assert_eq!(strip_markers(&indented), "  a  \n    b  ");
    }
}
