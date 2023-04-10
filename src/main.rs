use lexer::Lexer;
use parser::{Expr, Parser};
pub mod error;
pub mod lexer;
pub mod parser;
use std::{env, fs};

fn main() {
    let argv: Vec<String> = env::args().collect();
    let contents_raw = "";
    let contents = fs::read_to_string(argv[1].clone())
        .expect("Should have been able to read the file")
        .replace("\r", &contents_raw);
    let lexer = Lexer {
        text: contents,
        filename: String::from("test.bs"),
    };

    let tokens = lexer.lex();
    let mut parser = Parser { tokens };
    let program = parser.parse_program();
    if let Expr::Program(p) = program {
        dbg!(p);
    }
}
