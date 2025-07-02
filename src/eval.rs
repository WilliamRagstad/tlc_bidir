use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use crate::{
    parser::{parse_prog, Expr, Program, Term, Type},
    print,
    types::{self, Ctx},
};

/// Environment mapping variable names to terms
pub type Env = HashMap<String, Term>;

/// Substitute a variable in a term with another term
/// This is used in β-reduction.
///
/// See https://en.wikipedia.org/wiki/Lambda_calculus#Substitution.
pub fn substitute(term: &Term, var: &str, value: &Term) -> Term {
    match term {
        // (e1 e2)[var := value] = (e1[var := value]) (e2[var := value])
        Term::Application(e1, e2) => Term::Application(
            Box::new(substitute(e1, var, value)),
            Box::new(substitute(e2, var, value)),
        ),
        // (λx. e)[var := value] = λx. e  (x == var)
        Term::Abstraction(s, _) if s == var => term.clone(), // Bound variable, no substitution needed
        // (λx. e)[var := value] = λx. e  (x in free_vars(value))
        Term::Abstraction(s, body) if free_vars(value).contains(s) => {
            // Avoid variable capture collisions by generating a fresh variable name
            let mut s_new = s.clone();
            while free_vars(value).contains(&s_new) {
                s_new.push('\'');
            }
            let new_body = substitute(&rename_var(body, s, &s_new), var, value);
            Term::Abstraction(s_new, Box::new(new_body))
        }
        // (λx. e)[var := value] = λx. e[var := value]  (x != var and x not in free_vars(value))
        Term::Abstraction(s, body) => {
            // Substitute inside the abstraction's body
            Term::Abstraction(s.clone(), Box::new(substitute(body, var, value)))
        }
        // var[var := value] = value
        Term::Variable(v, _) if v == var => value.clone(),
        // x[var := value] = x   (x != var)
        Term::Variable(_, _) => term.clone(),
        // Nat and Bool are not affected by substitution
        Term::Nat(n) => Term::Nat(*n),
        Term::Bool(b) => Term::Bool(*b),
    }
}

/// Collect free variables in a term
///
/// See https://en.wikipedia.org/wiki/Lambda_calculus#Free_and_bound_variables.
pub fn free_vars(term: &Term) -> HashSet<String> {
    match term {
        // free_vars(λx. e) = free_vars(e) - {x}
        Term::Abstraction(s, body) => {
            let mut set = free_vars(body);
            set.remove(s);
            set
        }
        // free_vars(e1 e2) = free_vars(e1) + free_vars(e2)
        Term::Application(e1, e2) => {
            let mut set = free_vars(e1);
            set.extend(free_vars(e2));
            set
        }
        // free_vars(x) = {x}
        Term::Variable(s, _) => {
            let mut set = HashSet::new();
            set.insert(s.clone());
            set
        }
        // free_vars(n) = {}
        Term::Nat(_) => HashSet::new(),
        // free_vars(b) = {}
        Term::Bool(_) => HashSet::new(),
    }
}

// Rename a variable in a term
pub fn rename_var(term: &Term, old_var: &str, new_var: &str) -> Term {
    match term {
        Term::Abstraction(s, body) if s == old_var => Term::Abstraction(
            new_var.to_string(),
            Box::new(rename_var(body, old_var, new_var)),
        ),
        Term::Abstraction(s, body) => {
            Term::Abstraction(s.clone(), Box::new(rename_var(body, old_var, new_var)))
        }

        Term::Application(e1, e2) => Term::Application(
            Box::new(rename_var(e1, old_var, new_var)),
            Box::new(rename_var(e2, old_var, new_var)),
        ),
        Term::Variable(s, t) if s == old_var => Term::Variable(new_var.to_string(), t.clone()),
        Term::Variable(_, _) => term.clone(),
        Term::Nat(n) => Term::Nat(*n),
        Term::Bool(b) => Term::Bool(*b),
    }
}

