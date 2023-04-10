use crate::{
    error::error_at,
    lexer::{Token, TWL},
};

#[derive(Debug, Clone)]
pub enum LiteralType {
    String,
    Number,
}
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negative,
    LogicalNot,
}
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralType, String),
    Program(Vec<Expr>),
    Group(Box<Expr>),
    Unary(UnaryOperator, Box<Expr>),
    BinaryOperator(BinaryOperator, Box<Expr>, Box<Expr>),
}
pub enum Node {
    ProcCall(/*Arguments*/ Vec<Expr>),
}

pub struct Parser<'a> {
    pub tokens: Vec<TWL<'a>>,
}

macro_rules! eat_token {
    ($self: ident) => {
        &$self.tokens.remove(0)
    };
}
macro_rules! peek_token {
    ($self: ident) => {
        &$self.tokens[0]
    };
}

macro_rules! return_check {
    ($self: ident, $ret: expr) => {{
        let ret = $ret;
        return $self.check_binary(ret);
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
    pub fn check_binary(&mut self, ret: Expr) -> Expr {
        if self.tokens.len() < 1 {
            return ret;
        }
        let p = peek_token!(self);

        match &p.token {
            Token::OperatorAdd()
            | Token::OperatorSubtract()
            | Token::OperatorDivide()
            | Token::OperatorMultiply()
            | Token::OperatorEquals()
            | Token::OperatorMod()
            | Token::OperatorGreater()
            | Token::OperatorLesser()
            | Token::OperatorLogicalAnd()
            | Token::OperatorLogicalNot()
            | Token::OperatorLogicalOr() => {
                let _ = eat_token!(self);
                let expr = self.parse_expression();
                // TODO: ONLY DOES ADD, FIX THIS L8R WITH SOME JANKY FIXES IDK
                return Expr::BinaryOperator(BinaryOperator::Add, Box::new(ret), Box::new(expr));
            }
            _ => return ret,
        }
    }
    pub fn parse_expression(&mut self) -> Expr {
        let p = eat_token!(self);
        if let Token::StringLiteral(str) = &p.token {
            return_check!(self, Expr::Literal(LiteralType::String, str.to_string()));
        }
        if let Token::NumericLiteral(num, _) = &p.token {
            return_check!(self, Expr::Literal(LiteralType::Number, num.to_string()));
        }
        if let Token::Identifier(parts) = &p.token {
            if let Token::OpenParen() = peek_token!(self).token {
                self.parse_expression();
            }
        }
        if let Token::OpenParen() = &p.token {
            let expr = self.parse_expression();
            return_check!(self, Expr::Group(Box::new(expr)));
        }

        // operators

        // unary
        match &p.token {
            Token::OperatorSubtract() => {
                let expr = self.parse_expression();
                return_check!(self, Expr::Unary(UnaryOperator::Negative, Box::new(expr)));
            }
            Token::OperatorLogicalNot() => {
                let expr = self.parse_expression();
                return_check!(self, Expr::Unary(UnaryOperator::LogicalNot, Box::new(expr)));
            }
            _ => {}
        }

        error_at(
            &p.filen,
            &p.linen,
            &p.charn,
            &format!("Token not implemented or invalid token: {:?}", p),
        )
    }
    pub fn parse_program(&mut self) -> Expr {
        let mut l: Vec<Expr> = vec![];
        while self.tokens.len() > 0 {
            l.push(self.parse_expression());
        }
        return Expr::Program(l);
    }
}
