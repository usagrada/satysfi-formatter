//! 文 (`let` / `type` / `module` など) の Doc 化。

use super::{s, with_leading, Doc, Kid, Kids, Printer};
use satysfi_parser::{Cst, Rule};

impl<'a> Printer<'a> {
    pub(super) fn stmt_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::let_stmt => {
                let kids = self.kids(cst);
                match kids.items.split_last() {
                    Some((value, head)) => {
                        let doc = s("let ")
                            .append(self.bind_head(head))
                            .append(self.after_op(" =", value));
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
                    doc = doc.append(s(" ")).append(kid.doc.clone());
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
        let head_kids = Kids {
            items: items
                .into_iter()
                .map(|c| {
                    let mut leading = c.leading;
                    // 行末のコメントは、次の要素の前の単独行のコメントとして扱う
                    let doc = with_leading(std::mem::take(&mut leading), self.node(c.cst));
                    Kid { cst: c.cst, doc }
                })
                .collect(),
            dangling: vec![],
        };
        let mut doc = match value_index {
            // let-rec f x = expr (| args = expr)*
            Some(i) => self
                .bind_head(&head_kids.items[..i])
                .append(self.after_op(" =", &head_kids.items[i])),
            // let-rec f (| args = expr)+
            None => head_kids.items[0].doc.clone(),
        };
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
            doc = doc.append(s(" ")).append(kid.doc.clone());
        }
        kids.finish(doc.append(self.after_op(" =", value)))
    }

    fn type_inner(&self, cst: &Cst) -> Doc {
        let kids = self.kids(cst);
        let name = kids
            .items
            .iter()
            .position(|k| k.cst.rule == Rule::type_name)
            .unwrap_or(0);
        let mut doc = Doc::nil();
        for kid in &kids.items[..name] {
            doc = doc.append(kid.doc.clone()).append(s(" "));
        }
        doc = doc.append(kids.items[name].doc.clone());

        let variants: Vec<&Kid> = kids.items[name + 1..]
            .iter()
            .filter(|k| k.cst.rule == Rule::type_variant)
            .collect();
        if !variants.is_empty() {
            // 1 行に収まれば `= A | B`、収まらなければ `|` を行頭に置いて縦に並べる
            let mut body = s("| ").flat_alt(Doc::nil());
            for (i, variant) in variants.iter().enumerate() {
                if i > 0 {
                    body = body.append(Doc::line()).append(s("| "));
                }
                body = body.append(variant.doc.clone());
            }
            doc = doc
                .append(s(" ="))
                .append(Doc::line().append(body).nest(self.indent).group());
        } else if let Some(ty) = kids.items[name + 1..]
            .iter()
            .find(|k| k.cst.rule == Rule::type_expr)
        {
            doc = doc.append(self.after_op(" =", ty));
        }
        let mut constraints = Doc::nil();
        for kid in kids.items[name + 1..]
            .iter()
            .filter(|k| k.cst.rule == Rule::constraint)
        {
            constraints = constraints.append(Doc::hardline()).append(kid.doc.clone());
        }
        kids.finish(doc.append(constraints.nest(self.indent)))
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
