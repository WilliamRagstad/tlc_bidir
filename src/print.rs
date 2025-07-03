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
        Term::Abstraction(param, expected, body, _) => {
            let body = term(body);
            format!(
                "{YELLOW}λ{RESET}{}{DARK_GRAY}.{RESET}{}",
                typed_var(param, expected),
                body
            )
        }
        Term::Application(f, x, _) => format!(
            "{DARK_GRAY}({RESET}{} {}{DARK_GRAY}){RESET}",
            term(f),
            term(x)
        ),
        Term::Variable(v, t, _) => {
            if let Some(t) = t {
                format!("{} {DARK_GRAY}:{RESET} {}", var(v), r#type(t))
            } else {
                var(v)
            }
        }
    }
}

pub fn typed_var(v: &str, ty: &Option<Type>) -> String {
    if let Some(t) = ty {
        format!("{} {DARK_GRAY}:{RESET} {}", var(v), r#type(t))
    } else {
        var(v)
    }
}

pub fn assign(target: &str, ty: &Option<Type>, body: &Term) -> String {
    format!(
        "{} {DARK_GRAY}={RESET} {}",
        typed_var(target, ty),
        term(body)
    )
}

pub fn r#type(t: &Type) -> String {
    match t {
        Type::Any => format!("{CYAN}*{RESET}"),
        Type::Variable(name) => format!("{PURPLE}{}{RESET}", name),
        Type::Abstraction(t1, t2) => format!("{} {DARK_GRAY}->{RESET} {}", r#type(t1), r#type(t2)),
    }
}

pub fn ty_err(err: TypeError) -> String {
    let type_error = format!("{RED}Type error{RESET}");
    match err {
        TypeError::Mismatch {
            expected,
            found,
            info,
        } => {
            format!(
                "{type_error}: expected {} but found {} at line {} col {}",
                r#type(&expected),
                r#type(&found),
                info.0,
                info.1
            )
        }
        TypeError::NotAFunction(t, info) => {
            format!(
                "{type_error}: {} is not a function type at line {} col {}",
                r#type(&t),
                info.0,
                info.1
            )
        }
        TypeError::Unbound(name, info) => {
            format!(
                "{type_error}: unbound variable `{}` at line {} col {}",
                var(&name),
                info.0,
                info.1
            )
        }
    }
}

pub fn ctx(ctx: &crate::types::Ctx) -> String {
    let mut ctx_str = "Γ = {\n".to_string();
    for (name, t) in ctx.iter() {
        ctx_str.push_str(&format!(
            "  {} {DARK_GRAY}:{RESET} {}{DARK_GRAY},{RESET}\n",
            var(name),
            r#type(t)
        ));
    }
    ctx_str.push('}');
    ctx_str
}
