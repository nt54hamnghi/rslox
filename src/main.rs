#![allow(unused_variables)]
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use codecrafters_interpreter::cli;
use codecrafters_interpreter::scanner::ScanResult;
use codecrafters_interpreter::scanner::Scanner;
use codecrafters_interpreter::scanner::token::Token;

fn main() {
    let args = cli::Args::parse();

    match args.subcommand {
        cli::Command::Tokenize { filename } => tokenize(filename),
        cli::Command::Parse { filename } => parse(filename),
    }
}

fn parse(filename: PathBuf) {
    let Some(content) = read_file(filename) else {
        return;
    };

    let tokens = Scanner::new(&content)
        .scan_tokens()
        .filter_map(|s| match s {
            ScanResult::Result(token) => Some(token),
            ScanResult::Ignore => None,
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    println!("{tokens:#?}");
}

fn tokenize(filename: PathBuf) {
    let Some(content) = read_file(filename) else {
        return;
    };

    let scanner = Scanner::new(&content);
    let mut has_error = false;
    for result in scanner.scan_tokens() {
        match result {
            ScanResult::Ignore => continue,
            ScanResult::Result(Ok(t)) => println!("{t}"),
            ScanResult::Result(Err(e)) => {
                if !has_error {
                    has_error = true;
                }
                eprintln!("{e}");
            }
        }
    }

    if has_error {
        std::process::exit(65);
    } else {
        std::process::exit(0);
    }
}

fn read_file(filename: PathBuf) -> Option<String> {
    let Ok(file_contents) = fs::read_to_string(&filename) else {
        eprintln!("Failed to read file {}", filename.display());
        println!("{}", Token::new_eof(0));
        return None;
    };
    Some(file_contents)
}
