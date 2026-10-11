//! コメントの位置と保存の回帰テスト。
//! 行コメントは行末まで続くので、後ろに続く内容を飲み込まないことと、コメントが消えないことを確認する。
use super::test_tmpl;

#[test]
fn trailing_comment_of_statements() {
    let text = r#"let x = 1 %c
let y = 2%d
"#;
    let expect = r#"let x = 1 % c
let y = 2 % d
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_at_end_of_file_without_newline() {
    let text = r#"let x = [1; 2; 3] % c"#;
    let expect = r#"let x = [1; 2; 3] % c
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_before_and_after_equal() {
    let text = r#"let y = % after equal
  2
"#;
    let expect = r#"let y =
    % after equal
    2
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_function_body() {
    let text = r#"let f x y = % comment in def
  x + y % after
let z = 1
"#;
    let expect = r#"let f x y =
    % comment in def
    x + y % after
let z = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_after_the_last_expression_of_a_statement() {
    let text = r#"let f x =
  % head
  x % tail
  % end
let y = 1
"#;
    // 本体より深く字下げされたコメントは、本体の続きとして、本体と同じ字下げで出す
    let expect = r#"let f x =
    % head
    x % tail
    % end
let y = 1
"#;
    test_tmpl(text, expect);
}

// ---- 文の後ろの単独行のコメントが、その文の本体か、次の文の見出しか ----
// 直前の文より深く字下げされ、直前の文や他のコメントと空行で区切られていないものは、直前の文の本体の続き。
// 文と同じ (またはより浅い) 字下げのものや、空行で区切られているものは、次の文の見出し。

#[test]
fn comment_at_the_same_indentation_as_the_statements_is_a_heading_of_the_next_statement() {
    let text = r#"let f x =
  x
% heading
let y = 1
"#;
    let expect = r#"let f x = x
% heading
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn indented_comment_after_a_blank_line_is_a_heading_of_the_next_statement() {
    let text = r#"let f x =
  x

  % separated by a blank line
let y = 1
"#;
    let expect = r#"let f x = x

% separated by a blank line
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn several_indented_comments_are_all_part_of_the_body() {
    let text = r#"let f x =
  x
  % body 1
  % body 2
let y = 1
"#;
    let expect = r#"let f x = x
    % body 1
    % body 2
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_are_split_where_the_indentation_stops_being_deeper() {
    let text = r#"let f x =
  x
  % body
% heading
let y = 1
"#;
    let expect = r#"let f x = x
    % body
% heading
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn a_blank_line_after_the_body_comment_is_kept_once() {
    let text = r#"let f x =
  x
  % body

% heading
let y = 1
"#;
    let expect = r#"let f x = x
    % body

% heading
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn a_blank_line_between_the_body_comment_and_the_next_statement_is_kept_once() {
    let text = r#"let f x =
  x
  % end

let y = 1
"#;
    let expect = r#"let f x = x
    % end

let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn indented_comment_at_the_end_of_a_struct_belongs_to_the_last_statement() {
    let text = r#"module M = struct
  let f x =
    x
    % end
end
"#;
    let expect = r#"module M = struct
    let f x = x
        % end
end
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_of_the_last_statement_before_end_of_struct_at_the_same_indentation_stays_in_the_struct()
{
    let text = r#"module M = struct
  let f x = x
  % last
end
"#;
    let expect = r#"module M = struct
    let f x = x
    % last
end
"#;
    test_tmpl(text, expect);
}

#[test]
fn a_comment_indented_more_than_the_previous_statement_is_its_body_even_if_it_looks_like_a_heading()
{
    // 字下げだけでは見出しと区別できない。直前の文を優先する
    let text = r#"let a = 1
   % indented
let b = 2
"#;
    let expect = r#"let a = 1
    % indented
let b = 2
"#;
    test_tmpl(text, expect);
}

#[test]
fn trailing_comment_stays_with_its_list_element() {
    let text = r#"let x = [
  1; % one
  % own line
  2;
  % dangling
]
"#;
    let expect = r#"let x = [
    1; % one
    % own line
    2;
    % dangling
]
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_empty_list() {
    let text = r#"let x = [
  % only
]
"#;
    let expect = r#"let x = [
    % only
]
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_empty_record() {
    let text = r#"let x = (|
  % only
|)
"#;
    let expect = r#"let x = (|
    % only
|)
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_before_first_list_element() {
    let text = r#"let x = [% c0
 a]
"#;
    let expect = r#"let x = [
    % c0
    a;
]
"#;
    test_tmpl(text, expect);
}

#[test]
fn trailing_comment_stays_with_its_record_field() {
    let text = r#"let x = (| a = 1; % c
  b = 2 |)
"#;
    let expect = r#"let x = (|
    a = 1; % c
    b = 2;
|)
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_in_tuple() {
    let text = r#"let t = (1, % first
  2)
"#;
    let expect = r#"let t = (
    1,
    % first
    2
)
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_after_the_last_tuple_element_is_kept_inside() {
    let text = r#"let t = (1, 2
  % last
)
let u = 1
"#;
    let expect = r#"let t = (
    1,
    2
    % last
)
let u = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_before_operator_goes_before_the_operator_line() {
    let text = r#"let g = (1 % a
  + 2 % b
  + 3)
"#;
    let expect = r#"let g =
    (1
        % a
        + 2
        % b
        + 3)
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_between_arguments() {
    let text = r#"let g = f a % c1
  b % c2
  c
"#;
    let expect = r#"let g =
    f
        a
        % c1
        b
        % c2
        c
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_if() {
    let text = r#"let g = if a % c1
 then b % c2
 else c % c3
"#;
    let expect = r#"let g =
    if a then
        % c1
        b
    else
        % c2
        c % c3
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_match() {
    let text = r#"let g = match x with % c0
 | A -> 1 % c1
 % own-line before arm
 | B -> 2 % c2
"#;
    let expect = r#"let g =
    match x with % c0
    | A -> 1 % c1
    % own-line before arm
    | B -> 2 % c2
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_type_variants() {
    let text = r#"type t = % c0
 | A % c1
 | B % c2
let x = 1
"#;
    let expect = r#"type t =
    % c0
    | A % c1
    | B % c2
let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn trailing_comment_of_the_type_name_is_kept() {
    let text = r#"type t = % c0
 A
let x = 1
"#;
    let expect = r#"type t =
    % c0
    | A
let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_let_rec() {
    let text = r#"let-rec f x = x % c1
| y = y % c2
let z = 1
"#;
    let expect = r#"let-rec f x = x % c1
    | y = y % c2
let z = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_module() {
    let text = r#"module M = struct % c0
  let x = 1 % c1
  % c2
end % c3
"#;
    let expect = r#"module M = struct
    % c0
    let x = 1 % c1
    % c2
end % c3
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_around_in_stay_on_their_side() {
    let text = r#"let x = 1 % c
% own line before in
in % after in
% own line after in
document(||)'<>
"#;
    let expect = r#"let x = 1 % c
% own line before in
in % after in

% own line after in
document (||) '<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_around_in_of_let_expression_stay_on_their_side() {
    let text = r#"let f =
  let x = 1 % c
  % own line before in
  in % after in
  % own line after in
  x
"#;
    let expect = r#"let f =
    let x = 1 % c
    % own line before in
    in % after in
    % own line after in
    x
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_path_before_cycle() {
    let text = r#"let p = <[ (0pt, 0pt) -- % c
 cycle ]>
let q = 1
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        % c
        -- cycle
    ]>
let q = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_after_path_stays_inside_brackets() {
    let text = r#"let p = <[ (0pt, 0pt) -- cycle%c
 ]>
in
document(||)'<>
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        -- cycle
        % c
    ]>
in

document (||) '<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_around_headers() {
    let text = r#"% license
@require: a % trailing
% between
@import: b

% before document
document(||)'<>
"#;
    let expect = r#"% license
@require: a % trailing
% between
@import: b

% before document
document (||) '<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_after_document() {
    let text = r#"document(||)'<>
% after
"#;
    let expect = r#"document (||) '<>

% after
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_between_blocks() {
    let text = r#"document(||)'<
  +p{a} % c1

  % c2
  +p{b}
  % end
>
"#;
    let expect = r#"document (||) '<
    +p { a } % c1

    % c2
    +p { b }
    % end
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_text_keeps_adjacency() {
    let text = r#"document(||)'<+p{a%c
b}>
"#;
    let expect = r#"document (||) '<
    +p {
        a% c
        b
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_after_text_gets_a_space() {
    let text = r#"document(||)'<+p{a%c
}>
"#;
    let expect = r#"document (||) '<
    +p {
        a % c
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_before_text() {
    let text = r#"document(||)'<+p{% c0
a}>
"#;
    let expect = r#"document (||) '<
    +p {
        % c0
        a
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_nested_command() {
    let text = r#"document(||)'<+p{\emph{x % c
 y}}>
"#;
    let expect = r#"document (||) '<
    +p {
        \emph {
            x % c
            y
        }
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_math() {
    let text = r#"document(||)'<+p{${a % c
 b}}>
"#;
    let expect = r#"document (||) '<
    +p {
        ${
            a % c
            b
        }
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_at_the_start_of_math_list() {
    let text = r#"document(||)'<+p{${|%cmt
 a | b + c |}}>
"#;
    let expect = r#"document (||) '<
    +p {
        ${
            % cmt
            | a | b + c |
        }
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_before_math_in_braces() {
    let text = r#"document(||)'<+p{${%cmt
 a}}>
"#;
    let expect = r#"document (||) '<
    +p {
        ${
            % cmt
            a
        }
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_in_horizontal_list_cell_breaks_the_line() {
    let text = r#"document(||)'<+tab{| a%cmt
 | b |}>
"#;
    let expect = r#"document (||) '<
    +tab {
        | a % cmt
        | b
        |
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_between_horizontal_list_cells() {
    let text = r#"document(||)'<+tab{| a |%cmt
 b |}>
"#;
    let expect = r#"document (||) '<
    +tab {
        | a
        | % cmt
        b
        |
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_between_bullets() {
    let text = r#"document(||)'<+listing{
 * a
 %cmt
 * b
}>
"#;
    let expect = r#"document (||) '<
    +listing {
        * a
        % cmt
        * b
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn comment_between_star_and_text() {
    let text = r#"document(||)'<+listing{
 * %cmt
 a
}>
"#;
    let expect = r#"document (||) '<
    +listing {
        % cmt
        * a
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn own_line_comment_before_a_bracketed_value() {
    let text = r#"let g x =
  % note
  (a, b)
let f x = match x with
  | A -> % c
    [1; 2]
  | B -> 2
"#;
    let expect = r#"let g x =
    % note
    (a, b)
let f x =
    match x with
    | A ->
        % c
        [1; 2]
    | B -> 2
"#;
    test_tmpl(text, expect);
}

#[test]
fn blank_line_between_comment_and_statement_is_kept_in_place() {
    let text = r#"let x = 1
% about x

let y = 2
"#;
    test_tmpl(text, text);
}

#[test]
fn blank_line_between_comment_and_block_is_kept_in_place() {
    let text = r#"document (||) '<
    +p { a }
    % about p

    +p { b }
>
"#;
    test_tmpl(text, text);
}

#[test]
fn blank_lines_between_comments_are_kept_in_place() {
    let text = r#"let a = 1
% section header

% note about x
let x = 1

% section header

% note about y

let y = 2
"#;
    test_tmpl(text, text);
}

#[test]
fn blank_lines_between_comments_in_a_vertical_area_are_kept_in_place() {
    let text = r#"document (||) '<
    +p { a }
    % section header

    % note about b
    +p { b }
>
"#;
    test_tmpl(text, text);
}
