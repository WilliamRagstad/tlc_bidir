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

pub fn check_program(ctx: &mut Ctx, prog: &mut Program) -> Result<(), TypeError> {
    for expr in prog.iter() {
        check_expr(ctx, expr)?;
    }
    // Remove all type definitions from the context after checking
    prog.retain(|expr| !matches!(expr, Expr::TypeDef(_, _)));
    Ok(())
}

pub fn check_expr(ctx: &mut Ctx, expr: &Expr) -> Result<Rc<Type>, TypeError> {
    match expr {
        Expr::Assignment(target, expected, body) => {
            // Infer the body and bind it to the target
            check_bind(ctx, target, expected, body)
        }
        Expr::TypeDef(target, ty) => {
            // Insert the type definition into the context
            println!("Inserting type definition: {} = {}", target, ty);
            ctx.insert(target.clone(), Rc::new(ty.clone()));
            Ok(Rc::new(ty.clone()))
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

    // if let Some(expected_ty) = expected {
    //     if let Some(existing_ty) = ctx.get(target) {
    //         if *expected_ty != **existing_ty {
    //             Err(TypeError::Mismatch {
    //                 expected: (*expected_ty).clone(),
    //                 found: (**existing_ty).clone(),
    //                 info: body.info().clone(),
    //             })
    //         } else {
    //             Ok(Rc::new(expected_ty.clone()))
    //         }
    //     } else {
    //         // If not bound, insert the expected type
    //         ctx.insert(target.to_string(), Rc::new(expected_ty.clone()));
    //         // Now check the body against the expected type
    //         let inferred = infer_term(ctx, body)?;
    //         if *expected_ty != *inferred {
    //             return Err(TypeError::Mismatch {
    //                 expected: (*expected_ty).clone(),
    //                 found: (*inferred).clone(),
    //                 info: body.info().clone(),
    //             });
    //         }
    //         Ok(Rc::new(expected_ty.clone()))
    //     }
    // } else {
    //     let inferred = infer_term(ctx, body)?;
    //     ctx.insert(target.to_string(), inferred.clone());
    //     // If no expected type, just return the inferred type
    //     Ok(inferred)
    // }
    match infer_var(ctx, target, expected, body.info()) {
        Ok(ty) => {
            // Now check the body against the inferred type
            check_term(ctx, body, &ty)?;
            Ok(ty)
        }
        Err(TypeError::Unbound(_, _)) if expected.is_some() => {
            let expected_ty = Rc::new(resolve_type(ctx, expected.as_ref().unwrap()));
            println!(
                "Variable `{}` is unbound, expected type: {}",
                target,
                expected.clone().unwrap_or_default()
            );
            // If the variable is unbound but we have an expected type, we can insert it
            ctx.insert(target.to_string(), expected_ty.clone());
            check_term(ctx, body, &expected_ty)?;
            Ok(expected_ty)
        }
        Err(TypeError::Unbound(_, _)) => {
            // If the variable is unbound and no expected type, we can infer it
            let inferred_ty = infer_term(ctx, body)?;
            println!(
                "Variable `{}` is unbound, inferred type: {}",
                target, inferred_ty
            );
            ctx.insert(target.to_string(), inferred_ty.clone());
            Ok(inferred_ty)
        }
        Err(err) => Err(err),
    }
}

/// Checking: Γ ⊢ e ⇐ T   (returns () on success)
pub fn check_term(ctx: &mut Ctx, e: &Term, expected: &Rc<Type>) -> Result<(), TypeError> {
    println!("Checking term: {}, expected: {}", e, expected);
    match (e, expected.as_ref()) {
        (Term::Abstraction(x, _, body, _), Type::Abstraction(param, ret)) => {
            ctx.insert(x.clone(), param.clone());
            let res = check_term(ctx, body, ret);
            ctx.remove(x);
            res
        }
        // fall back to synthesis + equality
        _ => {
            let inferred = infer_term(ctx, e)?;
            if compare_types(expected, &inferred) {
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
            // if let Some(ex_ty) = expected {
            //     // Lookup expected type name in context
            //     let ex_ty = if let Type::Variable(name) = ex_ty {
            //         if let Some(var_ty) = ctx.get(name) {
            //             var_ty
            //         } else {
            //             ex_ty
            //         }
            //     } else {
            //         ex_ty
            //     };

            //     // If there's an expected type, we should compare it
            //     if let Some(var_ty) = ctx.get(x) {
            //         if *ex_ty != **var_ty {
            //             return Err(TypeError::Mismatch {
            //                 expected: (*ex_ty).clone(),
            //                 found: (**var_ty).clone(),
            //                 info: e.info().clone(),
            //             });
            //         }
            //     }
            // }
            // ctx.get(x)
            //     .cloned()
            //     .ok_or(TypeError::Unbound(x.clone(), e.info().clone()))
            println!(
                "Inferring variable: {}, expected: {}",
                x,
                expected.clone().unwrap_or_default()
            );
            infer_var(ctx, x, expected, e.info())
        }
        Term::Abstraction(param, _, body, _) => {
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

fn infer_var(
    ctx: &mut Ctx,
    name: &str,
    expected: &Option<Type>,
    info: &LineInfo,
) -> Result<Rc<Type>, TypeError> {
    if let Some(expected) = expected {
        let expected = resolve_type(ctx, expected);

        // If there's an expected type, we should compare it
        if let Some(var_ty) = ctx.get(name) {
            if !compare_types(&expected, var_ty) {
                return Err(TypeError::Mismatch {
                    expected,
                    found: (**var_ty).clone(),
                    info: info.clone(),
                });
            }
        }
    }
    ctx.get(name)
        .cloned()
        .ok_or(TypeError::Unbound(name.to_string(), info.clone())) // Placeholder for line info
}

// Lookup type names in context
fn resolve_type(ctx: &Ctx, ty: &Type) -> Type {
    match ty {
        Type::Any => Type::Any, // Represents any type
        Type::Variable(name) => {
            if let Some(resolved) = ctx.get(name) {
                resolved.as_ref().clone()
            } else {
                ty.clone()
            }
        }
        Type::Abstraction(param, ret) => Type::Abstraction(
            Rc::new(resolve_type(ctx, param)),
            Rc::new(resolve_type(ctx, ret)),
        ),
    }
}

fn compare_types(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Any, _) | (_, Type::Any) => true, // Any type matches with any type
        (Type::Variable(name_a), Type::Variable(name_b)) => name_a == name_b,
        (Type::Abstraction(param_a, ret_a), Type::Abstraction(param_b, ret_b)) => {
            compare_types(param_a, param_b) && compare_types(ret_a, ret_b)
        }
        _ => false,
    }
}
