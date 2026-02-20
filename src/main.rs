#![allow(unused_variables)]
use std::fs;
use std::path::PathBuf;

use clap::Parser as _;
use codecrafters_interpreter::cli;
use codecrafters_interpreter::parser::Parser;
use codecrafters_interpreter::parser::printer::AstPrinter;
use codecrafters_interpreter::scanner::ScanItem;
use codecrafters_interpreter::scanner::Scanner;

fn main() {
    let args = cli::Args::parse();

    match args.subcommand {
        cli::Command::Tokenize { filename } => tokenize(filename),
        cli::Command::Parse { filename } => parse(filename),
    }
}

fn parse(filename: PathBuf) {
    let content = read_file(filename);

    let mut has_error = false;
    let mut tokens = Vec::new();

    let scanner = Scanner::new(&content);
    for result in scanner.scan_tokens() {
        match result {
            Ok(ScanItem::Ignore) => continue,
            Ok(ScanItem::Token(tkn)) => tokens.push(tkn),
            Err(err) => {
                has_error = true;
                eprintln!("{err}");
            }
        }
    }

    if has_error {
        std::process::exit(65);
    }

    let mut parser = Parser::from(tokens);
    match parser.parse() {
        Ok(expr) => {
            let expr_str = AstPrinter.print(expr);
            println!("{expr_str}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(65);
        }
    };
}

fn tokenize(filename: PathBuf) {
    let content = read_file(filename);

    let mut has_error = false;

    let scanner = Scanner::new(&content);
    for result in scanner.scan_tokens() {
        match result {
            Ok(ScanItem::Ignore) => continue,
            Ok(ScanItem::Token(tkn)) => println!("{tkn}"),
            Err(err) => {
                has_error = true;
                eprintln!("{err}");
            }
        }
    }

    if has_error {
        std::process::exit(65);
    }
}

fn read_file(filename: PathBuf) -> String {
    let Ok(file_contents) = fs::read_to_string(&filename) else {
        eprintln!("Failed to read file {}", filename.display());
        std::process::exit(1);
    };
    file_contents
}
