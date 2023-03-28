use crate::lexer::{Token, TWL};

//pub enum

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

impl Parser<'_> {
    pub fn parse_expression(&mut self) {
        let p = eat_token!(self);
        if let Token::Identifier(parts) = &p.token {}
    }
}
