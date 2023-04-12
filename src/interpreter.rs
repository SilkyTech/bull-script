use std::collections::HashMap;

use crate::{parser::{Expr, Parser}, lexer::Lexer};

pub struct Interpreter {
}
impl Interpreter {
    pub fn run_code(&mut self, code: Vec<Expr>, mut variables: HashMap<String, Expr>) -> HashMap<String, Expr> {
        for expr in code {
            if let Expr::Import(rel, path) = expr.clone() {
                if rel {
                    // TODO: do relative imports
                } else {
                    let raw = match path.as_str() {
                        "std" => include_str!("lib/std.bs"),
                        _ => panic!("TODO: add better error reporting | invalid std import")
                    }.to_string().replace("\r", &"");
                    let lexer = Lexer {
                        text: raw,
                        filename: String::from("std/".to_string() + &path + &".bs".to_string()),
                    };
                    let tokens = lexer.lex();
                    let mut parser = Parser { tokens };
                    let program = parser.parse_program();
                    let mut inter = Interpreter::new();
                    let vars = inter.run_program(program);
                    for (k, v) in vars {
                        if variables.contains_key(&k) {
                            panic!("TODO: add better error reporting | variable already defined")
                        }
                        variables.insert(k, v);
                    }
                    dbg!(variables.clone());
                }
            }
            else if let Expr::Proc(name, args, prog) = expr.clone() {
                let joinname = name.join(".");
                if variables.contains_key(&joinname) {
                    panic!("TODO: add better error reporting | variable already defined")
                } else {
                    variables.insert(joinname, expr);
                }
            } else {
                println!("Unknown instruction: {:?}", expr);
            }
        }

        return variables.clone();
    }

    pub fn run_program(&mut self, code: Expr) -> HashMap<String, Expr> {
        if let Expr::Program(p) = code {
            return self.run_code(p, HashMap::new());
        } else {
            panic!("Root expression is not a program")
        }
    }

    pub fn new() -> Self {
        Interpreter {  }
    }
}