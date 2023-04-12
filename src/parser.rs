use crate::{
    error::error_at,
    lexer::{Token, TWL},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String,
    Number,
    Boolean,
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
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
    Identifier(Vec<String>),
    Call(Vec<String>, Vec<Expr>),
    // Import(Relative import?, path)
    Import(bool, String),
    Proc(Vec<String>, Vec<String>, Vec<Expr>),
    If(Box<Expr>, Vec<Expr>),
    For(Vec<String>, Box<Expr>, Box<Expr>, Vec<Expr>),
    VariableDeclaration(Vec<String>, Box<Expr>),
    ConstantDeclaration(Vec<String>, Box<Expr>),
    VariableSet(Vec<String>, Box<Expr>),
    Namespace(Vec<String>, Vec<Expr>),
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
        let peek = peek_token!(self);
        if let Token::ImportKeyword() = &peek.token.clone() {
            let peek = eat_token!(self);
            let path = eat_token!(self);
            match path.token.clone() {
                Token::StringLiteral(str) => return Expr::Import(true, str),
                Token::Identifier(vec) => return Expr::Import(false, vec.join(".")),
                _ => error_at(
                    &peek.filen,
                    &peek.linen,
                    &peek.charn,
                    &format!("Expected string literal or identifier after import statement, instead found {:?}", path.token),
                ),
            }
        }

        if let Token::Namespace() = &peek.token.clone() {
            eat_token!(self);

            let nmspc_name = {
                let then = eat_token!(self);
                if let Token::Identifier(ve) = then.token.clone() {
                    ve
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected Identifier, got {:?}", then.token),
                    )
                }
            };

            eat_token!(self);

            let mut key = peek_token!(self);
            let mut program: Vec<Expr> = vec![];
            loop {
                if let Token::End() = key.token {
                    eat_token!(self);
                    break;
                }
                if let Token::EOF() = key.token {
                    error_at(
                        &key.filen,
                        &key.linen,
                        &key.charn,
                        &format!("Prematurely reached EOF, did you end your proc?"),
                    )
                }
                program.push(self.parse_expression());
                key = peek_token!(self);
            }
            return Expr::Namespace(nmspc_name, program);
        }

        if let Token::Let() = &peek.token.clone() {
            _ = eat_token!(self);
            let varname = {
                let then = eat_token!(self);
                if let Token::Identifier(ve) = then.token.clone() {
                    ve
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected Identifier, got {:?}", then.token),
                    )
                }
            };
            _ = {
                let then = eat_token!(self);
                if let Token::OperatorSet() = then.token.clone() {
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"=\", got {:?}", then.token),
                    )
                }
            };
            let expr = self.parse_expression().clone();
            return Expr::VariableDeclaration(varname, Box::new(expr));
        }
        if let Token::Const() = &peek.token.clone() {
            _ = eat_token!(self);
            let varname = {
                let then = eat_token!(self);
                if let Token::Identifier(ve) = then.token.clone() {
                    ve
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected Identifier, got {:?}", then.token),
                    )
                }
            };
            _ = {
                let then = eat_token!(self);
                if let Token::OperatorSet() = then.token.clone() {
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"=\", got {:?}", then.token),
                    )
                }
            };
            let expr = self.parse_expression().clone();
            return Expr::ConstantDeclaration(varname, Box::new(expr));
        }

        if let Token::If() = &peek.token.clone() {
            eat_token!(self);

            let expr = self.parse_expression();

            // get body of program
            {
                let then = eat_token!(self);
                if !matches!(then.token, Token::Then(..)) {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"then\" keyword, got {:?}", then.token),
                    )
                };
            }
            let mut key = peek_token!(self);
            let mut program: Vec<Expr> = vec![];
            loop {
                if let Token::End() = key.token {
                    eat_token!(self);
                    break;
                }
                if let Token::EOF() = key.token {
                    error_at(
                        &key.filen,
                        &key.linen,
                        &key.charn,
                        &format!("Prematurely reached EOF, did you end your proc?"),
                    )
                }
                program.push(self.parse_expression());
                key = peek_token!(self);
            }
            return Expr::If(Box::new(expr), program);
        }
        if let Token::For() = &peek.token.clone() {
            eat_token!(self);

            let varname = {
                let then = eat_token!(self);
                if let Token::Identifier(ve) = then.token.clone() {
                    ve
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected Identifier, got {:?}", then.token),
                    )
                }
            };

            _ = {
                let then = eat_token!(self);
                if let Token::OperatorSet() = then.token.clone() {
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"=\", got {:?}", then.token),
                    )
                }
            };

            let startval = {
                let then = eat_token!(self);
                if let Token::NumericLiteral(f, _s) = then.token.clone() {
                    Expr::Literal(LiteralType::Number, f.to_string())
                } else if let Token::Identifier(ve) = then.token.clone() {
                    Expr::Identifier(ve)
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!(
                            "Expected Identifier or numeric literal, got {:?}",
                            then.token
                        ),
                    )
                }
            };

            _ = {
                let then = eat_token!(self);
                if let Token::To() = then.token.clone() {
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"to\", got {:?}", then.token),
                    )
                }
            };
            let endval = {
                let then = eat_token!(self);
                if let Token::NumericLiteral(f, _s) = then.token.clone() {
                    Expr::Literal(LiteralType::Number, f.to_string())
                } else if let Token::Identifier(ve) = then.token.clone() {
                    Expr::Identifier(ve)
                } else {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!(
                            "Expected Identifier or numeric literal, got {:?}",
                            then.token
                        ),
                    )
                }
            };

            // get body of program
            {
                let then = eat_token!(self);
                if !matches!(then.token, Token::Then(..)) {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"then\" keyword, got {:?}", then.token),
                    )
                };
            }
            let mut key = peek_token!(self);
            let mut program: Vec<Expr> = vec![];
            loop {
                if let Token::End() = key.token {
                    eat_token!(self);
                    break;
                }
                if let Token::EOF() = key.token {
                    error_at(
                        &key.filen,
                        &key.linen,
                        &key.charn,
                        &format!("Prematurely reached EOF, did you end your proc?"),
                    )
                }
                program.push(self.parse_expression());
                key = peek_token!(self);
            }
            return Expr::For(varname, Box::new(startval), Box::new(endval), program);
        }
        /*if let Token::IfKeyword() = &peek.token {
            eat_token!(self);

            let expr = self.parse_expression();

            // get body of program
            {
                let then = eat_token!(self);
                if !matches!(then.token, Token::ThenKeyword(..)) {
                    error_at(
                        &then.filen,
                        &then.linen,
                        &then.charn,
                        &format!("Expected \"then\" keyword, got {:?}", then.token),
                    )
                };
            }
            let mut key = peek_token!(self);
            let mut program: Vec<Expr> = vec![];
            loop {
                if let Token::EndKeyword() = key.token {
                    eat_token!(self);
                    break;
                }
                if let Token::EOF() = key.token {
                    error_at(
                        &key.filen,
                        &key.linen,
                        &key.charn,
                        &format!("Prematurely reached EOF, did you end your proc?"),
                    )
                }
                program.push(self.parse_expression());
                key = peek_token!(self);
            }
            return Expr::If(Box::new(expr), program);
        }
        */
        // TODO: ^ add while loop
        return self.equality();
    }
    pub fn parse_program(&mut self) -> Expr {
        let mut l: Vec<Expr> = vec![];
        while self.tokens.len() > 1 {
            let expr = self.parse_expression();
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
                    let _tmp = eat_token!(self);
                    let right = self.comparison();
                    expr = Expr::Binary(BinaryOperator::Equal, Box::new(expr), Box::new(right));
                }
                Token::OperatorNotEquals() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.comparison();
                    expr = Expr::Binary(BinaryOperator::NotEqual, Box::new(expr), Box::new(right));
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
                    expr = Expr::Binary(BinaryOperator::Greater, Box::new(expr), Box::new(right));
                }
                Token::OperatorLesser() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::Binary(BinaryOperator::Lesser, Box::new(expr), Box::new(right));
                }
                Token::OperatorLesserEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::Binary(BinaryOperator::Lesser, Box::new(expr), Box::new(right));
                }
                Token::OperatorGreaterEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = Expr::Binary(BinaryOperator::Lesser, Box::new(expr), Box::new(right));
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
                    expr = Expr::Binary(BinaryOperator::Subtract, Box::new(expr), Box::new(right));
                }
                Token::OperatorAdd() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.factor();
                    expr = Expr::Binary(BinaryOperator::Add, Box::new(expr), Box::new(right));
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
                    let _temp = eat_token!(self).token.clone();
                    let right = self.unary();
                    expr = Expr::Binary(BinaryOperator::Multiply, Box::new(expr), Box::new(right));
                }
                Token::OperatorDivide() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.unary();
                    expr = Expr::Binary(BinaryOperator::Divide, Box::new(expr), Box::new(right));
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
        if let Token::BooleanLiteral(b) = &p.token {
            return if b.clone() {
                Expr::Literal(LiteralType::Boolean, "1".to_string())
            } else {
                Expr::Literal(LiteralType::Boolean, "0".to_string())
            };
        }
        if let Token::Proc() = &p.token {
            let name = eat_token!(self);
            if let Token::Identifier(n) = name.token.clone() {
                let _open = eat_token!(self);
                let mut args: Vec<String> = vec![];
                let do_loop = true;
                let mut depth = 0;
                while do_loop {
                    depth += 1;
                    if depth > 1000 {
                        panic!("Reached maximum argument find depth of 1000! You have way too many arguments!");
                    }
                    let peek = peek_token!(self);
                    if let Token::CloseParen() = peek.token {
                        _ = eat_token!(self);
                        break;
                    }
                    let d = eat_token!(self);
                    if let Token::Identifier(ve) = d.token.clone() {
                        args.push(ve[0].clone());
                    } else {
                        error_at(
                            &d.filen,
                            &d.linen,
                            &d.charn,
                            &format!("Expected Identifier, got {:?}", d.token),
                        );
                    };

                    let peek = peek_token!(self);
                    if let Token::Comma() = peek.token {
                        _ = eat_token!(self);
                        continue;
                    }
                }

                // get body of program
                {
                    let then = eat_token!(self);
                    if !matches!(then.token, Token::Then(..)) {
                        error_at(
                            &then.filen,
                            &then.linen,
                            &then.charn,
                            &format!("Expected \"then\" keyword, got {:?}", then.token),
                        )
                    };
                }
                let mut key = peek_token!(self);
                let mut program: Vec<Expr> = vec![];
                loop {
                    if let Token::End() = key.token {
                        eat_token!(self);
                        break;
                    }
                    if let Token::EOF() = key.token {
                        error_at(
                            &p.filen,
                            &p.linen,
                            &p.charn,
                            &format!("Prematurely reached EOF, did you end your proc?"),
                        )
                    }
                    program.push(self.parse_expression());
                    key = peek_token!(self);
                }
                return Expr::Proc(n, args, program);
            } else {
                error_at(
                    &name.filen,
                    &name.linen,
                    &name.charn,
                    &format!("Expected Identifier, got {:?}", name.token),
                )
            }
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
            } else if let Token::OperatorSet() = peek_token!(self).token {
                // setting variable
                eat_token!(self);
                let expr = self.parse_expression();
                return Expr::VariableSet(parts.clone(), Box::new(expr));
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
