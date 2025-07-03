#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        eval::{eval_expr, inline_vars},
        parser::{parse_prog, Expr, Term},
        PRINT_NONE,
    };

    impl Expr {
        fn term(&self) -> &Term {
            match self {
                Expr::Assignment(_, _, term) => term,
                Expr::TypeDef(_, _) => panic!("Type definitions should not be used as terms"),
                Expr::Term(term) => term,
            }
        }
    }

    #[test]
    fn test_parse() {
        let input = "x = y; λx. (x y); x y;";
        let terms = parse_prog(input);

        if let Expr::Assignment(target, _, body) = &terms[0] {
            assert_eq!(target, "x");
            if let Term::Variable(var_name, _, _) = body {
                assert_eq!(var_name, "y");
            } else {
                panic!("Expected a variable for assignment body");
            }
        } else {
            panic!("Expected an assignment expression");
        }
        if let Expr::Term(term) = &terms[1] {
            if let Term::Abstraction(param, body, _) = term {
                assert_eq!(param, "x");
                if let Term::Application(f, x, _) = &**body {
                    if let Term::Variable(var_name, _, _) = &**f {
                        assert_eq!(var_name, "x");
                        if let Term::Variable(arg_name, _, _) = &**x {
                            assert_eq!(arg_name, "y");
                        } else {
                            panic!("Expected a variable for argument");
                        }
                    } else {
                        panic!("Expected a variable for function");
                    }
                } else {
                    panic!("Expected an application in the body of abstraction");
                }
            } else {
                panic!("Expected an abstraction term");
            }
        } else {
            panic!("Expected a term expression");
        }
        if let Expr::Term(term) = &terms[2] {
            if let Term::Application(f, x, _) = term {
                if let Term::Variable(var_name, _, _) = &**f {
                    assert_eq!(var_name, "x");
                    if let Term::Variable(arg_name, _, _) = &**x {
                        assert_eq!(arg_name, "y");
                    } else {
                        panic!("Expected a variable for argument in application");
                    }
                } else {
                    panic!("Expected a variable for function in application");
                }
            } else {
                panic!("Expected an application term");
            }
        } else {
            panic!("Expected a term expression");
        }
    }

    #[test]
    fn test_multi_app() {
        let input = "λx. λy. λz. ((x y) z);";
        let terms = parse_prog(input);

        if let Expr::Term(Term::Abstraction(_, body, _)) = &terms[0] {
            if let Term::Application(f, x, _) = &**body {
                if let Term::Application(g, y, _) = &**f {
                    if let Term::Variable(x_var, None, _) = &**g {
                        assert_eq!(x_var, "x");
                        if let Term::Variable(y_var, None, _) = &**y {
                            assert_eq!(y_var, "y");
                            if let Term::Variable(z_var, None, _) = &**x {
                                assert_eq!(z_var, "z");
                            } else {
                                panic!("Expected a variable for z");
                            }
                        } else {
                            panic!("Expected a variable for y");
                        }
                    } else {
                        panic!("Expected a variable for x");
                    }
                } else {
                    panic!("Expected an application in the body");
                }
            } else {
                panic!("Expected an application in the body");
            }
        } else {
            panic!("Expected a term abstraction");
        }
    }

    #[test]
    fn test_eval() {
        let mut env = HashMap::new();
        let input = "x = λx. (x y); x y;";
        let prog = parse_prog(input);
        assert_eq!(prog.len(), 2);
        eval_expr(&prog[0], &mut env, false, PRINT_NONE);
        let result = eval_expr(&prog[1], &mut env, false, PRINT_NONE);

        if let Term::Application(f, x, _) = result {
            if let Term::Variable(var_name, _, _) = &*f {
                assert_eq!(var_name, "x");
                if let Term::Variable(arg_name, _, _) = &*x {
                    assert_eq!(arg_name, "y");
                } else {
                    panic!("Expected a variable for argument in application");
                }
            } else {
                panic!("Expected a variable for function in application");
            }
        } else {
            panic!("Expected a term expression for evaluation result");
        }
    }

    /// We should be able to have recursive function definitions
    /// and inline them in one step at a time without any issues.
    #[test]
    fn test_inline_vars_one_step() {
        let mut env = HashMap::new();
        let input = "A = λx. (A x); A y;";
        let expected = "(λx. (A x)) y";
        let prog = parse_prog(input);
        let binding = parse_prog(expected).pop().unwrap();
        let prog_expected = binding.term();
        assert_eq!(prog.len(), 2);
        eval_expr(&prog[0], &mut env, false, PRINT_NONE);
        let inlined = inline_vars(prog[1].term(), &env);
        assert_eq!(&inlined, prog_expected);
    }
}