// Perform β-reduction on a lambda calculus term
pub fn beta_reduce(term: &Term, env: &Env, mut bound_vars: HashSet<String>) -> Term {
    match term {
        Term::Abstraction(var, body) => {
            bound_vars.insert(var.clone());
            Term::Abstraction(var.clone(), Box::new(beta_reduce(body, env, bound_vars)))
        }
        Term::Application(e1, e2) => {
            // Only when application is reduced, lookup env variables and substitute
            let e1 = if let Term::Variable(var, ty) = e1.borrow() {
                if !bound_vars.contains(var) {
                    env_var(var, ty, env)
                } else {
                    *e1.clone()
                }
            } else {
                *e1.clone()
            };
            if let Term::Abstraction(var, body) = e1.borrow() {
                substitute(body, var, e2)
            } else {
                Term::Application(
                    Box::new(beta_reduce(&e1, env, bound_vars.clone())),
                    Box::new(beta_reduce(e2, env, bound_vars)),
                )
            }
        }
        Term::Variable(_, _) => term.clone(),
        Term::Nat(n) => Term::Nat(*n),
        Term::Bool(b) => Term::Bool(*b),
    }
}

/// Reduce a term to normal form by repeatedly applying β-reduction
pub fn reduce_to_normal_form(term: &Term, env: &Env, verbose: bool, printer: PrinterFn) -> Term {
    let mut term = term.clone();
    loop {
        let mut next = beta_reduce(&term, env, HashSet::new());
        if next == term {
            // Try to inline variables in the term
            next = inline_vars(&next, env);
            if next == term {
                return term;
            }
        }
        term = next;
        if verbose {
            printer(print::term(&term));
        }
    }
}

/// Inline a free variable in env into a term
pub fn env_var(var: &str, ty: &Option<Type>, env: &Env) -> Term {
    if let Some(expr) = env.get(var) {
        // If the variable is in the environment, loop until it is not a variable
        let mut expr = expr.clone();
        while let Term::Variable(v, _) = &expr {
            if let Some(new_expr) = env.get(v) {
                expr = new_expr.clone();
            } else {
                break;
            }
        }
        return expr;
    }
    Term::Variable(var.to_string(), ty.clone())
}

/// Inline variables in a term using the given environment
pub fn inline_vars(term: &Term, env: &Env) -> Term {
    match &term {
        Term::Abstraction(param, body) => {
            Term::Abstraction(param.clone(), Box::new(inline_vars(body, env)))
        }
        Term::Application(f, x) => {
            Term::Application(Box::new(inline_vars(f, env)), Box::new(inline_vars(x, env)))
        }
        Term::Variable(var, ty) => env_var(var, ty, env),
        Term::Nat(n) => Term::Nat(*n),
        Term::Bool(b) => Term::Bool(*b),
    }
}

pub fn eval_expr(expr: &Expr, env: &mut Env, verbose: bool, printer: PrinterFn) -> Term {
    match expr {
        Expr::Assignment(name, val) => {
            if verbose {
                printer(print::assign(name, val));
            }
            let name = match name {
                Term::Variable(v, _) => v.clone(),
                _ => panic!("Assignment target must be a variable"),
            };
            // Explicitly DON'T apply beta reduction here!
            // We want recursive combinators to not be evaluated until they are used
            env.insert(name, val.clone());
            val.clone()
        }
        Expr::Term(term) => {
            let term = inline_vars(term, env);
            if verbose {
                printer(print::term(&term));
            }
            reduce_to_normal_form(&term, env, verbose, printer)
        }
    }
}

/// Run the given input program in the given environment
pub fn eval_prog(input: String, env: &mut Env, verbose: bool, printer: PrinterFn) {
    let terms: Program = parse_prog(input.replace("\r", "").trim());
    let mut ctx = Ctx::new();
    if let Err(err) = types::check_program(&mut ctx, &terms) {
        printer(print::ty_err(err));
        return;
    }
    if verbose {
        printer(print::ctx(&ctx));
    }
    for (i, expr) in terms.iter().enumerate() {
        let term = eval_expr(expr, env, verbose, printer);
        if matches!(expr, Expr::Assignment(_, _)) {
            continue;
        }
        if verbose {
            // Print all terms and their reduction steps
            // println!("{}", print::term(&term));
            if i < terms.len() - 1 {
                print::line(20);
            }
        }
        if !verbose && i == terms.len() - 1 {
            // Always print the last term if not in verbose mode
            printer(print::term(&term));
        }
    }
}

pub type PrinterFn = fn(String);
