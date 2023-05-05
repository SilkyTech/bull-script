use std::collections::HashMap;

use crate::chainmap::NamedChainMap;
use crate::error::error_at;
use crate::lexer::Lexer;
use crate::parser::{Expr, ExprWL, Parser};

pub struct Interpreter {
    stack: NamedChainMap<Vec<String>, ExprWL>,
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

macro_rules! error {
    ($loc: expr, $reason: expr) => {
        error_at(&$loc.filen, &$loc.linen, &$loc.charn, &$reason)
    };
}
macro_rules! vec_append {
    ($vec: expr, $add: expr) => {{
        let mut p = $vec.clone();
        p.append(&mut $add);
        p
    }};
}

enum InterpreterEat {
    AddEat,
    AddNo,
    No,
}

impl Interpreter {
    fn run_code(
        &mut self,
        code: Vec<ExprWL>,
        scope_name: String,
        namespace: Vec<String>,
        eat: InterpreterEat,
    ) {
        if let InterpreterEat::No = eat {
        } else {
            self.stack.push_hash(scope_name);
        }
        for ex in code {
            let expr = ex.clone().expr;

            if let Expr::VariableDeclaration(name, value) = expr.clone() {
                self.stack.insert_top(name.clone(), unbox(value));
            } else if let Expr::Proc(mut name, _args, _prog) = expr.clone() {
                let new_name = vec_append!(namespace, name);
                self.stack.insert_top(new_name.clone(), ex.clone());
                dbg!(new_name.clone());
            } else if let Expr::Import(relative, path) = expr.clone() {
                if relative {
                    error!(ex, format!("Relative imports are not implemented yet"));
                } else {
                    let raw = match path.as_str() {
                        "std" => include_str!("lib/std.bs"),
                        "math" => include_str!("lib/math.bs"),
                        _ => error!(ex, format!("{} is not a valid std import!", path)),
                    }
                    .replace("\r", "");
                    let filename = "lib/".to_string() + &path + &".bs";
                    let lexer = Lexer {
                        filename: filename.clone(),
                        text: raw,
                    };
                    let lexed = lexer.lex();
                    let mut parser = Parser { tokens: lexed };
                    let parsed = parser.parse_program();
                    self.run_program(parsed, filename);
                    //self.stack.move_to_back();
                }
            } else if let Expr::Namespace(nmspc, prog) = expr.clone() {
                self.run_code(
                    prog.clone(),
                    "namespace: ".to_string() + &nmspc.clone().join("."),
                    vec_append!(namespace.clone(), nmspc.clone()),
                    InterpreterEat::No,
                )
            } else {
                eprintln!("{:?} has not been implemented yet!", expr);
            }
        }
        //dbg!(self.stack.top());
        if let InterpreterEat::AddNo | InterpreterEat::No = eat {
        } else {
            self.stack.pop_hash();
        };
    }

    pub fn run_proc(&mut self, main: ExprWL) {
        panic!()
    }

    pub fn run_program(&mut self, prog: ExprWL, filename: String) {
        let expr = prog.clone().expr;

        if let Expr::Program(prog) = expr.clone() {
            self.run_code(prog, filename.clone(), vec![], InterpreterEat::AddNo);
        }
        dbg!(self.stack.clone());
        self.run_proc(self.stack.find(vec!["main".to_string()]).unwrap());
    }

    pub fn new() -> Self {
        Self {
            stack: NamedChainMap::new(),
        }
    }
}
