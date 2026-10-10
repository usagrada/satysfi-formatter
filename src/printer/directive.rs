//! コメントによる整形の制御。
//!
//! * `% satysfi-fmt: skip` : 直後のノードを整形しない。行末に書いた場合は直前のノード。
//! * `% satysfi-fmt: off` / `% satysfi-fmt: on` : 間にあるノードを整形しない。
//!   `on` がなければ、同じ親の中の最後のノードまで。
//!
//! 整形しないノードは元の文字列をそのまま出力する。
//! ノード間の空白や改行、コメントは通常どおり整形される。

use satysfi_parser::{Cst, CstText, Item, Rule};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Directive {
    Skip,
    Off,
    On,
}

/// コメント文字列 (`%` を含む) から directive を取り出す。
pub fn parse(comment: &str) -> Option<Directive> {
    let body = comment.trim_start_matches('%').trim();
    match body.strip_prefix("satysfi-fmt:")?.trim() {
        "skip" => Some(Directive::Skip),
        "off" => Some(Directive::Off),
        "on" => Some(Directive::On),
        _ => None,
    }
}

/// 整形しないノードの span (start, end) の集合を求める。
pub fn collect_skip_spans(csttext: &CstText) -> HashSet<(usize, usize)> {
    let mut skip = HashSet::new();
    walk(csttext, &csttext.cst, &mut skip, false, false);
    skip
}

/// 文や header の並びを束ねるだけのノード。
/// 直前の directive は、このノードではなく中の最初の要素にかかる。
fn is_container(node: &Cst) -> bool {
    matches!(node.rule, Rule::preamble | Rule::headers)
}

/// `off` / `skip_next` は親から引き継ぐ状態。最後の `off` の状態を返す。
fn walk(
    csttext: &CstText,
    node: &Cst,
    skip: &mut HashSet<(usize, usize)>,
    inherited_off: bool,
    inherited_skip_next: bool,
) -> bool {
    let mut off = inherited_off;
    let mut skip_next = inherited_skip_next;
    let mut prev: Option<(usize, usize)> = None;
    for item in csttext.items(node) {
        match item {
            Item::Comment { span, own_line } => {
                match parse(csttext.get_text_from_span(span)) {
                    Some(Directive::Skip) if own_line => skip_next = true,
                    Some(Directive::Skip) => {
                        if let Some(prev) = prev {
                            skip.insert(prev);
                        }
                    }
                    Some(Directive::Off) => off = true,
                    Some(Directive::On) => off = false,
                    None => {}
                }
            }
            Item::Node(child) => {
                let key = (child.span.start, child.span.end);
                // 水平モードの空白だけの文字列は、directive の対象にならない
                let blank_text = child.rule == Rule::regular_text
                    && csttext.get_text(child).trim().is_empty();
                if blank_text {
                    continue;
                }
                if key.0 == key.1 {
                    // 空のノード (header がない場合の headers など)
                } else if is_container(child) {
                    off = walk(csttext, child, skip, off, skip_next);
                    skip_next = false;
                } else if (off || skip_next) && key.0 < key.1 {
                    skip.insert(key);
                    skip_next = false;
                } else {
                    walk(csttext, child, skip, false, false);
                }
                prev = Some(key);
            }
            Item::BlankLines(_) => {}
        }
    }
    off
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
        assert_eq!(parse("% satysfi-fmt: skip"), Some(Directive::Skip));
        assert_eq!(parse("%satysfi-fmt:off"), Some(Directive::Off));
        assert_eq!(parse("%%  satysfi-fmt:  on  "), Some(Directive::On));
        assert_eq!(parse("% satysfi-fmt: unknown"), None);
        assert_eq!(parse("% other: skip"), None);
    }

    #[test]
    fn verbatim_roundtrip() {
        let v = verbatim("  a  \n    b  ");
        // Doc が付けるインデントを想定
        let indented = v.replace('\n', "\n        ");
        assert_eq!(strip_markers(&indented), "  a  \n    b  ");
    }
}
