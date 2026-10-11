//! コメントによる整形の制御 (clang-format の `// clang-format off` / `on` と同じ考え方)。
//!
//! * `% satysfi-fmt off` から `% satysfi-fmt on` までの文字列は、整形せずに元のまま出力する。
//!   空白・改行・字下げ・間にあるコメントも元のまま残る。ノードや行の途中に書いてもよい。
//! * `off` / `on` のコメント自体は、通常どおり整形 (字下げ) される。
//! * `on` がなければ、ファイルの最後まで整形しない。
//! * `% satysfi-fmt off: 理由` のように、`off` / `on` の後ろに `:` で説明を書いてよい。
//!
//! いったん全体を整形してから、`off` と `on` の間を整形前の文字列に戻す ([`restore_off_regions`])。
//!
//! 構文の途中 (`let-rec` と関数名の間など) に書くと、整形でコメントの位置が変わって
//! 範囲を正確には戻せないことがある。その場合は、`off` と `on` の位置は変えないまま、
//! 整形しない範囲を広げる。`off` と `on` を含む同じ階層のノードの並び
//! (文の並びなら、`off` を含む文から `on` を含む文まで) を丸ごと元のまま出力し、
//! それでも `off` / `on` が元の位置に残らなければ 1 つ上の階層へ広げ、
//! 最後はファイル全体を元のまま返す。そのため整形がエラーになることはない。

use satysfi_parser::{Cst, CstText, Item, Span, grammar};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Directive {
    Off,
    On,
}

/// コメント文字列 (`%` を含む) から directive を取り出す。
///
/// `% satysfi-fmt off` / `% satysfi-fmt on` の形のコメントだけが対象。
/// `%` はいくつ重ねてもよい (`%% satysfi-fmt off`)。
/// コメントの途中に書いたものや、他の文章が続くものは普通のコメントとして扱う。
pub fn parse(comment: &str) -> Option<Directive> {
    let body = comment.strip_prefix('%')?.trim_start_matches('%');
    // `satysfi-fmt` と `off` / `on` の間には空白が要る (`satysfi-fmt: off` や `satysfi-fmt-off` は対象外)
    let rest = body.trim().strip_prefix("satysfi-fmt")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
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

/// 整形結果 `output` の `% satysfi-fmt off` から `on` までを、整形前 (`input`) の文字列に戻す。
///
/// 範囲を正確に戻せないときは、`off` と `on` を含むノードの並びを丸ごと元のまま出力する
/// (さらに戻せなければ階層を広げ、最後はファイル全体を元のまま返す)。
pub fn restore_off_regions(input: &CstText, output: &str) -> String {
    match restore_precisely(input, output) {
        Ok(result) => result,
        Err(_) => restore_by_nodes(input, output),
    }
}

/// 整形結果の `off` から `on` までの文字列だけを、整形前の文字列に戻す。
///
/// 整形でコメントとコードの並びが変わり、範囲を正しく戻せないときは、
/// その `off` コメントの行 (0 始まり) を返す。
pub(crate) fn restore_precisely(input: &CstText, output: &str) -> Result<String, usize> {
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
        if !same_comment(off) || !on.is_none_or(same_comment) {
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

/// 元に戻す範囲。同じ親を持つノードの並び (`first` から `last` まで)、またはファイル全体。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Region {
    Whole,
    /// 戻すノードがない (`off` と `on` の間に、キーワードなどノード以外しかない)。
    /// `node` は両方のコメントを含む最小のノードへの道筋。コメントが動いたときは、このノードを戻す。
    Empty {
        node: Vec<usize>,
    },
    /// ルートから `path` の順に子をたどったノードの、`first` 番目から `last` 番目の子
    Children {
        path: Vec<usize>,
        first: usize,
        last: usize,
    },
}

impl Region {
    /// 1 つ上の階層に広げる。ルートの子の並びを広げるとファイル全体になる。
    fn widen(&self) -> Region {
        match self {
            Region::Whole => Region::Whole,
            Region::Empty { node: path } | Region::Children { path, .. } => match path.split_last()
            {
                None => Region::Whole,
                Some((index, parent)) => Region::Children {
                    path: parent.to_vec(),
                    first: *index,
                    last: *index,
                },
            },
        }
    }

    /// 範囲が覆う文字列の位置 (`root` はこの範囲を求めた木)。
    fn span(&self, root: &Cst) -> Option<(usize, usize)> {
        match self {
            Region::Whole | Region::Empty { .. } => None,
            Region::Children { path, first, last } => {
                let node = node_at(root, path)?;
                Some((
                    node.inner.get(*first)?.span.start,
                    node.inner.get(*last)?.span.end,
                ))
            }
        }
    }

    /// 2 つの範囲を覆う、最も小さい範囲。
    fn merge(&self, other: &Region) -> Region {
        let (a, b) = match (self, other) {
            (
                Region::Children {
                    path: a,
                    first: af,
                    last: al,
                },
                Region::Children {
                    path: b,
                    first: bf,
                    last: bl,
                },
            ) => ((a, *af, *al), (b, *bf, *bl)),
            _ => return Region::Whole,
        };
        let common =
            a.0.iter()
                .zip(b.0.iter())
                .take_while(|(x, y)| x == y)
                .count();
        // 共通の祖先ノードの、どの子に属するか (その祖先自身の子の範囲ならそのまま)
        let range_at = |r: (&Vec<usize>, usize, usize)| -> (usize, usize) {
            if r.0.len() == common {
                (r.1, r.2)
            } else {
                (r.0[common], r.0[common])
            }
        };
        let (a_range, b_range) = (range_at(a), range_at(b));
        Region::Children {
            path: a.0[..common].to_vec(),
            first: a_range.0.min(b_range.0),
            last: a_range.1.max(b_range.1),
        }
    }
}

fn node_at<'a>(root: &'a Cst, path: &[usize]) -> Option<&'a Cst> {
    path.iter().try_fold(root, |node, &i| node.inner.get(i))
}

