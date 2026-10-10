//! `pretty` の Doc を組み立てて整形する新しい printer。
//!
//! CST を `Doc` に変換 (lower) し、最後に `pretty` で描画する。
//! コメントと空行は `CstText::items` で子ノードの間から取り出す。
//!
//! 移行中は、まだ Doc 化していない rule を旧 `Formatter` に委譲する
//! (`Printer::legacy`)。rule ごとに Doc 化を進め、委譲を減らしていく。
//! 旧 `Formatter` の出力と一致させる必要はない。

pub mod directive;
mod expr;

use crate::comment::{get_comments, subtree_with_comments, to_comment_string, Comment};
use crate::formatter::Formatter;
use lspower::lsp::{FormattingOptions, FormattingProperty};
use pretty::RcDoc;
use satysfi_parser::{Cst, CstText, Item, Rule};
use std::collections::{HashSet, VecDeque};

type Doc = RcDoc<'static, ()>;

/// 行幅の既定値。`FormattingOptions::properties["lineWidth"]` で変更できる。
pub const DEFAULT_LINE_WIDTH: usize = 80;
/// インデント幅の既定値以外は `FormattingOptions::tab_size` に従う。
const INDENT: isize = 4;

pub struct Printer<'a> {
    csttext: &'a CstText,
    option: FormattingOptions,
    comments: VecDeque<Comment>,
    legacy: Formatter<'a>,
    skip: HashSet<(usize, usize)>,
    width: usize,
}

impl<'a> Printer<'a> {
    pub fn new(csttext: &'a CstText, option: FormattingOptions) -> Self {
        let width = match option.properties.get("lineWidth") {
            Some(FormattingProperty::Number(n)) if *n > 0 => *n as usize,
            _ => DEFAULT_LINE_WIDTH,
        };
        Self {
            csttext,
            comments: get_comments(csttext),
            legacy: Formatter::new(csttext, option.clone()),
            skip: directive::collect_skip_spans(csttext),
            width,
            option,
        }
    }

    pub fn print(&self) -> String {
        let doc = self.node(&self.csttext.cst);
        let mut buf = Vec::new();
        doc.render(self.width, &mut buf).expect("render to memory");
        let output = String::from_utf8(buf).expect("doc is valid utf-8");
        self.legacy.finish(output)
    }

    /// cst の子の隙間 (cst 自身が持つ記号の周り) にコメントがあるか。
    /// 子の内部にあるコメントは、その子が責任を持つので含めない。
    fn has_comment(&self, cst: &Cst) -> bool {
        self.csttext
            .items(cst)
            .iter()
            .any(|item| matches!(item, Item::Comment { .. }))
    }

