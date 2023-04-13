use std::collections::HashMap;

use crate::{
    ctwl,
    error::error_at,
    lexer::Lexer,
    parser::{BinaryOperator, Expr, ExprWL, LiteralType, Parser, UnaryOperator},
};

fn unbox<T>(value: Box<T>) -> T {
    *value
}

macro_rules! insert_variables_from_run_proc {
    ($vars: ident, $insert: ident) => {{
        for (k, _) in $insert.clone() {
            let val = $vars.get(&k).unwrap();
            $insert.insert(k, val.clone());
        }
    }};
}
macro_rules! qerror {
    ($p: expr, $f: expr) => {
        error_at(
            &$p.filen.clone(),
            &$p.linen.clone(),
            &$p.charn.clone(),
            &$f.to_string(),
        )
    };
}

pub struct Interpreter {}
impl Interpreter {
    pub fn run_proc_name(
        &mut self,
        exprwl: ExprWL,
        mut variables: HashMap<Vec<String>, ExprWL>,
        namespace: Vec<String>,
    ) -> (ExprWL, HashMap<Vec<String>, ExprWL>) {
        let expr = exprwl.clone().expr.clone();
        if let Expr::Call(name, args) = expr {
            if name.starts_with(vec!["builtin".to_string()].as_slice()) {
                let subcmd = name.iter().nth(1);
                match subcmd {
                    None => qerror!(exprwl, &"No subcommand for 'builtin' namespace"),
                    Some(v) => match v.as_str() {
                        "printval" => {
                            assert_eq!(args.len(), 1, "Argument length required to be one: usage `builtin.printstr($val)`");

                            let res = self.resolve_variable(
                                args[0].clone(),
                                variables.clone(),
                                namespace.clone(),
                            );
                            print!("{}", res.1);
                        }
                        "debug" => {
                            assert_eq!(
                                args.len(),
                                1,
                                "Argument length required to be one: usage `builtin.debug($expr)`"
                            );
                            println!("{:?}", args[0].clone());
                        }
                        "debugval" => {
                            assert_eq!(
                                args.len(),
                                1,
                                "Argument length required to be one: usage `builtin.debug($expr)`"
                            );
                            println!(
                                "{:?}",
                                self.resolve_variable(
                                    args[0].clone(),
                                    variables.clone(),
                                    namespace.clone()
                                )
                            );
                        }
                        _ => qerror!(
                            exprwl,
                            format!("No subcommand called '{}' in builtin functions", v)
                        ),
                    },
                }
                return (
                    ctwl!(Expr::Literal(LiteralType::Null, "".to_string()), exprwl),
                    variables.clone(),
                );
            } else {
                match variables.get(&name) {
                    Some(v) => {
                        if let Expr::Proc(_name, _arg, _prog) = &v.expr {
                            let vars = self.run_proc(
                                args.clone(),
                                v.clone(),
                                variables.clone(),
                                namespace.clone(),
                            );
                            let hash = vars.1;

                            insert_variables_from_run_proc!(hash, variables);

                            return (vars.0.clone(), variables.clone());
                        } else {
                            qerror!(exprwl, format!("{} is not a proc", name.join(".")))
                        }
                    }
                    None => qerror!(exprwl, format!("{} is not defined", name.join("."))),
                }
            }
        } else {
            panic!();
        }
    }

