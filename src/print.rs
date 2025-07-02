use std::io::Write;

use crate::{parser::Type, types::TypeError, Term};

const RED: &str = "\x1b[31m";
const DARK_GRAY: &str = "\x1b[90m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const PINK: &str = "\x1b[35m";
const PURPLE: &str = "\x1b[95m";
const ITALIC: &str = "\x1b[3m";
const RESET: &str = "\x1b[0m";

pub fn line(len: usize) {
    println!("{}{}{}", DARK_GRAY, "-".repeat(len), RESET);
}

pub fn pause(s: &str) {
    print!("{YELLOW}<{}>{RESET}", s);
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();
    print!("\x1b[1A"); // Move up one line
    print!("\x1b[2K"); // Clear the line
}

pub fn var(v: &str) -> String {
    match v {
        // booleans
        "true" => format!("{CYAN}{ITALIC}true{RESET}"),
        "false" => format!("{CYAN}{ITALIC}false{RESET}"),
        // function names
        _ if char::is_uppercase(v.chars().next().unwrap()) => {
            format!("{PINK}{}{RESET}", v)
        }
        // digits
        _ if v.chars().all(char::is_numeric) => {
            format!("{GREEN}{}{RESET}", v)
        }
        // variable names
        _ => format!("{ITALIC}{}{RESET}", v),
    }
}

/// Pretty print a term
pub fn term(t: &Term) -> String {
    match t {
        Term::Abstraction(param, body) => {
            let body = term(body);
            format!("{YELLOW}λ{RESET}{}{DARK_GRAY}.{RESET}{}", var(param), body)
        }
        Term::Application(f, x) => format!(
            "{DARK_GRAY}({RESET}{} {}{DARK_GRAY}){RESET}",
            term(f),
            term(x)
        ),
        Term::Variable(v, t) => {
            if let Some(t) = t {
                format!("{} {DARK_GRAY}:{RESET} {}", var(v), ty(t))
            } else {
                var(v)
            }
        }
        Term::Nat(n) => format!("{GREEN}{}{RESET}", n),
        Term::Bool(b) => format!("{CYAN}{}{RESET}", if *b { "true" } else { "false" }),
    }
}

pub fn assign(target: &Term, body: &Term) -> String {
    format!("{} = {}{DARK_GRAY};{RESET}", term(target), term(body))
}

pub fn ty(t: &Type) -> String {
    match t {
        Type::Variable(name) => format!("{PURPLE}{}{RESET}", name),
        Type::Abstraction(t1, t2) => format!("{} {DARK_GRAY}->{RESET} {}", ty(t1), ty(t2)),
    }
}

pub fn ty_err(err: TypeError) -> String {
    let type_error = format!("{RED}Type error{RESET}");
    match err {
        TypeError::Mismatch { expected, found } => {
            format!(
                "{type_error}: expected {} but found {}",
                ty(&expected),
                ty(&found)
            )
        }
        TypeError::NotAFunction(t) => {
            format!("{type_error}: {} is not a function type", ty(&t))
        }
        TypeError::Unbound(name) => {
            format!("{type_error}: unbound variable `{}`", var(&name))
        }
    }
}

pub fn ctx(ctx: &crate::types::Ctx) -> String {
    let mut ctx_str = "Γ = {\n".to_string();
    for (name, t) in ctx.iter() {
        ctx_str.push_str(&format!(
            "  {} {DARK_GRAY}:{RESET} {}{DARK_GRAY},{RESET}\n",
            var(name),
            ty(t)
        ));
    }
    ctx_str.push('}');
    ctx_str
}
