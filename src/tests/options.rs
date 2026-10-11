//! 整形のオプションと、整形できない入力の扱い
use super::format;
use crate::FormatConfig;

fn config(indent_width: usize, line_width: usize) -> FormatConfig {
    FormatConfig {
        indent_width,
        line_width,
        ..Default::default()
    }
}

#[test]
fn indent_width_is_configurable() {
    let input = r#"let x = [aaaaaaaa; bbbbbbbb]
in
document(||)'<+p{a}>
"#;
    assert_eq!(
        format(input, &config(2, 16)),
        r#"let x = [
  aaaaaaaa;
  bbbbbbbb;
]
in

document(||)'<
  +p{ a }
>
"#
    );
    assert_eq!(
        format(input, &config(8, 16)),
        r#"let x = [
        aaaaaaaa;
        bbbbbbbb;
]
in

document(||)'<
        +p{ a }
>
"#
    );
}

#[test]
fn indent_with_tabs() {
    let tabs = FormatConfig {
        use_tabs: true,
        ..config(4, 16)
    };
    let input = r#"let x = [[aaaaaaaa; bbbbbbbb]; [c]]
in
document(||)'<+p{a+++b}+section{s}<+p{c}>>
"#;
    let output = format(input, &tabs);
    assert_eq!(
        output,
        "let x = [\n\t[\n\t\taaaaaaaa;\n\t\tbbbbbbbb;\n\t];\n\t[c];\n]\nin\n\ndocument(||)'<\n\t+p{ a+++b }\n\t+section{ s }<\n\t\t+p{ c }\n\t>\n>\n"
    );
    assert_eq!(format(&output, &tabs), output, "not idempotent");
}

#[test]
fn tabs_keep_verbatim_lines() {
    let tabs = FormatConfig {
        use_tabs: true,
        ..Default::default()
    };
    // off から on までは元の空白のまま。off / on のコメント自体はタブで字下げする
    let input = r#"document(||)'<
    % satysfi-fmt off
    +p{
        a
    }
    % satysfi-fmt on
    +p{b}
>
"#;
    assert_eq!(
        format(input, &tabs),
        "document(||)'<\n\t% satysfi-fmt off\n    +p{\n        a\n    }\n\t% satysfi-fmt on\n\t+p{ b }\n>\n"
    );
}

#[test]
fn inline_command_arguments_are_not_separated_by_spaces() {
    let config = FormatConfig::default();
    let input = r#"document(||)'<+section{a} <+p{\emph{b} \cmd(1) [2] {c} {d}}>>
"#;
    let output = format(input, &config);
    assert_eq!(
        output,
        r#"document(||)'<
    +section{ a }<
        +p{ \emph{ b } \cmd(1)[2]{ c }{ d } }
    >
>
"#
    );
    assert_eq!(format(&output, &config), output, "not idempotent");
}

#[test]
fn block_command_space_puts_a_space_before_text_arguments() {
    let spaced = FormatConfig {
        block_command_space: true,
        ..Default::default()
    };
    let input = r#"document(||)'<+section{a}<+cmd(1)[2]{b}+p{\emph{c}}>>
"#;
    let output = format(input, &spaced);
    assert_eq!(
        output,
        r#"document(||)'<
    +section { a } <
        +cmd(1)[2] { b }
        +p { \emph{ c } }
    >
>
"#
    );
    assert_eq!(format(&output, &spaced), output, "not idempotent");
}

#[test]
fn long_inline_command_breaks_after_the_brace() {
    let config = FormatConfig::default();
    let input = r#"let-inline \cmd arg = { \code-printer ?:(make-config CodeSyntax.rust CodeTheme.basic-dark |> set-basic-font-size 9pt) (arg); }
in
document(||)'<>
"#;
    let output = format(input, &config);
    assert_eq!(
        output,
        r#"let-inline \cmd arg = {
    \code-printer?:(make-config CodeSyntax.rust CodeTheme.basic-dark
        |> set-basic-font-size 9pt)
        (arg);
}
in

document(||)'<>
"#
    );
    assert_eq!(format(&output, &config), output, "not idempotent");
    // 1 行に収まるときはそのまま
    assert_eq!(
        format("document(||)'<+p{\\emph{a}}>", &config),
        "document(||)'<\n    +p{ \\emph{ a } }\n>\n"
    );
}

#[test]
fn argument_follows_a_multiline_argument() {
    let config = FormatConfig::default();
    let input = r#"let-inline \cmd arg = {
\code-printer ?:(make-config CodeSyntax.rust CodeTheme.basic-dark
  |> set-basic-font-size 9pt
  |> set-number-fun CodeDesign.number-fun-null) (arg);
}
in
document(||)'<>
"#;
    let output = format(input, &config);
    assert_eq!(
        output,
        r#"let-inline \cmd arg = {
    \code-printer?:(make-config CodeSyntax.rust CodeTheme.basic-dark
        |> set-basic-font-size 9pt
        |> set-number-fun CodeDesign.number-fun-null)(arg);
}
in

document(||)'<>
"#
    );
    assert_eq!(format(&output, &config), output, "not idempotent");
}

#[test]
fn space_is_kept_before_comment() {
    let compact = FormatConfig::default();
    let input = r#"document(||)'<+p % c
{a}>
"#;
    let output = format(input, &compact);
    assert!(
        output.contains(
            r#"+p % c
"#
        ),
        "{output}"
    );
    assert_eq!(format(&output, &compact), output, "not idempotent");
}

#[test]
fn final_newline_is_inserted_by_default() {
    assert_eq!(
        format("let x = 1", &FormatConfig::default()),
        r#"let x = 1
"#
    );
}

#[test]
fn final_newline_can_be_disabled() {
    let config = FormatConfig {
        final_newline: false,
        ..Default::default()
    };
    assert_eq!(format("let x = 1", &config), "let x = 1");
}

#[test]
fn default_line_width_is_80() {
    let config = FormatConfig::default();
    // 80 桁に収まるリストは 1 行のまま
    let fits = format!("let x = [{}]\n", ["aaaaaaaa"; 7].join("; "));
    assert!(fits.trim_end().chars().count() <= 80);
    assert_eq!(format(&fits, &config), fits);
    // 80 桁を超えると縦に展開される
    let long = format!("let x = [{}]\n", ["aaaaaaaa"; 9].join("; "));
    assert!(format(&long, &config).lines().count() > 2);
}

#[test]
fn input_that_cannot_be_parsed_is_an_error() {
    let config = FormatConfig::default();
    assert!(
        crate::format(
            r#"let x = = 1
"#,
            &config
        )
        .is_err()
    );
    assert!(crate::format("", &config).is_err());
}
