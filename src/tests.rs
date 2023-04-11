macro_rules! test_file {
    ($file: expr, $correct: expr) => {{
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
mod parser {
    use crate::lexer::Lexer;
    use crate::parser::Expr;
    use crate::parser::Parser;
    use crate::parser::{BinaryOperator, UnaryOperator};
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
    fn function() {
        test_file!(
            "../tests/function.bs",
            vec![Expr::Call(
                vec!["print".to_string()],
                vec![Expr::Literal(
                    crate::parser::LiteralType::String,
                    "Hello, world!".to_string()
                )]
            ),]
        )
    }

    #[test]
    fn math() {
        test_file!(
            "../tests/math.bs",
            vec![Expr::BinaryOperator(
                BinaryOperator::Multiply,
                Box::new(Expr::Group(Box::new(Expr::BinaryOperator(
                    BinaryOperator::Subtract,
                    Box::new(Expr::BinaryOperator(
                        BinaryOperator::Add,
                        Box::new(Expr::Literal(
                            crate::parser::LiteralType::Number,
                            "3".to_string()
                        )),
                        Box::new(Expr::BinaryOperator(
                            BinaryOperator::Multiply,
                            Box::new(Expr::Literal(
                                crate::parser::LiteralType::Number,
                                "5".to_string()
                            )),
                            Box::new(Expr::Literal(
                                crate::parser::LiteralType::Number,
                                "3".to_string()
                            ))
                        ))
                    )),
                    Box::new(Expr::Literal(
                        crate::parser::LiteralType::Number,
                        "4".to_string()
                    )),
                )))),
                Box::new(Expr::Unary(
                    UnaryOperator::Negative,
                    Box::new(Expr::Literal(
                        crate::parser::LiteralType::Number,
                        "4".to_string()
                    ))
                )),
            )]
        )
    }
}
