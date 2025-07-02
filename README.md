# Typed Lambda Calculus with Bidirectional Checking

This is a fork of the [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) repository, extending with **bidirectional type checking**, type annotations, natural numbers and booleans!

The extended grammar:

```go
x ::= v       // variable
    | v : t   // variable with type annotation

e ::= x       // variable
    | λx. e   // abstraction
    | e e     // application
    | x = e   // binding
	| n       // natural number
	| true    // boolean true
	| false   // boolean false

t ::= Bool    // boolean type
	| Nat     // natural number type
	| t -> t  // function type
```

## See [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) for usage reference
