//! 設定の読み込みと優先順位
use crate::config::{ConfigError, PartialConfig, find_config_file, resolve_config};
use crate::{FormatConfig, format};
use std::fs;
use std::path::PathBuf;

/// テストごとの一時ディレクトリ。設定ファイルの探索はパスを正規化するので、正規化して返す
fn temp_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("satysfi-fmt-test-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

#[test]
fn default_config() {
    assert_eq!(
        PartialConfig::default().build(),
        FormatConfig {
            line_width: 80,
            indent_width: 4,
            use_tabs: false,
            final_newline: true,
            block_command_space: false,
        }
    );
}

#[test]
fn reads_toml() {
    let config = PartialConfig::from_toml(
        r#"line_width = 100
indent_width = 2
"#,
    )
    .unwrap();
    assert_eq!(config.line_width, Some(100));
    assert_eq!(config.indent_width, Some(2));
    assert_eq!(config.final_newline, None);
}

#[test]
fn rejects_unknown_keys_and_invalid_values() {
    assert!(matches!(
        PartialConfig::from_toml(
            r#"max_width = 100
"#
        ),
        Err(ConfigError::Parse(_))
    ));
    assert!(matches!(
        PartialConfig::from_toml(
            r#"line_width = 0
"#
        ),
        Err(ConfigError::Invalid(_))
    ));
    assert!(matches!(
        PartialConfig::from_toml(
            r#"indent_width = 0
"#
        ),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn upper_layer_takes_precedence() {
    let file = PartialConfig {
        line_width: Some(100),
        indent_width: Some(2),
        use_tabs: Some(true),
        ..Default::default()
    };
    let cli = PartialConfig {
        line_width: Some(40),
        ..Default::default()
    };
    assert_eq!(
        cli.or(file).build(),
        FormatConfig {
            line_width: 40,
            indent_width: 2,
            use_tabs: true,
            final_newline: true,
            block_command_space: false,
        }
    );
}

#[test]
fn reads_editor_options() {
    assert_eq!(
        PartialConfig::from_editor_options(2, false, Some(false)),
        PartialConfig {
            indent_width: Some(2),
            use_tabs: Some(true),
            final_newline: Some(false),
            ..Default::default()
        }
    );
    // tab_size が 0 なら無視する
    assert_eq!(
        PartialConfig::from_editor_options(0, true, None),
        PartialConfig {
            use_tabs: Some(false),
            ..Default::default()
        }
    );
}

#[test]
fn reads_all_keys_from_toml() {
    let config = PartialConfig::from_toml(
        r#"line_width = 100
indent_width = 2
use_tabs = true
final_newline = false
block_command_space = true
"#,
    )
    .unwrap();
    assert_eq!(
        config.build(),
        FormatConfig {
            line_width: 100,
            indent_width: 2,
            use_tabs: true,
            final_newline: false,
            block_command_space: true,
        }
    );
}

#[test]
fn finds_config_file_in_ancestors() {
    let root = temp_dir("ancestors");
    let sub = root.join("a").join("b");
    fs::create_dir_all(&sub).unwrap();
    let file = sub.join("main.saty");
    fs::write(&file, "").unwrap();
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"line_width = 100
"#,
    )
    .unwrap();

    assert_eq!(
        find_config_file(&file),
        Some(root.join(".satysfi-fmt.toml"))
    );
    assert_eq!(resolve_config(&file).unwrap().line_width, Some(100));

    // 先頭に `.` のない satysfi-fmt.toml は読まない
    fs::write(
        sub.join("satysfi-fmt.toml"),
        r#"line_width = 40
"#,
    )
    .unwrap();
    assert_eq!(
        find_config_file(&file),
        Some(root.join(".satysfi-fmt.toml"))
    );
    // 近いほうの設定ファイルを使う
    fs::write(
        sub.join(".satysfi-fmt.toml"),
        r#"line_width = 60
"#,
    )
    .unwrap();
    assert_eq!(find_config_file(&file), Some(sub.join(".satysfi-fmt.toml")));
    assert_eq!(resolve_config(&file).unwrap().line_width, Some(60));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn finds_config_file_through_parent_references() {
    // `b/..` の親は `a` であって `b` ではない
    let root = temp_dir("dotdot");
    let a = root.join("a");
    let b = a.join("b");
    fs::create_dir_all(&b).unwrap();
    fs::write(a.join("main.saty"), "").unwrap();
    fs::write(
        b.join(".satysfi-fmt.toml"),
        r#"line_width = 60
"#,
    )
    .unwrap();
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"line_width = 100
"#,
    )
    .unwrap();

    let file = b.join("..").join("main.saty");
    assert_eq!(
        find_config_file(&file),
        Some(root.join(".satysfi-fmt.toml"))
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn invalid_config_file_reports_its_path() {
    let root = temp_dir("invalid");
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"line_width = "wide"
"#,
    )
    .unwrap();
    let err = resolve_config(&root.join("main.saty")).unwrap_err();
    assert!(err.to_string().contains(".satysfi-fmt.toml"), "{err}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn format_uses_config() {
    let input = r#"let x = [aaaaaaaa; bbbbbbbb]
"#;
    let narrow = FormatConfig {
        line_width: 16,
        indent_width: 2,
        ..Default::default()
    };
    assert_eq!(
        format(input, &narrow).unwrap(),
        r#"let x = [
  aaaaaaaa;
  bbbbbbbb;
]
"#
    );
    assert_eq!(format(input, &FormatConfig::default()).unwrap(), input);
}

#[test]
fn format_reports_parse_error() {
    let err = format(
        r#"let x = = 1
"#,
        &FormatConfig::default(),
    )
    .unwrap_err();
    assert!(
        matches!(err, crate::FormatError::Parse { line: 0, .. }),
        "{err:?}"
    );
}

/// language server での使い方: 設定ファイルを `FormattingOptions` より優先する
#[test]
fn config_file_takes_precedence_over_editor_options() {
    let root = temp_dir("editor");
    let file = root.join("src").join("main.saty");
    let input = r#"let x = [aaaaaaaa; bbbbbbbb]
"#;
    // エディタは「タブ幅 8、タブでインデント」
    let request = PartialConfig::from_editor_options(8, false, None);
    let config = || resolve_config(&file).unwrap().or(request.clone()).build();

    // 設定ファイルがなければエディタの指定
    assert_eq!(
        config(),
        FormatConfig {
            indent_width: 8,
            use_tabs: true,
            ..Default::default()
        }
    );

    // 設定ファイルの項目が優先され、書いていない項目はエディタの指定
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"line_width = 16
use_tabs = false
"#,
    )
    .unwrap();
    assert_eq!(
        format(input, &config()).unwrap(),
        r#"let x = [
        aaaaaaaa;
        bbbbbbbb;
]
"#
    );

    fs::remove_dir_all(&root).unwrap();
}