/// `off` コメントから `on` コメント (なければファイルの最後) までを含む、最も小さいノードの並び。
/// 範囲にノードが 1 つもなければ `Region::Empty`。
fn initial_region(root: &Cst, off: Span, on: Option<Span>) -> Region {
    // `on` がないときは、ファイルの最後のコードまで及ぶ
    let code_end = root.inner.iter().map(|c| c.span.end).max().unwrap_or(0);
    let (lo, hi) = (off.start, on.map_or(code_end, |span| span.end));
    let mut path = vec![];
    let mut node = root;
    while let Some(k) = node
        .inner
        .iter()
        .position(|c| c.span.start <= lo && c.span.end >= hi && c.span.start < c.span.end)
    {
        path.push(k);
        node = &node.inner[k];
    }
    let non_empty = |c: &Cst| c.span.start < c.span.end;
    let first = node
        .inner
        .iter()
        .position(|c| non_empty(c) && c.span.end > lo);
    let last = node
        .inner
        .iter()
        .rposition(|c| non_empty(c) && c.span.start < hi);
    match (first, last) {
        (Some(first), Some(last)) if first <= last => Region::Children { path, first, last },
        _ => Region::Empty { node: path },
    }
}

/// 整形後の木 `out` の中で、`region` に対応する文字列の位置。
/// 木の形が入力と違うときは `None`。
fn output_span(input: &Cst, out: &Cst, region: &Region) -> Option<(usize, usize)> {
    let Region::Children { path, first, last } = region else {
        return None;
    };
    let (mut a, mut b) = (input, out);
    for &i in path {
        if a.rule != b.rule || a.inner.len() != b.inner.len() {
            return None;
        }
        a = a.inner.get(i)?;
        b = b.inner.get(i)?;
    }
    if a.rule != b.rule || a.inner.len() != b.inner.len() {
        return None;
    }
    let same_rules = (*first..=*last).all(|i| a.inner[i].rule == b.inner[i].rule);
    if !same_rules {
        return None;
    }
    Some((b.inner[*first].span.start, b.inner[*last].span.end))
}

/// 重なる範囲を 1 つにまとめる。
fn merge_regions(root: &Cst, mut regions: Vec<Region>) -> Vec<Region> {
    loop {
        let spans: Vec<Option<(usize, usize)>> = regions.iter().map(|r| r.span(root)).collect();
        let mut merged = false;
        'search: for i in 0..regions.len() {
            for j in i + 1..regions.len() {
                let overlap = match (&regions[i], &regions[j], spans[i], spans[j]) {
                    // 戻すノードがない範囲は、他の範囲と重ならない
                    (Region::Empty { .. }, _, _, _) | (_, Region::Empty { .. }, _, _) => false,
                    (_, _, Some(a), Some(b)) => a.0 < b.1 && b.0 < a.1,
                    _ => true,
                };
                if overlap {
                    let m = regions[i].merge(&regions[j]);
                    regions.remove(j);
                    regions[i] = m;
                    merged = true;
                    break 'search;
                }
            }
        }
        if !merged {
            return regions;
        }
    }
}