    pub fn run_code(
        &mut self,
        code: Vec<ExprWL>,
        mut variables: HashMap<Vec<String>, ExprWL>,
        namespace: Vec<String>,
    ) -> (ExprWL, HashMap<Vec<String>, ExprWL>) {
        for exprwl in code {
            let expr = exprwl.clone().expr;
            if let Expr::Import(rel, path) = expr.clone() {
                if rel {
                    // TODO: do relative imports
                } else {
                    let raw = match path.as_str() {
                        "std" => include_str!("lib/std.bs"),
                        "math" => include_str!("lib/math.bs"),
                        _ => qerror!(exprwl, format!("Invaild std import!")),
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
                            qerror!(exprwl, format!("Variable already defined"));
                        }
                        let new_namespace = namespace.clone();
                        let new_namespace = new_namespace.iter().chain(&k).map(|f| f.clone());
                        let new_namespace: Vec<String> = new_namespace.collect();
                        variables.insert(new_namespace, v);
                    }
                }
            } else if let Expr::Proc(name, _args, _prog) = expr.clone() {
                if variables.contains_key(&name) {
                    qerror!(exprwl, format!("Variable already defined"));
                } else {
                    let new_namespace = namespace.clone();
                    let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                    let new_namespace: Vec<String> = new_namespace.collect();
                    variables.insert(new_namespace, exprwl);
                }
            } else if let Expr::Namespace(name, space) = expr.clone() {
                let new_namespace = namespace.clone();
                let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                let new_namespace: Vec<String> = new_namespace.collect();
                let def = self.run_code(space, variables.clone(), new_namespace.clone());
                for (k, v) in def.1 {
                    if k.starts_with(new_namespace.clone().as_slice()) {
                        variables.insert(k.clone(), v);
                    }
                }
            } else if let Expr::Call(_name, _args) = expr.clone() {
                self.run_proc_name(exprwl, variables.clone(), namespace.clone());
            } else if let Expr::VariableDeclaration(name, expr) = expr.clone() {
                if !variables.contains_key(&name) {
                    variables.insert(name, unbox(expr));
                } else {
                    qerror!(
                        exprwl,
                        format!(
                            "Variable named \"{}\" has already been defined",
                            name.join(".")
                        )
                    );
                }
            } else if let Expr::ConstantDeclaration(name, expr) = expr.clone() {
                if !variables.contains_key(&name) {
                    let new_namespace = namespace.clone();
                    let new_namespace = new_namespace.iter().chain(&name).map(|f| f.clone());
                    let new_namespace: Vec<String> = new_namespace.collect();
                    variables.insert(new_namespace, unbox(expr));
                } else {
                    qerror!(
                        exprwl,
                        format!(
                            "Variable named \"{}\" has already been defined",
                            name.join(".")
                        )
                    );
                }
            } else if let Expr::VariableSet(name, expr) = expr.clone() {
                // setting a variable
                if variables.contains_key(&name) {
                    let newval =
                        self.resolve_variable(unbox(expr), variables.clone(), namespace.clone());
                    let newval = ctwl!(Expr::Literal(newval.0, newval.1), exprwl);
                    variables.insert(name, newval);
                } else {
                    qerror!(
                        exprwl,
                        format!("Variable named \"{}\" isn't defined", name.join("."))
                    );
                }
            } else if let Expr::For(name, start, end, prog) = expr.clone() {
                let start =
                    self.resolve_variable(unbox(start), variables.clone(), namespace.clone());
                let end = self.resolve_variable(unbox(end), variables.clone(), namespace.clone());
                if let LiteralType::Number = start.0 {
                    if let LiteralType::Number = end.0 {
                        let start = start.1.parse::<i32>().unwrap();
                        let end = end.1.parse::<i32>().unwrap();
                        for i in start..end {
                            let mut new_vars = variables.clone();
                            new_vars.insert(
                                name.clone(),
                                ctwl!(
                                    Expr::Literal(LiteralType::Number, i.to_string()),
                                    exprwl.clone()
                                ),
                            );
                            let res =
                                self.run_code(prog.clone(), new_vars.clone(), namespace.clone());

                            let hash = res.1;
                            insert_variables_from_run_proc!(hash, variables);
                        }
                    }
                }
            } else if let Expr::Return(expr) = expr.clone() {
                return (unbox(expr), variables.clone());
            } else {
                println!("Unknown instruction: {:?}", expr);
            }
        }

