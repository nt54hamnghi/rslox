#![allow(unused_variables)]
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

use clap::Parser as _;
use codecrafters_interpreter::cli;
use codecrafters_interpreter::interpreter::Interpreter;
use codecrafters_interpreter::parser::Parser;
use codecrafters_interpreter::parser::expr::AstNode;
use codecrafters_interpreter::parser::printer::AstPrinter;
use codecrafters_interpreter::scanner::ScanItem;
use codecrafters_interpreter::scanner::Scanner;
use codecrafters_interpreter::scanner::token::Token;

fn main() {
    let args = cli::Args::parse();

    match args.subcommand {
        cli::Command::Tokenize { filename } => {
            tokenize(filename, io::stdout());
        }
        cli::Command::Parse { filename } => {
            parse(filename, io::stdout());
        }
        cli::Command::Evaluate { filename } => {
            evaluate(filename);
        }
    };
}

fn evaluate(filename: PathBuf) {
    let expr = parse(filename, null());
    let interpreter = Interpreter;
    if let Err(err) = interpreter.interpret(&expr) {
        eprintln!("{err}");
        std::process::exit(70);
    }
}

fn parse(filename: PathBuf, mut sink: impl io::Write) -> AstNode {
    let tokens = tokenize(filename, null());
    let mut parser = Parser::from(tokens);
    match parser.parse() {
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

fn read_file(filename: PathBuf) -> String {
    let Ok(file_contents) = fs::read_to_string(&filename) else {
        eprintln!("Failed to read file {}", filename.display());
        std::process::exit(1);
    };
    file_contents
}

fn null() -> File {
    OpenOptions::new().write(true).open("/dev/null").unwrap()
}
