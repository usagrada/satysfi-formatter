//! SATySFi の標準パッケージ (`~/.satysfi/dist/packages`) を整形しても壊れないことを確認する。
//! パッケージがない環境ではスキップする。
use super::check_invariants;
use std::fs;

fn check_package(file: &str) {
    let dir = match dirs::home_dir() {
        Some(home) => home.join(".satysfi/dist/packages"),
        None => return,
    };
    match fs::read_to_string(dir.join(file)) {
        Ok(input) => check_invariants(file, &input),
        Err(_) => eprintln!("skip: {file} is not installed"),
    }
}

#[test]
fn module_deco() {
    check_package("deco.satyh");
}

#[test]
fn module_table() {
    check_package("table.satyh");
}

#[test]
fn test_option() {
    check_package("option.satyg");
}
