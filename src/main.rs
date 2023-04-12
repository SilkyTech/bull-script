use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
pub mod error;
pub mod lexer;
pub mod parser;
use std::{env, fs};
pub mod interpreter;
pub mod tests;

fn main() {
    let argv: Vec<String> = env::args().collect();
    let contents_raw = "";
    let contents = fs::read_to_string(argv[1].clone())
        .expect("Should have been able to read the file")
        .replace("\r", &contents_raw);
    let lexer = Lexer {
        text: contents,
        filename: String::from(argv[1].clone()),
    };

    let tokens = lexer.lex();
    let mut parser = Parser { tokens };
    let program = parser.parse_program();

    let mut inter = Interpreter::new();
    inter.run_program(program, true, vec![]);
}
