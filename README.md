# Lambda Calculus with Bidirectional Types

This is a fork of the [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) repository, which implements a simple interpreter for the untyped [lambda calculus](https://en.wikipedia.org/wiki/Lambda_calculus) in Rust.
This version extends the original implementation with **bidirectional type checking**, type annotations, natural numbers and booleans!

The extended grammar:

```go
e ::= x       // variable
    | λx. e   // abstraction
    | e e     // application
    | x = e   // binding
	| (e : T) // type annotation
	| n       // natural number
	| true    // boolean true
	| false   // boolean false

T ::= Bool    // boolean type
	| Nat     // natural number type
	| T -> T  // function type
```

## See [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) for usage reference
