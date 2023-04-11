use crate::{
    error::error_at,
    lexer::{Token, TWL},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String,
    Number,
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negative,
    LogicalNot,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Equal,
    NotEqual,
    Lesser,
    Greater,
    LesserEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralType, String),
    Program(Vec<Expr>),
    Group(Box<Expr>),
    Unary(UnaryOperator, Box<Expr>),
    BinaryOperator(BinaryOperator, Box<Expr>, Box<Expr>),
    Identifier(Vec<String>),
    Call(Vec<String>, Vec<Expr>),
}
pub enum Node {
    ProcCall(/*Arguments*/ Vec<Expr>),
}

#[derive(Clone)]
pub struct Parser<'a> {
    pub tokens: Vec<TWL<'a>>,
}

macro_rules! eat_token {
    ($self: ident) => {
        &$self.tokens.remove(0)
    };
}
macro_rules! peek_token {
    ($self: ident) => {{
        let tok = &$self.tokens[0];

        tok
    }};
}

/*
expression     → literal
               | unary
               | binary
               | grouping ;

literal        → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
               | "+"  | "-"  | "*" | "/" ;
 */

impl Parser<'_> {
    pub fn check_binary(&mut self, op: TWL) -> bool {
        match op.token {
            Token::OperatorAdd()
            | Token::OperatorSubtract()
            | Token::OperatorMultiply()
            | Token::OperatorDivide()
            | Token::OperatorMod()
            | Token::OperatorSet()
            | Token::OperatorEquals()
            | Token::OperatorNotEquals()
            | Token::OperatorLogicalAnd()
            | Token::OperatorLogicalOr()
            | Token::OperatorLogicalNot()
            | Token::OperatorGreater()
            | Token::OperatorLesser() => {
                let mut p = self.clone();
                let expr = p.parse_expression();
                match expr {
                    Expr::Group(..) | Expr::Literal(..) => return true,
                    _ => return false,
                }
            }
            _ => {
                return false;
            }
        }
    }
    pub fn parse_expression(&mut self) -> Expr {
        return self.equality();
    }
    pub fn parse_program(&mut self) -> Expr {
        let mut l: Vec<Expr> = vec![];
        while self.tokens.len() > 1 {
            let expr = self.parse_expression();
            dbg!(expr.clone());
            l.push(expr);
        }
        return Expr::Program(l);
    }
    // MATH
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        loop {
            match peek_token!(self).token {
                Token::OperatorEquals() => {
                    let tmp = eat_token!(self);
                    let right = self.comparison();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Equal,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorNotEquals() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.comparison();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::NotEqual,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        loop {
            match peek_token!(self).token {
                Token::OperatorGreater() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Greater,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorLesser() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Lesser,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorLesserEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Lesser,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorGreaterEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Lesser,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        loop {
            match peek_token!(self).token {
                Token::OperatorSubtract() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.factor();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Subtract,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorAdd() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.factor();
                    expr =
                        Expr::BinaryOperator(BinaryOperator::Add, Box::new(expr), Box::new(right));
                }
                _ => break,
            }
        }
        return expr;
    }
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        loop {
            match peek_token!(self).token {
                Token::OperatorMultiply() => {
                    let temp = eat_token!(self).token.clone();
                    dbg!(self.tokens.clone());
                    let right = self.unary();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Multiply,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                Token::OperatorDivide() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.unary();
                    expr = Expr::BinaryOperator(
                        BinaryOperator::Divide,
                        Box::new(expr),
                        Box::new(right),
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn unary(&mut self) -> Expr {
        match peek_token!(self).token {
            Token::OperatorSubtract() => {
                _ = eat_token!(self).token.clone();
                let right = self.primary();
                return Expr::Unary(UnaryOperator::Negative, Box::new(right));
            }
            Token::OperatorLogicalNot() => {
                _ = eat_token!(self).token.clone();
                let right = self.primary();
                return Expr::Unary(UnaryOperator::LogicalNot, Box::new(right));
            }
            _ => {}
        }
        return self.primary();
    }
    fn primary(&mut self) -> Expr {
        let p = eat_token!(self);

        if let Token::StringLiteral(str) = &p.token {
            return Expr::Literal(LiteralType::String, str.to_string());
        }
        if let Token::NumericLiteral(num, _) = &p.token {
            return Expr::Literal(LiteralType::Number, num.to_string());
        }
        if let Token::Identifier(parts) = &p.token {
            if let Token::OpenParen() = peek_token!(self).token {
                eat_token!(self);
                let mut arguments: Vec<Expr> = vec![];
                loop {
                    if let Token::CloseParen() = peek_token!(self).token {
                        eat_token!(self);
                        break;
                    }
                    let expr = self.parse_expression();
                    arguments.push(expr);
                    if let Token::Comma() = peek_token!(self).token {
                        eat_token!(self);
                        continue;
                    }
                }
                return Expr::Call(parts.clone(), arguments);
            } else {
                return Expr::Identifier(parts.to_vec());
            }
        }
        if let Token::OpenParen() = &p.token {
            let expr = self.parse_expression();
            if let Token::EOF() = peek_token!(self).token {
                error_at(
                    &p.filen,
                    &p.linen,
                    &p.charn,
                    &format!("Prematurely reached EOF, did you end your grouping?"),
                )
            }
            let close = eat_token!(self).token.clone();
            if let Token::CloseParen() = close {
                return Expr::Group(Box::new(expr));
            }
            error_at(
                &p.filen,
                &p.linen,
                &p.charn,
                &format!("Expected ')' after expression, instead got: {:?}", close),
            )
        }
        if let Token::EOF() = &p.token {
            error_at(
                &p.filen,
                &p.linen,
                &p.charn,
                &format!("Prematurely reached EOF"),
            )
        }

        error_at(
            &p.filen,
            &p.linen,
            &p.charn,
            &format!("Token not implemented or invalid token: {:?}", p),
        )
    }
}
