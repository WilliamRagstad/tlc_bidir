# Typed Lambda Calculus with Bidirectional Checking

This is a fork of the [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) repository, extending with **bidirectional type checking**, type annotations, natural numbers and booleans!

The extended grammar:

```go
e ::= X       // variable
    | λX. e   // abstraction
    | e e     // application
    | X = e   // binding
	| n       // natural number
	| true    // boolean true
	| false   // boolean false

X ::= v       // variable
    | v : T   // variable with type annotation

T ::= Bool    // boolean type
	| Nat     // natural number type
	| T -> T  // function type
```

## See [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) for usage reference
