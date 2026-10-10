//! 式・リスト・レコードなどの Doc 化。
//!
//! 幅に収まれば 1 行、収まらなければ縦に展開する。

use super::{join, s, with_leading, Doc, Kid, Kids, Printer};
use satysfi_parser::{Cst, Rule};

/// `a |> b |> c` の 1 項
struct ChainPart {
    /// この項の前の演算子
    op: Option<String>,
    /// 演算子の前にあったコメント
    comments: Vec<String>,
    operand: Doc,
}

impl<'a> Printer<'a> {
    pub(super) fn expr_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::expr => {
                let kids = self.kids(cst);
                match kids.items.as_slice() {
                    [only] => kids.finish(only.doc.clone()),
                    _ => self.raw(cst),
                }
            }
            Rule::match_expr => self.match_expr(cst),
            Rule::match_arm => self.match_arm(cst),
            Rule::match_guard => {
                let kids = self.kids(cst);
                kids.finish(s("when ").append(kids.items[0].doc.clone()))
            }
            Rule::ctrl_while => self.ctrl_while(cst),
            Rule::ctrl_if => self.ctrl_if(cst),
            // then / else の中身。`if` が `then` / `else` を付ける
            Rule::ctrl_then | Rule::ctrl_else => {
                let kids = self.kids(cst);
                kids.finish(kids.items[0].doc.clone())
            }
            Rule::lambda => self.lambda(cst),
            Rule::assignment => {
                let kids = self.kids(cst);
                let doc = kids.items[0]
                    .doc
                    .clone()
                    .append(self.after_op(" <-", &kids.items[1]));
                kids.finish(doc)
            }
            Rule::dyadic_expr => self.chain(cst, Rule::dyadic_expr),
            Rule::sequential_expr => self.chain(cst, Rule::sequential_expr),
            Rule::unary_operator_expr => {
                let kids = self.kids(cst);
                let op = self.text_of(kids.items[0].cst).trim();
                let sep = if op == "-" { "" } else { " " };
                let doc = s(op).append(s(sep)).append(kids.items[1].doc.clone());
                kids.finish(doc)
            }
            Rule::application => self.application(cst),
            Rule::application_args_normal => {
                let kids = self.kids(cst);
                kids.finish(kids.items[0].doc.clone())
            }
            Rule::application_args_optional => {
                let kids = self.kids(cst);
                match kids.items.first() {
                    Some(arg) => kids.finish(s("?:").append(arg.doc.clone())),
                    None => s("?*"),
                }
            }
            Rule::command_application => {
                let kids = self.kids(cst);
                kids.finish(s("command ").append(kids.items[0].doc.clone()))
            }
            Rule::variant_constructor => {
                let kids = self.kids(cst);
                let doc = match kids.items.as_slice() {
                    [name, arg] => {
                        let sep = self.adjacent_sep(name.cst, arg.cst);
                        name.doc.clone().append(s(sep)).append(arg.doc.clone())
                    }
                    [name] => name.doc.clone(),
                    _ => self.raw(cst),
                };
                kids.finish(doc)
            }
            Rule::unary => self.unary(cst),
            Rule::path => self.path(cst),
            Rule::path_line => {
                let kids = self.kids(cst);
                kids.finish(s("-- ").append(kids.items[0].doc.clone()))
            }
            Rule::path_curve => {
                let kids = self.kids(cst);
                let doc = s(".. controls ")
                    .append(kids.items[0].doc.clone())
                    .append(s(" and "))
                    .append(kids.items[1].doc.clone())
                    .append(s(" .. "))
                    .append(kids.items[2].doc.clone());
                kids.finish(doc)
            }
            Rule::path_line_cycle => {
                // `--` と `cycle` の間のコメントは前に出す
                let children = self.children(cst);
                let comments = children
                    .items
                    .into_iter()
                    .flat_map(|c| c.leading.into_iter().chain(c.trailing))
                    .chain(children.dangling);
                with_leading(comments.collect(), s("-- cycle"))
            }
            Rule::path_curve_cycle => {
                let kids = self.kids(cst);
                let doc = s(".. controls ")
                    .append(kids.items[0].doc.clone())
                    .append(s(" and "))
                    .append(kids.items[1].doc.clone())
                    .append(s(" .. cycle"));
                kids.finish(doc)
            }
            Rule::list => self.seq(cst, "[", "]", None, &|c| self.node(c)),
            Rule::record => self.seq(cst, "(|", "|)", Some(Rule::unary), &|c| self.node(c)),
            Rule::record_unit => {
                let kids = self.kids(cst);
                let doc = kids.items[0]
                    .doc
                    .clone()
                    .append(self.after_op(" =", &kids.items[1]));
                kids.finish(doc)
            }
            Rule::tuple => {
                let kids = self.kids(cst);
                self.tuple_doc(kids)
            }
            Rule::expr_with_mod => {
                let kids = self.kids(cst);
                let doc = kids.items[0]
                    .doc
                    .clone()
                    .append(s(".("))
                    .append(kids.items[1].doc.clone())
                    .append(s(")"));
                kids.finish(doc)
            }
            // 葉
            _ => self.raw(cst),
        }
    }

    /// expr / unary の包みを外した中身。
    pub(super) fn core<'c>(&self, cst: &'c Cst) -> &'c Cst {
        match cst.rule {
            Rule::expr if cst.inner.len() == 1 => self.core(&cst.inner[0]),
            Rule::unary
                if cst.inner.len() == 1
                    && matches!(
                        cst.inner[0].rule,
                        Rule::list
                            | Rule::record
                            | Rule::tuple
                            | Rule::block_text
                            | Rule::horizontal_text
                            | Rule::math_text
                    ) =>
            {
                self.core(&cst.inner[0])
            }
            _ => cst,
        }
    }

    /// 開き括弧を行末に置いたまま、自分で折り返せる式か。
    pub(super) fn is_hugging(&self, cst: &Cst) -> bool {
        let core = self.core(cst);
        match core.rule {
            Rule::list
            | Rule::record
            | Rule::tuple
            | Rule::block_text
            | Rule::horizontal_text
            | Rule::math_text => true,
            // `f a [ ... ]` のように、最後の引数が自分で折り返せる適用
            Rule::application => core
                .inner
                .last()
                .and_then(|arg| arg.inner.first())
                .map_or(false, |inner| self.is_hugging(inner)),
            _ => false,
        }
    }

    /// `op value` の value を `op` の行末に置けるか。
    /// 前に単独行のコメントがあるときは、コメントが `op` の行末に移ってしまうので置かない。
    pub(super) fn hugs(&self, value: &Kid) -> bool {
        !value.commented && self.is_hugging(value.cst)
    }

    /// `op value`。括弧で始まる式は行末に置き、それ以外は収まらなければ次の行に送る。
    pub(super) fn after_op(&self, op: &str, value: &Kid) -> Doc {
        if self.hugs(value) {
            s(op).append(s(" ")).append(value.doc.clone())
        } else {
            s(op).append(
                Doc::line()
                    .append(value.doc.clone())
                    .nest(self.indent)
                    .group(),
            )
        }
    }

    /// `( expr )`。`let ... in` や `match` のように必ず複数行になる式は、
    /// 括弧を単独行に置いて中身を字下げする。それ以外は括弧に密着させ、
    /// 収まらなければ中身が自分で折り返す。
    pub(super) fn paren(&self, inner: &Kid) -> Doc {
        let statement_like = matches!(
            self.core(inner.cst).rule,
            Rule::bind_stmt | Rule::match_expr | Rule::sequential_expr
        );
        if statement_like {
            s("(")
                .append(Doc::hardline().append(inner.doc.clone()).nest(self.indent))
                .append(Doc::hardline())
                .append(s(")"))
        } else {
            s("(").append(inner.doc.clone()).append(s(")"))
        }
    }

    /// 2 つのノードの間に空白がなければ空文字、あれば 1 つの空白。
    /// `Some(x)` と `Some x` のように、元の書き方を保つために使う。
    pub(super) fn adjacent_sep(&self, left: &Cst, right: &Cst) -> &'static str {
        if left.span.end == right.span.start {
            ""
        } else {
            " "
        }
    }

    fn match_expr(&self, cst: &Cst) -> Doc {
        let mut children = self.children(cst);
        if children.items.is_empty() {
            return self.raw(cst);
        }
        let scrutinee = children.items.remove(0);
        let mut doc = s("match ")
            .append(with_leading(scrutinee.leading, self.node(scrutinee.cst)))
            .append(s(" with"));
        // 腕の前の行末コメントは `with` の行に残す
        if let Some(comment) = scrutinee.trailing {
            doc = doc.append(s(" ")).append(Doc::text(comment));
        }
        doc = doc.append(self.prefixed_lines(children.items, "| "));
        for comment in children.dangling {
            doc = doc.append(Doc::hardline()).append(Doc::text(comment));
        }
        doc
    }

    fn match_arm(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let (value, head) = kids.items.split_last().unwrap();
        let mut doc = Doc::nil();
        for (i, k) in head.iter().enumerate() {
            if i > 0 {
                doc = doc.append(s(" "));
            }
            doc = doc.append(k.doc.clone());
        }
        kids.finish(doc.append(self.after_op(" ->", value)))
    }

    fn ctrl_while(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let doc = s("while ")
            .append(kids.items[0].doc.clone())
            .append(self.after_op(" do", &kids.items[1]));
        kids.finish(doc)
    }

    fn ctrl_if(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let [cond, then, els] = match kids.items.as_slice() {
            [a, b, c] => [a, b, c],
            _ => return self.raw(cst),
        };
        // `else if` は入れ子にせず並べる
        let else_is_if = els
            .cst
            .inner
            .first()
            .map_or(false, |e| self.core(e).rule == Rule::ctrl_if);
        let else_part = if else_is_if {
            s(" ").append(els.doc.clone())
        } else {
            Doc::line().append(els.doc.clone()).nest(self.indent)
        };
        let doc = s("if ")
            .append(cond.doc.clone())
            .append(s(" then"))
            .append(Doc::line().append(then.doc.clone()).nest(self.indent))
            .append(Doc::line())
            .append(s("else"))
            .append(else_part)
            .group();
        kids.finish(doc)
    }

    fn lambda(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let (body, pats) = kids.items.split_last().unwrap();
        let mut doc = s("fun");
        for p in pats {
            doc = doc.append(s(" ")).append(p.doc.clone());
        }
        kids.finish(doc.append(self.after_op(" ->", body)))
    }

    /// `a |> b |> c` や `a before b before c` のような、右に入れ子になった並び。
    /// 収まらなければ 1 項ずつ改行する。
    fn chain(&self, cst: &Cst, chain_rule: Rule) -> Doc {
        let mut parts: Vec<ChainPart> = vec![];
        let mut carry: Vec<String> = vec![];
        self.collect_chain(cst, chain_rule, &mut parts, &mut carry);
        let is_seq = chain_rule == Rule::sequential_expr;

        let mut iter = parts.into_iter();
        let first = iter.next().unwrap();
        let doc = if is_seq {
            // `a before` の後で改行する。インデントは変えない。
            let mut doc = first.operand;
            for part in iter {
                doc = doc
                    .append(s(" before"))
                    .append(Doc::line())
                    .append(part.operand);
            }
            doc.group()
        } else {
            let mut rest = Doc::nil();
            for part in iter {
                // 演算子の前にあったコメントは、演算子の行の前に置く
                let mut line = Doc::line();
                for comment in part.comments {
                    line = line.append(Doc::text(comment)).append(Doc::hardline());
                }
                rest = rest
                    .append(line)
                    .append(s(&part.op.unwrap_or_default()))
                    .append(s(" "))
                    .append(part.operand);
            }
            first.operand.append(rest.nest(self.indent)).group()
        };
        carry.into_iter().fold(doc, |doc, comment| {
            doc.append(Doc::hardline()).append(Doc::text(comment))
        })
    }

    fn collect_chain(
        &self,
        cst: &Cst,
        chain_rule: Rule,
        parts: &mut Vec<ChainPart>,
        carry: &mut Vec<String>,
    ) {
        let children = self.children(cst);
        // 直前の演算子と、その前にあったコメント
        let mut op: Option<String> = None;
        let mut op_comments: Vec<String> = vec![];
        for child in children.items {
            let mut leading = std::mem::take(carry);
            leading.extend(child.leading);
            carry.extend(child.trailing);
            match child.cst.rule {
                Rule::bin_operator => {
                    op = Some(self.text_of(child.cst).to_string());
                    op_comments = leading;
                }
                rule if rule == chain_rule => {
                    // 入れ子の並びの先頭の項に、ここで拾った演算子を渡す
                    let before = parts.len();
                    let mut inner_carry = leading;
                    self.collect_chain(child.cst, chain_rule, parts, &mut inner_carry);
                    if let Some(first) = parts.get_mut(before) {
                        first.op = op.take();
                        let mut comments = std::mem::take(&mut op_comments);
                        comments.append(&mut first.comments);
                        first.comments = comments;
                    }
                    *carry = inner_carry;
                }
                _ => {
                    // 演算子がない先頭の項のコメントは、項の前に置く
                    let (comments, leading) = if op.is_some() {
                        (std::mem::take(&mut op_comments), leading)
                    } else {
                        (vec![], leading)
                    };
                    parts.push(ChainPart {
                        op: op.take(),
                        comments,
                        operand: with_leading(leading, self.node(child.cst)),
                    });
                }
            }
        }
        carry.extend(children.dangling);
    }

    fn application(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        if kids.len() < 2 {
            return self.raw(cst);
        }
        let func = kids.items[0].doc.clone();
        // 末尾に連続する、括弧で包まれた引数 (リスト・レコード・ブロックなど) は
        // 直前の引数と同じ行に置き、中身を自分で折り返す。
        // それ以外の引数は、収まらなければ 1 引数ずつ改行する。
        let args = &kids.items[1..];
        let hugged_from = args
            .iter()
            .rposition(|arg| {
                arg.commented
                    || !arg
                        .cst
                        .inner
                        .first()
                        .map_or(false, |inner| self.is_hugging(inner))
            })
            .map_or(0, |i| i + 1);
        let mut broken = Doc::nil();
        let mut hugged = Doc::nil();
        for (i, arg) in args.iter().enumerate() {
            if i >= hugged_from {
                hugged = hugged.append(s(" ")).append(arg.doc.clone());
            } else {
                broken = broken.append(Doc::line()).append(arg.doc.clone());
            }
        }
        let doc = func.append(broken.nest(self.indent)).group().append(hugged);
        kids.finish(doc)
    }

    fn unary(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let mut doc = Doc::nil();
        let mut first = true;
        for kid in &kids.items {
            let part = match kid.cst.rule {
                // 接頭辞 (`!` `&` `~`)
                Rule::unary_prefix => {
                    doc = doc.append(s(self.text_of(kid.cst).trim()));
                    continue;
                }
                // `( expr )`
                Rule::expr => self.paren(kid),
                // `( op )`
                Rule::bin_operator => s(&format!("({})", self.text_of(kid.cst).trim())),
                _ => kid.doc.clone(),
            };
            if !first {
                doc = doc.append(s("#"));
            }
            doc = doc.append(part);
            first = false;
        }
        kids.finish(doc)
    }

    fn path(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let mut rest = Doc::nil();
        for k in &kids.items[1..] {
            rest = rest.append(Doc::line()).append(k.doc.clone());
        }
        // 最後のコメントは閉じ記号の前に置く (整形後も同じ階層のコメントになる)
        for comment in &kids.dangling {
            rest = rest
                .append(Doc::hardline())
                .append(Doc::text(comment.clone()));
        }
        s("<[")
            .append(
                Doc::line()
                    .append(kids.items[0].doc.clone())
                    .append(rest)
                    .nest(self.indent),
            )
            .append(Doc::line())
            .append(s("]>"))
            .group()
    }

    /// `(a, b, c)`。収まらなければ 1 要素ずつ改行する。
    pub(super) fn tuple_doc(&self, kids: Kids) -> Doc {
        let mut elems = join(
            kids.items.iter().map(|k| k.doc.clone()),
            s(",").append(Doc::line()),
        );
        // 最後のコメントは閉じ記号の前に置く
        for comment in &kids.dangling {
            elems = elems
                .append(Doc::hardline())
                .append(Doc::text(comment.clone()));
        }
        s("(")
            .append(Doc::line_().append(elems).nest(self.indent))
            .append(Doc::line_())
            .append(s(")"))
            .group()
    }

    /// `[a; b; c]` や `(| a = 1; b = 2 |)` のような `;` 区切りの並び。
    /// 収まれば 1 行 (末尾に `;` を付けない)、収まらなければ縦に展開する (末尾にも `;` を付ける)。
    /// `head` に指定した rule が先頭にあれば、`head with` として先頭に置く。
    pub(super) fn seq(
        &self,
        cst: &Cst,
        open: &'static str,
        close: &'static str,
        head: Option<Rule>,
        elem: &dyn Fn(&Cst) -> Doc,
    ) -> Doc {
        struct Entry {
            leading: Vec<String>,
            doc: Doc,
            trailing: Option<String>,
            is_head: bool,
        }
        let children = self.children(cst);
        let entries: Vec<Entry> = children
            .items
            .into_iter()
            .enumerate()
            .map(|(i, child)| Entry {
                is_head: i == 0 && Some(child.cst.rule) == head,
                doc: elem(child.cst),
                leading: child.leading,
                trailing: child.trailing,
            })
            .collect();
        let pending = children.dangling;
        if entries.is_empty() && pending.is_empty() {
            return s(&format!("{open}{close}"));
        }

        let last_value = entries.iter().rposition(|e| !e.is_head);
        let mut body = Doc::nil();
        // 直前が行コメントで、必ず改行が必要か
        let mut must_break = false;
        let mut first = true;
        for (i, entry) in entries.into_iter().enumerate() {
            if !first {
                body = body.append(if must_break {
                    Doc::hardline()
                } else {
                    Doc::line()
                });
            }
            for comment in entry.leading {
                body = body.append(Doc::text(comment)).append(Doc::hardline());
            }
            let tail = if entry.is_head {
                s(" with")
            } else if Some(i) == last_value {
                // 末尾の `;` は縦に展開したときだけ付ける
                s(";").flat_alt(Doc::nil())
            } else {
                s(";")
            };
            body = body.append(entry.doc).append(tail);
            must_break = entry.trailing.is_some();
            if let Some(comment) = entry.trailing {
                body = body.append(s(" ")).append(Doc::text(comment));
            }
            first = false;
        }
        for comment in pending {
            if !first {
                body = body.append(Doc::hardline());
            }
            body = body.append(Doc::text(comment));
            must_break = true;
            first = false;
        }
        let closing = if must_break {
            Doc::hardline()
        } else {
            Doc::line_()
        };
        s(open)
            .append(Doc::line_().append(body).nest(self.indent))
            .append(closing)
            .append(s(close))
            .group()
    }
}
