//! `pretty` の Doc を組み立てて整形する printer。
//!
//! CST を `Doc` に変換 (lower) し、最後に `pretty` で描画する。
//!
//! * 行幅に収まれば 1 行、収まらなければ縦に展開する (`group`)。
//! * コメントと空行は `CstText::items` で子ノードの隙間から取り出す。
//!   行コメントは行末まで続くので、後ろには必ず改行を置く (`hardline`)。
//! * `% satysfi-fmt: skip` などのコメントで、ノードを元の文字列のまま出力できる。
//!   → [`directive`]

pub mod directive;
mod expr;
mod pattern;
mod stmt;
mod text;
mod types;

use lspower::lsp::{FormattingOptions, FormattingProperty};
use pretty::RcDoc;
use satysfi_parser::{Cst, CstText, Item, Rule, Span};
use std::collections::HashSet;

pub(self) type Doc = RcDoc<'static, ()>;

/// 行幅の既定値。`FormattingOptions::properties["lineWidth"]` で変更できる。
pub const DEFAULT_LINE_WIDTH: usize = 80;

pub struct Printer<'a> {
    csttext: &'a CstText,
    option: FormattingOptions,
    skip: HashSet<(usize, usize)>,
    width: usize,
    indent: isize,
}

/// 子ノードと、その前後にあるコメント。
pub(self) struct Child<'c> {
    pub cst: &'c Cst,
    /// 子の前にある、単独行のコメント
    pub leading: Vec<String>,
    /// 子と同じ行の末尾にあるコメント
    pub trailing: Option<String>,
    /// 子の前に空行があるか
    pub blank_before: bool,
}

pub(self) struct Children<'c> {
    pub items: Vec<Child<'c>>,
    /// 最後の子の後 (または子がないとき) にある、単独行のコメント
    pub dangling: Vec<String>,
}

/// 子ノードを Doc にしたもの。コメントは Doc に含まれている。
pub(self) struct Kid<'c> {
    pub cst: &'c Cst,
    pub doc: Doc,
}

pub(self) struct Kids<'c> {
    pub items: Vec<Kid<'c>>,
    /// 最後の子の後にあるコメント。`finish` で Doc の後ろに付ける。
    pub dangling: Vec<String>,
}

impl<'c> Kids<'c> {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 最後の子の後ろのコメントを付ける。
    pub fn finish(&self, doc: Doc) -> Doc {
        let mut doc = doc;
        for comment in &self.dangling {
            doc = doc
                .append(Doc::hardline())
                .append(Doc::text(comment.clone()));
        }
        doc
    }
}

impl<'a> Printer<'a> {
    pub fn new(csttext: &'a CstText, option: FormattingOptions) -> Self {
        let width = match option.properties.get("lineWidth") {
            Some(FormattingProperty::Number(n)) if *n > 0 => *n as usize,
            _ => DEFAULT_LINE_WIDTH,
        };
        let indent = (option.tab_size as isize).max(1);
        Self {
            csttext,
            skip: directive::collect_skip_spans(csttext),
            width,
            indent,
            option,
        }
    }

    pub fn print(&self) -> String {
        let doc = self.node(&self.csttext.cst);
        let mut buf = Vec::new();
        doc.render(self.width, &mut buf).expect("render to memory");
        let output = String::from_utf8(buf).expect("doc is valid utf-8");
        // 行末の空白を除く (整形しないノードは目印があるので残る)
        let output = output
            .split('\n')
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        let mut output = directive::strip_markers(&output);
        if self.option.insert_final_newline.unwrap_or(true) && !output.ends_with('\n') {
            output.push('\n');
        }
        output
    }

    // ---- 基本の部品 ----

