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
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            if !file_contents.is_empty() {
                let scanner = Scanner::new(&file_contents);
                for token in scanner.scan_tokens() {
                    println!("{}", token);
                }
            } else {
                println!("{}", Token::eof_token());
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
