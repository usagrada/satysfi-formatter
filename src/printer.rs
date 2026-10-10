//! `pretty` の Doc を組み立てて整形する新しい printer。
//!
//! CST を `Doc` に変換 (lower) し、最後に `pretty` で描画する。
//! コメントと空行は `CstText::items` で子ノードの間から取り出す。
//!
//! 移行中は、まだ Doc 化していない rule を旧 `Formatter` に委譲する
//! (`Printer::legacy`)。rule ごとに Doc 化を進め、委譲を減らしていく。

use crate::comment::{get_comments, subtree_with_comments, to_comment_string, Comment};
use crate::formatter::Formatter;
use lspower::lsp::FormattingOptions;
use pretty::RcDoc;
use satysfi_parser::{Cst, CstText, Item, Rule};
use std::collections::VecDeque;

type Doc = RcDoc<'static, ()>;

/// 描画時の行幅。折り返しを行う group を導入するまでは実質使われない。
const LINE_WIDTH: usize = 100;

pub struct Printer<'a> {
    csttext: &'a CstText,
    option: FormattingOptions,
    comments: VecDeque<Comment>,
    legacy: Formatter<'a>,
}

impl<'a> Printer<'a> {
    pub fn new(csttext: &'a CstText, option: FormattingOptions) -> Self {
        Self {
            csttext,
            comments: get_comments(csttext),
            legacy: Formatter::new(csttext, option.clone()),
            option,
        }
    }

    pub fn print(&self) -> String {
        let doc = self.node(&self.csttext.cst);
        let mut buf = Vec::new();
        doc.render(LINE_WIDTH, &mut buf).expect("render to memory");
        let output = String::from_utf8(buf).expect("doc is valid utf-8");
        self.legacy.finish(output)
    }

    fn text_of(&self, cst: &Cst) -> &'a str {
        self.csttext.get_text(cst)
    }

    /// cst を Doc に変換する。
    fn node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::program_saty | Rule::program_satyh => self.program(cst),
            Rule::stage => Doc::text(format!("@stage: {}", self.text_of(cst))),
            Rule::headers => self.headers(cst),
            Rule::preamble => self.preamble(cst),
            _ => self.legacy(cst),
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

    /// 前処理部の文の並び。`module` の前だけ空行を入れる。
    fn preamble(&self, cst: &Cst) -> Doc {
        // (空行を前に置くか, Doc)
        let mut stmts: Vec<(bool, Doc)> = vec![];
        let mut pending: Vec<String> = vec![];
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => {
                    let doc = leading_comments(&mut pending, self.node(child));
                    stmts.push((child.rule == Rule::module_stmt, doc));
                }
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match stmts.pop() {
                        Some((blank, last)) if !own_line && pending.is_empty() => stmts.push((
                            blank,
                            last.append(Doc::text(" ")).append(Doc::text(comment)),
                        )),
                        Some(last) => {
                            stmts.push(last);
                            pending.push(comment);
                        }
                        None => pending.push(comment),
                    }
                }
                Item::BlankLines(_) => {}
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
fn lines(text: &str) -> Doc {
    join(
        text.split('\n').map(|line| Doc::text(line.to_string())),
        Doc::hardline(),
    )
}

fn join(docs: impl IntoIterator<Item = Doc>, sep: Doc) -> Doc {
    Doc::intersperse(docs, sep)
}

/// 溜まっているコメントを doc の前に 1 行ずつ置く。
fn leading_comments(pending: &mut Vec<String>, doc: Doc) -> Doc {
    let mut result = Doc::nil();
    for comment in pending.drain(..) {
        result = result.append(Doc::text(comment)).append(Doc::hardline());
    }
    result.append(doc)
}
