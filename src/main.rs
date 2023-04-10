use lexer::Lexer;
use parser::{Expr, Parser};
pub mod error;
pub mod lexer;
pub mod parser;
use std::{env, fs};

macro_rules! test_file {
    ($file: expr, $correct: expr) => {{
        use crate::lexer::Lexer;
        use crate::parser::Expr;
        use crate::parser::Parser;

        let raw = include_str!($file).replace("\r", &"");
        dbg!(raw.clone());
        let lexer = Lexer {
            text: raw.to_string(),
            filename: String::from($file),
        };

        let tokens = lexer.lex();
        let mut parser = Parser { tokens };
        let program = parser.parse_program();
        if let Expr::Program(p) = program {
            println!("{:?}", p);
            assert_eq!(p, $correct)
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::parser::BinaryOperator;

    #[test]
    fn literal() {
        test_file!(
            "../tests/literal.bs",
            vec![
                Expr::Literal(
                    crate::parser::LiteralType::String,
                    "Hello, world!".to_string()
                ),
                Expr::Literal(crate::parser::LiteralType::Number, "10".to_string())
            ]
        )
    }

    #[test]
    fn math() {
        test_file!(
            "../tests/math.bs",
            vec![
                Expr::Unary(
                    crate::parser::UnaryOperator::Negative,
                    Box::new(Expr::Literal(
                        crate::parser::LiteralType::Number,
                        "9".to_string()
                    ))
                ),
                Expr::BinaryOperator(
                    BinaryOperator::Subtract,
                    Box::new(Expr::BinaryOperator(
                        BinaryOperator::Add,
                        Box::new(Expr::Literal(
                            crate::parser::LiteralType::Number,
                            "1".to_string()
                        )),
                        Box::new(Expr::BinaryOperator(
                            BinaryOperator::Multiply,
                            Box::new(Expr::Literal(
                                crate::parser::LiteralType::Number,
                                "3".to_string()
                            )),
                            Box::new(Expr::Literal(
                                crate::parser::LiteralType::Number,
                                "6".to_string()
                            ))
                        ))
                    )),
                    Box::new(Expr::Literal(
                        crate::parser::LiteralType::Number,
                        "5".to_string()
                    )),
                )
            ]
        )
    }
}

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
