//! `satysfi-fmt`: SATySFi のフォーマッタ。
//!
//! language server やエディタからも、このコマンドをプロセスとして呼んで使う。
//! 引数・標準入出力・終了コードは公開インターフェースとして扱う (README 参照)。
//!
//! 整形の設定は設定ファイル (`.satysfi-fmt.toml`) だけで行い、コマンドライン引数では変えない。
//!
//! 終了コード: 0 = 成功 / 1 = `--check` で整形されていないファイルがあった / 2 = エラー
use clap::Parser;
use satysfi_formatter::{format, resolve_config, FormatError, PartialConfig};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// input files (read from stdin if omitted or `-`)
    #[clap(parse(from_os_str), value_name = "FILE")]
    files: Vec<PathBuf>,
    /// file name of the stdin input, used to search the config file and in messages
    #[clap(
        long,
        visible_alias = "stdin-filepath",
        parse(from_os_str),
        value_name = "PATH"
    )]
    assume_filename: Option<PathBuf>,
    /// overwrite the input files
    #[clap(short, long, conflicts_with_all = &["check", "output"])]
    write: bool,
    /// print nothing; exit with 1 if some input is not formatted
    #[clap(long, conflicts_with = "output")]
    check: bool,
    /// output file
    #[clap(short, long, parse(from_os_str), value_name = "PATH")]
    output: Option<PathBuf>,

    /// config file (searched from the directory of the input file upward if omitted)
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    config: Option<PathBuf>,
}

/// 整形の入力
enum Input<'a> {
    Stdin,
    File(&'a Path),
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Unchanged,
    Changed,
}

/// 1 つの入力を整形する。失敗は `path: message` の形のメッセージにして返す。
fn run_one(cli: &Cli, explicit: Option<&PartialConfig>, input: Input) -> Result<Outcome, String> {
    let (code, name) = match input {
        Input::File(path) => (
            fs::read_to_string(path).map_err(|err| format!("{}: {}", path.display(), err))?,
            path.to_path_buf(),
        ),
        Input::Stdin => {
            let mut code = String::new();
            io::stdin()
                .read_to_string(&mut code)
                .map_err(|err| format!("<stdin>: {}", err))?;
            let name = cli
                .assume_filename
                .clone()
                .unwrap_or_else(|| PathBuf::from("<stdin>"));
            (code, name)
        }
    };

    // 設定ファイルは、ファイルのディレクトリから探す。
    // `--assume-filename` がない標準入力は、カレントディレクトリから探す。
    let from_file = match explicit {
        Some(config) => config.clone(),
        None => {
            let base = match (&input, &cli.assume_filename) {
                (Input::Stdin, None) => std::env::current_dir()
                    .map_err(|err| err.to_string())?
                    .join("<stdin>"),
                _ => name.clone(),
            };
            resolve_config(&base).map_err(|err| err.to_string())?
        }
    };
    let config = from_file.build();

    // 位置は `path:line:column: error: message` の形式 (1 始まり。column は文字数)
    let output = format(&code, &config).map_err(|err| match err {
        FormatError::Parse { line, column, .. } => format!(
            "{}:{}:{}: error: syntax error",
            name.display(),
            line + 1,
            column + 1
        ),
    })?;
    let outcome = if output == code {
        Outcome::Unchanged
    } else {
        Outcome::Changed
    };

    if cli.check {
        if outcome == Outcome::Changed {
            eprintln!("{}: not formatted", name.display());
        }
        return Ok(outcome);
    }
    match (&cli.output, cli.write, &input) {
        (Some(path), _, _) => {
            fs::write(path, &output).map_err(|err| format!("{}: {}", path.display(), err))?
        }
        // 変わらないファイルは書き直さない (更新時刻を保つ)
        (None, true, Input::File(path)) => {
            if outcome == Outcome::Changed {
                fs::write(path, &output).map_err(|err| format!("{}: {}", path.display(), err))?
            }
        }
        _ => {
            // パイプが閉じられていても panic しない
            let _ = io::stdout().write_all(output.as_bytes());
        }
    }
    Ok(outcome)
}

fn run(cli: &Cli) -> ExitCode {
    let files: Vec<&Path> = cli
        .files
        .iter()
        .map(PathBuf::as_path)
        .filter(|path| path.as_os_str() != "-")
        .collect();
    let from_stdin = files.is_empty();

    if from_stdin && cli.write {
        eprintln!("error: --write needs FILE");
        return ExitCode::from(2);
    }
    // 結果を 1 つの出力にまとめられないので、複数ファイルは --write / --check のときだけ
    if files.len() > 1 && !cli.write && !cli.check {
        eprintln!("error: several files need --write or --check");
        return ExitCode::from(2);
    }

    let explicit = match &cli.config {
        Some(path) => match PartialConfig::from_file(path) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("{}", err);
                return ExitCode::from(2);
            }
        },
        None => None,
    };

    let inputs: Vec<Input> = if from_stdin {
        vec![Input::Stdin]
    } else {
        files.into_iter().map(Input::File).collect()
    };
    let (mut failed, mut unformatted) = (false, false);
    // 1 つ失敗しても残りは処理する
    for input in inputs {
        match run_one(cli, explicit.as_ref(), input) {
            Ok(Outcome::Changed) => unformatted = true,
            Ok(Outcome::Unchanged) => {}
            Err(message) => {
                eprintln!("{}", message);
                failed = true;
            }
        }
    }
    match (failed, cli.check && unformatted) {
        (true, _) => ExitCode::from(2),
        (false, true) => ExitCode::from(1),
        (false, false) => ExitCode::SUCCESS,
    }
}

fn main() -> ExitCode {
    run(&Cli::parse())
}
