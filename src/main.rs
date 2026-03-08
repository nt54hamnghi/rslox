#![allow(unused_variables)]
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::{fs, io};

use clap::Parser as _;
use codecrafters_interpreter::cli::{Args, Command};
use codecrafters_interpreter::error::Report;
use codecrafters_interpreter::interpreter::Interpreter;
use codecrafters_interpreter::parser::Parser;
use codecrafters_interpreter::parser::expr::ExprNode;
use codecrafters_interpreter::parser::printer::AstPrinter;
use codecrafters_interpreter::scanner::token::Token;
use codecrafters_interpreter::scanner::{ScanItem, Scanner};

/// Parses CLI arguments and dispatches to the selected subcommand.
fn main() {
    let args = Args::parse();

    match args.subcommand {
        Command::Tokenize { filename } => {
            tokenize(filename, io::stdout());
        }
        Command::Parse { filename } => {
            parse(filename, io::stdout());
        }
        Command::Evaluate { filename } => {
            evaluate(filename, io::stdout());
        }
        Command::Run { filename } => {
            let res = run(filename);
            if let Err(err) = res {
                err.exit()
            }
        }
    };
}

fn run(filename: PathBuf) -> Result<(), Report> {
    let tokens = tokenize(filename, null());
    let mut parser = Parser::from(tokens);
    let ast = parser.parse()?;
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast)?;

    Ok(())
}

/// Parses and evaluates a single expression file, writing the result to `sink`.
///
/// Exits with code `70` if runtime evaluation fails.
fn evaluate(filename: PathBuf, mut sink: impl io::Write) {
    let expr = parse(filename, null());
    let interpreter = Interpreter::new();
    match interpreter.evaluate(&expr) {
        Ok(val) => writeln!(sink, "{}", val).unwrap(),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(70);
        }
    }
}

/// Tokenizes and parses a single expression file, prints its AST form to `sink`,
/// and returns the parsed expression node.
///
/// Exits with code `65` if parsing fails.
fn parse(filename: PathBuf, mut sink: impl io::Write) -> ExprNode {
    let tokens = tokenize(filename, null());
    let mut parser = Parser::from(tokens);
    match parser.parse_expression() {
        Ok(expr) => {
            writeln!(sink, "{}", AstPrinter.print(&expr)).unwrap();
            expr
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(65);
        }
    }
}

/// Scans tokens from `filename`, writing each token to `sink`, and returns all
/// successfully scanned tokens.
///
/// Exits with code `65` if any scan error occurs.
fn tokenize(filename: PathBuf, mut sink: impl io::Write) -> Vec<Token> {
    let content = read_file(filename);

    let mut has_error = false;
    let mut tokens = Vec::new();

    let scanner = Scanner::new(&content);
    for result in scanner.scan_tokens() {
        match result {
            Ok(ScanItem::Ignore) => continue,
            Ok(ScanItem::Token(tkn)) => {
                writeln!(sink, "{tkn}").unwrap();
                tokens.push(tkn);
            }
            Err(err) => {
                has_error = true;
                eprintln!("{err}");
            }
        }
    }

    if has_error {
        std::process::exit(65);
    }

    tokens
}

/// Reads an input file into a string.
///
/// Exits with code `1` when the file cannot be read.
fn read_file(filename: PathBuf) -> String {
    let Ok(file_contents) = fs::read_to_string(&filename) else {
        eprintln!("Failed to read file {}", filename.display());
        std::process::exit(1);
    };
    file_contents
}

/// Returns a writable sink that discards all bytes (`/dev/null`).
fn null() -> File {
    OpenOptions::new().write(true).open("/dev/null").unwrap()
}
