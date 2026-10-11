//! パス式 `<[ ... ]>` の整形 (行幅に収まるときと、収まらないとき)。
//! `test_width` は整形結果をもう一度整形しても変わらないこと (冪等) も確認する。
use super::{test_tmpl, test_width};

#[test]
fn fits_in_one_line() {
    let text = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>
"#;
    let expect = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>
"#;
    test_tmpl(text, expect);
}

#[test]
fn two_points_fit_in_one_line() {
    let text = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) ]>
"#;
    let expect = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) ]>
"#;
    test_tmpl(text, expect);
}

#[test]
fn single_point_fits_in_one_line() {
    let text = r#"let p = <[ (0pt, 0pt) ]>
"#;
    let expect = r#"let p = <[ (0pt, 0pt) ]>
"#;
    test_tmpl(text, expect);
}

#[test]
fn variables_and_expressions_as_points() {
    let text = r#"let p = <[ (x +' 1pt, y) -- (x, y +' 1pt) ]>
"#;
    let expect = r#"let p = <[ (x +' 1pt, y) -- (x, y +' 1pt) ]>
"#;
    test_tmpl(text, expect);
}

#[test]
fn line_breaks_in_the_source_are_ignored() {
    let text = r#"let p = <[ (0pt, 0pt)
 -- (10pt, 10pt)
 -- cycle ]>
"#;
    let expect = r#"let p = <[ (0pt, 0pt) -- (10pt, 10pt) -- cycle ]>
"#;
    test_tmpl(text, expect);
}

#[test]
fn straight_path_breaks_one_segment_per_line() {
    let text = r#"let p = <[ (0pt, 0pt) -- (10pt, 10pt) -- (20pt, 0pt) -- cycle ]>
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        -- (10pt, 10pt)
        -- (20pt, 0pt)
        -- cycle
    ]>
"#;
    test_width(text, 30, expect);
}

#[test]
fn let_moves_the_path_to_the_next_line_after_equal() {
    let text = r#"let p = <[ (0pt, 0pt) -- (10pt, 10pt) -- (20pt, 0pt) -- cycle ]>
let q = 1
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        -- (10pt, 10pt)
        -- (20pt, 0pt)
        -- cycle
    ]>
let q = 1
"#;
    test_width(text, 40, expect);
}

#[test]
fn curve_segment_stays_on_one_line() {
    let text = r#"let p = <[ p0 .. controls c1 and c2 .. p1 -- cycle ]>
"#;
    let expect = r#"let p =
    <[
        p0
        .. controls c1 and c2 .. p1
        -- cycle
    ]>
"#;
    test_width(text, 30, expect);
}

#[test]
fn curve_to_cycle() {
    let text = r#"let p = <[ p0 .. controls c1 and c2 .. cycle ]>
"#;
    let expect = r#"let p =
    <[
        p0
        .. controls c1 and c2 .. cycle
    ]>
"#;
    test_width(text, 20, expect);
}

#[test]
fn segment_longer_than_the_width_breaks_inside_its_points() {
    let text = r#"let p = <[ (0pt, 0pt) .. controls (10pt, 0pt) and (20pt, 10pt) .. (30pt, 30pt) -- (40pt, 30pt) ]>
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        .. controls (
            10pt,
            0pt
        ) and (
            20pt,
            10pt
        ) .. (
            30pt,
            30pt
        )
        -- (
            40pt,
            30pt
        )
    ]>
"#;
    test_width(text, 20, expect);
}

#[test]
fn short_segments_are_not_split_when_only_the_whole_path_is_too_long() {
    let text = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>
"#;
    let expect = r#"let p =
    <[
        (0pt, 0pt)
        -- (
            1pt,
            1pt
        )
        -- cycle
    ]>
"#;
    test_width(text, 20, expect);
}

#[test]
fn path_as_an_argument_breaks_all_the_arguments() {
    let text = r#"let g = Gr.stroke 1pt Color.black <[ (0pt, 0pt) -- (10pt, 10pt) -- (20pt, 0pt) ]>
"#;
    let expect = r#"let g =
    Gr.stroke
        1pt
        Color.black
        <[
            (0pt, 0pt)
            -- (10pt, 10pt)
            -- (20pt, 0pt)
        ]>
"#;
    test_width(text, 40, expect);
}

#[test]
fn path_in_a_module_is_indented_with_the_statement() {
    let text = r#"module M = struct
  let p = <[ (0pt, 0pt) -- (10pt, 10pt) -- (20pt, 0pt) -- cycle ]>
end
"#;
    let expect = r#"module M = struct
    let p =
        <[
            (0pt, 0pt)
            -- (10pt, 10pt)
            -- (20pt, 0pt)
            -- cycle
        ]>
end
"#;
    test_width(text, 40, expect);
}
