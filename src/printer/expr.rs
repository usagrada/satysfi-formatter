//! 式・リスト・レコードなどの Doc 化。
//!
//! 幅に収まれば 1 行、収まらなければ縦に展開する。
//! コメントを含むノードは、行が必ず折り返されるようにする。

use super::{join, Doc, Printer, INDENT};
use satysfi_parser::{Cst, Item, Rule};

impl<'a> Printer<'a> {
    /// 式まわりの rule を Doc にする。対応していない rule は旧 Formatter に委譲する。
    pub(super) fn expr_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::expr => match cst.inner.as_slice() {
                [inner] if !self.has_comment(cst) => self.node(inner),
                _ => self.legacy(cst),
            },
            Rule::unary => self.unary(cst),
            Rule::list => self.list(cst),
            Rule::record => self.record(cst),
            Rule::tuple => self.tuple(cst),
            Rule::application => self.application(cst),
            Rule::dyadic_expr => self.dyadic(cst),
            Rule::let_stmt => self.let_stmt(cst),
            Rule::var
            | Rule::modvar
            | Rule::const_unit
            | Rule::const_bool
            | Rule::const_int
            | Rule::const_float
            | Rule::const_length
            | Rule::const_string
            | Rule::variant_name
            | Rule::bin_operator
            | Rule::unary_operator => Doc::text(self.text_of(cst).to_string()),
            _ => self.legacy(cst),
        }
    }

    /// expr / unary の包みを外した中身。
    fn core<'c>(&self, cst: &'c Cst) -> &'c Cst {
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
    fn is_hugging(&self, cst: &Cst) -> bool {
        let core = self.core(cst);
        match core.rule {
            Rule::list
            | Rule::record
            | Rule::tuple
            | Rule::block_text
            | Rule::horizontal_text
            | Rule::math_text => true,
            // `f a [ ... ]` のように、最後の引数が自分で折り返せる適用
            Rule::application if !self.has_comment(core) => core
                .inner
                .last()
                .and_then(|arg| arg.inner.first())
                .map_or(false, |inner| self.is_hugging(inner)),
            _ => false,
        }
    }

    /// `= value` の部分。括弧で始まる式は行末に置き、それ以外は収まらなければ次の行に送る。
    fn after_equal(&self, value: &Cst) -> Doc {
        let doc = self.node(value);
        if self.is_hugging(value) {
            Doc::text(" =").append(Doc::text(" ")).append(doc)
        } else {
            Doc::text(" =").append(Doc::line().append(doc).nest(INDENT).group())
        }
    }

    fn let_stmt(&self, cst: &Cst) -> Doc {
        let simple = !self.has_comment(cst)
            && cst.inner.last().map_or(false, |c| c.rule == Rule::expr)
            && cst.inner[..cst.inner.len() - 1]
                .iter()
                .all(|c| matches!(c.rule, Rule::pattern | Rule::arg));
        if !simple {
            return self.legacy(cst);
        }
        let (value, head) = cst.inner.split_last().unwrap();
        let mut doc = Doc::text("let");
        for c in head {
            doc = doc.append(Doc::text(" ")).append(Doc::text(self.text_of(c).to_string()));
        }
        doc.append(self.after_equal(value))
    }

    fn unary(&self, cst: &Cst) -> Doc {
        match cst.inner.as_slice() {
            [inner] if !self.has_comment(cst) => match inner.rule {
                // `( expr )`
                Rule::expr => Doc::text("(").append(self.node(inner)).append(")"),
                // `( op )`
                Rule::bin_operator => Doc::text(format!("({})", self.text_of(inner))),
                _ => self.node(inner),
            },
            _ => self.legacy(cst),
        }
    }

    fn application(&self, cst: &Cst) -> Doc {
        if self.has_comment(cst) || cst.inner.len() < 2 {
            return self.legacy(cst);
        }
        let func = self.node(&cst.inner[0]);
        let args = &cst.inner[1..];
        let arg_doc = |arg: &Cst| -> Doc {
            match (arg.rule, arg.inner.first()) {
                (Rule::application_args_normal, Some(inner)) => self.node(inner),
                (Rule::application_args_optional, Some(inner)) => {
                    Doc::text("?:").append(self.node(inner))
                }
                _ => Doc::text(self.text_of(arg).to_string()),
            }
        };
        // 括弧で包まれた引数 (リスト・レコード・ブロックなど) は直前の引数と同じ行に置き、
        // 中身を自分で折り返す。それ以外の引数は、収まらなければ 1 引数ずつ改行する。
        let mut broken = Doc::nil();
        let mut hugged = Doc::nil();
        let mut seen_hugged = false;
        for arg in args {
            let hugging = arg
                .inner
                .first()
                .map_or(false, |inner| self.is_hugging(inner));
            if hugging {
                seen_hugged = true;
                hugged = hugged.append(Doc::text(" ")).append(arg_doc(arg));
            } else if seen_hugged {
                // 括弧の後に続く引数は同じ行に置く
                hugged = hugged.append(Doc::text(" ")).append(arg_doc(arg));
            } else {
                broken = broken.append(Doc::line()).append(arg_doc(arg));
            }
        }
        func.append(broken.nest(INDENT)).group().append(hugged)
    }

    /// `a |> b |> c` のような演算子の並び。収まらなければ演算子を行頭に置いて縦に並べる。
    fn dyadic(&self, cst: &Cst) -> Doc {
        if self.has_comment(cst) {
            return self.legacy(cst);
        }
        let mut operands = vec![];
        let mut ops = vec![];
        let mut node = cst;
        loop {
            match node.inner.as_slice() {
                [lhs, op, rhs] if node.rule == Rule::dyadic_expr && op.rule == Rule::bin_operator => {
                    operands.push(lhs);
                    ops.push(op);
                    node = rhs;
                }
                _ => {
                    operands.push(node);
                    break;
                }
            }
        }
        let mut rest = Doc::nil();
        for (op, operand) in ops.iter().zip(operands.iter().skip(1)) {
            rest = rest
                .append(Doc::line())
                .append(Doc::text(self.text_of(op).to_string()))
                .append(Doc::text(" "))
                .append(self.node(operand));
        }
        self.node(operands[0]).append(rest.nest(INDENT)).group()
    }

    fn tuple(&self, cst: &Cst) -> Doc {
        if self.has_comment(cst) {
            return self.legacy(cst);
        }
        let elems = cst.inner.iter().map(|c| self.node(c));
        let body = join(elems, Doc::text(",").append(Doc::line()));
        Doc::text("(")
            .append(Doc::line_().append(body).nest(INDENT))
            .append(Doc::line_())
            .append(")")
            .group()
    }

    fn list(&self, cst: &Cst) -> Doc {
        self.sequence(cst, "[", "]", |c| self.node(c))
    }

    fn record(&self, cst: &Cst) -> Doc {
        // `(| r with ... |)` は未対応
        if cst.inner.iter().any(|c| c.rule != Rule::record_unit) {
            return self.legacy(cst);
        }
        self.sequence(cst, "(|", "|)", |c| self.record_unit(c))
    }

    fn record_unit(&self, cst: &Cst) -> Doc {
        match cst.inner.as_slice() {
            [name, value] if !self.has_comment(cst) => {
                Doc::text(self.text_of(name).to_string()).append(self.after_equal(value))
            }
            _ => self.legacy(cst),
        }
    }

    /// `[a; b; c]` や `(| a = 1; b = 2 |)` のような `;` 区切りの並び。
    /// 収まれば 1 行 (末尾に `;` を付けない)、収まらなければ縦に展開する (末尾にも `;` を付ける)。
    fn sequence(
        &self,
        cst: &Cst,
        open: &'static str,
        close: &'static str,
        elem: impl Fn(&Cst) -> Doc,
    ) -> Doc {
        struct Entry {
            leading: Vec<String>,
            doc: Doc,
            trailing: Option<String>,
        }
        let mut entries: Vec<Entry> = vec![];
        let mut pending: Vec<String> = vec![];
        for item in self.csttext.items(cst) {
            match item {
                Item::Node(child) => entries.push(Entry {
                    leading: std::mem::take(&mut pending),
                    doc: elem(child),
                    trailing: None,
                }),
                Item::Comment { span, own_line } => {
                    let comment = self.comment(span);
                    match entries.last_mut() {
                        Some(last) if !own_line && last.trailing.is_none() && pending.is_empty() => {
                            last.trailing = Some(comment)
                        }
                        _ => pending.push(comment),
                    }
                }
                Item::BlankLines(_) => {}
            }
        }
        if entries.is_empty() && pending.is_empty() {
            return Doc::text(format!("{open}{close}"));
        }

        let count = entries.len();
        let mut body = Doc::nil();
        // 直前が行コメントで、必ず改行が必要か
        let mut must_break = false;
        let mut first = true;
        for (i, entry) in entries.into_iter().enumerate() {
            if !first {
                body = body.append(if must_break { Doc::hardline() } else { Doc::line() });
            }
            for comment in entry.leading {
                body = body.append(Doc::text(comment)).append(Doc::hardline());
            }
            let semicolon = if i + 1 < count {
                Doc::text(";")
            } else {
                // 末尾の `;` は縦に展開したときだけ付ける
                Doc::text(";").flat_alt(Doc::nil())
            };
            body = body.append(entry.doc).append(semicolon);
            must_break = entry.trailing.is_some();
            if let Some(comment) = entry.trailing {
                body = body.append(Doc::text(" ")).append(Doc::text(comment));
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
        let closing = if must_break { Doc::hardline() } else { Doc::line_() };
        Doc::text(open)
            .append(Doc::line_().append(body).nest(INDENT))
            .append(closing)
            .append(Doc::text(close))
            .group()
    }
}
