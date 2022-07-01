use std::{collections::HashMap, fmt::Display, iter::Peekable, slice::Iter};

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    SetVal,
}

impl Operator {
    pub fn from_str(s: &str) -> Self {
        match s {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            "==" => Self::Eq,
            "!=" => Self::Neq,
            ":=" => Self::SetVal,
            _ => panic!("Unknown operator"),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Eq => write!(f, "="),
            Operator::Neq => write!(f, "!="),
            Operator::SetVal => write!(f, ":="),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Token(Token),
    BinaryExpr {
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    FnCall {
        name: String,
        args: Vec<Expr>,
    },
    FnDef {
        name: String,
        args: HashMap<(usize, String), Expr>,
        body: Vec<Expr>,
        return_type: String,
    },
    Return {
        inner: Box<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Token(t) => write!(f, "{}", t),
            Expr::BinaryExpr { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            Expr::FnCall { name, args } => write!(f, "{}({:?})", name, args),
            Expr::FnDef {
                name,
                args,
                body,
                return_type,
            } => {
                write!(
                    f,
                    "func {}({:?}) {{{:?}}} -> {:?}",
                    name, args, body, return_type
                )
            }
            Expr::Return { inner } => write!(f, "return {}", inner),
        }
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens }
    }

    pub fn parse(&self) -> Vec<Expr> {
        let mut exprs = Vec::new();
        let mut tokens = &mut self.tokens.iter().peekable();
        while {
            let this = &tokens.clone();
            this.len() != 0
        } {
            let (expr, tokens_new) = Self::parse_expr(tokens, true);
            tokens = tokens_new;
            exprs.push(expr);
        }
        exprs
    }

    pub fn parse_expr<'a>(
        tokens: &'a mut Peekable<Iter<'a, Token>>,
        mut sc_check: bool,
    ) -> (Expr, &'a mut Peekable<Iter<'a, Token>>) {
        let (expr, tokens_new) = match tokens.next() {
            Some(Token::Return) => {
                let (expr, tokens_new) = Self::parse_expr(tokens, false);
                (
                    Expr::Return {
                        inner: Box::new(expr),
                    },
                    tokens_new,
                )
            }
            Some(Token::Identifier(ident)) => match tokens.peek() {
                Some(Token::SetVal) => {
                    tokens.next();
                    let (expr, tokens_new) = Self::parse_expr(tokens, false);
                    (
                        Expr::BinaryExpr {
                            op: Operator::SetVal,
                            lhs: Box::new(Expr::Token(Token::Identifier(ident.into()))),
                            rhs: Box::new(expr),
                        },
                        tokens_new,
                    )
                }
                Some(Token::LParen) => {
                    tokens.next();
                    Self::parse_fn_call(ident.to_string(), tokens)
                }
                Some(Token::Operator(op)) => {
                    tokens.next();
                    let (expr, tokens_new) = Self::parse_expr(tokens, false);
                    (
                        Expr::BinaryExpr {
                            op: Operator::from_str(op),
                            lhs: Box::new(Expr::Token(Token::Identifier(ident.into()))),
                            rhs: Box::new(expr),
                        },
                        tokens_new,
                    )
                }
                _ => (Expr::Token(Token::Identifier(ident.into())), tokens),
            },
            Some(Token::Num(num)) => match tokens.peek() {
                Some(Token::Operator(op)) => {
                    tokens.next();
                    let (expr, tokens_new) = Self::parse_expr(tokens, false);
                    match op.as_str() {
                        "+" => (
                            Expr::BinaryExpr {
                                op: Operator::Add,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        "-" => (
                            Expr::BinaryExpr {
                                op: Operator::Sub,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        "*" => (
                            Expr::BinaryExpr {
                                op: Operator::Mul,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        "/" => (
                            Expr::BinaryExpr {
                                op: Operator::Div,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        "=" => (
                            Expr::BinaryExpr {
                                op: Operator::Eq,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        "!=" => (
                            Expr::BinaryExpr {
                                op: Operator::Neq,
                                lhs: Box::new(Expr::Token(Token::Num(*num))),
                                rhs: Box::new(expr),
                            },
                            tokens_new,
                        ),
                        _ => panic!("Expected operator"),
                    }
                }
                _ => (Expr::Token(Token::Num(*num)), tokens),
            },
            Some(Token::Bool(bool)) => (Expr::Token(Token::Bool(*bool)), tokens),
            Some(Token::String(string)) => (Expr::Token(Token::String(string.into())), tokens),
            Some(Token::Func) => {
                sc_check = false;
                Self::parse_fn_def(tokens)
            }
            _ => (Expr::Token(Token::Error), tokens),
        };
        if sc_check {
            if tokens_new.next() == Some(&Token::Semicolon) {
                (expr, tokens_new)
            } else {
                panic!("Expected semicolon");
            }
        } else {
            (expr, tokens_new)
        }
    }

    pub fn parse_fn_def<'a>(
        tokens: &'a mut Peekable<Iter<'a, Token>>,
    ) -> (Expr, &'a mut Peekable<Iter<'a, Token>>) {
        match tokens.next() {
            Some(Token::Identifier(ident)) => match tokens.next() {
                Some(Token::LParen) => {
                    let mut vals = HashMap::new();
                    let mut return_type: String = String::from("_none");
                    if tokens.peek() == Some(&&Token::RParen) {
                        tokens.next();
                    } else {
                        let mut idx = 0;
                        loop {
                            match tokens.peek() {
                                Some(Token::Type(t)) => {
                                    tokens.next();
                                    match tokens.peek() {
                                        Some(Token::Identifier(ident)) => {
                                            tokens.next();
                                            vals.insert(
                                                (idx, ident.to_string()),
                                                Expr::Token(Token::Type(t.clone())),
                                            );
                                        }
                                        _ => panic!("Expected identifier"),
                                    }
                                }
                                _ => panic!("Expected type"),
                            }
                            match tokens.next() {
                                Some(Token::Comma) => (),
                                Some(Token::RParen) => {
                                    break;
                                }
                                _ => panic!("Expected comma, or ')'"),
                            };
                            idx += 1;
                        }
                    }
                    let (body, tokens_new) = match tokens.next() {
                        Some(Token::ReturnType) => {
                            return_type = match tokens.next() {
                                Some(Token::Type(t)) => t.clone(),
                                _ => panic!("Expected type"),
                            };
                            match tokens.next() {
                                Some(Token::LBrace) => Self::handle_func_block(tokens),
                                _ => panic!("Expected brace"),
                            }
                        }
                        Some(Token::LBrace) => Self::handle_func_block(tokens),
                        _ => panic!("Expected a return statement or brace"),
                    };
                    return (
                        Expr::FnDef {
                            name: ident.into(),
                            args: vals,
                            body,
                            return_type,
                        },
                        tokens_new,
                    );
                }
                _ => panic!("Expected '(', got {:?}", tokens.peek()),
            },
            _ => panic!("Expected identifier"),
        }
    }

    pub fn handle_func_block<'a>(
        mut tokens: &'a mut Peekable<Iter<'a, Token>>,
    ) -> (Vec<Expr>, &'a mut Peekable<Iter<'a, Token>>) {
        let mut exprs = Vec::new();
        loop {
            let (expr, tokens_new) = Self::parse_expr(tokens, true);
            exprs.push(expr);
            match tokens_new.peek() {
                Some(Token::RBrace) => {
                    tokens_new.next();
                    return (exprs, tokens_new);
                }
                _ => (),
            };
            tokens = tokens_new;
        }
    }

    pub fn parse_fn_call<'a>(
        ident: String,
        mut tokens: &'a mut Peekable<Iter<'a, Token>>,
    ) -> (Expr, &'a mut Peekable<Iter<'a, Token>>) {
        let mut args = Vec::new();
        if tokens.peek() == Some(&&Token::RParen) {
            tokens.next();
            (Expr::FnCall { name: ident, args }, tokens)
        } else {
            loop {
                let (arg, tokens_new) = Self::parse_expr(tokens, false);
                args.push(arg);
                match tokens_new.next() {
                    Some(Token::Comma) => (),
                    Some(Token::RParen) => return (Expr::FnCall { name: ident, args }, tokens_new),
                    _ => panic!("Expect comma, or ')'"),
                };
                tokens = tokens_new;
            }
        }
    }
}
