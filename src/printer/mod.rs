//! `pretty` の Doc を組み立てて整形する printer。
//!
//! CST を `Doc` に変換 (lower) し、最後に `pretty` で描画する。
//!
//! * 行幅に収まれば 1 行、収まらなければ縦に展開する (`group`)。
//! * コメントと空行は `CstText::items` で子ノードの隙間から取り出す。
//!   行コメントは行末まで続くので、後ろには必ず改行を置く (`hardline`)。
//! * `% satysfi-fmt off` から `on` までは整形せず、元の文字列に戻す。→ [`directive`]

pub mod directive;
mod expr;
mod pattern;
mod stmt;
mod text;
mod types;

use crate::config::FormatConfig;
use pretty::RcDoc;
use satysfi_parser::{Cst, CstText, Item, Rule, Span};
use std::cell::RefCell;
use std::collections::HashMap;

pub(self) type Doc = RcDoc<'static, ()>;

pub struct Printer<'a> {
    csttext: &'a CstText,
    final_newline: bool,
    use_tabs: bool,
    command_space: bool,
    width: usize,
    indent: isize,
    /// `token_multiline` / `math_multiline` の結果
    token_multiline_cache: RefCell<HashMap<(usize, usize, Rule), bool>>,
    math_multiline_cache: RefCell<HashMap<(usize, usize, Rule), bool>>,
}

/// 子ノードと、その前後にあるコメント。
pub(self) struct Child<'c> {
    pub cst: &'c Cst,
    /// 子の前にある、単独行のコメント
    pub leading: Vec<String>,
    /// 子と同じ行の末尾にあるコメント
    pub trailing: Option<String>,
    /// 子の前 (leading があればその前) に空行があるか
    pub blank_before: bool,
    /// leading の各コメントの前に空行があるか (最初のコメントの前は `blank_before`)
    pub leading_blanks: Vec<bool>,
    /// leading の各コメントの列 (行頭からの文字数)
    pub leading_cols: Vec<usize>,
    /// leading と子の間に空行があるか
    pub blank_after_leading: bool,
}

pub(self) struct Children<'c> {
    pub items: Vec<Child<'c>>,
    /// 最後の子の後 (または子がないとき) にある、単独行のコメント
    pub dangling: Vec<String>,
    /// dangling の各コメントの前に空行があるか
    pub dangling_blanks: Vec<bool>,
    /// dangling の各コメントの列
    pub dangling_cols: Vec<usize>,
}

