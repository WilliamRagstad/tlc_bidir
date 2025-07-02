use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

/// Lambda calculus parser using pest
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LambdaCalcParser;

/// AST for our extended lambda calculus program
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assignment(Term, Term),
    Term(Term),
}

/// A program is a list of expressions
pub type Program = Vec<Expr>;

/// AST for lambda calculus
///
/// See https://en.wikipedia.org/wiki/Lambda_calculus#Definition.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Abstraction(String, Box<Term>),
    Application(Box<Term>, Box<Term>),
    Variable(String, Option<Type>), // Variable with optional type annotation
    Nat(u32),                       // Natural number
    Bool(bool),                     // Boolean value
}

/// Type system for lambda calculus
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Variable(String), // Type variable
    Abstraction(Box<Type>, Box<Type>),
}

/// Parse a top-level program into a list of terms
pub fn parse_prog(input: &str) -> Program {
    /// Transform a Pest pair into our own AST Expr node format
    fn parse_term(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::abstraction => {
                let mut inner = pair.into_inner();
                let param = inner.next().unwrap().as_str().to_string();
                let body = parse_term(inner.next().unwrap());
                Term::Abstraction(param, Box::new(body))
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
                let mut inner = pair.into_inner();
                let mut lhs = parse_term(inner.next().unwrap());
                for rhs in inner {
                    lhs = Term::Application(Box::new(lhs), Box::new(parse_term(rhs)));
                }
                lhs
            }
            Rule::variable => {
                let mut inner = pair.into_inner();
                let var_name = inner.next().unwrap().as_str().to_string();
                let type_annotation = inner.next().map(parse_type);
                Term::Variable(var_name, type_annotation)
            }
            Rule::nat => {
                let nat_str = pair.as_str();
                Term::Nat(nat_str.parse().unwrap())
            }
            Rule::bool => {
                let bool_str = pair.as_str();
                Term::Bool(bool_str == "true")
            }
            r => unreachable!("Rule {:?} not expected", r),
        }
    }

    fn parse_type(pair: Pair<Rule>) -> Type {
        match pair.as_rule() {
            Rule::base_type => Type::Variable(pair.as_str().to_string()),
            Rule::app_type => {
                let mut inner = pair.into_inner();
                let base = parse_type(inner.next().unwrap());
                let next = parse_type(inner.next().unwrap());
                Type::Abstraction(Box::new(base), Box::new(next))
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
                let term = parse_term(inner.next().unwrap());
                prog.push(Expr::Assignment(name, term));
            }
            // Parse a lambda calculus term
            _ => prog.push(Expr::Term(parse_term(pair))),
        }
    }
    prog
}
