extern crate wasmer;

use wasmer::{LazyInit, Memory, WasmerEnv};

#[derive(WasmerEnv)]
struct ExportNotWrappedInLazyInit {
    #[wasmer(export)]
    memory: Memory, //~ WasmerEnv derive expects all `export`s to be wrapped in `LazyInit`
}

fn main() {}
