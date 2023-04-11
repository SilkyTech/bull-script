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
}

#[derive(Debug, Clone, PartialEq)]
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
    ($self: ident) => {
        &$self.tokens[0]
    };
}

macro_rules! peekpeek_token {
    ($self: ident) => {
        &$self.tokens.iter().nth(1)
    };
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
        let p = eat_token!(self);

        if let Token::StringLiteral(str) = &p.token {
            return Expr::Literal(LiteralType::String, str.to_string());
        }
        if let Token::NumericLiteral(num, _) = &p.token {
            return Expr::Literal(LiteralType::Number, num.to_string());
        }
        if let Token::Identifier(parts) = &p.token {
            if let Token::OpenParen() = peek_token!(self).token {
                self.parse_expression();
            }
        }
        if let Token::OpenParen() = &p.token {
            let expr = self.parse_expression();
            return Expr::Group(Box::new(expr));
        }

        // operators

        // binary
        if self.check_binary(p.clone()) {
            match &p.token {
                Token::OperatorEquals() | Token::OperatorNotEquals() => {
                    // Equality
                }
                _ => {}
            }
        }
        // unary
        match &p.token {
            Token::OperatorSubtract() => {
                let expr = self.parse_expression();
                return Expr::Unary(UnaryOperator::Negative, Box::new(expr));
            }
            Token::OperatorLogicalNot() => {
                let expr = self.parse_expression();
                return Expr::Unary(UnaryOperator::LogicalNot, Box::new(expr));
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
    // MATH
    pub fn equality(&mut self) -> Expr {
        /*
        private Expr equality() {
            Expr expr = comparison();

            while (match(BANG_EQUAL, EQUAL_EQUAL)) {
            Token operator = previous();
            Expr right = comparison();
            expr = new Expr.Binary(expr, operator, right);
            }

            return expr;
        } */
    }
}
