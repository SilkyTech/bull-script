use crate::{
    error::error_at,
    lexer::{Token, TWL},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String,
    Number,
    Boolean,
    Null,
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
    Program(Vec<ExprWL>),
    Group(Box<ExprWL>),
    Unary(UnaryOperator, Box<ExprWL>),
    Binary(BinaryOperator, Box<ExprWL>, Box<ExprWL>),
    Identifier(Vec<String>),
    Call(Vec<String>, Vec<ExprWL>),
    // Import(Relative import?, path)
    Import(bool, String),
    Proc(Vec<String>, Vec<String>, Vec<ExprWL>),
    If(Box<ExprWL>, Vec<ExprWL>),
    For(Vec<String>, Box<ExprWL>, Box<ExprWL>, Vec<ExprWL>),
    While(Box<ExprWL>, Vec<ExprWL>),
    Return(Box<ExprWL>),
    VariableDeclaration(Vec<String>, Box<ExprWL>),
    ConstantDeclaration(Vec<String>, Box<ExprWL>),
    VariableSet(Vec<String>, Box<ExprWL>),
    Namespace(Vec<String>, Vec<ExprWL>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprWL {
    pub expr: Expr,
    pub linen: i32,
    pub charn: i32,
    pub filen: String,
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

        tok.clone()
    }};
}

