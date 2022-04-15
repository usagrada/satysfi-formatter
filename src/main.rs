use clap::Parser;
use satysfi_formatter::{format, OptionData};
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
    /// Add space before arguments in command
    #[clap(long)]
    cspace: bool,
}

fn main() {
    let cli = Cli::parse();
    let code = fs::read_to_string(&cli.file).expect("Failed to read file");
    let option = OptionData {
        indent_space: cli.indent_space,
        command_args_space: cli.cspace,
        ..Default::default()
    };
    let output = format(&code, option);

    match (cli.output, cli.write) {
        (Some(path), _) => fs::write(&path, output).expect("Failed to write file"),
        (None, true) => fs::write(&cli.file, output).expect("Failed to write file"),
        (None, false) => println!("{}", output),
    }
}
