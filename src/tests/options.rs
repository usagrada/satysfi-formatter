//! 整形のオプションと、整形できない入力の扱い
use crate::format;
use lspower::lsp::{FormattingOptions, FormattingProperty};

fn option(tab_size: u32) -> FormattingOptions {
    FormattingOptions {
        tab_size,
        insert_spaces: true,
        ..Default::default()
    }
}

#[test]
fn indent_width_follows_tab_size() {
    let input = "let x = [aaaaaaaa; bbbbbbbb]\nin\ndocument(||)'<+p{a}>\n";
    let mut two = option(2);
    two.properties
        .insert("lineWidth".to_string(), FormattingProperty::Number(16));
    assert_eq!(
        format(input, two),
        "let x = [\n  aaaaaaaa;\n  bbbbbbbb;\n]\nin\n\ndocument (||) '<\n  +p { a }\n>\n"
    );
    let mut eight = option(8);
    eight
        .properties
        .insert("lineWidth".to_string(), FormattingProperty::Number(16));
    assert_eq!(
        format(input, eight),
        "let x = [\n        aaaaaaaa;\n        bbbbbbbb;\n]\nin\n\ndocument (||) '<\n        +p { a }\n>\n"
    );
}

#[test]
fn final_newline_is_inserted_by_default() {
    assert_eq!(format("let x = 1", option(4)), "let x = 1\n");
}

#[test]
fn final_newline_can_be_disabled() {
    let mut option = option(4);
    option.insert_final_newline = Some(false);
    assert_eq!(format("let x = 1", option), "let x = 1");
}

#[test]
fn default_line_width_is_80() {
    // 80 桁に収まるリストは 1 行のまま
    let fits = format!("let x = [{}]\n", ["aaaaaaaa"; 7].join("; "));
    assert!(fits.trim_end().chars().count() <= 80);
    assert_eq!(format(&fits, option(4)), fits);
    // 80 桁を超えると縦に展開される
    let long = format!("let x = [{}]\n", ["aaaaaaaa"; 9].join("; "));
    assert!(format(&long, option(4)).lines().count() > 2);
}

#[test]
fn invalid_line_width_falls_back_to_the_default() {
    for width in [0, -5] {
        let mut option = option(4);
        option
            .properties
            .insert("lineWidth".to_string(), FormattingProperty::Number(width));
        assert_eq!(format("let x = [1; 2]\n", option), "let x = [1; 2]\n");
    }
}

#[test]
fn input_that_cannot_be_parsed_is_returned_as_is() {
    let input = "let x = = 1\n";
    assert_eq!(format(input, option(4)), input);
    assert_eq!(format("", option(4)), "");
}
