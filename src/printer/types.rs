//! 型の Doc 化。

use super::{join, s, Doc, Kid, Printer};
use satysfi_parser::{Cst, Rule};

impl<'a> Printer<'a> {
    pub(super) fn type_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::type_expr => {
                // `A -> B ?-> C`
                let kids = self.kids(cst);
                let count = kids.len();
                let mut iter = kids.items.iter().enumerate();
                let first = match iter.next() {
                    Some((i, kid)) => self.arrow(kid, i + 1 < count),
                    None => return self.raw(cst),
                };
                let mut rest = Doc::nil();
                for (i, kid) in iter {
                    rest = rest.append(Doc::line()).append(self.arrow(kid, i + 1 < count));
                }
                kids.finish(first.append(rest.nest(self.indent)).group())
            }
            Rule::type_optional | Rule::type_prod => {
                let kids = self.kids(cst);
                let docs = kids.items.iter().map(|k| self.type_atom(k));
                kids.finish(join(docs, s(" * ")))
            }
            Rule::type_inline_cmd => self.type_command(cst, "inline-cmd"),
            Rule::type_block_cmd => self.type_command(cst, "block-cmd"),
            Rule::type_math_cmd => self.type_command(cst, "math-cmd"),
            Rule::type_list_unit_optional => {
                let kids = self.kids(cst);
                kids.finish(kids.items[0].doc.clone().append(s("?")))
            }
            Rule::type_application => {
                let kids = self.kids(cst);
                let docs = kids.items.iter().map(|k| self.type_atom(k));
                kids.finish(join(docs, s(" ")))
            }
            Rule::type_record => self.seq(cst, "(|", "|)", None, &|c| self.node(c)),
            Rule::type_record_unit => {
                let kids = self.kids(cst);
                let doc = kids.items[0]
                    .doc
                    .clone()
                    .append(s(": "))
                    .append(kids.items[1].doc.clone());
                kids.finish(doc)
            }
            // type_name / type_param
            _ => self.raw(cst),
        }
    }

    /// 括弧で包まれた型は `type_expr` として子に現れる。
    fn type_atom(&self, kid: &Kid) -> Doc {
        if kid.cst.rule == Rule::type_expr {
            s("(").append(kid.doc.clone()).append(s(")"))
        } else {
            kid.doc.clone()
        }
    }

    fn arrow(&self, kid: &Kid, has_next: bool) -> Doc {
        match kid.cst.rule {
            Rule::type_optional => kid.doc.clone().append(s(" ?->")),
            _ if has_next => kid.doc.clone().append(s(" ->")),
            _ => kid.doc.clone(),
        }
    }

    /// `[int; string?] inline-cmd`
    fn type_command(&self, cst: &Cst, keyword: &str) -> Doc {
        self.seq(cst, "[", "]", None, &|c| self.node(c))
            .append(s(" "))
            .append(s(keyword))
    }
}
