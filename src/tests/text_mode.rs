//! 水平・垂直モードとコマンドの整形
use super::{test_tmpl, test_width};

#[test]
fn empty_vertical() {
    let text = r##"document(||)'<>
"##;
    let expect = r##"document (||) '<>
"##;
    test_tmpl(text, expect);
}

#[test]
fn empty_horizontal() {
    let text = r##"document(||)'<+p{}>
"##;
    let expect = r##"document (||) '<
    +p {}
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn blank_only_horizontal() {
    let text = r##"document(||)'<+p{ }>
"##;
    let expect = r##"document (||) '<
    +p {}
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn blocks_are_split() {
    let text = r##"document(||)'<+p{a}+p{b}>
"##;
    let expect = r##"document (||) '<
    +p { a }
    +p { b }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn blank_lines_between_blocks_are_kept_up_to_one() {
    let text = r##"document(||)'<+p{a}


+p{b}>
"##;
    let expect = r##"document (||) '<
    +p { a }

    +p { b }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn block_command_without_arguments() {
    let text = r##"document(||)'<+newpage;>
"##;
    let expect = r##"document (||) '<
    +newpage;
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn block_embedding() {
    let text = r##"document(||)'<#blk;>
"##;
    let expect = r##"document (||) '<
    #blk;
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn inline_embedding_and_commands() {
    let text = r##"document(||)'<+p{#inl; \emph{x} #M.y;}>
"##;
    let expect = r##"document (||) '<
    +p { #inl; \emph { x } #M.y; }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn commands_without_arguments() {
    let text = r##"document(||)'<+p{\a;\b;}>
"##;
    let expect = r##"document (||) '<
    +p { \a;\b; }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn text_after_arguments_is_kept_adjacent() {
    let text = r##"document(||)'<+p{\a{x}{y}(1)(2)}>
"##;
    let expect = r##"document (||) '<
    +p { \a { x } { y }(1)(2) }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn optional_and_omitted_arguments() {
    let text = r##"document(||)'<+p{\a?:(1) ?* (2){x}}>
"##;
    let expect = r##"document (||) '<
    +p { \a ?:(1) ?* (2) { x } }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn block_command_with_arguments_and_vertical() {
    let text = r##"document(||)'<+p(1)?:(2)?*{x}<+p{y}>>
"##;
    let expect = r##"document (||) '<
    +p (1) ?:(2) ?* { x } <
        +p { y }
    >
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn command_with_module() {
    let text = r##"document(||)'<+p{\M.f{x}}>
"##;
    let expect = r##"document (||) '<
    +p { \M.f { x } }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn escaped_characters() {
    let text = r##"document(||)'<+p{a \% b \\ c \{ \} \@ \` \| \* \$ \# \;}>
"##;
    let expect = r##"document (||) '<
    +p { a \% b \\ c \{ \} \@ \` \| \* \$ \# \; }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn inline_code() {
    let text = r##"document(||)'<+p{`code` and ``co`de``}>
"##;
    let expect = r##"document (||) '<
    +p { `code` and ``co`de`` }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn line_breaks_in_text_are_kept() {
    let text = r##"document(||)'<+p{a
   b

   c}>
"##;
    let expect = r##"document (||) '<
    +p {
        a
        b
        c
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn spaces_are_collapsed_and_trimmed() {
    let text = r##"document(||)'<+p{   a   b   }>
"##;
    let expect = r##"document (||) '<
    +p { a b }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn spaces_around_commands_are_collapsed() {
    let text = r##"document(||)'<+p{a   \emph{b}   c}>
"##;
    let expect = r##"document (||) '<
    +p { a \emph { b } c }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn adjacent_commands_stay_adjacent() {
    let text = r##"document(||)'<+p{\emph{b}\emph{c}}>
"##;
    let expect = r##"document (||) '<
    +p { \emph { b }\emph { c } }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn inline_math_in_text() {
    let text = r##"document(||)'<+p{x ${y} z}>
"##;
    let expect = r##"document (||) '<
    +p { x ${y} z }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn bullet_list_with_nesting() {
    let text = r##"document(||)'<+p{
  * a
  * b
    ** c
  * d
}>
"##;
    let expect = r##"document (||) '<
    +p {
        * a
        * b
        ** c
        * d
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn bullet_list_in_one_line() {
    let text = r##"document(||)'<+listing{* a * b}>
"##;
    let expect = r##"document (||) '<
    +listing {
        * a
        * b
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn horizontal_list() {
    let text = r##"document(||)'<+tab{| a | b | c |}>
"##;
    let expect = r##"document (||) '<
    +tab { | a | b | c | }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn horizontal_list_with_empty_cell() {
    let text = r##"document(||)'<+tab{| a | b |
 | c | d |}>
"##;
    let expect = r##"document (||) '<
    +tab { | a | b | | c | d | }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn embedding_only() {
    let text = r##"document(||)'<+p{#a;}>
"##;
    let expect = r##"document (||) '<
    +p { #a; }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn multiline_argument_breaks_braces() {
    let text = r##"document(||)'<+p {
\emph{a
b}
}>
"##;
    let expect = r##"document (||) '<
    +p {
        \emph {
            a
            b
        }
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn one_line_text_is_never_split() {
    let text = r##"document(||)'<+p{aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee ffffffffff gggggggggg}>
"##;
    let expect = r##"document (||) '<
    +p { aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee ffffffffff gggggggggg }
>
"##;
    test_width(text, 30, expect);
}

#[test]
fn command_arguments_break_when_too_long() {
    let text = r##"document(||)'<+very-long-command-name(aaaaaaaaaaaaaaaa)(bbbbbbbbbbbbbbbb){text}>
"##;
    let expect = r##"document (||) '<
    +very-long-command-name
        (aaaaaaaaaaaaaaaa)
        (bbbbbbbbbbbbbbbb) { text }
>
"##;
    test_width(text, 30, expect);
}

#[test]
fn command_hugs_a_list_argument() {
    let text = r##"document(||)'<+fig(vconcat [aaaaaaaaaaaaaaaa; bbbbbbbbbbbbbbbb; cccccccccccccccc]);>
"##;
    let expect = r##"document (||) '<
    +fig (vconcat [
        aaaaaaaaaaaaaaaa;
        bbbbbbbbbbbbbbbb;
        cccccccccccccccc;
    ]);
>
"##;
    test_width(text, 40, expect);
}

#[test]
fn section_with_nested_blocks() {
    let text = r##"document(||)'<+section{Sec}<+p{a}+p{b}>>
"##;
    let expect = r##"document (||) '<
    +section { Sec } <
        +p { a }
        +p { b }
    >
>
"##;
    test_tmpl(text, expect);
}