#[macro_export]
macro_rules! ctwl {
    ($t: expr, $s: expr) => {
        ExprWL {
            expr: $t.clone(),
            linen: $s.clone().linen.clone(),
            filen: $s.clone().filen.clone(),
            charn: $s.clone().charn.clone(),
        }
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
    pub fn parse_expression(&mut self) -> ExprWL {
        let peek = peek_token!(self);
        if let Token::ImportKeyword() = &peek.token.clone() {
            let peek = eat_token!(self);
            let path = eat_token!(self);
            match path.token.clone() {
                Token::StringLiteral(str) => return ctwl!(Expr::Import(true, str), peek),
                Token::Identifier(vec) => return ctwl!(Expr::Import(false, vec.join(".")), peek),
                _ => error_at(
                    &peek.filen,
                    &peek.linen,
                    &peek.charn,
                    &format!("Expected string literal or identifier after import statement, instead found {:?}", path.token),
                ),
            }
        }

        if let Token::Namespace() = peek.token.clone() {
            _ = eat_token!(self);

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

            _ = eat_token!(self);

            let mut key = peek_token!(self);
            let mut program: Vec<ExprWL> = vec![];
            loop {
                if let Token::End() = key.token {
                    _ = eat_token!(self);
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
            return ctwl!(Expr::Namespace(nmspc_name, program), peek);
        }

        if let Token::Let() = peek.token.clone() {
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
            return ctwl!(Expr::VariableDeclaration(varname, Box::new(expr)), peek);
        }
        if let Token::Const() = peek.token.clone() {
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
            return ctwl!(Expr::ConstantDeclaration(varname, Box::new(expr)), peek);
        }

        if let Token::If() = peek.token.clone() {
            _ = eat_token!(self);

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
            let mut program: Vec<ExprWL> = vec![];
            loop {
                if let Token::End() = key.token {
                    _ = eat_token!(self);
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
            return ctwl!(Expr::If(Box::new(expr), program), peek);
        }
        if let Token::For() = peek.token.clone() {
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

            let startval = ctwl!(
                {
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
                },
                peek
            );

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
            let endval = ctwl!(
                {
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
                },
                peek
            );

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
            let mut program: Vec<ExprWL> = vec![];
            loop {
                if let Token::End() = key.token {
                    _ = eat_token!(self);
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
            return ctwl!(
                Expr::For(varname, Box::new(startval), Box::new(endval), program),
                peek
            );
        }
        if let Token::While() = peek.token.clone() {
            _ = eat_token!(self);

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
            let mut program: Vec<ExprWL> = vec![];
            loop {
                if let Token::End() = key.token {
                    _ = eat_token!(self);
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
            return ctwl!(Expr::While(Box::new(expr), program), peek);
        }
        if let Token::Return() = peek.token.clone() {
            _ = eat_token!(self);
            let expr = self.parse_expression();
            return ctwl!(Expr::Return(Box::new(expr)), peek);
        }
        return self.equality();
    }
    pub fn parse_program(&mut self) -> ExprWL {
        let mut l: Vec<ExprWL> = vec![];
        while self.tokens.len() > 1 {
            let expr = self.parse_expression();
            l.push(expr);
        }
        return ctwl!(Expr::Program(l), self.tokens[0]);
    }
    // MATH
    fn equality(&mut self) -> ExprWL {
        let mut expr = self.comparison();
        loop {
            match peek_token!(self).token {
                Token::OperatorEquals() => {
                    let _tmp = eat_token!(self);
                    let right = self.comparison();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Equal,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                Token::OperatorNotEquals() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.comparison();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::NotEqual,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr.clone()
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn comparison(&mut self) -> ExprWL {
        let mut expr = self.term();
        loop {
            match peek_token!(self).token {
                Token::OperatorGreater() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Greater,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                Token::OperatorLesser() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Lesser,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                Token::OperatorLesserEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Lesser,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                Token::OperatorGreaterEqual() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.term();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Lesser,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn term(&mut self) -> ExprWL {
        let mut expr = self.factor();
        loop {
            match peek_token!(self).token {
                Token::OperatorSubtract() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.factor();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Subtract,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr
                    );
                }
                Token::OperatorAdd() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.factor();
                    expr = ctwl!(
                        Expr::Binary(BinaryOperator::Add, Box::new(expr.clone()), Box::new(right)),
                        expr
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn factor(&mut self) -> ExprWL {
        let mut expr = self.unary();
        loop {
            match peek_token!(self).token {
                Token::OperatorMultiply() => {
                    let _temp = eat_token!(self).token.clone();
                    let right = self.unary();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Multiply,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr.clone()
                    );
                }
                Token::OperatorDivide() => {
                    _ = eat_token!(self).token.clone();
                    let right = self.unary();
                    expr = ctwl!(
                        Expr::Binary(
                            BinaryOperator::Divide,
                            Box::new(expr.clone()),
                            Box::new(right)
                        ),
                        expr.clone()
                    );
                }
                _ => break,
            }
        }
        return expr;
    }
    fn unary(&mut self) -> ExprWL {
        match peek_token!(self).token {
            Token::OperatorSubtract() => {
                _ = eat_token!(self).token.clone();
                let right = self.primary();
                return ctwl!(
                    Expr::Unary(UnaryOperator::Negative, Box::new(right.clone())),
                    right.clone()
                );
            }
            Token::OperatorLogicalNot() => {
                _ = eat_token!(self).token.clone();
                let right = self.primary();
                return ctwl!(
                    Expr::Unary(UnaryOperator::LogicalNot, Box::new(right.clone())),
                    right.clone()
                );
            }
            _ => {}
        }
        return self.primary();
    }
    fn primary(&mut self) -> ExprWL {
        let p = eat_token!(self);

        if let Token::StringLiteral(str) = &p.token {
            return ctwl!(Expr::Literal(LiteralType::String, str.to_string()), p);
        }
        if let Token::NumericLiteral(num, _) = &p.token {
            return ctwl!(Expr::Literal(LiteralType::Number, num.to_string()), p);
        }
        if let Token::BooleanLiteral(b) = &p.token {
            return if b.clone() {
                ctwl!(Expr::Literal(LiteralType::Boolean, "1".to_string()), p)
            } else {
                ctwl!(Expr::Literal(LiteralType::Boolean, "0".to_string()), p)
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
                let mut program: Vec<ExprWL> = vec![];
                loop {
                    if let Token::End() = key.token {
                        _ = eat_token!(self);
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
                return ctwl!(Expr::Proc(n, args, program), p);
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
                _ = eat_token!(self);
                let mut arguments: Vec<ExprWL> = vec![];
                loop {
                    if let Token::CloseParen() = peek_token!(self).token {
                        _ = eat_token!(self);
                        break;
                    }
                    let expr = self.parse_expression();
                    arguments.push(expr);
                    if let Token::Comma() = peek_token!(self).token {
                        _ = eat_token!(self);
                        continue;
                    }
                }
                return ctwl!(Expr::Call(parts.clone(), arguments), p);
            } else if let Token::OperatorSet() = peek_token!(self).token {
                // setting variable
                _ = eat_token!(self);
                let expr = self.parse_expression();
                return ctwl!(Expr::VariableSet(parts.clone(), Box::new(expr)), p);
            } else {
                return ctwl!(Expr::Identifier(parts.to_vec()), p);
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
                return ctwl!(Expr::Group(Box::new(expr)), p);
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
