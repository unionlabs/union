#![allow(dead_code)]

use wasmer::{Function, Global, LazyInit, Memory, NativeFunc, Table, WasmerEnv};

#[derive(WasmerEnv, Clone)]
struct MyEnv {
    num: u32,
    nums: Vec<i32>,
}

fn impls_wasmer_env<T: WasmerEnv>() -> bool {
    true
}

#[test]
fn test_derive() {
    let _my_env = MyEnv {
        num: 3,
        nums: vec![1, 2, 3],
    };
    assert!(impls_wasmer_env::<MyEnv>());
}

#[derive(WasmerEnv, Clone)]
struct MyEnvWithMemory {
    num: u32,
    nums: Vec<i32>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

#[derive(WasmerEnv, Clone)]
struct MyEnvWithFuncs {
    num: u32,
    nums: Vec<i32>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export)]
    sum: LazyInit<NativeFunc<(i32, i32), i32>>,
}

#[derive(WasmerEnv, Clone)]
struct MyEnvWithEverything {
    num: u32,
    nums: Vec<i32>,
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    #[wasmer(export)]
    sum: LazyInit<NativeFunc<(), i32>>,
    #[wasmer(export)]
    multiply: LazyInit<Function>,
    #[wasmer(export)]
    counter: LazyInit<Global>,
    #[wasmer(export)]
    functions: LazyInit<Table>,
}

#[derive(WasmerEnv, Clone)]
struct MyEnvWithLifetime<'a> {
    name: &'a str,
    #[wasmer(export(name = "memory"))]
    memory: LazyInit<Memory>,
}

#[derive(WasmerEnv, Clone)]
struct MyUnitStruct;

#[derive(WasmerEnv, Clone)]
struct MyTupleStruct(u32);

#[derive(WasmerEnv, Clone)]
struct MyTupleStruct2(u32, u32);

#[derive(WasmerEnv, Clone)]
struct MyTupleStructWithAttribute(#[wasmer(export(name = "memory"))] LazyInit<Memory>, u32);

#[test]
fn test_derive_with_attribute() {
    assert!(impls_wasmer_env::<MyEnvWithMemory>());
    assert!(impls_wasmer_env::<MyEnvWithFuncs>());
    assert!(impls_wasmer_env::<MyEnvWithEverything>());
    assert!(impls_wasmer_env::<MyEnvWithLifetime>());
    assert!(impls_wasmer_env::<MyUnitStruct>());
    assert!(impls_wasmer_env::<MyTupleStruct>());
    assert!(impls_wasmer_env::<MyTupleStruct2>());
    assert!(impls_wasmer_env::<MyTupleStructWithAttribute>());
}

#[derive(WasmerEnv, Clone)]
struct StructWithOptionalField {
    #[wasmer(export(optional = true))]
    memory: LazyInit<Memory>,
    #[wasmer(export(optional = true, name = "real_memory"))]
    memory2: LazyInit<Memory>,
    #[wasmer(export(optional = false))]
    memory3: LazyInit<Memory>,
}

#[test]
fn test_derive_with_optional() {
    assert!(impls_wasmer_env::<StructWithOptionalField>());
}

#[derive(WasmerEnv, Clone)]
struct StructWithAliases {
    #[wasmer(export(alias = "_memory"))]
    memory: LazyInit<Memory>,
    #[wasmer(export(alias = "_real_memory", optional = true, name = "real_memory"))]
    memory2: LazyInit<Memory>,
    #[wasmer(export(alias = "_memory3", alias = "__memory3"))]
    memory3: LazyInit<Memory>,
    #[wasmer(export(alias = "_memory3", name = "memory4", alias = "__memory3"))]
    memory4: LazyInit<Memory>,
}

#[test]
fn test_derive_with_aliases() {
    assert!(impls_wasmer_env::<StructWithAliases>());
}
