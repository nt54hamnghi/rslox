#![allow(unused_variables)]
use std::env;
use std::fs;

use codecrafters_interpreter::scanner::Scanner;
use codecrafters_interpreter::scanner::token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let Ok(file_contents) = fs::read_to_string(filename) else {
                eprintln!("Failed to read file {}", filename);
                println!("{}", Token::new_eof(0));
                return;
            };

            let scanner = Scanner::new(&file_contents);

            let mut has_error = false;
            for item in scanner.scan_tokens() {
                match item {
                    Ok(t) => println!("{t}"),
                    Err(e) => {
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
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
