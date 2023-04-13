use std::collections::HashMap;

use crate::{
    lexer::Lexer,
    parser::{BinaryOperator, Expr, LiteralType, Parser, UnaryOperator},
};

fn unbox<T>(value: Box<T>) -> T {
    *value
}
fn resolve_math(val: Expr, variables: HashMap<Vec<String>, Expr>) -> (LiteralType, String) {
    fn to_num(v: (LiteralType, String)) -> f64 {
        if let LiteralType::Number = v.0 {
            return v.1.parse::<f64>().unwrap();
        } else if let LiteralType::Boolean = v.0 {
            return v.1.parse::<f64>().unwrap();
        } else {
            panic!("")
        }
    }
    if let Expr::Binary(op, left, right) = val {
        match op {
            BinaryOperator::Add => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        + to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Subtract => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        - to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Multiply => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        * to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Divide => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        / to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Mod => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        % to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Lesser => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        < to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::LesserEqual => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        <= to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Greater => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        > to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::GreaterEqual => {
                return (
                    LiteralType::Number,
                    (to_num(resolve_variable(unbox(left), variables.clone()))
                        >= to_num(resolve_variable(unbox(right), variables.clone())))
                    .to_string(),
                )
            }
            BinaryOperator::Equal => {
                return (
                    LiteralType::Boolean,
                    (resolve_variable(unbox(left), variables.clone())
                        == resolve_variable(unbox(right), variables.clone()))
                    .to_string(),
                )
            }
            BinaryOperator::NotEqual => {
                return (
                    LiteralType::Boolean,
                    (resolve_variable(unbox(left), variables.clone())
                        != resolve_variable(unbox(right), variables.clone()))
                    .to_string(),
                )
            }
        }
    } else if let Expr::Unary(op, right) = val {
        match op {
            UnaryOperator::Negative => {
                return (
                    LiteralType::Number,
                    (-to_num(resolve_variable(unbox(right), variables.clone()))).to_string(),
                )
            }
            UnaryOperator::LogicalNot => {
                return (
                    LiteralType::Boolean,
                    (!(to_num(resolve_variable(unbox(right), variables.clone())).round() as i32))
                        .to_string(),
                )
            }
        }
    } else {
        return resolve_variable(val.clone(), variables.clone());
    }
}

fn resolve_variable(y: Expr, variables: HashMap<Vec<String>, Expr>) -> (LiteralType, String) {
    match y {
        Expr::Literal(_ty, v) => return (_ty, v),
        Expr::Unary(..) | Expr::Binary(..) => return resolve_math(y, variables.clone()),
        Expr::Identifier(name) => {
            let var = variables.get(&name.clone());
            match var {
                None => panic!("Variable \"{}\" doesn't exist", name.join(".")),
                Some(v) => resolve_variable(v.clone(), variables.clone()),
            }
        }
        _ => {
            panic!("Invalid argument")
        }
    }
}

macro_rules! insert_variables_from_run_proc {
    ($vars: ident, $insert: ident) => {{
        for (k, _) in $insert.clone() {
            let val = $vars.get(&k).unwrap();
            $insert.insert(k, val.clone());
        }
    }};
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

                                let res = resolve_variable(args[0].clone(), variables.clone());
                                print!("{}", res.1);
                            }
                            "debug" => {
                                assert_eq!(args.len(), 1, "Argument length required to be one: usage `builtin.debug($expr)`");
                                println!("{:?}", args[0].clone());
                            }
                            "debugval" => {
                                assert_eq!(args.len(), 1, "Argument length required to be one: usage `builtin.debug($expr)`");
                                println!(
                                    "{:?}",
                                    resolve_variable(args[0].clone(), variables.clone())
                                );
                            }
                            _ => panic!("No subcommand called '{}' in builtin functions", v),
                        },
                    }
                } else {
                    match variables.get(&name) {
                        Some(v) => {
                            if let Expr::Proc(_name, _arg, _prog) = v {
                                let vars = self.run_proc(
                                    args.clone(),
                                    v.clone(),
                                    variables.clone(),
                                    namespace.clone(),
                                );

                                insert_variables_from_run_proc!(vars, variables);
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
            } else if let Expr::ConstantDeclaration(name, expr) = expr.clone() {
                if !variables.contains_key(&name) {
                    let new_namespace = namespace.clone();
                    let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                    let new_namespace: Vec<String> = new_namespace.collect();
                    variables.insert(new_namespace, unbox(expr));
                } else {
                    panic!(
                        "Variable named \"{}\" has already been defined",
                        name.join(".")
                    );
                }
            } else if let Expr::VariableSet(name, expr) = expr.clone() {
                // setting a variable
                if variables.contains_key(&name) {
                    let newval = resolve_variable(unbox(expr), variables.clone());
                    let newval = Expr::Literal(newval.0, newval.1);
                    variables.insert(name, newval);
                } else {
                    panic!("Variable named \"{}\" isn't defined", name.join("."));
                }
            } else if let Expr::For(name, start, end, prog) = expr.clone() {
                let start = resolve_variable(unbox(start), variables.clone());
                let end = resolve_variable(unbox(end), variables.clone());
                if let LiteralType::Number = start.0 {
                    if let LiteralType::Number = end.0 {
                        let start = start.1.parse::<i32>().unwrap();
                        let end = end.1.parse::<i32>().unwrap();
                        for i in start..end {
                            let mut new_vars = variables.clone();
                            new_vars.insert(
                                name.clone(),
                                Expr::Literal(LiteralType::Number, i.to_string()),
                            );
                            let res =
                                self.run_code(prog.clone(), new_vars.clone(), namespace.clone());
                            insert_variables_from_run_proc!(res, variables);
                        }
                    }
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
    ) -> HashMap<Vec<String>, Expr> {
        if let Expr::Proc(_name, pargs, prog) = func {
            for (i, arg) in pargs.iter().enumerate() {
                scope.insert(vec![arg.to_string()], args[i].clone());
            }
            return self.run_code(prog, scope, namespace.clone());
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
                let _res = self.run_proc(vec![], main.clone(), ret.clone(), namespace.clone());
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
