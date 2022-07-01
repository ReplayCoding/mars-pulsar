use {
    crate::{
        builtins,
        lexer::Token,
        parser::{Expr, Operator},
    },
    std::{
        collections::HashMap,
        fmt::{Display, Formatter},
        ops::Deref,
    },
};

pub struct Interpreter<'a> {
    pub state: State<'a>,
    pub exprs: Vec<Expr>,
}

pub struct State<'a> {
    pub toplevel_scope: HashMap<String, &'a Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    String(String),
    Bool(bool),
    Fn(FnType),
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Int,    /* (i64) */
    String, /* (String) */
    Bool,   /* (bool) */
    Fn,     /* (FnType) */
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnType {
    Builtin(BuiltinFn),
    User(UserFn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuiltinFn {
    pub name: String,
    pub return_type: Box<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserFn {
    pub name: String,
    pub args: HashMap<(usize, String), ValueType>,
    pub body: Vec<Expr>,
    pub return_type: ValueType,
}

impl Display for FnType {
    fn fmt(&self, _f: &mut Formatter) -> std::fmt::Result {
        match self {
            FnType::Builtin(_f) => Ok(()),
            FnType::User(_f) => Ok(()),
        }
    }
}

impl Display for BuiltinFn {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for UserFn {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Fn(_f) => Ok(()),
            Value::Nothing => write!(f, "Nothing"),
        }
    }
}

fn call_fn(name: &str, passed_args: Vec<&Value>, scope: &mut HashMap<String, &Value>) -> Value {
    match scope.get(name) {
        Some(key) => match key {
            Value::Fn(FnType::Builtin(BuiltinFn {
                name, return_type, ..
            })) => builtins::call_builtin(name, passed_args, return_type.deref().to_owned()),
            Value::Fn(FnType::User(UserFn {
                name,
                args,
                body,
                return_type,
                ..
            })) => {
                let mut new_scope = scope.clone();
                for ((index, name), _) in args {
                    new_scope.insert(name.clone(), passed_args[*index]);
                };
                Value::Nothing
            }
            _ => {
                panic!("Not a function")
            }
        },
        _ => panic!("Undefined function: {}", name),
    }
}

fn get_valuetype_from(name: &str) -> ValueType {
    match name {
        "bool" => ValueType::Bool,
        "int" => ValueType::Int,
        "string" => ValueType::String,
        "_none" => ValueType::Nothing,
        _ => panic!("Invalid type name: {}", name),
    }
}

fn interpret_expr(expr: &Expr, scope: &mut HashMap<String, &Value>) -> Value {
    match expr {
        Expr::BinaryExpr {
            op: Operator::SetVal,
            lhs,
            rhs,
        } => {
            let right_side = interpret_expr(rhs, scope);
            scope.insert(lhs.to_string(), &right_side);
            Value::Nothing
        }
        Expr::BinaryExpr {
            op: Operator::Add,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                (Value::String(left), Value::String(right)) => Value::String(left + &right),
                _ => panic!("Cannot add non-numeric values"),
            }
        }
        Expr::BinaryExpr {
            op: Operator::Sub,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
                _ => panic!("Cannot subtract non-numeric values"),
            }
        }
        Expr::BinaryExpr {
            op: Operator::Mul,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
                _ => panic!("Cannot multiply non-numeric values"),
            }
        }
        Expr::BinaryExpr {
            op: Operator::Div,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
                _ => panic!("Cannot divide non-numeric values"),
            }
        }
        Expr::BinaryExpr {
            op: Operator::Eq,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left == right),
                (Value::String(left), Value::String(right)) => Value::Bool(left == right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool(left == right),
                _ => panic!("Cannot compare non-numeric values"),
            }
        }
        Expr::BinaryExpr {
            op: Operator::Neq,
            lhs,
            rhs,
        } => {
            let left_side = interpret_expr(lhs, scope);
            let right_side = interpret_expr(rhs, scope);
            match (left_side, right_side) {
                (Value::Int(left), Value::Int(right)) => Value::Bool(left != right),
                (Value::String(left), Value::String(right)) => Value::Bool(left != right),
                (Value::Bool(left), Value::Bool(right)) => Value::Bool(left != right),
                _ => panic!("Cannot compare non-numeric values"),
            }
        }
        Expr::Token(x) => match x {
            Token::Num(x) => Value::Int(*x),
            Token::String(x) => Value::String(x.to_string()),
            Token::Bool(x) => Value::Bool(*x),
            Token::Identifier(x) => {
                if let Some(val) = scope.get(x) {
                    **val
                } else {
                    panic!("Undefined variable: {}", x)
                }
            }
            _ => Value::Nothing,
        },
        Expr::FnCall { name, args } => {
            let mut args_vec = Vec::new();
            for arg in args {
                args_vec.push(&interpret_expr(arg, scope));
            }
            call_fn(name, args_vec, scope)
        }
        Expr::FnDef {
            name,
            args,
            body,
            return_type,
        } => {
            let funcdef = Value::Fn(FnType::User(UserFn {
                // FIXME: Is there a better way to do this?
                name: name.clone(),
                args: args
                    .into_iter()
                    .map(|((i, n), v)| {
                        if let Expr::Token(Token::Type(name)) = v {
                            ((*i, n.clone()), get_valuetype_from(name))
                        } else {
                            unreachable!("This should always a be a type token")
                        }
                    })
                    .collect(),
                body: body.clone(),
                return_type: get_valuetype_from(return_type),
            }));
            scope.insert(name.clone(), &funcdef);
            Value::Nothing
        }
        Expr::Return { .. } => todo!(),
    }
}
impl Interpreter {
    pub fn new(exprs: Vec<Expr>) -> Self {
        let mut toplevel_scope = HashMap::new();
        builtins::make_builtins(&mut toplevel_scope);
        Self {
            state: State {
                toplevel_scope: toplevel_scope,
            },
            exprs,
        }
    }

    pub fn run(&mut self) {
        for expr in &self.exprs {
            interpret_expr(expr, &mut self.state.toplevel_scope);
        }
    }
}
