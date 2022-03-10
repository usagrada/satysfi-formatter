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
        if let Some(inner) = text.find('%') {
            if inner > 0 && &text[(inner - 1)..inner] == "\\" {
                // escaped percent
                continue;
            }
            let comment = format!("% {}", &text[inner + 1..].trim_start());

            // println!("comment: {comment}");
            comments.push_back(Comment {
                text: comment,
                span: Span { start, end },
            });
        }
    }
    comments
}

pub fn check_comment(cst: &Cst, comment: &Comment) -> bool {
    let inner_contain_comment = cst.inner.iter().fold(false, |current, inner_cst| {
        current || inner_cst.span.contains(&comment.span)
    });
    let contain_comment = cst.span.contains(&comment.span);
    // コメントを含みかつコメントが内部の要素に含まれていない場合出力する
    contain_comment && !inner_contain_comment
}

// csttext にコメントを追加して組み直す関数
pub fn csttext_insert_comments(csttext: CstText) -> CstText {
    let mut comments = get_comments(&csttext);
    if let Some(comment) = comments.pop_front() {
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