/// 子ノードを Doc にしたもの。コメントは Doc に含まれている。
pub(self) struct Kid<'c> {
    pub cst: &'c Cst,
    pub doc: Doc,
    /// 前にコメントがあるか (`doc` はコメントで始まる)
    pub commented: bool,
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
    pub fn new(csttext: &'a CstText, config: &FormatConfig) -> Self {
        Self {
            csttext,
            width: config.line_width.max(1),
            indent: config.indent_width.max(1) as isize,
            final_newline: config.final_newline,
            use_tabs: config.use_tabs,
            command_space: config.command_space,
            token_multiline_cache: RefCell::default(),
            math_multiline_cache: RefCell::default(),
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
            .map(|line| {
                let line = line.trim_end();
                if self.use_tabs {
                    self.indent_with_tabs(line)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut output = directive::strip_markers(&output);
        if self.final_newline && !output.ends_with('\n') {
            output.push('\n');
        }
        output
    }

    /// 行頭のインデント (空白) をタブにする。インデント幅に満たない端数は空白のまま残す。
    /// 整形しない文字列の 2 行目以降は、元のインデントを残すので変えない。
    fn indent_with_tabs(&self, line: &str) -> String {
        if directive::is_verbatim_continuation(line) {
            return line.to_string();
        }
        let body = line.trim_start_matches(' ');
        let spaces = line.len() - body.len();
        let indent = self.indent as usize;
        format!(
            "{}{}{}",
            "\t".repeat(spaces / indent),
            " ".repeat(spaces % indent),
            body
        )
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
        // pending の最初のコメントの前に空行があったか
        let mut blank_before_pending = false;
        // pending の各コメントの前に空行があったか (最初は常に false)
        let mut pending_blanks: Vec<bool> = vec![];
        let mut pending_cols: Vec<usize> = vec![];
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => {
                    let (blank_before, blank_after_leading) = if pending.is_empty() {
                        (blank, false)
                    } else {
                        (blank_before_pending, blank)
                    };
                    items.push(Child {
                        cst: child,
                        leading: std::mem::take(&mut pending),
                        leading_blanks: std::mem::take(&mut pending_blanks),
                        leading_cols: std::mem::take(&mut pending_cols),
                        trailing: None,
                        blank_before,
                        blank_after_leading,
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
                        _ => {
                            if pending.is_empty() {
                                blank_before_pending = blank;
                                pending_blanks.push(false);
                            } else {
                                pending_blanks.push(blank);
                            }
                            blank = false;
                            pending_cols.push(self.column(span.start));
                            pending.push(comment)
                        }
                    }
                }
                Item::BlankLines(_) => blank = true,
            }
        }
        // dangling の各コメントの前の空行 (最初のコメントの前は blank_before_pending)
        let mut dangling_blanks = pending_blanks;
        if let Some(first) = dangling_blanks.first_mut() {
            *first = blank_before_pending;
        }
        Children {
            items,
            dangling: pending,
            dangling_blanks,
            dangling_cols: pending_cols,
        }
    }

    /// 位置 `pos` の列 (行頭からの文字数)。
    pub(self) fn column(&self, pos: usize) -> usize {
        let text = &self.csttext.text;
        let line_start = text[..pos].rfind('\n').map_or(0, |i| i + 1);
        text[line_start..pos].chars().count()
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
                commented: !leading.is_empty(),
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
        // `in` の後ろにあった単独行のコメント。本体の前に置く
        // (children では preamble の行末コメントと本体の前のコメントに分かれてしまうので、置き換える)
        let mut after_in: Option<Vec<String>> = None;

        for child in children.items {
            if child.cst.rule == Rule::headers && child.cst.inner.is_empty() {
                pending.extend(child.leading);
                continue;
            }
            match after_in.take() {
                Some(comments) => pending.extend(comments),
                None => pending.extend(child.leading),
            }
            let mut doc = self.node(child.cst);
            let mut trailing = child.trailing;
            if child.cst.rule == Rule::preamble && is_saty {
                // preamble と本体の間のコメントは、元の位置に合わせて `in` の前後に置く
                let (before, after) = self.comments_around_in(cst, child.cst);
                for (comment, own_line) in before {
                    doc = doc
                        .append(if own_line { Doc::hardline() } else { s(" ") })
                        .append(Doc::text(comment));
                }
                doc = doc.append(Doc::hardline()).append(s("in"));
                let mut after = after.into_iter().peekable();
                trailing = after.next_if(|(_, own_line)| !own_line).map(|(c, _)| c);
                after_in = Some(after.map(|(c, _)| c).collect());
            }
            let mut doc = with_leading(std::mem::take(&mut pending), doc);
            if let Some(comment) = trailing {
                doc = doc.append(Doc::text(" ")).append(Doc::text(comment));
            }
            blocks.push(doc);
        }
        pending.extend(after_in.unwrap_or_default());
        pending.extend(children.dangling);
        if !pending.is_empty() {
            blocks.push(join(pending.into_iter().map(Doc::text), Doc::hardline()));
        }
        join(blocks, Doc::hardline().append(Doc::hardline()))
    }

    /// `parent` の子 `def` (preamble や `let` の定義) と、次の子 (本体) の間にあるコメントを、
    /// `in` の前と後ろに分ける。それぞれ (コメント, 単独行か) の並び。
    pub(self) fn comments_around_in(
        &self,
        parent: &Cst,
        def: &Cst,
    ) -> (Vec<(String, bool)>, Vec<(String, bool)>) {
        let in_pos = self.in_keyword_pos(def.span.end);
        let mut before = vec![];
        let mut after = vec![];
        let items = self.csttext.items(parent);
        let gap = items
            .iter()
            .skip_while(|item| !matches!(item, Item::Node(n) if std::ptr::eq(*n, def)))
            .skip(1)
            .take_while(|item| !matches!(item, Item::Node(_)));
        for item in gap {
            if let Item::Comment { span, own_line } = item {
                let comment = (self.comment(*span), *own_line);
                if span.start < in_pos {
                    before.push(comment);
                } else {
                    after.push(comment);
                }
            }
        }
        (before, after)
    }

    /// `from` 以降で、空白とコメントを読み飛ばした最初の位置 (定義の後ろの `in`)。
    fn in_keyword_pos(&self, from: usize) -> usize {
        let text = &self.csttext.text;
        let mut pos = from;
        loop {
            let rest = &text[pos..];
            let trimmed = rest.trim_start();
            pos += rest.len() - trimmed.len();
            if !trimmed.starts_with('%') {
                return pos;
            }
            pos += trimmed.find('\n').unwrap_or(trimmed.len());
        }
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
        let mut children = self.children(cst);
        // 文の後ろにある単独行のコメントのうち、その文より深く字下げされ、空行で区切られていないものは、
        // 次の文の見出しではなく、その文の本体の続きとして、本体と同じ字下げで出す。
        let count = children.items.len();
        let mut tails: Vec<Vec<String>> = vec![vec![]; count];
        for j in 1..count {
            let prev_col = self.column(children.items[j - 1].cst.span.start);
            let child = &mut children.items[j];
            let taken = (0..child.leading.len())
                .take_while(|&k| {
                    let blank = if k == 0 {
                        child.blank_before
                    } else {
                        child.leading_blanks[k]
                    };
                    // `off` / `on` は、次の文に付く指示なので本体のコメントにはしない
                    !blank
                        && child.leading_cols[k] > prev_col
                        && directive::parse(&child.leading[k]).is_none()
                })
                .count();
            if taken > 0 {
                tails[j - 1] = child.leading.drain(..taken).collect();
                // 残りの先頭のコメント (なければ子) の前の空行は、そのまま引き継ぐ
                child.blank_before = match child.leading_blanks.get(taken) {
                    Some(blank) => *blank,
                    None => child.blank_after_leading,
                };
                if child.leading.is_empty() {
                    // コメントがなくなったので、コメントと子の間の空行は文の間の空行になった
                    child.blank_after_leading = false;
                }
                child.leading_blanks.drain(..taken);
                child.leading_cols.drain(..taken);
                if let Some(first) = child.leading_blanks.first_mut() {
                    *first = false;
                }
            }
        }
        if let Some(last) = children.items.last() {
            let prev_col = self.column(last.cst.span.start);
            let taken = (0..children.dangling.len())
                .take_while(|&k| {
                    !children.dangling_blanks[k]
                        && children.dangling_cols[k] > prev_col
                        && directive::parse(&children.dangling[k]).is_none()
                })
                .count();
            if taken > 0 {
                tails[count - 1] = children.dangling.drain(..taken).collect();
            }
        }

        let mut doc = Doc::nil();
        let mut first = true;
        let mut prev_is_module = false;
        for (child, tail) in children.items.into_iter().zip(tails) {
            if !first {
                doc = doc.append(Doc::hardline());
                if child.blank_before || prev_is_module || child.cst.rule == Rule::module_stmt {
                    doc = doc.append(Doc::hardline());
                }
            }
            prev_is_module = child.cst.rule == Rule::module_stmt;
            let mut stmt = leading_lines(&child).append(self.node(child.cst));
            if let Some(comment) = child.trailing {
                stmt = stmt.append(Doc::text(" ")).append(Doc::text(comment));
            }
            if !tail.is_empty() {
                let comments = join(tail.into_iter().map(Doc::text), Doc::hardline());
                stmt = stmt.append(Doc::hardline().append(comments).nest(self.indent));
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

/// 子の前の単独行のコメントを 1 行ずつ並べる。
/// コメントの間と、コメントと子の間にあった空行は 1 行だけ残す。
pub(self) fn leading_lines(child: &Child) -> Doc {
    let mut doc = Doc::nil();
    for (comment, blank) in child.leading.iter().zip(&child.leading_blanks) {
        if *blank {
            doc = doc.append(Doc::hardline());
        }
        doc = doc
            .append(Doc::text(comment.clone()))
            .append(Doc::hardline());
    }
    if child.blank_after_leading {
        doc = doc.append(Doc::hardline());
    }
    doc
}

/// 文字列 s を Doc にする。
pub(self) fn s(text: &str) -> Doc {
    Doc::text(text.to_string())
}
