#[cfg(feature = "js")]
mod js {
    use anyhow::Result;
    use wasm_bindgen_test::*;
    use wasmer::*;

    #[wasm_bindgen_test]
    fn test_exported_memory() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
      (memory (export "mem") 1)
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![],
                exports: vec![ExternType::Memory(MemoryType::new(Pages(1), None, false))],
            })
            .unwrap();

        let import_object = imports! {};
        let instance = Instance::new(&module, &import_object).unwrap();

        let memory = instance.exports.get_memory("mem").unwrap();
        assert_eq!(memory.ty(), MemoryType::new(Pages(1), None, false));
        assert_eq!(memory.size(), Pages(1));
        assert_eq!(memory.data_size(), 65536);

        memory.grow(Pages(1)).unwrap();
        assert_eq!(memory.ty(), MemoryType::new(Pages(2), None, false));
        assert_eq!(memory.size(), Pages(2));
        assert_eq!(memory.data_size(), 65536 * 2);
    }

    #[wasm_bindgen_test]
    fn test_exported_function() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func (export "get_magic") (result i32)
          (i32.const 42)
        )
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![],
                exports: vec![ExternType::Function(FunctionType::new(
                    vec![],
                    vec![Type::I32],
                ))],
            })
            .unwrap();

        let import_object = imports! {};
        let instance = Instance::new(&module, &import_object).unwrap();

        let get_magic = instance.exports.get_function("get_magic").unwrap();
        assert_eq!(
            get_magic.ty().clone(),
            FunctionType::new(vec![], vec![Type::I32])
        );

        let expected = vec![Val::I32(42)].into_boxed_slice();
        assert_eq!(get_magic.call(&[]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_function_dynamic() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
            })
            .unwrap();

        let imported_signature = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let imported = Function::new(&store, &imported_signature, |args| {
            println!("Calling `imported`...");
            let result = args[0].unwrap_i32() * 2;
            println!("Result of `imported`: {:?}", result);
            Ok(vec![Value::I32(result)])
        });

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let exported = instance.exports.get_function("exported").unwrap();

        let expected = vec![Val::I32(6)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(3)]), Ok(expected));
    }

    // We comment it for now because in old versions of Node, only single return values are supported

    // #[wasm_bindgen_test]
    // fn test_imported_function_dynamic_multivalue() {
    //     let store = Store::default();
    //     let mut module = Module::new(
    //         &store,
    //         br#"
    //     (module
    //         (func $multivalue (import "env" "multivalue") (param i32 i32) (result i32 i32))
    //         (func (export "multivalue") (param i32 i32) (result i32 i32)
    //             (call $multivalue (local.get 0) (local.get 1))
    //         )
    //     )
    //     "#,
    //     )
    //     .unwrap();
    //     module.set_type_hints(ModuleTypeHints {
    //         imports: vec![
    //             ExternType::Function(FunctionType::new(
    //                 vec![Type::I32, Type::I32],
    //                 vec![Type::I32, Type::I32],
    //             )),
    //         ],
    //         exports: vec![
    //             ExternType::Function(FunctionType::new(
    //                 vec![Type::I32, Type::I32],
    //                 vec![Type::I32, Type::I32],
    //             )),
    //         ],
    //     });

    //     let multivalue_signature =
    //         FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32, Type::I32]);
    //     let multivalue = Function::new(&store, &multivalue_signature, |args| {
    //         println!("Calling `imported`...");
    //         // let result = args[0].unwrap_i32() * ;
    //         // println!("Result of `imported`: {:?}", result);
    //         Ok(vec![args[1].clone(), args[0].clone()])
    //     });

    //     let import_object = imports! {
    //         "env" => {
    //             "multivalue" => multivalue,
    //         }
    //     };
    //     let instance = Instance::new(&module, &import_object).unwrap();

    //     let exported_multivalue = instance
    //         .exports
    //         .get_function("multivalue")
    //         .unwrap();

    //     let expected = vec![Val::I32(2), Val::I32(3)].into_boxed_slice();
    //     assert_eq!(
    //         exported_multivalue.call(&[Val::I32(3), Val::I32(2)]),
    //         Ok(expected)
    //     );
    // }

    #[wasm_bindgen_test]
    fn test_imported_function_dynamic_with_env() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
            })
            .unwrap();

        #[derive(WasmerEnv, Clone)]
        struct Env {
            multiplier: i32,
        }

        let imported_signature = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let imported = Function::new_with_env(
            &store,
            &imported_signature,
            Env { multiplier: 3 },
            |env, args| {
                println!("Calling `imported`...");
                let result = args[0].unwrap_i32() * env.multiplier;
                println!("Result of `imported`: {:?}", result);
                Ok(vec![Value::I32(result)])
            },
        );

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let exported = instance.exports.get_function("exported").unwrap();

        let expected = vec![Val::I32(9)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(3)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_function_native() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
            })
            .unwrap();

        fn imported_fn(arg: u32) -> u32 {
            return arg + 1;
        }

        let imported = Function::new_native(&store, imported_fn);

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let exported = instance.exports.get_function("exported").unwrap();

        let expected = vec![Val::I32(5)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_function_native_with_env() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
            })
            .unwrap();

        #[derive(WasmerEnv, Clone)]
        struct Env {
            multiplier: u32,
        }

        fn imported_fn(env: &Env, arg: u32) -> u32 {
            return env.multiplier * arg;
        }

        let imported = Function::new_native_with_env(&store, Env { multiplier: 3 }, imported_fn);

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let exported = instance.exports.get_function("exported").unwrap();

        let expected = vec![Val::I32(12)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_function_native_with_wasmer_env() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
        (memory (export "memory") 1)
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![
                    ExternType::Function(FunctionType::new(vec![Type::I32], vec![Type::I32])),
                    ExternType::Memory(MemoryType::new(Pages(1), None, false)),
                ],
            })
            .unwrap();

        #[derive(WasmerEnv, Clone)]
        struct Env {
            multiplier: u32,
            #[wasmer(export)]
            memory: LazyInit<Memory>,
        }

        fn imported_fn(env: &Env, arg: u32) -> u32 {
            let memory = env.memory_ref().unwrap();
            let memory_val = memory.uint8view().get_index(0);
            return (memory_val as u32) * env.multiplier * arg;
        }

        let imported = Function::new_native_with_env(
            &store,
            Env {
                multiplier: 3,
                memory: LazyInit::new(),
            },
            imported_fn,
        );

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let memory = instance.exports.get_memory("memory").unwrap();
        assert_eq!(memory.data_size(), 65536);
        let memory_val = memory.uint8view().get_index(0);
        assert_eq!(memory_val, 0);

        memory.uint8view().set_index(0, 2);
        let memory_val = memory.uint8view().get_index(0);
        assert_eq!(memory_val, 2);

        let exported = instance.exports.get_function("exported").unwrap();

        // It works with the provided memory
        let expected = vec![Val::I32(24)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));

        // It works if we update the memory
        memory.uint8view().set_index(0, 3);
        let expected = vec![Val::I32(36)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_unit_native_function_env() {
        let store = Store::default();
        #[derive(WasmerEnv, Clone)]
        struct Env {
            multiplier: u32,
        }

        fn imported_fn(env: &Env, args: &[Val]) -> Result<Vec<Val>, RuntimeError> {
            let value = env.multiplier * args[0].unwrap_i32() as u32;
            return Ok(vec![Val::I32(value as _)]);
        }

        let imported_signature = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let imported = Function::new_with_env(
            &store,
            imported_signature,
            Env { multiplier: 3 },
            imported_fn,
        );

        let expected = vec![Val::I32(12)].into_boxed_slice();
        assert_eq!(imported.call(&[Val::I32(4)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_function_with_wasmer_env() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (func $imported (import "env" "imported") (param i32) (result i32))
        (func (export "exported") (param i32) (result i32)
            (call $imported (local.get 0))
        )
        (memory (export "memory") 1)
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Function(FunctionType::new(
                    vec![Type::I32],
                    vec![Type::I32],
                ))],
                exports: vec![
                    ExternType::Function(FunctionType::new(vec![Type::I32], vec![Type::I32])),
                    ExternType::Memory(MemoryType::new(Pages(1), None, false)),
                ],
            })
            .unwrap();

        #[derive(WasmerEnv, Clone)]
        struct Env {
            multiplier: u32,
            #[wasmer(export)]
            memory: LazyInit<Memory>,
        }

        fn imported_fn(env: &Env, args: &[Val]) -> Result<Vec<Val>, RuntimeError> {
            let memory = env.memory_ref().unwrap();
            let memory_val = memory.uint8view().get_index(0);
            let value = (memory_val as u32) * env.multiplier * args[0].unwrap_i32() as u32;
            return Ok(vec![Val::I32(value as _)]);
        }

        let imported_signature = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let imported = Function::new_with_env(
            &store,
            imported_signature,
            Env {
                multiplier: 3,
                memory: LazyInit::new(),
            },
            imported_fn,
        );

        let import_object = imports! {
            "env" => {
                "imported" => imported,
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let memory = instance.exports.get_memory("memory").unwrap();
        assert_eq!(memory.data_size(), 65536);
        let memory_val = memory.uint8view().get_index(0);
        assert_eq!(memory_val, 0);

        memory.uint8view().set_index(0, 2);
        let memory_val = memory.uint8view().get_index(0);
        assert_eq!(memory_val, 2);

        let exported = instance.exports.get_function("exported").unwrap();

        // It works with the provided memory
        let expected = vec![Val::I32(24)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));

        // It works if we update the memory
        memory.uint8view().set_index(0, 3);
        let expected = vec![Val::I32(36)].into_boxed_slice();
        assert_eq!(exported.call(&[Val::I32(4)]), Ok(expected));
    }

    #[wasm_bindgen_test]
    fn test_imported_exported_global() {
        let store = Store::default();
        let mut module = Module::new(
            &store,
            br#"
    (module
        (global $mut_i32_import (import "" "global") (mut i32))
        (func (export "getGlobal") (result i32) (global.get $mut_i32_import))
        (func (export "incGlobal") (global.set $mut_i32_import (
            i32.add (i32.const 1) (global.get $mut_i32_import)
        )))
    )
    "#,
        )
        .unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                imports: vec![ExternType::Global(GlobalType::new(
                    ValType::I32,
                    Mutability::Var,
                ))],
                exports: vec![
                    ExternType::Function(FunctionType::new(vec![], vec![Type::I32])),
                    ExternType::Function(FunctionType::new(vec![], vec![])),
                ],
            })
            .unwrap();
        let global = Global::new_mut(&store, Value::I32(0));
        let import_object = imports! {
            "" => {
                "global" => global.clone()
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let get_global = instance.exports.get_function("getGlobal").unwrap();
        assert_eq!(
            get_global.call(&[]),
            Ok(vec![Val::I32(0)].into_boxed_slice())
        );

        global.set(Value::I32(42)).unwrap();
        assert_eq!(
            get_global.call(&[]),
            Ok(vec![Val::I32(42)].into_boxed_slice())
        );

        let inc_global = instance.exports.get_function("incGlobal").unwrap();
        inc_global.call(&[]).unwrap();
        assert_eq!(
            get_global.call(&[]),
            Ok(vec![Val::I32(43)].into_boxed_slice())
        );
        assert_eq!(global.get(), Val::I32(43));
    }

    #[wasm_bindgen_test]
    fn test_native_function() {
        let store = Store::default();
        let module = Module::new(
            &store,
            br#"(module
            (func $add (import "env" "sum") (param i32 i32) (result i32))
            (func (export "add_one") (param i32) (result i32)
                (call $add (local.get 0) (i32.const 1))
            )
        )"#,
        )
        .unwrap();

        fn sum(a: i32, b: i32) -> i32 {
            a + b
        }

        let import_object = imports! {
            "env" => {
                "sum" => Function::new_native(&store, sum),
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let add_one: NativeFunc<i32, i32> =
            instance.exports.get_native_function("add_one").unwrap();
        assert_eq!(add_one.call(1), Ok(2));
    }

    #[wasm_bindgen_test]
    fn test_panic() {
        let store = Store::default();
        let module = Module::new(
            &store,
            br#"
(module
  (type $run_t (func (param i32 i32) (result i32)))
  (type $early_exit_t (func (param) (result)))
  (import "env" "early_exit" (func $early_exit (type $early_exit_t)))
  (func $run (type $run_t) (param $x i32) (param $y i32) (result i32)
    (call $early_exit)
    (i32.add
        local.get $x
        local.get $y))
  (export "run" (func $run)))
"#,
        )
        .unwrap();

        fn early_exit() {
            panic!("Do panic")
        }

        let import_object = imports! {
            "env" => {
                "early_exit" => Function::new_native(&store, early_exit),
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        let run_func: NativeFunc<(i32, i32), i32> =
            instance.exports.get_native_function("run").unwrap();

        assert!(run_func.call(1, 7).is_err(), "Expected early termination",);
        let run_func = instance.exports.get_function("run").unwrap();

        assert!(
            run_func.call(&[Val::I32(1), Val::I32(7)]).is_err(),
            "Expected early termination",
        );
    }

    #[wasm_bindgen_test]
    fn test_custom_error() {
        let store = Store::default();
        let module = Module::new(
            &store,
            br#"
(module
  (type $run_t (func (param i32 i32) (result i32)))
  (type $early_exit_t (func (param) (result)))
  (import "env" "early_exit" (func $early_exit (type $early_exit_t)))
  (func $run (type $run_t) (param $x i32) (param $y i32) (result i32)
    (call $early_exit)
    (i32.add
        local.get $x
        local.get $y))
  (export "run" (func $run)))
"#,
        )
        .unwrap();

        use std::fmt;

        #[derive(Debug, Clone, Copy)]
        struct ExitCode(u32);

        impl fmt::Display for ExitCode {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::error::Error for ExitCode {}

        fn early_exit() -> Result<(), ExitCode> {
            Err(ExitCode(1))
        }

        let import_object = imports! {
            "env" => {
                "early_exit" => Function::new_native(&store, early_exit),
            }
        };
        let instance = Instance::new(&module, &import_object).unwrap();

        fn test_result<T: core::fmt::Debug>(result: Result<T, RuntimeError>) {
            match result {
                Ok(result) => {
                    assert!(
                        false,
                        "Expected early termination with `ExitCode`, found: {:?}",
                        result
                    );
                }
                Err(e) => {
                    match e.downcast::<ExitCode>() {
                        // We found the exit code used to terminate execution.
                        Ok(exit_code) => {
                            assert_eq!(exit_code.0, 1);
                        }
                        Err(e) => {
                            assert!(false, "Unknown error `{:?}` found. expected `ErrorCode`", e);
                        }
                    }
                }
            }
        }

        let run_func: NativeFunc<(i32, i32), i32> =
            instance.exports.get_native_function("run").unwrap();
        test_result(run_func.call(1, 7));

        let run_func = instance.exports.get_function("run").unwrap();
        test_result(run_func.call(&[Val::I32(1), Val::I32(7)]));
    }

    #[wasm_bindgen_test]
    fn test_start_function_fails() {
        let store = Store::default();
        let module = Module::new(
            &store,
            br#"
    (module
        (func $start_function
            (i32.div_u
                (i32.const 1)
                (i32.const 0)
            )
            drop
        )
        (start $start_function)
    )
    "#,
        )
        .unwrap();

        let import_object = imports! {};
        let result = Instance::new(&module, &import_object);
        let err = result.unwrap_err();
        assert!(format!("{:?}", err).contains("zero"))
    }
}
