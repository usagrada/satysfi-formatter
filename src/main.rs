use clap::Parser;
use satysfi_formatter::format;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// input file
    #[clap(parse(from_os_str), value_name = "FILE")]
    file: PathBuf,
    #[clap(short, long)]
    output: Option<PathBuf>,
    #[clap(long)]
    option: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let code = fs::read_to_string(&cli.file).expect("Failed to read file");
    let output = format(&code);

    match cli.output {
        Some(path) => fs::write(&path, output).expect("Failed to write file"),
        None => println!("{}", output),
    }
}
