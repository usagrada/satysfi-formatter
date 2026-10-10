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
    let input = "let x = [aaaaaaaa; bbbbbbbb]\nin\ndocument(||)'<+p{a}>\n";
    assert_eq!(
        format(input, &config(2, 16)),
        "let x = [\n  aaaaaaaa;\n  bbbbbbbb;\n]\nin\n\ndocument (||) '<\n  +p { a }\n>\n"
    );
    assert_eq!(
        format(input, &config(8, 16)),
        "let x = [\n        aaaaaaaa;\n        bbbbbbbb;\n]\nin\n\ndocument (||) '<\n        +p { a }\n>\n"
    );
}

#[test]
fn indent_with_tabs() {
    let tabs = FormatConfig {
        use_tabs: true,
        ..config(4, 16)
    };
    let input =
        "let x = [[aaaaaaaa; bbbbbbbb]; [c]]\nin\ndocument(||)'<+p{a+++b}+section{s}<+p{c}>>\n";
    let output = format(input, &tabs);
    assert_eq!(
        output,
        "let x = [\n\t[\n\t\taaaaaaaa;\n\t\tbbbbbbbb;\n\t];\n\t[c];\n]\nin\n\ndocument (||) '<\n\t+p { a+++b }\n\t+section { s } <\n\t\t+p { c }\n\t>\n>\n"
    );
    assert_eq!(format(&output, &tabs), output, "not idempotent");
}

#[test]
fn tabs_keep_verbatim_lines() {
    let tabs = FormatConfig {
        use_tabs: true,
        ..Default::default()
    };
    // 整形しないノードの 2 行目以降は元の空白のまま
    let input = "document(||)'<\n    % satysfi-fmt: skip\n    +p{\n        a\n    }\n>\n";
    assert_eq!(
        format(input, &tabs),
        "document (||) '<\n\t% satysfi-fmt: skip\n\t+p{\n        a\n    }\n>\n"
    );
}

#[test]
fn command_space_can_be_disabled() {
    let compact = FormatConfig {
        command_space: false,
        ..Default::default()
    };
    let input = "document(||)'<+section{a}<+p{\\emph{b}\\cmd(1)[2]{c}{d}}>>\n";
    let output = format(input, &compact);
    assert_eq!(
        output,
        "document (||) '<\n    +section{ a }<\n        +p{ \\emph{ b }\\cmd (1) [2]{ c }{ d } }\n    >\n>\n"
    );
    assert_eq!(format(&output, &compact), output, "not idempotent");
    // 既定では空白を入れる
    assert_eq!(
        format(input, &FormatConfig::default()),
        "document (||) '<\n    +section { a } <\n        +p { \\emph { b }\\cmd (1) [2] { c } { d } }\n    >\n>\n"
    );
}

#[test]
fn command_space_is_kept_before_comment() {
    let compact = FormatConfig {
        command_space: false,
        ..Default::default()
    };
    let input = "document(||)'<+p % c\n{a}>\n";
    let output = format(input, &compact);
    assert!(output.contains("+p % c\n"), "{output}");
    assert_eq!(format(&output, &compact), output, "not idempotent");
}

#[test]
fn final_newline_is_inserted_by_default() {
    assert_eq!(format("let x = 1", &FormatConfig::default()), "let x = 1\n");
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
    assert!(crate::format("let x = = 1\n", &config).is_err());
    assert!(crate::format("", &config).is_err());
}
