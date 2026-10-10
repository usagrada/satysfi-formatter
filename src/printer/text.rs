//! 水平・垂直・数式モードとコマンドの Doc 化。
//!
//! 水平モードでは空白が意味を持つ。ただし連続する空白は 1 つにまとまるので、
//! 空白の連続は「空白 1 つ」または「改行 1 つ」に正規化する (改行があれば改行を残す)。
//! `{` の直後と `}` の直前の空白は無視される。コメントはその後ろの空白も読み飛ばす。

use super::{s, Doc, Printer};
use satysfi_parser::{Cst, Rule};

/// 水平モードの要素。
enum Piece {
    Token(Tok),
    Space,
    Newline,
    Comment(String),
}

impl<'a> Printer<'a> {
    pub(super) fn text_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::block_text => {
                let kids = self.kids(cst);
                kids.finish(s("'").append(kids.items[0].doc.clone()))
            }
            Rule::horizontal_text | Rule::cmd_text_arg => {
                let kids = self.kids(cst);
                kids.finish(kids.items[0].doc.clone())
            }
            Rule::math_text => {
                let kids = self.kids(cst);
                let (body, multiline) = self.math_body(&kids.items[0]);
                kids.finish(self.wrap_math("${", body, multiline))
            }
            Rule::vertical => self.vertical(cst),
            Rule::horizontal_single => {
                let (body, multiline) = self.horizontal_body(cst);
                self.braces(body, multiline)
            }
            Rule::horizontal_list => self.horizontal_list(cst),
            Rule::horizontal_bullet_list => self.horizontal_bullet_list(cst),
            Rule::inline_cmd | Rule::block_cmd => self.command(cst),
            Rule::cmd_expr_arg | Rule::cmd_expr_option => {
                let kids = self.kids(cst);
                let prefix = if cst.rule == Rule::cmd_expr_option {
                    "?:"
                } else {
                    ""
                };
                let doc = match kids.items.first() {
                    // `?*`
                    None => return self.raw(cst),
                    Some(inner) => {
                        // マクロの先行引数 `~(expr)`
                        let tilde = self.csttext.text[cst.span.start..inner.cst.span.start]
                            .contains('~');
                        let body = match inner.cst.rule {
                            Rule::expr => self.paren(inner),
                            _ => inner.doc.clone(),
                        };
                        s(prefix).append(s(if tilde { "~" } else { "" })).append(body)
                    }
                };
                kids.finish(doc)
            }
            Rule::math_cmd => {
                let kids = self.kids(cst);
                let mut doc = Doc::nil();
                for kid in &kids.items {
                    doc = doc.append(kid.doc.clone());
                }
                kids.finish(doc)
            }
            Rule::math_cmd_expr_arg | Rule::math_cmd_expr_option => {
                let kids = self.kids(cst);
                let prefix = if cst.rule == Rule::math_cmd_expr_option {
                    "?:"
                } else {
                    ""
                };
                let inner = &kids.items[0];
                let body = match inner.cst.rule {
                    Rule::math_single | Rule::math_list => {
                        let (body, multiline) = self.math_body(inner);
                        self.wrap_math("{", body, multiline)
                    }
                    Rule::expr => s("!").append(self.paren(inner)),
                    // 水平モード `!{ ... }`、垂直モード `!<...>`、`![...]`、`!(|...|)`
                    _ => s("!").append(inner.doc.clone()),
                };
                kids.finish(s(prefix).append(body))
            }
            Rule::math_single | Rule::math_list => {
                let kid = super::Kid {
                    cst,
                    doc: Doc::nil(),
                };
                self.math_body(&kid).0
            }
            Rule::math_token => {
                let kids = self.kids(cst);
                let mut doc = Doc::nil();
                for kid in &kids.items {
                    doc = doc.append(kid.doc.clone());
                }
                kids.finish(doc)
            }
            Rule::math_sup | Rule::math_sub => {
                let kids = self.kids(cst);
                let prefix = if cst.rule == Rule::math_sup { "^" } else { "_" };
                let inner = &kids.items[0];
                let body = match inner.cst.rule {
                    Rule::math_single => {
                        let (body, multiline) = self.math_body(inner);
                        self.wrap_math("{", body, multiline)
                    }
                    _ => inner.doc.clone(),
                };
                kids.finish(s(prefix).append(body))
            }
            Rule::math_unary => {
                let kids = self.kids(cst);
                match kids.items.first() {
                    None => self.raw(cst),
                    Some(inner) => kids.finish(inner.doc.clone()),
                }
            }
            // 葉
            _ => self.raw(cst),
        }
    }

    /// `{ body }`。中身が複数行なら、`{` と `}` を単独行に置いて縦に展開する。
    /// 1 行の文章は、行幅を超えても `{` と `}` を割らない。
    fn braces(&self, body: Option<Doc>, multiline: bool) -> Doc {
        match body {
            None => s("{}"),
            Some(body) if multiline => s("{")
                .append(Doc::hardline().append(body).nest(self.indent))
                .append(Doc::hardline())
                .append(s("}")),
            Some(body) => s("{ ").append(body).append(s(" }")),
        }
    }

    /// 整形後に必ず改行を含むか。元の文字列の改行ではなく、整形後の構造だけで決める
    /// (整形の前後で結果が変わらないようにするため)。
    fn token_multiline(&self, cst: &Cst) -> bool {
        match cst.rule {
            Rule::inline_cmd | Rule::cmd_text_arg | Rule::horizontal_text => {
                cst.inner.iter().any(|c| self.token_multiline(c))
            }
            Rule::horizontal_single => {
                self.horizontal_pieces(cst, &|c| Tok {
                    doc: Doc::nil(),
                    multiline: self.token_multiline(c),
                })
                .1
            }
            Rule::horizontal_list => cst.inner.iter().any(|c| self.token_multiline(c)),
            Rule::horizontal_bullet_list | Rule::vertical | Rule::block_text => {
                !cst.inner.is_empty()
            }
            Rule::math_text => cst.inner.iter().any(|c| self.math_multiline(c)),
            Rule::const_string => self.text_of(cst).contains('\n'),
            _ => false,
        }
    }

    /// 数式が整形後に複数行になるか。`math_body` が改行を残す条件と同じ。
    fn math_multiline(&self, cst: &Cst) -> bool {
        match cst.rule {
            Rule::math_single => {
                let children = self.children(cst);
                if !children.dangling.is_empty() {
                    return true;
                }
                let mut prev_end: Option<usize> = None;
                for child in &children.items {
                    if !child.leading.is_empty() || child.trailing.is_some() {
                        return true;
                    }
                    if let Some(end) = prev_end {
                        if self.csttext.text[end..child.cst.span.start].contains('\n') {
                            return true;
                        }
                    }
                    if self.math_multiline(child.cst) {
                        return true;
                    }
                    prev_end = Some(child.cst.span.end);
                }
                false
            }
            Rule::math_list
            | Rule::math_token
            | Rule::math_sup
            | Rule::math_sub
            | Rule::math_unary
            | Rule::math_cmd
            | Rule::math_cmd_expr_arg
            | Rule::math_cmd_expr_option => cst.inner.iter().any(|c| self.math_multiline(c)),
            Rule::horizontal_single
            | Rule::horizontal_list
            | Rule::horizontal_bullet_list
            | Rule::vertical => self.token_multiline(cst),
            _ => false,
        }
    }

    /// horizontal_single を要素の列にして、空白を正規化する。(要素, 複数行になるか)
    fn horizontal_pieces(&self, cst: &Cst, token: &dyn Fn(&Cst) -> Tok) -> (Vec<Piece>, bool) {
        let children = self.children(cst);
        let mut pieces: Vec<Piece> = vec![];
        for child in children.items {
            pieces.extend(child.leading.into_iter().map(Piece::Comment));
            match child.cst.rule {
                Rule::regular_text => split_text(self.text_of(child.cst), &mut pieces),
                _ => pieces.push(Piece::Token(token(child.cst))),
            }
            pieces.extend(child.trailing.map(Piece::Comment));
        }
        pieces.extend(children.dangling.into_iter().map(Piece::Comment));

        // 空白を正規化する
        let mut normalized: Vec<Piece> = vec![];
        for piece in pieces {
            match piece {
                Piece::Space | Piece::Newline => match normalized.last_mut() {
                    // 先頭とコメントの直後の空白は読み飛ばされる
                    None | Some(Piece::Comment(_)) => {}
                    Some(Piece::Space) => {
                        if matches!(piece, Piece::Newline) {
                            *normalized.last_mut().unwrap() = Piece::Newline;
                        }
                    }
                    Some(Piece::Newline) => {}
                    Some(Piece::Token(_)) => normalized.push(piece),
                },
                _ => normalized.push(piece),
            }
        }
        // 末尾の空白は読み飛ばされる
        while matches!(normalized.last(), Some(Piece::Space | Piece::Newline)) {
            normalized.pop();
        }
        let multiline = normalized.iter().any(|p| match p {
            Piece::Newline | Piece::Comment(_) => true,
            Piece::Token(t) => t.multiline,
            Piece::Space => false,
        });
        (normalized, multiline)
    }

    /// horizontal_single の中身。(Doc, 複数行になるか)
    /// 中身が空なら None。
    fn horizontal_body(&self, cst: &Cst) -> (Option<Doc>, bool) {
        let multiline_of = |c: &Cst| self.token_multiline(c);
        let (pieces, multiline) = self.horizontal_pieces(cst, &|c| {
            Tok {
                doc: self.node(c),
                multiline: multiline_of(c),
            }
        });
        if pieces.is_empty() {
            return (None, false);
        }
        let count = pieces.len();
        // 後ろに続くものがないコメントは、直前の文字との間に空白を入れても意味が変わらない
        let only_comments_follow: Vec<bool> = {
            let mut flags = vec![false; count];
            let mut only = true;
            for i in (0..count).rev() {
                flags[i] = only;
                only &= matches!(pieces[i], Piece::Comment(_));
            }
            flags
        };
        let mut doc = Doc::nil();
        let mut prev_is_token = false;
        for (i, piece) in pieces.into_iter().enumerate() {
            let is_token = matches!(piece, Piece::Token(_));
            doc = match piece {
                Piece::Token(t) => doc.append(t.doc),
                Piece::Space => doc.append(s(" ")),
                Piece::Newline => doc.append(Doc::hardline()),
                Piece::Comment(c) => {
                    let doc = if prev_is_token && only_comments_follow[i] {
                        doc.append(s(" "))
                    } else {
                        doc
                    };
                    let doc = doc.append(Doc::text(c));
                    // コメントは行末まで続くので、後ろには必ず改行を置く
                    if i + 1 < count {
                        doc.append(Doc::hardline())
                    } else {
                        doc
                    }
                }
            };
            prev_is_token = is_token;
        }
        (Some(doc), multiline)
    }

    /// `{ | a | b | }`
    fn horizontal_list(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        let mut body = Doc::nil();
        for child in &children.items {
            let (cell, _) = self.horizontal_body(child.cst);
            body = body
                .append(Doc::line())
                .append(s("| "))
                .append(cell.unwrap_or_else(Doc::nil));
        }
        body = body.append(Doc::line()).append(s("|"));
        s("{")
            .append(body.nest(self.indent))
            .append(Doc::line())
            .append(s("}"))
            .group()
    }

    /// `{ * a * b }`。項目ごとに改行する。
    fn horizontal_bullet_list(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        let mut body = Doc::nil();
        for child in &children.items {
            let bullet = child.cst;
            let star = bullet
                .inner
                .iter()
                .find(|c| c.rule == Rule::horizontal_bullet_star)
                .map_or("*", |c| self.text_of(c).trim());
            let single = bullet
                .inner
                .iter()
                .find(|c| c.rule == Rule::horizontal_single);
            let content = single
                .and_then(|single| self.horizontal_body(single).0)
                .unwrap_or_else(Doc::nil);
            body = body
                .append(Doc::hardline())
                .append(s(star))
                .append(s(" "))
                .append(content);
        }
        s("{")
            .append(body.nest(self.indent))
            .append(Doc::hardline())
            .append(s("}"))
    }

    /// `<+p { a } +p { b }>`。要素ごとに改行する。
    fn vertical(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        if children.items.is_empty() && children.dangling.is_empty() {
            return s("<>");
        }
        let mut body = Doc::nil();
        let mut first = true;
        for child in children.items {
            if !first {
                body = body.append(Doc::hardline());
                if child.blank_before {
                    body = body.append(Doc::hardline());
                }
            }
            for comment in child.leading {
                body = body.append(Doc::text(comment)).append(Doc::hardline());
            }
            body = body.append(self.node(child.cst));
            if let Some(comment) = child.trailing {
                body = body.append(s(" ")).append(Doc::text(comment));
            }
            first = false;
        }
        for comment in children.dangling {
            if !first {
                body = body.append(Doc::hardline());
            }
            body = body.append(Doc::text(comment));
            first = false;
        }
        s("<")
            .append(Doc::hardline().append(body).nest(self.indent))
            .append(Doc::hardline())
            .append(s(">"))
    }

    /// `\name(arg){ text }` / `+name(arg){ text }`。テキスト引数がなければ `;` で終える。
    fn command(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let mut doc = kids.items[0].doc.clone();
        let mut has_text = false;
        for kid in &kids.items[1..] {
            has_text |= kid.cst.rule == Rule::cmd_text_arg;
            doc = doc.append(s(" ")).append(kid.doc.clone());
        }
        if !has_text {
            doc = doc.append(s(";"));
        }
        kids.finish(doc)
    }

    // ---- 数式 ----

    /// `${ ... }` / `{ ... }` で囲む。複数行なら中身を字下げして囲み記号を単独行に置く。
    fn wrap_math(&self, open: &str, body: Doc, multiline: bool) -> Doc {
        if multiline {
            s(open)
                .append(Doc::hardline().append(body).nest(self.indent))
                .append(Doc::hardline())
                .append(s("}"))
        } else {
            s(open).append(body).append(s("}"))
        }
    }

    /// math_single / math_list の中身。(Doc, 複数行か)
    /// 空白は、元の文字列で空白があったかどうかを保つ。
    fn math_body(&self, kid: &super::Kid) -> (Doc, bool) {
        let cst = kid.cst;
        let children = self.children(cst);
        let mut doc = Doc::nil();
        let mut multiline = false;
        let mut prev_end: Option<usize> = None;
        let is_list = cst.rule == Rule::math_list;
        let mut first = true;
        for child in children.items {
            for comment in child.leading {
                if !first {
                    doc = doc.append(Doc::hardline());
                }
                doc = doc.append(Doc::text(comment)).append(Doc::hardline());
                multiline = true;
                prev_end = None;
                first = false;
            }
            if is_list {
                // `| a | b |`
                doc = doc.append(s(if first { "| " } else { " | " }));
                let (cell, cell_multiline) = self.math_body(&super::Kid {
                    cst: child.cst,
                    doc: Doc::nil(),
                });
                multiline |= cell_multiline;
                doc = doc.append(cell);
            } else {
                if let Some(end) = prev_end {
                    let gap = &self.csttext.text[end..child.cst.span.start];
                    if gap.contains('\n') {
                        doc = doc.append(Doc::hardline());
                        multiline = true;
                    } else if gap.chars().any(char::is_whitespace) {
                        doc = doc.append(s(" "));
                    }
                }
                doc = doc.append(self.node(child.cst));
            }
            prev_end = Some(child.cst.span.end);
            first = false;
            if let Some(comment) = child.trailing {
                doc = doc
                    .append(s(" "))
                    .append(Doc::text(comment))
                    .append(Doc::hardline());
                multiline = true;
                prev_end = None;
            }
        }
        if is_list {
            doc = doc.append(s(" |"));
        }
        for comment in children.dangling {
            doc = doc.append(Doc::hardline()).append(Doc::text(comment));
            multiline = true;
        }
        (doc, multiline)
    }
}

/// 命令など、単語以外の要素。
struct Tok {
    doc: Doc,
    multiline: bool,
}

/// 水平モードの文字列を、単語と空白に分ける。
fn split_text(text: &str, pieces: &mut Vec<Piece>) {
    let mut word = String::new();
    let mut whitespace = String::new();
    let flush_word = |word: &mut String, pieces: &mut Vec<Piece>| {
        if !word.is_empty() {
            pieces.push(Piece::Token(Tok {
                doc: Doc::text(std::mem::take(word)),
                multiline: false,
            }));
        }
    };
    let flush_space = |whitespace: &mut String, pieces: &mut Vec<Piece>| {
        if !whitespace.is_empty() {
            pieces.push(if whitespace.contains('\n') {
                Piece::Newline
            } else {
                Piece::Space
            });
            whitespace.clear();
        }
    };
    for ch in text.chars() {
        if ch.is_whitespace() {
            flush_word(&mut word, pieces);
            whitespace.push(ch);
        } else {
            flush_space(&mut whitespace, pieces);
            word.push(ch);
        }
    }
    flush_word(&mut word, pieces);
    flush_space(&mut whitespace, pieces);
}
