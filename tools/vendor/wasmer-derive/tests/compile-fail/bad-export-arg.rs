extern crate wasmer;

use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(WasmerEnv)]
struct BadExportArg {
    #[wasmer(export(this_is_not_a_real_argument = "hello, world"))]
    //~ Unrecognized argument in export options: expected `name` found `this_is_not_a_real_argument
    memory: LazyInit<Memory>,
}

#[derive(WasmerEnv)]
struct BadExportArgRawString {
    #[wasmer(export("hello"))] //~ Failed to parse `wasmer` attribute: unexpected token
    memory: LazyInit<Memory>,
}

fn main() {}
