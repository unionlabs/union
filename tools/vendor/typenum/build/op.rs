#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum OpType {
    Operator,
    Function,
}

use self::OpType::*;

struct Op {
    token: &'static str,
    operator: &'static str,
    example: (&'static str, &'static str),
    precedence: u8,
    n_args: u8,
    op_type: OpType,
}

pub fn write_op_macro() -> ::std::io::Result<()> {
    let out_dir = ::std::env::var("OUT_DIR").unwrap();
    let dest = ::std::path::Path::new(&out_dir).join("op.rs");
    #[cfg(not(feature = "force_unix_path_separator"))]
    println!("cargo:rustc-env=TYPENUM_BUILD_OP={}", dest.display());
    let mut f = ::std::fs::File::create(&dest).unwrap();

    // Operator precedence is taken from
    // https://doc.rust-lang.org/reference.html#operator-precedence
    //
    // We choose 16 as the highest precedence (functions are set to 255 but it doesn't matter
    // for them).  We also only use operators that are left associative so we don't have to worry
    // about that.
    let ops = &[
        Op {
            token: "*",
            operator: "Prod",
            example: ("P2 * P3", "P6"),
            precedence: 16,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "/",
            operator: "Quot",
            example: ("P6 / P2", "P3"),
            precedence: 16,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "%",
            operator: "Mod",
            example: ("P5 % P3", "P2"),
            precedence: 16,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "+",
            operator: "Sum",
            example: ("P2 + P3", "P5"),
            precedence: 15,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "-",
            operator: "Diff",
            example: ("P2 - P3", "N1"),
            precedence: 15,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "<<",
            operator: "Shleft",
            example: ("U1 << U5", "U32"),
            precedence: 14,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: ">>",
            operator: "Shright",
            example: ("U32 >> U5", "U1"),
            precedence: 14,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "&",
            operator: "And",
            example: ("U5 & U3", "U1"),
            precedence: 13,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "^",
            operator: "Xor",
            example: ("U5 ^ U3", "U6"),
            precedence: 12,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "|",
            operator: "Or",
            example: ("U5 | U3", "U7"),
            precedence: 11,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "==",
            operator: "Eq",
            example: ("P5 == P3 + P2", "True"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "!=",
            operator: "NotEq",
            example: ("P5 != P3 + P2", "False"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "<=",
            operator: "LeEq",
            example: ("P6 <= P3 + P2", "False"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: ">=",
            operator: "GrEq",
            example: ("P6 >= P3 + P2", "True"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "<",
            operator: "Le",
            example: ("P4 < P3 + P2", "True"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: ">",
            operator: "Gr",
            example: ("P5 < P3 + P2", "False"),
            precedence: 10,
            n_args: 2,
            op_type: Operator,
        },
        Op {
            token: "cmp",
            operator: "Compare",
            example: ("cmp(P2, P3)", "Less"),
            precedence: !0,
            n_args: 2,
            op_type: Function,
        },
        Op {
            token: "sqr",
            operator: "Square",
            example: ("sqr(P2)", "P4"),
            precedence: !0,
            n_args: 1,
            op_type: Function,
        },
        Op {
            token: "sqrt",
            operator: "Sqrt",
            example: ("sqrt(U9)", "U3"),
            precedence: !0,
            n_args: 1,
            op_type: Function,
        },
        Op {
            token: "abs",
            operator: "AbsVal",
            example: ("abs(N2)", "P2"),
            precedence: !0,
            n_args: 1,
            op_type: Function,
        },
        Op {
            token: "cube",
            operator: "Cube",
            example: ("cube(P2)", "P8"),
            precedence: !0,
            n_args: 1,
            op_type: Function,
        },
        Op {
            token: "pow",
            operator: "Exp",
            example: ("pow(P2, P3)", "P8"),
            precedence: !0,
            n_args: 2,
            op_type: Function,
        },
        Op {
            token: "min",
            operator: "Minimum",
            example: ("min(P2, P3)", "P2"),
            precedence: !0,
            n_args: 2,
            op_type: Function,
        },
        Op {
            token: "max",
            operator: "Maximum",
            example: ("max(P2, P3)", "P3"),
            precedence: !0,
            n_args: 2,
            op_type: Function,
        },
        Op {
            token: "log2",
            operator: "Log2",
            example: ("log2(U9)", "U3"),
            precedence: !0,
            n_args: 1,
            op_type: Function,
        },
        Op {
            token: "gcd",
            operator: "Gcf",
            example: ("gcd(U9, U21)", "U3"),
            precedence: !0,
            n_args: 2,
            op_type: Function,
        },
    ];

    use std::io::Write;
    write!(
        f,
        "
/**
Convenient type operations.

Any types representing values must be able to be expressed as `ident`s. That means they need to be
in scope.

For example, `P5` is okay, but `typenum::P5` is not.

You may combine operators arbitrarily, although doing so excessively may require raising the
recursion limit.

# Example
```rust
#![recursion_limit=\"128\"]
#[macro_use] extern crate typenum;
use typenum::consts::*;

fn main() {{
    assert_type!(
        op!(min((P1 - P2) * (N3 + N7), P5 * (P3 + P4)) == P10)
    );
}}
```
Operators are evaluated based on the operator precedence outlined
[here](https://doc.rust-lang.org/reference.html#operator-precedence).

The full list of supported operators and functions is as follows:

{}

They all expand to type aliases defined in the `operator_aliases` module. Here is an expanded list,
including examples:

",
        ops.iter()
            .map(|op| format!("`{}`", op.token))
            .collect::<Vec<_>>()
            .join(", ")
    )?;

    //write!(f, "Token | Alias | Example\n ===|===|===\n")?;

    for op in ops.iter() {
        write!(
            f,
            "---\nOperator `{token}`. Expands to `{operator}`.

```rust
# #[macro_use] extern crate typenum;
# use typenum::*;
# fn main() {{
assert_type_eq!(op!({ex0}), {ex1});
# }}
```\n
",
            token = op.token,
            operator = op.operator,
            ex0 = op.example.0,
            ex1 = op.example.1
        )?;
    }

    write!(
        f,
        "*/
#[macro_export(local_inner_macros)]
macro_rules! op {{
    ($($tail:tt)*) => ( __op_internal__!($($tail)*) );
}}

    #[doc(hidden)]
    #[macro_export(local_inner_macros)]
    macro_rules! __op_internal__ {{
"
    )?;

    // We first us the shunting-yard algorithm to produce our tokens in Polish notation.
    // See: https://en.wikipedia.org/wiki/Shunting-yard_algorithm

    // Note: Due to macro asymmetry, "the top of the stack" refers to the first element, not the
    // last

    // -----------------------------------------------------------------------------------------
    // Stage 1: There are tokens to be read:

    // -------
    // Case 1: Token is a function => Push it onto the stack:
    for fun in ops.iter().filter(|f| f.op_type == Function) {
        write!(
            f,
            "
(@stack[$($stack:ident,)*] @queue[$($queue:ident,)*] @tail: {f_token} $($tail:tt)*) => (
    __op_internal__!(@stack[{f_op}, $($stack,)*] @queue[$($queue,)*] @tail: $($tail)*)
);",
            f_token = fun.token,
            f_op = fun.operator
        )?;
    }

    // -------
    // Case 2: Token is a comma => Until the top of the stack is a LParen,
    //                             Pop operators from stack to queue

    // Base case: Top of stack is LParen, ditch comma and continue
    write!(
        f,
        "
(@stack[LParen, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: , $($tail:tt)*) => (
    __op_internal__!(@stack[LParen, $($stack,)*] @queue[$($queue,)*] @tail: $($tail)*)
);"
    )?;
    // Recursive case: Not LParen, pop from stack to queue
    write!(
        f,
        "
(@stack[$stack_top:ident, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: , $($tail:tt)*) => (
    __op_internal__!(@stack[$($stack,)*] @queue[$stack_top, $($queue,)*] @tail: , $($tail)*)
);"
    )?;

    // -------
    // Case 3: Token is an operator, o1:
    for o1 in ops.iter().filter(|op| op.op_type == Operator) {
        // If top of stack is operator o2 with o1.precedence <= o2.precedence,
        // Then pop o2 off stack onto queue:
        for o2 in ops
            .iter()
            .filter(|op| op.op_type == Operator)
            .filter(|o2| o1.precedence <= o2.precedence)
        {
            write!(
                f,
                "
(@stack[{o2_op}, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: {o1_token} $($tail:tt)*) => (
    __op_internal__!(@stack[$($stack,)*] @queue[{o2_op}, $($queue,)*] @tail: {o1_token} $($tail)*)
);",
                o2_op = o2.operator,
                o1_token = o1.token
            )?;
        }
        // Base case: push o1 onto stack
        write!(
            f,
            "
(@stack[$($stack:ident,)*] @queue[$($queue:ident,)*] @tail: {o1_token} $($tail:tt)*) => (
    __op_internal__!(@stack[{o1_op}, $($stack,)*] @queue[$($queue,)*] @tail: $($tail)*)
);",
            o1_op = o1.operator,
            o1_token = o1.token
        )?;
    }

    // -------
    // Case 4: Token is "(": push it onto stack as "LParen". Also convert the ")" to "RParen" to
    // appease the macro gods:
    write!(
        f,
        "
(@stack[$($stack:ident,)*] @queue[$($queue:ident,)*] @tail: ( $($stuff:tt)* ) $($tail:tt)* )
 => (
    __op_internal__!(@stack[LParen, $($stack,)*] @queue[$($queue,)*]
                     @tail: $($stuff)* RParen $($tail)*)
);"
    )?;

    // -------
    // Case 5: Token is "RParen":
    //     1. Pop from stack to queue until we see an "LParen",
    //     2. Kill the "LParen",
    //     3. If the top of the stack is a function, pop it onto the queue
    // 2. Base case:
    write!(
        f,
        "
(@stack[LParen, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: RParen $($tail:tt)*) => (
    __op_internal__!(@rp3 @stack[$($stack,)*] @queue[$($queue,)*] @tail: $($tail)*)
);"
    )?;
    // 1. Recursive case:
    write!(
        f,
        "
(@stack[$stack_top:ident, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: RParen $($tail:tt)*)
 => (
    __op_internal__!(@stack[$($stack,)*] @queue[$stack_top, $($queue,)*] @tail: RParen $($tail)*)
);"
    )?;
    // 3. Check for function:
    for fun in ops.iter().filter(|f| f.op_type == Function) {
        write!(
            f,
            "
(@rp3 @stack[{fun_op}, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail: $($tail:tt)*) => (
    __op_internal__!(@stack[$($stack,)*] @queue[{fun_op}, $($queue,)*] @tail: $($tail)*)
);",
            fun_op = fun.operator
        )?;
    }
    // 3. If no function found:
    write!(
        f,
        "
(@rp3 @stack[$($stack:ident,)*] @queue[$($queue:ident,)*] @tail: $($tail:tt)*) => (
    __op_internal__!(@stack[$($stack,)*] @queue[$($queue,)*] @tail: $($tail)*)
);"
    )?;

    // -------
    // Case 6: Token is a number: Push it onto the queue
    write!(
        f,
        "
(@stack[$($stack:ident,)*] @queue[$($queue:ident,)*] @tail: $num:ident $($tail:tt)*) => (
    __op_internal__!(@stack[$($stack,)*] @queue[$num, $($queue,)*] @tail: $($tail)*)
);"
    )?;

    // -------
    // Case 7: Out of tokens:
    // Base case: Stack empty: Start evaluating
    write!(
        f,
        "
(@stack[] @queue[$($queue:ident,)*] @tail: ) => (
    __op_internal__!(@reverse[] @input: $($queue,)*)
);"
    )?;
    // Recursive case: Pop stack to queue
    write!(
        f,
        "
(@stack[$stack_top:ident, $($stack:ident,)*] @queue[$($queue:ident,)*] @tail:) => (
    __op_internal__!(@stack[$($stack,)*] @queue[$stack_top, $($queue,)*] @tail: )
);"
    )?;

    // -----------------------------------------------------------------------------------------
    // Stage 2: Reverse so we have RPN
    write!(
        f,
        "
(@reverse[$($revved:ident,)*] @input: $head:ident, $($tail:ident,)* ) => (
    __op_internal__!(@reverse[$head, $($revved,)*] @input: $($tail,)*)
);"
    )?;
    write!(
        f,
        "
(@reverse[$($revved:ident,)*] @input: ) => (
    __op_internal__!(@eval @stack[] @input[$($revved,)*])
);"
    )?;

    // -----------------------------------------------------------------------------------------
    // Stage 3: Evaluate in Reverse Polish Notation
    // Operators / Operators with 2 args:
    for op in ops.iter().filter(|op| op.n_args == 2) {
        // Note: We have to switch $a and $b here, otherwise non-commutative functions are backwards
        write!(
            f,
            "
(@eval @stack[$a:ty, $b:ty, $($stack:ty,)*] @input[{op}, $($tail:ident,)*]) => (
    __op_internal__!(@eval @stack[$crate::{op}<$b, $a>, $($stack,)*] @input[$($tail,)*])
);",
            op = op.operator
        )?;
    }
    // Operators with 1 arg:
    for op in ops.iter().filter(|op| op.n_args == 1) {
        write!(
            f,
            "
(@eval @stack[$a:ty, $($stack:ty,)*] @input[{op}, $($tail:ident,)*]) => (
    __op_internal__!(@eval @stack[$crate::{op}<$a>, $($stack,)*] @input[$($tail,)*])
);",
            op = op.operator
        )?;
    }

    // Wasn't a function or operator, so must be a value => push onto stack
    write!(
        f,
        "
(@eval @stack[$($stack:ty,)*] @input[$head:ident, $($tail:ident,)*]) => (
    __op_internal__!(@eval @stack[$head, $($stack,)*] @input[$($tail,)*])
);"
    )?;

    // No input left:
    write!(
        f,
        "
(@eval @stack[$stack:ty,] @input[]) => (
    $stack
);"
    )?;

    // -----------------------------------------------------------------------------------------
    // Stage 0: Get it started
    write!(
        f,
        "
($($tail:tt)* ) => (
    __op_internal__!(@stack[] @queue[] @tail: $($tail)*)
);"
    )?;

    write!(
        f,
        "
}}"
    )?;

    Ok(())
}