        return (
            ExprWL {
                expr: (Expr::Literal(LiteralType::Null, "".to_string())),
                linen: 0,
                charn: 0,
                filen: "".to_string(),
            },
            variables.clone(),
        );
    }

    pub fn resolve_math(
        &mut self,
        val: ExprWL,
        variables: HashMap<Vec<String>, ExprWL>,
        namespace: Vec<String>,
    ) -> (LiteralType, String) {
        fn to_num(v: (LiteralType, String)) -> f64 {
            if let LiteralType::Number = v.0 {
                return v.1.parse::<f64>().unwrap();
            } else if let LiteralType::Boolean = v.0 {
                return v.1.parse::<f64>().unwrap();
            } else {
                panic!("resolve_math didn't get number or boolean")
            }
        }
        if let Expr::Binary(op, left, right) = val.expr {
            match op {
                BinaryOperator::Add => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) + to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Subtract => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) - to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Multiply => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) * to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Divide => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) / to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Mod => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) % to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Lesser => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) < to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::LesserEqual => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) <= to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Greater => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) > to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::GreaterEqual => {
                    return (
                        LiteralType::Number,
                        (to_num(self.resolve_variable(
                            unbox(left),
                            variables.clone(),
                            namespace.clone(),
                        )) >= to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                BinaryOperator::Equal => {
                    return (
                        LiteralType::Boolean,
                        (self.resolve_variable(unbox(left), variables.clone(), namespace.clone())
                            == self.resolve_variable(
                                unbox(right),
                                variables.clone(),
                                namespace.clone(),
                            ))
                        .to_string(),
                    )
                }
                BinaryOperator::NotEqual => {
                    return (
                        LiteralType::Boolean,
                        (self.resolve_variable(unbox(left), variables.clone(), namespace.clone())
                            != self.resolve_variable(
                                unbox(right),
                                variables.clone(),
                                namespace.clone(),
                            ))
                        .to_string(),
                    )
                }
            }
        } else if let Expr::Unary(op, right) = val.expr {
            match op {
                UnaryOperator::Negative => {
                    return (
                        LiteralType::Number,
                        (-to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        )))
                        .to_string(),
                    )
                }
                UnaryOperator::LogicalNot => {
                    return (
                        LiteralType::Boolean,
                        (!(to_num(self.resolve_variable(
                            unbox(right),
                            variables.clone(),
                            namespace.clone(),
                        ))
                        .round() as i32))
                            .to_string(),
                    )
                }
            }
        } else {
            return self.resolve_variable(val.clone(), variables.clone(), namespace.clone());
        }
    }

    pub fn resolve_variable(
        &mut self,
        y: ExprWL,
        variables: HashMap<Vec<String>, ExprWL>,
        namespace: Vec<String>,
    ) -> (LiteralType, String) {
        match y.clone().expr {
            Expr::Literal(_ty, v) => return (_ty, v),
            Expr::Unary(..) | Expr::Binary(..) => {
                return self.resolve_math(y, variables.clone(), namespace.clone())
            }
            Expr::Identifier(name) => {
                let var = variables.get(&name.clone());
                match var {
                    None => qerror!(
                        y,
                        format!("Variable named \"{}\" hasn't been defined", name.join("."))
                    ),
                    Some(v) => {
                        self.resolve_variable(v.clone(), variables.clone(), namespace.clone())
                    }
                }
            }
            Expr::Call(name, args) => {
                let res = self.run_proc_name(y.clone(), variables.clone(), namespace.clone());
                return self.resolve_variable(res.0.clone(), variables.clone(), namespace.clone());
            }
            Expr::Group(expr) => {
                return self.resolve_variable(unbox(expr), variables.clone(), namespace.clone())
            }
            _ => {
                panic!("Invalid argument")
            }
        }
    }

    pub fn run_proc(
        &mut self,
        args: Vec<ExprWL>,
        func: ExprWL,
        mut scope: HashMap<Vec<String>, ExprWL>,
        namespace: Vec<String>,
    ) -> (ExprWL, HashMap<Vec<String>, ExprWL>) {
        if let Expr::Proc(_name, pargs, prog) = func.expr {
            for (i, arg) in pargs.iter().enumerate() {
                scope.insert(vec![arg.to_string()], args[i].clone());
            }

            let res = self.run_code(prog, scope, namespace.clone());

            return res.clone();
        } else {
            panic!("`run_proc` method didn't receive proc expression")
        }
    }

    pub fn run_program(
        &mut self,
        code: ExprWL,
        main: bool,
        namespace: Vec<String>,
    ) -> HashMap<Vec<String>, ExprWL> {
        if let Expr::Program(p) = code.expr {
            let ret = self.run_code(p, HashMap::new(), namespace.clone());
            if main {
                let main = ret
                    .1
                    .get(&vec!["main".to_string()])
                    .expect("Expected main function");
                let _res = self.run_proc(vec![], main.clone(), ret.1.clone(), namespace.clone());
            }
            return ret.1;
        } else {
            panic!("Root expression is not a program")
        }
    }

    pub fn new() -> Self {
        Interpreter {}
    }
}
