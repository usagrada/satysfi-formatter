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
        "indent_width = 2\nline_width = 16\n",
    )
    .unwrap();
    fs::write(sub.join("a.satyh"), "let x = [aaaaaaaa; bbbbbbbb]\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_satysfi-fmt"))
        .current_dir(&sub)
        .arg("a.satyh")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "let x = [\n  aaaaaaaa;\n  bbbbbbbb;\n]\n"
    );

    fs::remove_dir_all(&root).unwrap();
}
