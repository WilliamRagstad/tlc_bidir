use std::{fmt::Display, rc::Rc};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

/// Lambda calculus parser using pest
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LambdaCalcParser;

#[derive(Debug, Clone, PartialEq)]
pub struct LineInfo(pub usize, pub usize);

impl From<pest::Span<'_>> for LineInfo {
    fn from(span: pest::Span) -> Self {
        // Convert Pest span to our LineInfo
        LineInfo(span.start_pos().line_col().0, span.start_pos().line_col().1)
    }
}

/// AST for our extended lambda calculus program
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assignment(String, Option<Type>, Term),
    TypeDef(String, Type),
    Term(Term),
}

/// A program is a list of expressions
pub type Program = Vec<Expr>;

/// AST for lambda calculus
///
/// See https://en.wikipedia.org/wiki/Lambda_calculus#Definition.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Abstraction(String, Option<Type>, Box<Term>, LineInfo),
    Application(Box<Term>, Box<Term>, LineInfo),
    Variable(String, Option<Type>, LineInfo), // Variable with optional type annotation
}

impl Term {
    /// Get the line and column information for this term
    pub fn info(&self) -> &LineInfo {
        match self {
            Term::Abstraction(_, _, _, info) => info,
            Term::Application(_, _, info) => info,
            Term::Variable(_, _, info) => info,
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Abstraction(param, expected, term, _) => {
                write!(
                    f,
                    "λ{}: {}. {}",
                    param,
                    expected.clone().unwrap_or_default(),
                    term
                )
            }
            Term::Application(term1, term2, _) => {
                write!(f, "({} {})", term1, term2)
            }
            Term::Variable(name, expected, _) => {
                if let Some(expected) = expected {
                    write!(f, "{}: {}", name, expected)
                } else {
                    write!(f, "{}", name)
                }
            }
        }
    }
}

/// Type system for lambda calculus
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Type {
    #[default]
    Any, // Any type (used for untyped variables)
    Variable(String), // Type variable
    Abstraction(Rc<Type>, Rc<Type>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "*"),
            Type::Variable(name) => write!(f, "{}", name),
            Type::Abstraction(param, ret) => {
                write!(f, "({} -> {})", param, ret)
            }
        }
    }
}

/// Parse a top-level program into a list of terms
pub fn parse_prog(input: &str) -> Program {
    /// Transform a Pest pair into our own AST Expr node format
    fn parse_term(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::abstraction => {
                let span = pair.as_span();
                let mut inner = pair.into_inner();
                // let param = inner.next().unwrap().as_str().to_string();
                let (param, expected) = match inner.next().unwrap() {
                    // Parse variable with optional type annotation
                    pair if pair.as_rule() == Rule::variable => {
                        let mut inner_var = pair.into_inner();
                        let var_name = inner_var.next().unwrap().as_str().to_string();
                        let type_annotation = inner_var.next().map(parse_type);
                        (var_name, type_annotation)
                    }
                    // Parse untyped variable
                    pair if pair.as_rule() == Rule::untyped_variable => {
                        let var_name = pair.as_str().to_string();
                        (var_name, None)
                    }
                    _ => unreachable!("Expected variable or untyped variable"),
                };
                let body = parse_term(inner.next().unwrap());
                Term::Abstraction(param, expected, Box::new(body), span.into())
            }
            // Rule::application => {
            //     let mut inner = pair.into_inner();
            //     let lhs = parse_term(inner.next().unwrap());
            //     let rhs = parse_term(inner.next().unwrap());
            //     Term::Application(Box::new(lhs), Box::new(rhs))
            // }
            // rhs is one or more terms
            Rule::application => {
                // Syntax sugar: (e1 e2 e3 ...) -> (e1 (e2 (e3 ...)))
                // Previous (e1 e2) was only allowed
                let span = pair.as_span();
                let mut inner = pair.into_inner();
                let mut lhs = parse_term(inner.next().unwrap());
                for rhs in inner {
                    lhs = Term::Application(Box::new(lhs), Box::new(parse_term(rhs)), span.into());
                }
                lhs
            }
            Rule::variable => {
                let span = pair.as_span();
                let mut inner = pair.into_inner();
                let var_name = inner.next().unwrap().as_str().to_string();
                let type_annotation = inner.next().map(parse_type);
                Term::Variable(var_name, type_annotation, span.into())
            }
            Rule::untyped_variable => {
                // Variable without type annotation
                let var_name = pair.as_str().to_string();
                Term::Variable(var_name, None, pair.as_span().into())
            }
            r => unreachable!("Rule {:?} not expected", r),
        }
    }

    fn parse_type(pair: Pair<Rule>) -> Type {
        match pair.as_rule() {
            Rule::base_type => match pair.as_str() {
                "*" => Type::Any, // Represents any type
                name => Type::Variable(name.to_string()),
            },
            Rule::app_type => {
                let mut inner = pair.into_inner();
                let base = parse_type(inner.next().unwrap());
                let next = parse_type(inner.next().unwrap());
                Type::Abstraction(Rc::new(base), Rc::new(next))
            }
            r => unreachable!("Rule {:?} not expected", r),
        }
    }

    let mut prog = Program::new();
    let pairs = match LambdaCalcParser::parse(Rule::program, input) {
        Ok(pairs) => pairs,
        Err(e) => {
            eprintln!("{}", e);
            return prog;
        }
    };
    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => break,
            Rule::assignment => {
                let mut inner = pair.into_inner();
                let name = parse_term(inner.next().unwrap());
                let (name, expected) = match name {
                    Term::Variable(name, expected, _) => (name, expected),
                    _ => unreachable!("Assignment target must be a variable with type annotation"),
                };
                let term = parse_term(inner.next().unwrap());
                prog.push(Expr::Assignment(name, expected, term));
            }
            Rule::type_def => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let type_annotation = parse_type(inner.next().unwrap());
                prog.push(Expr::TypeDef(name, type_annotation));
            }
            // Parse a lambda calculus term
            _ => prog.push(Expr::Term(parse_term(pair))),
        }
    }
    prog
}
