use clap::Parser;
use satysfi_formatter::{format, resolve_config, PartialConfig};
use std::{fs, path::PathBuf, process};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// input file
    #[clap(parse(from_os_str), value_name = "FILE")]
    file: PathBuf,
    /// write to input file
    #[clap(short, long)]
    write: bool,
    /// output file
    #[clap(short, long)]
    output: Option<PathBuf>,
    /// indent size [default: 4]
    #[clap(short, long)]
    indent_space: Option<usize>,
    /// maximum line width [default: 80]
    #[clap(long)]
    line_width: Option<usize>,
    /// config file (searched from the directory of FILE upward if omitted)
    #[clap(long, parse(from_os_str), value_name = "PATH")]
    config: Option<PathBuf>,
    /// indent with tabs (`--use-tabs=false` to disable) [default: false]
    #[clap(
        long,
        value_name = "BOOL",
        min_values = 0,
        require_equals = true,
        default_missing_value = "true"
    )]
    use_tabs: Option<bool>,
    /// add space before text arguments of commands (`--cspace=false` to disable) [default: true]
    #[clap(
        long,
        value_name = "BOOL",
        min_values = 0,
        require_equals = true,
        default_missing_value = "true"
    )]
    cspace: Option<bool>,
}

fn main() {
    let cli = Cli::parse();
    let code = fs::read_to_string(&cli.file).expect("Failed to read file");

    let file_config = match &cli.config {
        Some(path) => PartialConfig::from_file(path),
        None => resolve_config(&cli.file),
    }
    .unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(2);
    });
    let cli_config = PartialConfig {
        line_width: cli.line_width.filter(|n| *n > 0),
        indent_width: cli.indent_space.filter(|n| *n > 0),
        use_tabs: cli.use_tabs,
        final_newline: None,
        command_space: cli.cspace,
    };
    let config = cli_config.or(file_config).build();

    let output = format(&code, &config).unwrap_or_else(|err| {
        eprintln!("{}: {}", cli.file.display(), err);
        process::exit(1);
    });

    match (cli.output, cli.write) {
        (Some(path), _) => fs::write(path, &output).expect("Failed to write file"),
        (None, true) => fs::write(&cli.file, &output).expect("Failed to write file"),
        (None, false) => print!("{}", output),
    }
}
