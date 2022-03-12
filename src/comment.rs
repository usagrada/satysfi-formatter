use satysfi_parser::{Cst, CstText, Rule, Span};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Comment {
    pub text: String,
    pub span: Span,
}

pub fn get_comments(csttext: &CstText) -> VecDeque<Comment> {
    let mut comments = VecDeque::new();
    // 全ての行を確認する
    for (index, line) in csttext.lines.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let start = csttext.lines[index - 1];
        let end = *line;
        let text = csttext.get_text_from_span(Span { start, end });
        if text.trim_start().starts_with("@require") || text.trim_start().starts_with("@import") {
            // @require, @import の行ではコメントではない
            continue;
        };
        if let Some(inner) = text.find('%') {
            // check whether percent is escaped
            let mut index = inner;
            let mut is_escaped = false;
            while index > 1 {
                if text[..index].ends_with("\\") {
                    // 再帰的に確認
                    if text[..index].ends_with("\\\\") {
                        // escaped percent
                        index -= 2;
                        continue;
                    }
                    is_escaped = true;
                    break;
                }
                break;
            }
            if is_escaped {
                continue;
            }
            let comment = format!("% {}", &text[inner + 1..].trim_start());

            comments.push_back(Comment {
                text: comment,
                // 行内部の開始位置を足す
                span: Span {
                    start: start + inner,
                    end,
                },
            });
        }
    }
    comments
}

pub fn check_comment(cst: &Cst, comment: &Comment) -> bool {
    let inner_contain_comment = cst.inner.iter().fold(false, |current, inner_cst| {
        // headers のみ例外
        current || (inner_cst.rule != Rule::headers && inner_cst.span.contains(&comment.span))
    });
    let contain_comment = cst.span.contains(&comment.span);
    // コメントを含みかつコメントが内部の要素に含まれていない場合出力する
    contain_comment && !inner_contain_comment
}

// csttext にコメントを追加して組み直す関数
pub fn csttext_insert_comments(csttext: CstText) -> CstText {
    let mut comments = get_comments(&csttext);
    if let Some(comment) = comments.pop_front() {
        comments.push_front(comment.clone());
        check_comment(&csttext.cst, &comment);
        let mut csttext = csttext;
        cst_insert_comment(&mut csttext.cst, &mut comments);
        csttext
    } else {
        csttext
    }
}

fn cst_insert_comment(cst: &mut Cst, comments: &mut VecDeque<Comment>) {
    let mut insert_comment = vec![];
    for comment in comments.iter() {
        let flag = check_comment(cst, &comment);

        if flag {
            #[cfg(debug_assertions)]
            println!("cst: {:?}, insert-comment: {}", cst.rule, comment.text);
            insert_comment.push(Cst {
                rule: Rule::comments,
                inner: vec![],
                span: comment.span,
            });
        }
    }
    if insert_comment.len() > 0 {
        insert_comment.iter().for_each(|comment| {
            cst.inner.push(Cst {
                rule: Rule::comments,
                inner: vec![],
                span: comment.span,
            });
            // insert した回数分減らせる
            // comment は前から順番に並んでいる
            comments.pop_front();
        });
        cst.inner.sort_by(|a, b| a.span.start.cmp(&b.span.start))
    }

    for inner_cst in cst.inner.iter_mut() {
        // 再帰的に探索
        cst_insert_comment(inner_cst, comments);
    }
}

pub fn to_comment_string(text: String) -> String {
    let index = text.find("%").unwrap();
    format!("% {}", &text[(index + 1)..].trim())
}
