use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;

use yarrlox::interpreter::Interpreter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Script to run. If not provided, a REPL session is started
    script: Option<PathBuf>,
}

fn main() {
    if let Err(err) = run_cli() {
        error_handler(err);
    }
}

fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.script {
        Some(script) => run_script(script)?,
        None => run_repl()?,
    }

    Ok(())
}

fn error_handler(err: anyhow::Error) {
    eprintln!("yarrlox encountered a critical error! Argh!");
    eprintln!("{}", err);

    exit(42)
}

fn run_script(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;
    let mut interpreter = Interpreter::new();
    yarrlox::eval(&source, yarrlox::errors::SimpleReporter, &mut interpreter);

    Ok(())
}

fn run_repl() -> anyhow::Result<()> {
    let mut interpreter = Interpreter::new();

    fn prompt() -> std::io::Result<()> {
        print!("> ");
        io::stdout().flush()
    }

    let stdin = io::stdin().lines();

    prompt()?;

    for line in stdin {
        match line {
            Ok(line) => yarrlox::eval(&line, yarrlox::errors::SimpleReporter, &mut interpreter),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
        prompt()?;
    }

    eprintln!();
    eprintln!("Buh-bye!");

    Ok(())
}
