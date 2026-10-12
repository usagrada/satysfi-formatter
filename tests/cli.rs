//! CLI (`satysfi-fmt`) の結合テスト。子プロセスとして起動する。
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// テストごとの一時ディレクトリ
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "satysfi-fmt-cli-test-{}-{}",
        std::process::id(),
        name
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn relative_path_finds_config_file_above_the_current_directory() {
    let root = temp_dir("relative");
    let sub = root.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"indent_width = 2
line_width = 16
"#,
    )
    .unwrap();
    fs::write(
        sub.join("a.satyh"),
        r#"let x = [aaaaaaaa; bbbbbbbb]
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_satysfi-fmt"))
        .current_dir(&sub)
        .arg("a.satyh")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        r#"let x = [
  aaaaaaaa;
  bbbbbbbb;
]
"#
    );

    fs::remove_dir_all(&root).unwrap();
}

/// 標準入力に `input` を渡して起動する
fn run_with_stdin(args: &[&str], dir: &std::path::Path, input: &str) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_satysfi-fmt"))
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn stdin_is_formatted_with_the_config_of_assumed_filename() {
    let root = temp_dir("stdin");
    fs::write(
        root.join(".satysfi-fmt.toml"),
        r#"indent_width = 2
line_width = 16
"#,
    )
    .unwrap();

    let output = run_with_stdin(
        &["--assume-filename", "sub/a.satyh"],
        &root,
        r#"let x = [aaaaaaaa; bbbbbbbb]
"#,
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        r#"let x = [
  aaaaaaaa;
  bbbbbbbb;
]
"#
    );

    // `-` でも同じ
    let output = run_with_stdin(&["-", "--stdin-filepath", "a.satyh"], &root, "let x  = 1\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "let x = 1\n");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn stdin_parse_error_is_reported_with_position() {
    let root = temp_dir("stdin-error");
    let output = run_with_stdin(&["--assume-filename", "a.saty"], &root, "let x = = 1\n");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("a.saty:1:"), "{stderr}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn check_reports_unformatted_input_by_exit_code() {
    let root = temp_dir("check");
    let formatted = run_with_stdin(&["--check"], &root, "let x = 1\n");
    assert_eq!(formatted.status.code(), Some(0));
    assert!(formatted.stdout.is_empty());

    let unformatted = run_with_stdin(&["--check"], &root, "let x  =  1\n");
    assert_eq!(unformatted.status.code(), Some(1));
    assert!(unformatted.stdout.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn final_newline_option_is_honored() {
    let root = temp_dir("final-newline");
    fs::write(root.join(".satysfi-fmt.toml"), "final_newline = false\n").unwrap();
    let output = run_with_stdin(&[], &root, "let x = 1");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "let x = 1");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn stdin_without_assumed_filename_searches_config_from_the_current_directory() {
    let root = temp_dir("stdin-cwd");
    fs::write(
        root.join(".satysfi-fmt.toml"),
        "indent_width = 2\nline_width = 16\n",
    )
    .unwrap();
    let output = run_with_stdin(&[], &root, "let x = [aaaaaaaa; bbbbbbbb]\n");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "let x = [\n  aaaaaaaa;\n  bbbbbbbb;\n]\n"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn explicit_config_file_is_used_instead_of_the_searched_one() {
    let root = temp_dir("explicit-config");
    fs::write(
        root.join(".satysfi-fmt.toml"),
        "indent_width = 2\nline_width = 16\n",
    )
    .unwrap();
    fs::write(
        root.join("other.toml"),
        "indent_width = 3\nline_width = 16\n",
    )
    .unwrap();
    let output = run_with_stdin(
        &["--config", "other.toml"],
        &root,
        "let x = [aaaaaaaa; bbbbbbbb]\n",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "let x = [\n   aaaaaaaa;\n   bbbbbbbb;\n]\n"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn formatting_options_are_not_accepted_as_arguments() {
    let root = temp_dir("no-format-args");
    for arg in [
        "--indent-width=2",
        "--line-width=10",
        "--use-tabs",
        "--command-space",
        "--final-newline",
        "--prefer-config-file",
    ] {
        let output = run_with_stdin(&[arg], &root, "let x = 1\n");
        assert_eq!(output.status.code(), Some(2), "{arg}: {output:?}");
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn several_files_are_checked_and_written() {
    let root = temp_dir("several");
    fs::write(root.join("a.saty"), "let x  = 1\n").unwrap();
    fs::write(root.join("b.saty"), "let y = 2\n").unwrap();
    let bin = env!("CARGO_BIN_EXE_satysfi-fmt");

    // --check だけ・--write だけが必要
    let output = Command::new(bin)
        .current_dir(&root)
        .args(["a.saty", "b.saty"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));

    let output = Command::new(bin)
        .current_dir(&root)
        .args(["--check", "a.saty", "b.saty"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("a.saty: not formatted") && !stderr.contains("b.saty"),
        "{stderr}"
    );

    let output = Command::new(bin)
        .current_dir(&root)
        .args(["--write", "a.saty", "b.saty"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(root.join("a.saty")).unwrap(),
        "let x = 1\n"
    );

    // 1 つが失敗しても残りは処理し、終了コードは 2
    fs::write(root.join("a.saty"), "let x  = 1\n").unwrap();
    fs::write(root.join("bad.saty"), "let x = = 1\n").unwrap();
    let output = Command::new(bin)
        .current_dir(&root)
        .args(["--write", "bad.saty", "a.saty"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(root.join("a.saty")).unwrap(),
        "let x = 1\n"
    );
    fs::remove_dir_all(&root).unwrap();
}
