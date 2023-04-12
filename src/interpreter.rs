use std::collections::HashMap;

use crate::{
    lexer::Lexer,
    parser::{Expr, LiteralType, Parser},
};

fn unbox<T>(value: Box<T>) -> T {
    *value
}

pub struct Interpreter {}
impl Interpreter {
    pub fn run_code(
        &mut self,
        code: Vec<Expr>,
        mut variables: HashMap<Vec<String>, Expr>,
        namespace: Vec<String>,
    ) -> HashMap<Vec<String>, Expr> {
        for expr in code {
            if let Expr::Import(rel, path) = expr.clone() {
                if rel {
                    // TODO: do relative imports
                } else {
                    let raw = match path.as_str() {
                        "std" => include_str!("lib/std.bs"),
                        "math" => include_str!("lib/math.bs"),
                        _ => panic!("TODO: add better error reporting | invalid std import"),
                    }
                    .to_string()
                    .replace("\r", &"");
                    let lexer = Lexer {
                        text: raw,
                        filename: String::from("std/".to_string() + &path + &".bs".to_string()),
                    };
                    let tokens = lexer.lex();
                    let mut parser = Parser { tokens };
                    let program = parser.parse_program();
                    let mut inter = Interpreter::new();
                    let vars = inter.run_program(program, false, namespace.clone());
                    for (k, v) in vars {
                        if variables.contains_key(&k) {
                            panic!("TODO: add better error reporting | variable already defined")
                        }
                        let new_namespace = namespace.clone();
                        let new_namespace = new_namespace.iter().chain(&k).map(|f| f.clone());
                        let new_namespace: Vec<String> = new_namespace.collect();
                        variables.insert(new_namespace, v);
                    }
                }
            } else if let Expr::Proc(name, _args, _prog) = expr.clone() {
                if variables.contains_key(&name) {
                    panic!("TODO: add better error reporting | variable already defined")
                } else {
                    let new_namespace = namespace.clone();
                    let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                    let new_namespace: Vec<String> = new_namespace.collect();
                    variables.insert(new_namespace, expr);
                }
            } else if let Expr::Namespace(name, space) = expr.clone() {
                let new_namespace = namespace.clone();
                let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                let new_namespace: Vec<String> = new_namespace.collect();
                let def = self.run_code(space, variables.clone(), new_namespace.clone());
                for (k, v) in def {
                    if k.starts_with(new_namespace.clone().as_slice()) {
                        variables.insert(k.clone(), v);
                    }
                }
            } else if let Expr::Call(name, args) = expr.clone() {
                if name.starts_with(vec!["builtin".to_string()].as_slice()) {
                    let subcmd = name.iter().nth(1);
                    match subcmd {
                        None => panic!("No subcommand for 'builtin' namespace"),
                        Some(v) => match v.as_str() {
                            "printval" => {
                                assert_eq!(args.len(), 1, "Argument length required to be one: usage `builtin.printstr($val)`");

                                fn func(y: Expr, variables: HashMap<Vec<String>, Expr>) {
                                    match y {
                                        Expr::Literal(_ty, v) => {
                                            print!("{}", v);
                                        }
                                        Expr::Identifier(name) => {
                                            let var = variables.get(&name.clone());
                                            match var {
                                                None => panic!(
                                                    "Variable \"{}\" doesn't exist",
                                                    name.join(".")
                                                ),
                                                Some(v) => func(v.clone(), variables.clone()),
                                            }
                                        }
                                        _ => {
                                            panic!("Invalid argument")
                                        }
                                    }
                                }

                                func(args[0].clone(), variables.clone());
                            }
                            _ => panic!("No subcommand called '{}' in builtin functions", v),
                        },
                    }
                } else {
                    match variables.get(&name) {
                        Some(v) => {
                            if let Expr::Proc(_name, arg, prog) = v {
                                self.run_proc(
                                    args.clone(),
                                    v.clone(),
                                    variables.clone(),
                                    namespace.clone(),
                                );
                            } else {
                                panic!("{} is not a proc", name.join("."))
                            }
                        }
                        None => panic!("{} is not defined", name.join(".")),
                    }
                }
            } else if let Expr::VariableDeclaration(name, expr) = expr.clone() {
                if !variables.contains_key(&name) {
                    variables.insert(name, unbox(expr));
                } else {
                    panic!(
                        "Variable named \"{}\" has already been defined",
                        name.join(".")
                    );
                }
            } else if let Expr::VariableSet(name, expr) = expr.clone() {
                // setting a variable
                if variables.contains_key(&name) {
                    variables.insert(name, unbox(expr));
                } else {
                    panic!("Variable named \"{}\" isn't defined", name.join("."));
                }
            } else {
                println!("Unknown instruction: {:?}", expr);
            }
        }

        return variables.clone();
    }

    pub fn run_proc(
        &mut self,
        args: Vec<Expr>,
        func: Expr,
        mut scope: HashMap<Vec<String>, Expr>,
        namespace: Vec<String>,
    ) {
        if let Expr::Proc(_name, pargs, prog) = func {
            for (i, arg) in pargs.iter().enumerate() {
                scope.insert(vec![arg.to_string()], args[i].clone());
            }
            self.run_code(prog, scope, namespace.clone());
        } else {
            panic!("`run_proc` method didn't receive proc expression")
        }
    }

    pub fn run_program(
        &mut self,
        code: Expr,
        main: bool,
        namespace: Vec<String>,
    ) -> HashMap<Vec<String>, Expr> {
        if let Expr::Program(p) = code {
            let ret = self.run_code(p, HashMap::new(), namespace.clone());
            if main {
                let main = ret
                    .get(&vec!["main".to_string()])
                    .expect("Expected main function");
                self.run_proc(vec![], main.clone(), ret.clone(), namespace.clone())
            }
            return ret;
        } else {
            panic!("Root expression is not a program")
        }
    }

    pub fn new() -> Self {
        Interpreter {}
    }
}