/// `off` から `on` までを含むノードの並びを、丸ごと元の文字列に戻す。
/// 戻した結果が正しいコードにならなければ、1 つ上の階層へ広げる。
fn restore_by_nodes(input: &CstText, output: &str) -> String {
    let src = input.text.as_str();
    let comments = comment_spans(input);
    let formatted = match CstText::parse(output, grammar::program) {
        Ok(formatted) => formatted,
        Err(_) => return src.to_string(),
    };
    // 整形してもコメントの数と順序は変わらないので、k 番目のコメント同士が対応する
    let out_comments = comment_spans(&formatted);
    if out_comments.len() != comments.len() {
        return src.to_string();
    }
    let off_on = off_regions(src, &comments);
    let mut regions: Vec<Region> = off_on
        .iter()
        .map(|(off, on)| initial_region(&input.cst, comments[*off], on.map(|i| comments[i])))
        .collect();
    if regions.is_empty() {
        return output.to_string();
    }
    // 広げるたびに範囲が大きくなり、最後はファイル全体になるので、必ず終わる
    loop {
        regions = merge_regions(&input.cst, regions);
        if regions.contains(&Region::Whole) {
            return src.to_string();
        }
        let mut spans = vec![];
        let mut widened = false;
        for region in regions.iter_mut() {
            // 戻すノードがない範囲は、コメントが動いたときだけ広げる
            if matches!(region, Region::Empty { .. }) {
                continue;
            }
            let in_span = region.span(&input.cst);
            let out_span = output_span(&input.cst, &formatted.cst, region);
            match (in_span, out_span) {
                (Some(in_span), Some(out_span)) => spans.push((in_span, out_span)),
                _ => {
                    *region = region.widen();
                    widened = true;
                }
            }
        }
        if widened {
            continue;
        }
        let result = replace_regions(src, output, &comments, &out_comments, spans);
        // 正しいコードで、コメントが増減していないこと
        let parsed = CstText::parse(&result, grammar::program).ok();
        let result_comments = parsed.as_ref().map(comment_spans);
        // `off` / `on` のコメントが、元の位置 (前にあるコードが同じ位置) から動いていないこと
        let stable = result_comments.as_ref().is_some_and(|spans| {
            spans.len() == comments.len()
                && off_on.iter().all(|(off, on)| {
                    std::iter::once(*off)
                        .chain(*on)
                        .all(|k| code_before(src, &comments, k) == code_before(&result, spans, k))
                })
        });
        if stable {
            return result;
        }
        // 動いてしまうときは、整形しない範囲を 1 つ上の階層に広げて、コメントを元のまま含める
        regions = regions.iter().map(Region::widen).collect();
    }
}

/// `k` 番目のコメントより前にあるコード (コメントと空白、`;` を除く)。
/// コメントが整形でコードに対して動いていないかの確認に使う。
fn code_before(text: &str, comments: &[Span], k: usize) -> String {
    let mut code = String::new();
    let mut pos = 0;
    for span in &comments[..k] {
        code.push_str(&text[pos..span.start]);
        pos = span.end;
    }
    code.push_str(&text[pos..comments[k].start]);
    significant(&code)
}