    pub(self) fn text_of(&self, cst: &Cst) -> &'a str {
        self.csttext.get_text(cst)
    }

    /// 元の文字列をそのまま Doc にする。
    /// 改行を含む文字列 (複数行の文字列リテラルなど) は、字下げの影響を受けないようにする。
    pub(self) fn raw(&self, cst: &Cst) -> Doc {
        let text = self.text_of(cst);
        if text.contains('\n') {
            lines(&directive::verbatim(text))
        } else {
            Doc::text(text.trim().to_string())
        }
    }

    pub(self) fn comment(&self, span: Span) -> String {
        to_comment_string(self.csttext.get_text_from_span(span))
    }

    /// cst の子と、その前後にあるコメントを取り出す。
    pub(self) fn children<'c>(&self, cst: &'c Cst) -> Children<'c> {
        let mut items: Vec<Child<'c>> = vec![];
        let mut pending: Vec<String> = vec![];
        let mut blank = false;
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => {
                    items.push(Child {
                        cst: child,
                        leading: std::mem::take(&mut pending),
                        trailing: None,
                        blank_before: blank,
                    });
                    blank = false;
                }
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match items.last_mut() {
                        Some(last)
                            if !own_line && last.trailing.is_none() && pending.is_empty() =>
                        {
                            last.trailing = Some(comment)
                        }
                        _ => pending.push(comment),
                    }
                }
                Item::BlankLines(_) => blank = true,
            }
        }
        Children {
            items,
            dangling: pending,
        }
    }

    /// 子を Doc にする。行末のコメントは次の子の前の単独行のコメントとして扱う
    /// (後ろに `;` や `)` などが続いても、コメントに飲み込まれないようにするため)。
    pub(self) fn kids<'c>(&self, cst: &'c Cst) -> Kids<'c> {
        let children = self.children(cst);
        let mut items = vec![];
        let mut carry: Vec<String> = vec![];
        for child in children.items {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(child.leading);
            carry.extend(child.trailing);
            items.push(Kid {
                cst: child.cst,
                doc: with_leading(leading, self.node(child.cst)),
            });
        }
        carry.extend(children.dangling);
        Kids {
            items,
            dangling: carry,
        }
    }

    /// 1 行ずつ並ぶ要素 (`match` の腕、`and` で繋ぐ定義など) を、行頭の記号 `prefix` を付けて並べる。
    /// 各要素の前には改行を置く。行末のコメントは、その要素の行に残す。
    pub(self) fn prefixed_lines(&self, children: Vec<Child>, prefix: &str) -> Doc {
        let mut doc = Doc::nil();
        for child in children {
            doc = doc.append(Doc::hardline());
            for comment in child.leading {
                doc = doc.append(Doc::text(comment)).append(Doc::hardline());
            }
            doc = doc.append(s(prefix)).append(self.node(child.cst));
            if let Some(comment) = child.trailing {
                doc = doc.append(s(" ")).append(Doc::text(comment));
            }
        }
        doc
    }

    // ---- ノードの変換 ----

    /// cst を Doc に変換する。
    pub(self) fn node(&self, cst: &Cst) -> Doc {
        // `satysfi-fmt` directive で整形しないと指定されたノード
        if self.skip.contains(&(cst.span.start, cst.span.end)) {
            return lines(&directive::verbatim(self.text_of(cst)));
        }
        match cst.rule {
            Rule::program_saty | Rule::program_satyh => self.program(cst),
            Rule::stage => Doc::text(format!("@stage: {}", self.text_of(cst).trim())),
            Rule::headers => self.headers(cst),
            Rule::preamble => self.stmt_list(cst),

            // 文
            Rule::let_stmt
            | Rule::let_rec_stmt
            | Rule::let_rec_inner
            | Rule::let_rec_matcharm
            | Rule::let_inline_stmt_ctx
            | Rule::let_inline_stmt_noctx
            | Rule::let_block_stmt_ctx
            | Rule::let_block_stmt_noctx
            | Rule::let_math_stmt
            | Rule::let_mutable_stmt
            | Rule::type_stmt
            | Rule::type_inner
            | Rule::type_variant
            | Rule::module_stmt
            | Rule::open_stmt
            | Rule::arg
            | Rule::sig_stmt
            | Rule::struct_stmt
            | Rule::sig_type_stmt
            | Rule::sig_val_stmt
            | Rule::sig_direct_stmt
            | Rule::constraint
            | Rule::bind_stmt => self.stmt_node(cst),

            // 型
            Rule::type_expr
            | Rule::type_optional
            | Rule::type_prod
            | Rule::type_inline_cmd
            | Rule::type_block_cmd
            | Rule::type_math_cmd
            | Rule::type_list_unit_optional
            | Rule::type_application
            | Rule::type_name
            | Rule::type_record
            | Rule::type_record_unit
            | Rule::type_param => self.type_node(cst),

            // パターン
            Rule::pat_as
            | Rule::pat_cons
            | Rule::pattern
            | Rule::pat_variant
            | Rule::pat_list
            | Rule::pat_tuple => self.pattern_node(cst),

            // 水平・垂直・数式モード
            Rule::block_text
            | Rule::horizontal_text
            | Rule::math_text
            | Rule::vertical
            | Rule::horizontal_single
            | Rule::horizontal_list
            | Rule::horizontal_bullet_list
            | Rule::horizontal_bullet
            | Rule::horizontal_bullet_star
            | Rule::regular_text
            | Rule::horizontal_escaped_char
            | Rule::inline_text_embedding
            | Rule::block_text_embedding
            | Rule::inline_cmd
            | Rule::block_cmd
            | Rule::math_cmd
            | Rule::cmd_expr_arg
            | Rule::cmd_expr_option
            | Rule::cmd_text_arg
            | Rule::math_cmd_expr_arg
            | Rule::math_cmd_expr_option
            | Rule::math_single
            | Rule::math_list
            | Rule::math_token
            | Rule::math_sup
            | Rule::math_sub
            | Rule::math_unary
            | Rule::math_embedding => self.text_node(cst),

            // 式
            _ => self.expr_node(cst),
        }
    }

    /// ファイル全体。stage / headers / preamble / 本体を空行で区切って並べる。
    fn program(&self, cst: &Cst) -> Doc {
        let is_saty = cst.rule == Rule::program_saty;
        let children = self.children(cst);
        let mut blocks: Vec<Doc> = vec![];
        // 次のブロックの前に置く、単独行のコメント
        let mut pending: Vec<String> = vec![];

        for child in children.items {
            if child.cst.rule == Rule::headers && child.cst.inner.is_empty() {
                pending.extend(child.leading);
                continue;
            }
            pending.extend(child.leading);
            let mut doc = self.node(child.cst);
            if child.cst.rule == Rule::preamble && is_saty {
                doc = doc.append(Doc::hardline()).append(Doc::text("in"));
            }
            let mut doc = with_leading(std::mem::take(&mut pending), doc);
            if let Some(comment) = child.trailing {
                doc = doc.append(Doc::text(" ")).append(Doc::text(comment));
            }
            blocks.push(doc);
        }
        pending.extend(children.dangling);
        if !pending.is_empty() {
            blocks.push(join(pending.into_iter().map(Doc::text), Doc::hardline()));
        }
        join(blocks, Doc::hardline().append(Doc::hardline()))
    }

    /// `@require:` / `@import:` の並び。
    fn headers(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        let mut docs: Vec<Doc> = vec![];
        for child in children.items {
            for comment in child.leading {
                docs.push(Doc::text(comment));
            }
            let mut doc = self.header(child.cst);
            if let Some(comment) = child.trailing {
                doc = doc.append(Doc::text(" ")).append(Doc::text(comment));
            }
            docs.push(doc);
        }
        docs.extend(children.dangling.into_iter().map(Doc::text));
        join(docs, Doc::hardline())
    }

    fn header(&self, cst: &Cst) -> Doc {
        if self.skip.contains(&(cst.span.start, cst.span.end)) {
            return lines(&directive::verbatim(self.text_of(cst)));
        }
        let keyword = match cst.rule {
            Rule::header_require => "@require:",
            Rule::header_import => "@import:",
            _ => return Doc::text(self.text_of(cst).trim().to_string()),
        };
        let pkgname = cst.inner.first().map_or("", |pkg| self.text_of(pkg).trim());
        Doc::text(format!("{keyword} {pkgname}"))
    }

    /// 文の並び (preamble / struct の中身)。元の空行は 1 行まで残し、
    /// `module` の前後には必ず空行を入れる。
    pub(self) fn stmt_list(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        let mut doc = Doc::nil();
        let mut first = true;
        let mut prev_is_module = false;
        for child in children.items {
            if !first {
                doc = doc.append(Doc::hardline());
                if child.blank_before || prev_is_module || child.cst.rule == Rule::module_stmt {
                    doc = doc.append(Doc::hardline());
                }
            }
            prev_is_module = child.cst.rule == Rule::module_stmt;
            let mut stmt = with_leading(child.leading, self.node(child.cst));
            if let Some(comment) = child.trailing {
                stmt = stmt.append(Doc::text(" ")).append(Doc::text(comment));
            }
            doc = doc.append(stmt);
            first = false;
        }
        for comment in children.dangling {
            if !first {
                doc = doc.append(Doc::hardline());
            }
            doc = doc.append(Doc::text(comment));
            first = false;
        }
        doc
    }
}

