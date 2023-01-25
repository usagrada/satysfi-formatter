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
            let is_escaped = {
                let text_removed_backslash = text[..inner].trim_matches('\\');
                (text[..inner].len() - text_removed_backslash.len()) % 2 == 1
            };
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

fn check_comment(cst: &Cst, comment: &Comment) -> bool {
    // headers は例外
    let inner_contain_comment = cst
        .inner
        .iter()
        .any(|inner_cst| inner_cst.rule != Rule::headers && inner_cst.span.contains(&comment.span));
    let contain_comment = cst.span.contains(&comment.span);
    // コメントを含みかつコメントが内部の要素に含まれていない場合出力する
    contain_comment && !inner_contain_comment && cst.rule != Rule::headers
}

/// csttext にコメントを追加して組み直す関数
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

/// コメントの挿入をDFSで行う
fn cst_insert_comment(cst: &mut Cst, comments: &mut VecDeque<Comment>) {
    let mut insert_comment = vec![];
    for comment in comments.iter() {
        let flag = check_comment(cst, comment);

        if flag {
            #[cfg(debug_assertions)]
            println!("cst: {:?}, insert-comment: {:?}", cst.rule, comment.text);
            insert_comment.push(Cst {
                rule: Rule::comments,
                inner: vec![],
                span: comment.span,
            });
        }
    }

    for inner_cst in cst.inner.iter_mut() {
        // 再帰的に探索
        cst_insert_comment(inner_cst, comments);
    }

    if !insert_comment.is_empty() {
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
}

/// コメントを文字列化する関数
pub fn to_comment_string(text: String) -> String {
    let index = text.find('%').unwrap();
    let comment = text[index + 1..].trim_end();
    if comment.is_empty() || comment.starts_with(char::is_whitespace) || comment.starts_with('%') {
        // 空白or複数の%で始まっていたらそのまま表示
        format!("%{}", comment)
    } else {
        // 空白がない場合は空白を1つ挿入
        format!("% {}", comment)
    }
}
