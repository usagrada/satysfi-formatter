use clap::Parser;
use lspower::lsp::{FormattingOptions, FormattingProperty};
use satysfi_formatter::format;
use std::{fs, path::PathBuf};

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
    /// indent size
    #[clap(short, long, default_value_t = 4)]
    indent_space: usize,
    /// maximum line width
    #[clap(long, default_value_t = 80)]
    line_width: usize,
    /// Add space before arguments in command
    #[clap(long)]
    cspace: bool,
}

fn main() {
    let cli = Cli::parse();
    let code = fs::read_to_string(&cli.file).expect("Failed to read file");
    let mut option = FormattingOptions {
        tab_size: cli.indent_space as u32,
        ..Default::default()
    };
    option.properties.insert(
        "lineWidth".to_string(),
        FormattingProperty::Number(cli.line_width as i32),
    );
    let output = format(&code, option);

    match (cli.output, cli.write) {
        (Some(path), _) => fs::write(path, &output).expect("Failed to write file"),
        (None, true) => fs::write(&cli.file, &output).expect("Failed to write file"),
        (None, false) => print!("{}", output),
    }
}