    fn text_of(&self, cst: &Cst) -> &'a str {
        self.csttext.get_text(cst)
    }

    /// cst を Doc に変換する。
    fn node(&self, cst: &Cst) -> Doc {
        // `satysfi-fmt` directive で整形しないと指定されたノード
        if self.skip.contains(&(cst.span.start, cst.span.end)) {
            return lines(&directive::verbatim(self.text_of(cst)));
        }
        match cst.rule {
            Rule::program_saty | Rule::program_satyh => self.program(cst),
            Rule::stage => Doc::text(format!("@stage: {}", self.text_of(cst))),
            Rule::headers => self.headers(cst),
            Rule::preamble => self.preamble(cst),
            _ => self.expr_node(cst),
        }
    }

    /// まだ Doc 化していない rule は旧 Formatter の出力をそのまま使う。
    fn legacy(&self, cst: &Cst) -> Doc {
        let cst = subtree_with_comments(cst, &self.comments);
        let output = self
            .legacy
            .to_string_cst(&self.csttext.text, &cst, 0);
        lines(output.trim())
    }

    /// 行コメントを Doc にする。
    fn comment(&self, span: satysfi_parser::Span) -> String {
        to_comment_string(self.csttext.get_text_from_span(span).to_string())
    }

    /// ファイル全体。stage / headers / preamble / 本体を空行で区切って並べる。
    fn program(&self, cst: &Cst) -> Doc {
        let is_saty = cst.rule == Rule::program_saty;
        let mut blocks: Vec<Doc> = vec![];
        // 次のブロックの前に置く、単独行のコメント
        let mut pending: Vec<String> = vec![];

        for item in self.csttext.items(cst) {
            match item {
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match blocks.pop() {
                        // 行末のコメントは直前のブロックの末尾に付ける
                        Some(last) if !own_line && pending.is_empty() => {
                            blocks.push(last.append(Doc::text(" ")).append(Doc::text(comment)));
                        }
                        Some(last) => {
                            blocks.push(last);
                            pending.push(comment);
                        }
                        None => pending.push(comment),
                    }
                }
                Item::BlankLines(_) => {}
                Item::Node(child) => {
                    if child.rule == Rule::headers && child.inner.is_empty() {
                        continue;
                    }
                    let mut doc = self.node(child);
                    if child.rule == Rule::preamble && is_saty {
                        doc = doc.append(Doc::hardline()).append(Doc::text("in"));
                    }
                    blocks.push(leading_comments(&mut pending, doc));
                }
            }
        }
        if !pending.is_empty() {
            blocks.push(join(pending.drain(..).map(Doc::text), Doc::hardline()));
        }
        join(blocks, Doc::hardline().append(Doc::hardline()))
    }

    /// `@require:` / `@import:` の並び。
    fn headers(&self, cst: &Cst) -> Doc {
        let mut docs: Vec<Doc> = vec![];
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => docs.push(self.header(child)),
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match docs.pop() {
                        Some(last) if !own_line => {
                            docs.push(last.append(Doc::text(" ")).append(Doc::text(comment)))
                        }
                        Some(last) => {
                            docs.push(last);
                            docs.push(Doc::text(comment));
                        }
                        None => docs.push(Doc::text(comment)),
                    }
                }
                Item::BlankLines(_) => {}
            }
        }
        join(docs, Doc::hardline())
    }

    fn header(&self, cst: &Cst) -> Doc {
        let keyword = match cst.rule {
            Rule::header_require => "@require:",
            Rule::header_import => "@import:",
            _ => return Doc::text(self.text_of(cst).trim().to_string()),
        };
        let pkgname = cst.inner.first().map_or("", |pkg| self.text_of(pkg).trim());
        Doc::text(format!("{keyword} {pkgname}"))
    }

    /// 前処理部の文の並び。元の空行は 1 行まで残し、`module` の前には必ず空行を入れる。
    fn preamble(&self, cst: &Cst) -> Doc {
        // (空行を前に置くか, Doc)
        let mut stmts: Vec<(bool, Doc)> = vec![];
        let mut pending: Vec<String> = vec![];
        // 次の要素の前に空行があったか
        let mut blank = false;
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => {
                    let doc = leading_comments(&mut pending, self.node(child));
                    stmts.push((blank || child.rule == Rule::module_stmt, doc));
                    blank = false;
                }
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match stmts.pop() {
                        Some((b, last)) if !own_line && pending.is_empty() => stmts.push((
                            b,
                            last.append(Doc::text(" ")).append(Doc::text(comment)),
                        )),
                        Some(last) => {
                            stmts.push(last);
                            pending.push(comment);
                        }
                        None => pending.push(comment),
                    }
                }
                Item::BlankLines(_) => blank = true,
            }
        }
        let mut doc = Doc::nil();
        let mut first = true;
        for (blank, stmt) in stmts {
            if !first {
                doc = doc.append(Doc::hardline());
                if blank {
                    doc = doc.append(Doc::hardline());
                }
            }
            doc = doc.append(stmt);
            first = false;
        }
        for comment in pending {
            if !first {
                doc = doc.append(Doc::hardline());
            }
            doc = doc.append(Doc::text(comment));
            first = false;
        }
        doc
    }
}

/// 複数行の文字列を、行ごとに hardline で繋いだ Doc にする。
/// 行頭の空白はそのまま残る (旧 Formatter の相対インデントを保つ)。
pub(self) fn lines(text: &str) -> Doc {
    join(
        text.split('\n').map(|line| Doc::text(line.to_string())),
        Doc::hardline(),
    )
}

pub(self) fn join(docs: impl IntoIterator<Item = Doc>, sep: Doc) -> Doc {
    Doc::intersperse(docs, sep)
}

/// 溜まっているコメントを doc の前に 1 行ずつ置く。
pub(self) fn leading_comments(pending: &mut Vec<String>, doc: Doc) -> Doc {
    let mut result = Doc::nil();
    for comment in pending.drain(..) {
        result = result.append(Doc::text(comment)).append(Doc::hardline());
    }
    result.append(doc)
}
