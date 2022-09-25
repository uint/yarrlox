use std::{
    env,
    io::{self, Write},
    path::Path,
    process::exit,
};

fn main() {
    if let Err(err) = run_cli() {
        error_handler(err);
    }
}

fn run_cli() -> anyhow::Result<()> {
    if env::args().len() > 2 {
        return Err(anyhow::anyhow!("too many arguments"));
    }

    match env::args().nth(1) {
        Some(path) => run_script(path)?,
        None => run_repl(),
    }

    Ok(())
}

/// This error handler is for basic errors when trying to use the CLI
/// (wrong path, wrong args, etc.). This isn't the handler for compilation
/// errors.
fn error_handler(err: anyhow::Error) {
    let exe_name = env::current_exe()
        .map(|p| {
            p.into_iter()
                .last()
                .map(|s| s.to_os_string().into_string().ok())
                .flatten()
        })
        .ok()
        .flatten()
        .unwrap_or(env!("CARGO_CRATE_NAME").to_string());

    eprintln!("Usage: {} [script]", exe_name,);
    eprintln!();
    eprintln!("The interpreter has met an awful fate. Argh!");
    eprintln!("{}", err);
    exit(1)
}

fn run_script(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;
    println!("{}", yarrlox::eval(&source));

    Ok(())
}

fn run_repl() {
    fn prompt() {
        print!("> ");
        io::stdout().flush().unwrap();
    }

    let stdin = io::stdin().lines();

    prompt();

    for line in stdin {
        match line {
            Ok(line) => println!("{}", yarrlox::eval(&line)),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
        prompt();
    }
}
