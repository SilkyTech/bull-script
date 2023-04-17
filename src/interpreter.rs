use std::collections::HashMap;

use crate::chainmap::ChainMap;
use crate::error::error_at;
use crate::parser::{Expr, ExprWL};

pub struct Interpreter {
    stack: Stack,
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

macro_rules! error {
    ($loc: expr, $reason: expr) => {
        error_at(&$loc.filen, &$loc.linen, &$loc.charn, &$reason);
    };
}

pub(crate) type Stack = Vec<HashMap<Vec<String>, ExprWL>>;

impl Interpreter {
    fn run_code(&mut self, code: Vec<ExprWL>) {
        let mut scope = HashMap::new();
        self.stack.push(scope);
        for ex in code {
            let expr = ex.clone().expr;

            if let Expr::VariableDeclaration(name, value) = expr.clone() {
                scope.insert(name.clone(), unbox(value));
            } else if let Expr::Import(relative, path) = expr.clone() {
                if relative {
                    error!(ex, format!("Relative imports are not implemented yet"));
                } else {
                }
            } else {
                eprintln!("{:?} has not been implemented yet!", expr);
            }
        }
        self.stack.pop();
    }

    pub fn run_program(&mut self, prog: ExprWL) {
        let expr = prog.clone().expr;
    }

    pub fn new() -> Self {
        Self { stack: vec![] }
    }
}
