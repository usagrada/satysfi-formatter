//! パターンの Doc 化。

use super::{s, Doc, Printer};
use satysfi_parser::{Cst, Rule};

impl<'a> Printer<'a> {
    pub(super) fn pattern_node(&self, cst: &Cst) -> Doc {
        match cst.rule {
            Rule::pat_as => {
                let kids = self.kids(cst);
                let mut doc = kids.items[0].doc.clone();
                if let Some(var) = kids.items.get(1) {
                    doc = doc.append(s(" as ")).append(var.doc.clone());
                }
                kids.finish(doc)
            }
            Rule::pat_cons => {
                let kids = self.kids(cst);
                let doc = match kids.items.as_slice() {
                    [head, tail] => head.doc.clone().append(s(" :: ")).append(tail.doc.clone()),
                    [only] => only.doc.clone(),
                    _ => self.raw(cst),
                };
                kids.finish(doc)
            }
            Rule::pattern => {
                let kids = self.kids(cst);
                let doc = match kids.items.as_slice() {
                    // `_` `x` `~x` や定数
                    [] => self.raw(cst),
                    [inner] => match inner.cst.rule {
                        Rule::pat_list | Rule::pat_tuple => inner.doc.clone(),
                        // `(op)`
                        Rule::bin_operator | Rule::unary_operator => {
                            s(&format!("({})", self.text_of(inner.cst).trim()))
                        }
                        // `(pattern)`
                        Rule::pat_as => s("(").append(inner.doc.clone()).append(s(")")),
                        // `~x` のように接頭辞があっても元の文字列を使う
                        _ => self.raw(cst),
                    },
                    _ => self.raw(cst),
                };
                kids.finish(doc)
            }
            Rule::pat_variant => {
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
            Rule::pat_list => self.seq(cst, "[", "]", None, &|c| self.node(c)),
            Rule::pat_tuple => {
                let kids = self.kids(cst);
                self.tuple_doc(kids)
            }
            _ => self.raw(cst),
        }
    }
}
