//! 文 (`let` / `type` / `module` など) の Doc 化。

use super::{s, with_leading, Child, Doc, Kid, Kids, Printer};
use satysfi_parser::{Cst, Rule};

impl<'a> Printer<'a> {
    pub(super) fn stmt_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::let_stmt => {
                let kids = self.kids(cst);
                match kids.items.split_last() {
                    Some((value, head)) => {
                        let doc = self.definition(s("let ").append(self.bind_head(head)), value);
                        kids.finish(doc)
                    }
                    None => self.raw(cst),
                }
            }
            Rule::let_rec_stmt => self.and_chain("let-rec ", cst),
            Rule::let_rec_inner => self.let_rec_inner(cst),
            Rule::let_rec_matcharm => {
                let kids = self.kids(cst);
                let (value, args) = kids.items.split_last().unwrap();
                let mut doc = Doc::nil();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        doc = doc.append(s(" "));
                    }
                    doc = doc.append(arg.doc.clone());
                }
                let eq = if args.is_empty() { "=" } else { " =" };
                kids.finish(doc.append(self.after_op(eq, value)))
            }
            Rule::let_inline_stmt_ctx | Rule::let_inline_stmt_noctx => {
                self.let_command("let-inline", cst)
            }
            Rule::let_block_stmt_ctx | Rule::let_block_stmt_noctx => {
                self.let_command("let-block", cst)
            }
            Rule::let_math_stmt => self.let_command("let-math", cst),
            Rule::let_mutable_stmt => {
                let kids = self.kids(cst);
                let doc = s("let-mutable ")
                    .append(kids.items[0].doc.clone())
                    .append(self.after_op(" <-", &kids.items[1]));
                kids.finish(doc)
            }
            Rule::type_stmt => self.and_chain("type ", cst),
            Rule::type_inner => self.type_inner(cst),
            Rule::type_variant => {
                let kids = self.kids(cst);
                let mut doc = kids.items[0].doc.clone();
                if let Some(ty) = kids.items.get(1) {
                    doc = doc.append(s(" of ")).append(ty.doc.clone());
                }
                kids.finish(doc)
            }
            Rule::constraint => {
                let kids = self.kids(cst);
                let doc = s("constraint ")
                    .append(kids.items[0].doc.clone())
                    .append(s(" :: "))
                    .append(kids.items[1].doc.clone());
                kids.finish(doc)
            }
            Rule::module_stmt => {
                let kids = self.kids(cst);
                let mut doc = s("module ").append(kids.items[0].doc.clone());
                for kid in &kids.items[1..] {
                    doc = match kid.cst.rule {
                        Rule::sig_stmt => doc.append(s(" : ")).append(kid.doc.clone()),
                        _ => doc.append(s(" = ")).append(kid.doc.clone()),
                    };
                }
                kids.finish(doc)
            }
            Rule::open_stmt => {
                let kids = self.kids(cst);
                kids.finish(s("open ").append(kids.items[0].doc.clone()))
            }
            Rule::arg => {
                let kids = self.kids(cst);
                let doc = match kids.items[0].cst.rule {
                    // `?:name`
                    Rule::var_ptn => s("?:").append(kids.items[0].doc.clone()),
                    _ => kids.items[0].doc.clone(),
                };
                kids.finish(doc)
            }
            Rule::sig_stmt => self.block_of("sig", cst),
            Rule::struct_stmt => self.block_of("struct", cst),
            Rule::sig_type_stmt => {
                let kids = self.kids(cst);
                let mut doc = s("type");
                let mut constraints = Doc::nil();
                for kid in &kids.items {
                    if kid.cst.rule == Rule::constraint {
                        constraints = constraints.append(Doc::hardline()).append(kid.doc.clone());
                    } else {
                        doc = doc.append(s(" ")).append(kid.doc.clone());
                    }
                }
                kids.finish(doc.append(constraints.nest(self.indent)))
            }
            Rule::sig_val_stmt | Rule::sig_direct_stmt => {
                let kids = self.kids(cst);
                let keyword = if cst.rule == Rule::sig_val_stmt {
                    "val "
                } else {
                    "direct "
                };
                let name = match kids.items[0].cst.rule {
                    Rule::bin_operator => s("(").append(kids.items[0].doc.clone()).append(s(")")),
                    _ => kids.items[0].doc.clone(),
                };
                let mut doc = s(keyword)
                    .append(name)
                    .append(s(" : "))
                    .append(kids.items[1].doc.clone());
                let mut constraints = Doc::nil();
                for kid in &kids.items[2..] {
                    constraints = constraints.append(Doc::hardline()).append(kid.doc.clone());
                }
                doc = doc.append(constraints.nest(self.indent));
                kids.finish(doc)
            }
            Rule::bind_stmt => {
                // `let ... in` の後に本体を同じインデントで続ける
                let kids = self.kids(cst);
                let doc = kids.items[0]
                    .doc
                    .clone()
                    .append(s(" in"))
                    .append(Doc::hardline())
                    .append(kids.items[1].doc.clone());
                kids.finish(doc)
            }
            _ => self.raw(cst),
        }
    }

    /// `pattern [: type] [| ]args` の部分。
    fn bind_head(&self, head: &[Kid]) -> Doc {
        let mut doc = Doc::nil();
        let mut has_type = false;
        let mut bar = false;
        for (i, kid) in head.iter().enumerate() {
            match kid.cst.rule {
                Rule::type_expr => {
                    doc = doc.append(s(" : ")).append(kid.doc.clone());
                    has_type = true;
                }
                Rule::arg => {
                    if has_type && !bar {
                        doc = doc.append(s(" |"));
                        bar = true;
                    }
                    doc = doc.append(self.param(kid));
                }
                _ => {
                    if i > 0 {
                        doc = doc.append(s(" "));
                    }
                    doc = doc.append(kid.doc.clone());
                }
            }
        }
        doc
    }

    /// 関数定義の引数 1 つ。収まる限り前の引数と同じ行に詰め、収まらなければ改行する。
    /// 続きの行は本体と見分けられるよう、本体より 1 段深く字下げする
    /// (ocamlformat や clang-format の ContinuationIndentWidth と同じ考え方)。
    /// パターンの中身はなるべく分解せず、引数ごと次の行に送る。
    fn param(&self, kid: &Kid) -> Doc {
        Doc::line()
            .append(kid.doc.clone())
            .nest(self.indent * 2)
            .group()
    }

    /// `keyword a and b and c`。2 つ目以降は改行して `and` を付ける。
    fn and_chain(&self, keyword: &str, cst: &Cst) -> Doc {
        let mut children = self.children(cst);
        if children.items.is_empty() {
            return self.raw(cst);
        }
        let first = children.items.remove(0);
        let mut doc = with_leading(first.leading, s(keyword).append(self.node(first.cst)));
        if let Some(comment) = first.trailing {
            doc = doc.append(s(" ")).append(Doc::text(comment));
        }
        doc = doc.append(self.prefixed_lines(children.items, "and "));
        for comment in children.dangling {
            doc = doc.append(Doc::hardline()).append(Doc::text(comment));
        }
        doc
    }

    fn let_rec_inner(&self, cst: &Cst) -> Doc {
        let children = self.children(cst);
        let value_index = children.items.iter().position(|c| c.cst.rule == Rule::expr);
        // 腕 (`| args = expr`) より前の部分と、腕に分ける
        let head_end = value_index.map_or(1, |i| i + 1);
        let mut items = children.items;
        let arms = items.split_off(head_end.min(items.len()));
        // 行末のコメントは、次の要素の前の単独行のコメントとして扱う
        let mut carry: Vec<String> = vec![];
        let mut head_items = vec![];
        for c in items {
            let mut leading = std::mem::take(&mut carry);
            leading.extend(c.leading);
            carry.extend(c.trailing);
            head_items.push(Kid {
                cst: c.cst,
                doc: with_leading(leading, self.node(c.cst)),
            });
        }
        let head_kids = Kids {
            items: head_items,
            dangling: vec![],
        };
        let mut doc = match value_index {
            // let-rec f x = expr (| args = expr)*
            Some(i) => self.definition(self.bind_head(&head_kids.items[..i]), &head_kids.items[i]),
            // let-rec f (| args = expr)+
            None => head_kids.items[0].doc.clone(),
        };
        // 腕の前の行末コメントは、本体の行に残す
        for comment in carry {
            doc = doc.append(s(" ")).append(Doc::text(comment));
        }
        doc = doc.append(self.prefixed_lines(arms, "| ").nest(self.indent));
        for comment in children.dangling {
            doc = doc.append(Doc::hardline()).append(Doc::text(comment));
        }
        doc
    }

    /// `let-inline ctx \cmd a b = body` などの形。最後の子が本体。
    fn let_command(&self, keyword: &str, cst: &Cst) -> Doc {
        let kids: Kids = self.kids(cst);
        let (value, head) = kids.items.split_last().unwrap();
        let mut doc = s(keyword);
        for kid in head {
            doc = match kid.cst.rule {
                Rule::arg => doc.append(self.param(kid)),
                _ => doc.append(s(" ")).append(kid.doc.clone()),
            };
        }
        kids.finish(self.definition(doc, value))
    }

    /// `head = value`。ヘッダを折り返したときは、本体を必ず次の行に送る。
    /// 括弧で始まる本体 (`= [` や `= (fun x ->`) は `=` の行末に置く。
    fn definition(&self, head: Doc, value: &Kid) -> Doc {
        if self.is_hugging(value.cst) {
            head.append(self.after_op(" =", value))
        } else {
            head.append(s(" ="))
                .append(Doc::line().append(value.doc.clone()).nest(self.indent))
                .group()
        }
    }

    fn type_inner(&self, cst: &Cst) -> Doc {
        let mut children = self.children(cst);
        // 行末のコメントは、次の子の前の単独行のコメントとして扱う
        let mut carry: Vec<String> = vec![];
        for child in children.items.iter_mut() {
            let mut leading = std::mem::take(&mut carry);
            leading.append(&mut child.leading);
            child.leading = leading;
            // variant の行末コメントはそのまま残す
            if child.cst.rule != Rule::type_variant {
                carry.extend(child.trailing.take());
            }
        }
        children.dangling.splice(0..0, carry);
        let name = children
            .items
            .iter()
            .position(|c| c.cst.rule == Rule::type_name)
            .unwrap_or(0);
        let mut doc = Doc::nil();
        let mut variants: Vec<&Child> = vec![];
        let mut constraints = Doc::nil();
        let mut synonym: Option<Kid> = None;
        for (i, child) in children.items.iter().enumerate() {
            let kid_doc = with_leading(child.leading.clone(), self.node(child.cst));
            if i < name {
                doc = doc.append(kid_doc).append(s(" "));
            } else if i == name {
                doc = doc.append(kid_doc);
            } else {
                match child.cst.rule {
                    Rule::type_variant => variants.push(child),
                    Rule::constraint => {
                        constraints = constraints.append(Doc::hardline()).append(kid_doc);
                    }
                    _ => {
                        synonym = Some(Kid {
                            cst: child.cst,
                            doc: kid_doc,
                        })
                    }
                }
            }
        }
        if !variants.is_empty() {
            // 1 行に収まれば `= A | B`、収まらなければ `|` を行頭に置いて縦に並べる。
            // コメントがあるときは必ず縦に並べる。
            let mut body = Doc::nil();
            let mut must_break = false;
            for (i, variant) in variants.iter().enumerate() {
                if i > 0 {
                    body = body.append(if must_break {
                        Doc::hardline()
                    } else {
                        Doc::line()
                    });
                }
                for comment in &variant.leading {
                    body = body
                        .append(Doc::text(comment.clone()))
                        .append(Doc::hardline());
                }
                let bar = if i == 0 {
                    s("| ").flat_alt(Doc::nil())
                } else {
                    s("| ")
                };
                body = body.append(bar).append(self.node(variant.cst));
                must_break = variant.trailing.is_some();
                if let Some(comment) = &variant.trailing {
                    body = body.append(s(" ")).append(Doc::text(comment.clone()));
                }
            }
            doc = doc
                .append(s(" ="))
                .append(Doc::line().append(body).nest(self.indent).group());
        } else if let Some(ty) = &synonym {
            doc = doc.append(self.after_op(" =", ty));
        }
        doc = doc.append(constraints.nest(self.indent));
        for comment in children.dangling {
            doc = doc.append(Doc::hardline()).append(Doc::text(comment));
        }
        doc
    }

    /// `sig ... end` / `struct ... end`
    fn block_of(&self, keyword: &str, cst: &Cst) -> Doc {
        let children = self.children(cst);
        if children.items.is_empty() && children.dangling.is_empty() {
            return s(&format!("{keyword} end"));
        }
        s(keyword)
            .append(
                Doc::hardline()
                    .append(self.stmt_list(cst))
                    .nest(self.indent),
            )
            .append(Doc::hardline())
            .append(s("end"))
    }
}
