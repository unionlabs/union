extern crate wasmer;

use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(WasmerEnv)]
struct BadAttribute {
    #[wasmer(extraport)] //~ Unexpected identifier `extraport`. Expected `export`.
    memory: LazyInit<Memory>,
}

fn main() {}