/// 整形結果 `output` の `out_span` を、入力 `src` の対応する `in_span` に置き換える。
///
/// 範囲に含まれるコメントが、整形で範囲の外に動いていたら、重複しないよう外側のコメントを取り除く。
fn replace_regions(
    src: &str,
    output: &str,
    in_comments: &[Span],
    out_comments: &[Span],
    mut spans: Vec<((usize, usize), (usize, usize))>,
) -> String {
    spans.sort_by_key(|(_, out_span)| out_span.0);
    // (出力中の範囲, 置き換える文字列)
    let mut edits: Vec<(usize, usize, &str)> = spans
        .iter()
        .map(|((in_start, in_end), (out_start, out_end))| {
            (*out_start, *out_end, &src[*in_start..*in_end])
        })
        .collect();
    for (k, in_comment) in in_comments.iter().enumerate() {
        let inside =
            |span: &Span, (start, end): (usize, usize)| span.start >= start && span.end <= end;
        let Some((_, out_span)) = spans
            .iter()
            .find(|(in_span, _)| inside(in_comment, *in_span))
        else {
            continue;
        };
        let out_comment = out_comments[k];
        if inside(&out_comment, *out_span) {
            continue;
        }
        // 範囲の外にある重複したコメントを取り除く (行がコメントだけなら行ごと)
        let line_start = output[..out_comment.start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = output[out_comment.end..]
            .find('\n')
            .map_or(output.len(), |i| out_comment.end + i + 1);
        let before = &output[line_start..out_comment.start];
        let range = if before.trim().is_empty() {
            (line_start, line_end)
        } else {
            (
                out_comment.start - (before.len() - before.trim_end().len()),
                out_comment.end,
            )
        };
        edits.push((range.0, range.1, ""));
    }
    edits.sort_by_key(|(start, _, _)| *start);
    let mut result = String::with_capacity(output.len());
    let mut pos = 0;
    for (start, end, replacement) in edits {
        // 重なる編集は、先にあるものを優先する
        if start < pos {
            continue;
        }
        result.push_str(&output[pos..start]);
        result.push_str(replacement);
        pos = end;
    }
    result.push_str(&output[pos..]);
    result
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
        assert_eq!(parse("% satysfi-fmt skip"), None);
        assert_eq!(parse("%satysfi-fmt off"), Some(Directive::Off));
        assert_eq!(parse("%   satysfi-fmt on  "), Some(Directive::On));
        assert_eq!(parse("% satysfi-fmt off"), Some(Directive::Off));
        assert_eq!(parse("% satysfi-fmt on"), Some(Directive::On));
        // `%` を重ねてもよい
        assert_eq!(parse("%% satysfi-fmt off"), Some(Directive::Off));
        assert_eq!(parse("%%%satysfi-fmt on"), Some(Directive::On));
        // `:` の後ろの説明は読み飛ばす
        assert_eq!(
            parse("% satysfi-fmt off: aligned table"),
            Some(Directive::Off)
        );
        assert_eq!(
            parse("% satysfi-fmt on: end of the table"),
            Some(Directive::On)
        );
    }

    #[test]
    fn other_comments_are_not_directives() {
        for comment in [
            // off / on だけ
            "% off",
            "% on",
            "% format: off",
            "% fmt: off",
            "% clang-format off",
            // 名前や区切りが違う
            "% satysfi-fmt-off",
            "% satysfi-fmt: off",
            "% satysfi-fmt:off",
            "% satysfi-fmt: on",
            "% satysfi-fmt",
            "% satysfi-fmt ",
            "% satysfi-fmt ",
            "% satysfi_fmt: off",
            "% SATYSFI-FMT: OFF",
            "% Satysfi-fmt: off",
            // 続きがある
            "% satysfi-fmt offset",
            "% satysfi-fmt off the table",
            "% satysfi-fmt online",
            "% satysfi-fmt on and off",
            "% satysfi-fmt onto",
            // コメントの途中に書いてある
            "% turn satysfi-fmt off here",
            "% see `% satysfi-fmt off`",
            "% TODO satysfi-fmt on",
            // % を重ねても、続きがあれば普通のコメント
            "%% satysfi-fmt off the table",
            "%%% turn satysfi-fmt on",
            // 空
            "%",
            "% ",
            // 未対応の指示
            "% satysfi-fmt skip",
            "% satysfi-fmt ignore",
            "% satysfi-fmt unknown",
        ] {
            assert_eq!(parse(comment), None, "{comment:?}");
        }
    }

    #[test]
    fn region_whose_code_changed_cannot_be_restored() {
        let input = CstText::parse(
            r#"let a = 1
% satysfi-fmt off
let b = 2
"#,
            grammar::program,
        )
        .unwrap();
        // 範囲内のコードが入力と違う整形結果 (コメントが別の位置に動いた場合など)
        let output = r#"let a = 1
% satysfi-fmt off
let c = 2
"#;
        assert_eq!(restore_precisely(&input, output), Err(1));
        // コードが同じなら、空白だけ入力に戻す
        let output = r#"let a = 1
% satysfi-fmt off
let b    =    2
"#;
        assert_eq!(
            restore_precisely(&input, output).as_deref(),
            Ok(r#"let a = 1
% satysfi-fmt off
let b = 2
"#)
        );
    }

    #[test]
    fn verbatim_roundtrip() {
        let v = verbatim("  a  \n    b  ");
        // Doc が付けるインデントを想定
        let indented = v.replace('\n', "\n        ");
        assert_eq!(strip_markers(&indented), "  a  \n    b  ");
    }

    fn parse_src(text: &str) -> CstText {
        CstText::parse(text, grammar::program).unwrap()
    }

    #[test]
    fn region_widens_one_level_at_a_time() {
        let region = Region::Children {
            path: vec![1, 2],
            first: 0,
            last: 3,
        };
        let wider = region.widen();
        assert_eq!(
            wider,
            Region::Children {
                path: vec![1],
                first: 2,
                last: 2
            }
        );
        let wider = wider.widen();
        assert_eq!(
            wider,
            Region::Children {
                path: vec![],
                first: 1,
                last: 1
            }
        );
        // ルートの子の並びを広げるとファイル全体になる
        assert_eq!(wider.widen(), Region::Whole);
        assert_eq!(Region::Whole.widen(), Region::Whole);
    }

    #[test]
    fn merged_region_covers_both_regions() {
        let a = Region::Children {
            path: vec![1, 2],
            first: 0,
            last: 1,
        };
        let b = Region::Children {
            path: vec![1, 4],
            first: 3,
            last: 3,
        };
        // 同じ親 (path = [1]) の 2 番目から 4 番目の子
        assert_eq!(
            a.merge(&b),
            Region::Children {
                path: vec![1],
                first: 2,
                last: 4
            }
        );
        // 一方が他方の祖先の子の並びなら、その範囲を広げる
        let c = Region::Children {
            path: vec![1],
            first: 0,
            last: 0,
        };
        assert_eq!(
            c.merge(&a),
            Region::Children {
                path: vec![1],
                first: 0,
                last: 2
            }
        );
        assert_eq!(a.merge(&Region::Whole), Region::Whole);
    }

    #[test]
    fn initial_region_is_the_smallest_list_of_nodes_containing_both_comments() {
        let src = r#"let a = 1
let-rec f x % satysfi-fmt off
 = 1
let y = 2
% satysfi-fmt on
let z = 3
"#;
        let input = parse_src(src);
        let comments = comment_spans(&input);
        // off は 2 番目の文の中、on は文の並びの隙間にある。文の並びの 2 番目から 3 番目の文
        let region = initial_region(&input.cst, comments[0], Some(comments[1]));
        let Region::Children { path, first, last } = region else {
            panic!("not a list of nodes");
        };
        let preamble = node_at(&input.cst, &path).unwrap();
        assert_eq!(preamble.rule, satysfi_parser::Rule::preamble);
        assert_eq!(
            &src[preamble.inner[first].span.start..preamble.inner[last].span.end],
            r#"let-rec f x % satysfi-fmt off
 = 1
let y = 2"#
        );
    }

    #[test]
    fn initial_region_without_on_extends_to_the_last_node() {
        let src = r#"let a = 1
let b = [1; % satysfi-fmt off
 2]
let c = 3
"#;
        let input = parse_src(src);
        let comments = comment_spans(&input);
        let Region::Children { path, first, last } = initial_region(&input.cst, comments[0], None)
        else {
            panic!("no region");
        };
        let preamble = node_at(&input.cst, &path).unwrap();
        assert_eq!(
            &src[preamble.inner[first].span.start..preamble.inner[last].span.end],
            r#"let b = [1; % satysfi-fmt off
 2]
let c = 3"#
        );
    }

    #[test]
    fn region_without_nodes_has_nothing_to_restore() {
        let src = r#"let a = 1
% satysfi-fmt off
% satysfi-fmt on
let b = 2
"#;
        let input = parse_src(src);
        let comments = comment_spans(&input);
        assert!(matches!(
            initial_region(&input.cst, comments[0], Some(comments[1])),
            Region::Empty { .. }
        ));
    }

    #[test]
    fn falls_back_to_the_whole_file_when_the_tree_shape_changed() {
        let src = r#"let a = 1
% satysfi-fmt off
let   b   =   2
"#;
        let input = parse_src(src);
        // 整形結果の構文木の形が入力と違う (どの階層でも対応づけられない) 場合
        let output = r#"let a = 1
% satysfi-fmt off
let b = 2
in
document(||)'<>
"#;
        assert_eq!(restore_by_nodes(&input, output), src);
    }

    #[test]
    fn restore_never_fails_even_if_the_region_cannot_be_restored_precisely() {
        let src = r#"let a = 1
% satysfi-fmt off
let b = 2
"#;
        let input = parse_src(src);
        let output = r#"let a = 1
% satysfi-fmt off
let c = 2
"#;
        assert!(restore_precisely(&input, output).is_err());
        let restored = restore_off_regions(&input, output);
        assert!(CstText::parse(&restored, grammar::program).is_ok());
    }
}
