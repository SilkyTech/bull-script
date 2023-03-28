use lexer::Lexer;
pub mod error;
pub mod lexer;
use std::fs;

fn main() {
    let contents_raw = "";
    let contents = fs::read_to_string("test.bs")
        .expect("Should have been able to read the file")
        .replace("\r", &contents_raw);
    let lexer = Lexer {
        text: contents,
        filename: String::from("test.bs"),
    };

    dbg!(lexer.lex());
}
