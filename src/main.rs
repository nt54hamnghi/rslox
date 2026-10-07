use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Parser;
use rslox::errors::Error;
use rslox::vm::VM;

#[derive(Debug, Parser)]
struct Cli {
    path: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();

    match cli.path {
        Some(path) => run_file(path),
        None => repl(),
    }
}

fn repl() -> color_eyre::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut buf = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;
        // read_line return Ok(0) on EOF and Ctrl+D sends EOF
        if stdin.read_line(&mut buf)? == 0 {
            break;
        }
        VM::interpret(buf.trim_end());
        buf.clear();
    }

    Ok(())
}

fn run_file(path: impl AsRef<Path>) -> color_eyre::Result<()> {
    let source = fs::read_to_string(path)?;
    match VM::interpret(&source) {
        Ok(_) => Ok(()),
        Err(err) => match err {
            Error::Compile => exit(65),
            Error::Runtime => exit(70),
        },
    }
}
