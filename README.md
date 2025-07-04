# Typed Lambda Calculus with Bidirectional Checking

This is a fork of the [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) repository, extending with **bidirectional type checking**, type annotations, natural numbers and booleans!

The extended grammar:

```go
e ::= X           // variable
    | λX. e       // abstraction
    | e e         // application
    | X = e       // binding
	| type A = B  // type definition

X ::= v           // variable
    | v : T       // variable with type annotation

T ::= t           // named type
	| *           // any type (hole)
    | T -> T      // application type
```

## See [lambda_calc](https://github.com/WilliamRagstad/lambda_calc) for usage reference