/// コメント (`%` を含む行) を整形する。`%` の後ろに空白がなければ 1 つ入れる。
pub fn to_comment_string(text: &str) -> String {
    let index = text.find('%').unwrap_or(0);
    let comment = text[index + 1..].trim_end();
    if comment.is_empty() || comment.starts_with(char::is_whitespace) || comment.starts_with('%') {
        // 空白or複数の%で始まっていたらそのまま表示
        format!("%{}", comment)
    } else {
        format!("% {}", comment)
    }
}

/// 複数行の文字列を、行ごとに hardline で繋いだ Doc にする。
/// 行頭の空白はそのまま残る。
pub(self) fn lines(text: &str) -> Doc {
    join(
        text.split('\n').map(|line| Doc::text(line.to_string())),
        Doc::hardline(),
    )
}

pub(self) fn join(docs: impl IntoIterator<Item = Doc>, sep: Doc) -> Doc {
    Doc::intersperse(docs, sep)
}

/// 単独行のコメントを doc の前に 1 行ずつ置く。
pub(self) fn with_leading(comments: Vec<String>, doc: Doc) -> Doc {
    let mut result = Doc::nil();
    for comment in comments {
        result = result.append(Doc::text(comment)).append(Doc::hardline());
    }
    result.append(doc)
}

/// 文字列 s を Doc にする。
pub(self) fn s(text: &str) -> Doc {
    Doc::text(text.to_string())
}
