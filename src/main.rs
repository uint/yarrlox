use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser as ClapParser;

use yarrlox::{interpreter::Interpreter, parser::Parser, EvalErrors};

#[derive(ClapParser)]
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
    eprintln!("{}", err);

    exit(42)
}

fn run_script(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;
    let mut parser = Parser::new();
    let mut interpreter = Interpreter::default();
    match yarrlox::eval(
        &source,
        yarrlox::errors::SimpleReporter,
        &mut parser,
        &mut interpreter,
    ) {
        Ok(_) => Ok(()),
        Err(EvalErrors::Syntax(_)) => Err(anyhow::anyhow!("syntax errors present")),
        Err(EvalErrors::Resolution(_)) => {
            Err(anyhow::anyhow!("variable resolution errors present"))
        }
        Err(EvalErrors::Interpreter(_)) => Err(anyhow::anyhow!("runtime errors present")),
    }
}

fn run_repl() -> anyhow::Result<()> {
    let mut parser = Parser::new();
    let mut interpreter = Interpreter::default();

    fn prompt() -> std::io::Result<()> {
        print!("> ");
        io::stdout().flush()
    }

    let stdin = io::stdin().lines();

    prompt()?;

    for line in stdin {
        match line {
            Ok(line) => {
                let _ = yarrlox::eval(
                    &line,
                    yarrlox::errors::SimpleReporter,
                    &mut parser,
                    &mut interpreter,
                );
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
        prompt()?;
    }

    eprintln!();
    eprintln!("Buh-bye!");

    Ok(())
}
