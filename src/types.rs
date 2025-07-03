use std::{collections::HashMap, rc::Rc};

use crate::parser::{Expr, LineInfo, Program, Term, Type};

pub type Ctx = HashMap<String, Rc<Type>>;

#[derive(Debug)]
pub enum TypeError {
    Mismatch {
        expected: Type,
        found: Type,
        info: LineInfo,
    },
    NotAFunction(Type, LineInfo),
    Unbound(String, LineInfo),
}

pub fn check_program(ctx: &mut Ctx, prog: &Program) -> Result<(), TypeError> {
    for expr in prog {
        check_expr(ctx, expr)?;
    }
    Ok(())
}

pub fn check_expr(ctx: &mut Ctx, expr: &Expr) -> Result<Rc<Type>, TypeError> {
    match expr {
        Expr::Assignment(target, body) => {
            // Infer the body and bind it to the target
            let (target, expected) = match target {
                Term::Variable(name, expected_ty, _) => (name, expected_ty),
                _ => unreachable!("Assignment target must be a variable with type annotation"),
            };
            check_bind(ctx, target, expected, body)
        }
        Expr::Term(term) => infer_term(ctx, term),
    }
}

/// Checking: Γ ⊢ x = body ⇒ T or Γ ⊢ x: T = body ⇒ T
fn check_bind(
    ctx: &mut Ctx,
    target: &str,
    expected: &Option<Type>,
    body: &Term,
) -> Result<Rc<Type>, TypeError> {
    // let ty_def = infer(ctx, def)?;
    //         ctx.insert(x.clone(), ty_def);
    //         let result = infer(ctx, body);
    //         ctx.remove(x);
    //         result

    // Check if the target is already bound

    if let Some(expected_ty) = expected {
        if let Some(existing_ty) = ctx.get(target) {
            if *expected_ty != **existing_ty {
                Err(TypeError::Mismatch {
                    expected: (*expected_ty).clone(),
                    found: (**existing_ty).clone(),
                    info: body.info().clone(),
                })
            } else {
                Ok(Rc::new(expected_ty.clone()))
            }
        } else {
            // If not bound, insert the expected type
            ctx.insert(target.to_string(), Rc::new(expected_ty.clone()));
            // Now check the body against the expected type
            let inferred = infer_term(ctx, body)?;
            if *expected_ty != *inferred {
                return Err(TypeError::Mismatch {
                    expected: (*expected_ty).clone(),
                    found: (*inferred).clone(),
                    info: body.info().clone(),
                });
            }
            Ok(Rc::new(expected_ty.clone()))
        }
    } else {
        let inferred = infer_term(ctx, body)?;
        ctx.insert(target.to_string(), inferred.clone());
        // If no expected type, just return the inferred type
        Ok(inferred)
    }
}

/// Checking: Γ ⊢ e ⇐ T   (returns () on success)
pub fn check_term(ctx: &mut Ctx, e: &Term, expected: &Rc<Type>) -> Result<(), TypeError> {
    match (e, expected.as_ref()) {
        (Term::Abstraction(x, body, _), Type::Abstraction(param, ret)) => {
            ctx.insert(x.clone(), param.clone());
            let res = check_term(ctx, body, ret);
            ctx.remove(x);
            res
        }
        // fall back to synthesis + equality
        _ => {
            let inferred = infer_term(ctx, e)?;
            if *expected == inferred {
                Ok(())
            } else {
                Err(TypeError::Mismatch {
                    expected: (*expected.as_ref()).clone(),
                    found: (*inferred).clone(),
                    info: e.info().clone(),
                })
            }
        }
    }
}

/// Synthesis: Γ ⊢ e ⇒ T
fn infer_term(ctx: &mut Ctx, e: &Term) -> Result<Rc<Type>, TypeError> {
    match e {
        Term::Variable(x, expected, _) => {
            if let Some(ex_ty) = expected {
                // If there's an expected type, we should compare it
                if let Some(var_ty) = ctx.get(x) {
                    if *ex_ty != **var_ty {
                        return Err(TypeError::Mismatch {
                            expected: (*ex_ty).clone(),
                            found: (**var_ty).clone(),
                            info: e.info().clone(),
                        });
                    }
                }
            }
            ctx.get(x)
                .cloned()
                .ok_or(TypeError::Unbound(x.clone(), e.info().clone()))
        }
        Term::Abstraction(param, body, _) => {
            let param_ty = Rc::new(Type::Variable(param.to_string()));
            ctx.insert(param.clone(), param_ty.clone());
            let ret_ty = infer_term(ctx, body)?;
            ctx.remove(param);
            Ok(Rc::new(Type::Abstraction(param_ty, ret_ty)))
        }
        Term::Application(lhs, rhs, _) => match infer_term(ctx, lhs)?.as_ref() {
            Type::Abstraction(param, ret) => {
                check_term(ctx, rhs, param)?;
                Ok(ret.clone())
            }
            other => Err(TypeError::NotAFunction((*other).clone(), e.info().clone())),
        },
    }
}
