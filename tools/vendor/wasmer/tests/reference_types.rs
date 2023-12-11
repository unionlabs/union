#[cfg(feature = "sys")]
pub mod reference_types {

    use anyhow::Result;
    use macro_wasmer_universal_test::universal_test;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    #[cfg(feature = "js")]
    use wasm_bindgen_test::*;
    use wasmer::*;

    #[universal_test]
    fn func_ref_passed_and_returned() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
(import "env" "func_ref_identity" (func (param funcref) (result funcref)))
(type $ret_i32_ty (func (result i32)))
(table $table (export "table") 2 2 funcref)

(func (export "run") (param) (result funcref)
      (call 0 (ref.null func)))
(func (export "call_set_value") (param $fr funcref) (result i32)
      (table.set $table (i32.const 0) (local.get $fr))
      (call_indirect $table (type $ret_i32_ty) (i32.const 0)))
)"#;
        let module = Module::new(&store, wat)?;
        #[derive(Clone, Debug)]
        pub struct Env(Arc<AtomicBool>);
        let env = Env(Arc::new(AtomicBool::new(false)));
        let env = FunctionEnv::new(&mut store, env);
        let imports = imports! {
            "env" => {
                "func_ref_identity" => Function::new_with_env(&mut store, &env, FunctionType::new([Type::FuncRef], [Type::FuncRef]), |_env: FunctionEnvMut<Env>, values: &[Value]| -> Result<Vec<_>, _> {
                    Ok(vec![values[0].clone()])
                })
            },
        };

        let instance = Instance::new(&mut store, &module, &imports)?;

        let f: &Function = instance.exports.get_function("run")?;
        let results = f.call(&mut store, &[]).unwrap();
        if let Value::FuncRef(fr) = &results[0] {
            assert!(fr.is_none());
        } else {
            panic!("funcref not found!");
        }

        let func_to_call =
            Function::new_typed_with_env(&mut store, &env, |env: FunctionEnvMut<Env>| -> i32 {
                env.data().0.store(true, Ordering::SeqCst);
                343
            });
        let call_set_value: &Function = instance.exports.get_function("call_set_value")?;
        let results: Box<[Value]> =
            call_set_value.call(&mut store, &[Value::FuncRef(Some(func_to_call))])?;
        assert!(env.as_ref(&store.as_store_ref()).0.load(Ordering::SeqCst));
        assert_eq!(&*results, &[Value::I32(343)]);

        Ok(())
    }

    #[universal_test]
    fn func_ref_passed_and_called() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
(func $func_ref_call (import "env" "func_ref_call") (param funcref) (result i32))
(type $ret_i32_ty (func (result i32)))
(table $table (export "table") 2 2 funcref)

(func $product (param $x i32) (param $y i32) (result i32)
      (i32.mul (local.get $x) (local.get $y)))
;; TODO: figure out exactly why this statement is needed
(elem declare func $product)
(func (export "call_set_value") (param $fr funcref) (result i32)
      (table.set $table (i32.const 0) (local.get $fr))
      (call_indirect $table (type $ret_i32_ty) (i32.const 0)))
(func (export "call_func") (param $fr funcref) (result i32)
      (call $func_ref_call (local.get $fr)))
(func (export "call_host_func_with_wasm_func") (result i32)
      (call $func_ref_call (ref.func $product)))
)"#;
        let module = Module::new(&store, wat)?;
        let env = FunctionEnv::new(&mut store, ());
        fn func_ref_call(
            mut env: FunctionEnvMut<()>,
            values: &[Value],
        ) -> Result<Vec<Value>, RuntimeError> {
            // TODO: look into `Box<[Value]>` being returned breakage
            let f = values[0].unwrap_funcref().as_ref().unwrap();
            let f: TypedFunction<(i32, i32), i32> = f.typed(&mut env)?;
            Ok(vec![Value::I32(f.call(&mut env, 7, 9)?)])
        }

        let imports = imports! {
            "env" => {
                "func_ref_call" => Function::new_with_env(
                    &mut store,
                    &env,
                    FunctionType::new([Type::FuncRef], [Type::I32]),
                    func_ref_call
                ),
                // "func_ref_call_native" => Function::new_native(&mut store, |f: Function| -> Result<i32, RuntimeError> {
                //     let f: TypedFunction::<(i32, i32), i32> = f.typed(&mut store)?;
                //     f.call(&mut store, 7, 9)
                // })
            },
        };

        let instance = Instance::new(&mut store, &module, &imports)?;
        {
            fn sum(a: i32, b: i32) -> i32 {
                a + b
            }
            let sum_func = Function::new_typed(&mut store, sum);

            let call_func: &Function = instance.exports.get_function("call_func")?;
            let result = call_func.call(&mut store, &[Value::FuncRef(Some(sum_func))])?;
            assert_eq!(result[0].unwrap_i32(), 16);
        }

        {
            let f: TypedFunction<(), i32> = instance
                .exports
                .get_typed_function(&mut store, "call_host_func_with_wasm_func")?;
            let result = f.call(&mut store)?;
            assert_eq!(result, 63);
        }

        Ok(())
    }

    #[macro_wasmer_universal_test::universal_test]
    fn extern_ref_passed_and_returned() -> Result<()> {
        use std::collections::HashMap;
        let mut store = Store::default();
        let wat = r#"(module
    (func $extern_ref_identity (import "env" "extern_ref_identity") (param externref) (result externref))
    (func $extern_ref_identity_native (import "env" "extern_ref_identity_native") (param externref) (result externref))
    (func $get_new_extern_ref (import "env" "get_new_extern_ref") (result externref))
    (func $get_new_extern_ref_native (import "env" "get_new_extern_ref_native") (result externref))

    (func (export "run") (param) (result externref)
          (call $extern_ref_identity (ref.null extern)))
    (func (export "run_native") (param) (result externref)
          (call $extern_ref_identity_native (ref.null extern)))
    (func (export "get_hashmap") (param) (result externref)
          (call $get_new_extern_ref))
    (func (export "get_hashmap_native") (param) (result externref)
          (call $get_new_extern_ref_native))
)"#;
        let module = Module::new(&store, wat)?;
        let env = FunctionEnv::new(&mut store, ());
        let imports = imports! {
            "env" => {
                "extern_ref_identity" => Function::new_with_env(&mut store, &env, FunctionType::new([Type::ExternRef], [Type::ExternRef]), |_env, values| -> Result<Vec<_>, _> {
                    Ok(vec![values[0].clone()])
                }),
                "extern_ref_identity_native" => Function::new_typed(&mut store, |er: Option<ExternRef>| -> Option<ExternRef> {
                    er
                }),
                "get_new_extern_ref" => Function::new_with_env(&mut store, &env, FunctionType::new([], [Type::ExternRef]), |mut env, _| -> Result<Vec<_>, _> {
                    let inner =
                        [("hello".to_string(), "world".to_string()),
                         ("color".to_string(), "orange".to_string())]
                        .iter()
                        .cloned()
                        .collect::<HashMap<String, String>>();
                    let new_extern_ref = ExternRef::new(&mut env, inner);
                    Ok(vec![Value::ExternRef(Some(new_extern_ref))])
                }),
                "get_new_extern_ref_native" => Function::new_typed_with_env(&mut store, &env, |mut env: FunctionEnvMut<()>| -> Option<ExternRef> {
                    let inner =
                        [("hello".to_string(), "world".to_string()),
                         ("color".to_string(), "orange".to_string())]
                        .iter()
                        .cloned()
                        .collect::<HashMap<String, String>>();
                    Some(ExternRef::new(&mut env.as_store_mut(), inner))
                })
            },
        };

        let instance = Instance::new(&mut store, &module, &imports)?;
        for run in &["run", "run_native"] {
            let f: &Function = instance.exports.get_function(run)?;
            let results = f.call(&mut store, &[]).unwrap();
            if let Value::ExternRef(er) = &results[0] {
                assert!(er.is_none());
            } else {
                panic!("result is not an extern ref!");
            }

            let f: TypedFunction<(), Option<ExternRef>> =
                instance.exports.get_typed_function(&store, run)?;
            let result: Option<ExternRef> = f.call(&mut store)?;
            assert!(result.is_none());
        }

        for get_hashmap in &["get_hashmap", "get_hashmap_native"] {
            let f: &Function = instance.exports.get_function(get_hashmap)?;
            let results = f.call(&mut store, &[]).unwrap();
            if let Value::ExternRef(er) = &results[0] {
                let inner: &HashMap<String, String> =
                    er.as_ref().unwrap().downcast(&mut store).unwrap();
                assert_eq!(inner["hello"], "world");
                assert_eq!(inner["color"], "orange");
            } else {
                panic!("result is not an extern ref!");
            }

            let f: TypedFunction<(), Option<ExternRef>> =
                instance.exports.get_typed_function(&store, get_hashmap)?;

            let result: Option<ExternRef> = f.call(&mut store)?;
            let inner: &HashMap<String, String> = result.unwrap().downcast(&mut store).unwrap();
            assert_eq!(inner["hello"], "world");
            assert_eq!(inner["color"], "orange");
        }

        Ok(())
    }

    #[universal_test]
    fn extern_ref_ref_counting_basic() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (func (export "drop") (param $er externref) (result)
          (drop (local.get $er)))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;
        let f: TypedFunction<Option<ExternRef>, ()> =
            instance.exports.get_typed_function(&store, "drop")?;

        let er = ExternRef::new(&mut store, 3u32);
        f.call(&mut store, Some(er.clone()))?;

        let tmp: Option<&u32> = er.downcast(&mut store);
        assert_eq!(tmp.unwrap(), &3);

        Ok(())
    }

    #[universal_test]
    fn refs_in_globals() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (global $er_global (export "er_global") (mut externref) (ref.null extern))
    (global $fr_global (export "fr_global") (mut funcref) (ref.null func))
    (global $fr_immutable_global (export "fr_immutable_global") funcref (ref.func $hello))
    (func $hello (param) (result i32)
          (i32.const 73))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;
        {
            let er_global: &Global = instance.exports.get_global("er_global")?;

            if let Value::ExternRef(er) = er_global.get(&mut store) {
                assert!(er.is_none());
            } else {
                panic!("Did not find extern ref in the global");
            }
            let extref = Some(ExternRef::new(&mut store, 3u32));
            er_global.set(&mut store, Value::ExternRef(extref))?;

            if let Value::ExternRef(er) = er_global.get(&mut store) {
                let tmp: Option<&u32> = er.unwrap().downcast(&mut store);
                assert_eq!(tmp.unwrap(), &3);
            } else {
                panic!("Did not find extern ref in the global");
            }
        }

        {
            let fr_global: &Global = instance.exports.get_global("fr_immutable_global")?;

            if let Value::FuncRef(Some(f)) = fr_global.get(&mut store) {
                let native_func: TypedFunction<(), u32> = f.typed(&mut store)?;
                assert_eq!(native_func.call(&mut store)?, 73);
            } else {
                panic!("Did not find non-null func ref in the global");
            }
        }

        {
            let fr_global: &Global = instance.exports.get_global("fr_global")?;

            if let Value::FuncRef(None) = fr_global.get(&mut store) {
            } else {
                panic!("Did not find a null func ref in the global");
            }

            let f = Function::new_typed(&mut store, |arg1: i32, arg2: i32| -> i32 { arg1 + arg2 });

            fr_global.set(&mut store, Value::FuncRef(Some(f)))?;

            if let Value::FuncRef(Some(f)) = fr_global.get(&mut store) {
                let native: TypedFunction<(i32, i32), i32> = f.typed(&mut store)?;
                assert_eq!(native.call(&mut store, 5, 7)?, 12);
            } else {
                panic!("Did not find extern ref in the global");
            }
        }

        Ok(())
    }

    #[universal_test]
    fn extern_ref_ref_counting_table_basic() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (global $global (export "global") (mut externref) (ref.null extern))
    (table $table (export "table") 4 4 externref)
    (func $insert (param $er externref) (param $idx i32)
           (table.set $table (local.get $idx) (local.get $er)))
    (func $intermediate (param $er externref) (param $idx i32)
          (call $insert (local.get $er) (local.get $idx)))
    (func $insert_into_table (export "insert_into_table") (param $er externref) (param $idx i32) (result externref)
          (call $intermediate (local.get $er) (local.get $idx))
          (local.get $er))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;

        let f: TypedFunction<(Option<ExternRef>, i32), Option<ExternRef>> = instance
            .exports
            .get_typed_function(&mut store, "insert_into_table")?;

        let er = ExternRef::new(&mut store, 3usize);

        let er = f.call(&mut store, Some(er), 1)?;
        assert!(er.is_some());

        let table: &Table = instance.exports.get_table("table")?;

        {
            let er2 = table.get(&mut store, 1).unwrap();
            let er2 = er2.externref().unwrap();
            assert!(er2.is_some());
        }

        assert!(er.is_some());
        table.set(&mut store, 1, Value::ExternRef(None))?;

        assert!(er.is_some());

        Ok(())
    }

    #[universal_test]
    fn extern_ref_ref_counting_global_basic() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (global $global (export "global") (mut externref) (ref.null extern))
    (func $get_from_global (export "get_from_global") (result externref)
          (drop (global.get $global))
          (global.get $global))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;

        let global: &Global = instance.exports.get_global("global")?;
        {
            let er = ExternRef::new(&mut store, 3usize);
            global.set(&mut store, Value::ExternRef(Some(er)))?;
        }
        let get_from_global: TypedFunction<(), Option<ExternRef>> = instance
            .exports
            .get_typed_function(&mut store, "get_from_global")?;

        let er = get_from_global.call(&mut store)?;
        assert!(er.is_some());
        global.set(&mut store, Value::ExternRef(None))?;
        assert!(er.is_some());

        Ok(())
    }

    #[universal_test]
    fn extern_ref_ref_counting_traps() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (func $pass_er (export "pass_extern_ref") (param externref)
          (local.get 0)
          (unreachable))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;

        let pass_extern_ref: TypedFunction<Option<ExternRef>, ()> = instance
            .exports
            .get_typed_function(&mut store, "pass_extern_ref")?;

        let er = ExternRef::new(&mut store, 3usize);

        let result = pass_extern_ref.call(&mut store, Some(er));
        assert!(result.is_err());

        Ok(())
    }

    #[universal_test]
    fn extern_ref_ref_counting_table_instructions() -> Result<()> {
        let mut store = Store::default();
        let wat = r#"(module
    (table $table1 (export "table1") 2 12 externref)
    (table $table2 (export "table2") 6 12 externref)
    (func $grow_table_with_ref (export "grow_table_with_ref") (param $er externref) (param $size i32) (result i32)
          (table.grow $table1 (local.get $er) (local.get $size)))
    (func $fill_table_with_ref (export "fill_table_with_ref") (param $er externref) (param $start i32) (param $end i32)
          (table.fill $table1 (local.get $start) (local.get $er) (local.get $end)))
    (func $copy_into_table2 (export "copy_into_table2")
          (table.copy $table2 $table1 (i32.const 0) (i32.const 0) (i32.const 4)))
)"#;
        let module = Module::new(&store, wat)?;
        let instance = Instance::new(&mut store, &module, &imports! {})?;

        let grow_table_with_ref: TypedFunction<(Option<ExternRef>, i32), i32> = instance
            .exports
            .get_typed_function(&mut store, "grow_table_with_ref")?;
        let fill_table_with_ref: TypedFunction<(Option<ExternRef>, i32, i32), ()> = instance
            .exports
            .get_typed_function(&mut store, "fill_table_with_ref")?;
        let copy_into_table2: TypedFunction<(), ()> = instance
            .exports
            .get_typed_function(&mut store, "copy_into_table2")?;
        let table1: &Table = instance.exports.get_table("table1")?;
        let table2: &Table = instance.exports.get_table("table2")?;

        let er1 = ExternRef::new(&mut store, 3usize);
        let er2 = ExternRef::new(&mut store, 5usize);
        let er3 = ExternRef::new(&mut store, 7usize);
        {
            let result = grow_table_with_ref.call(&mut store, Some(er1.clone()), 0)?;
            assert_eq!(result, 2);

            let result = grow_table_with_ref.call(&mut store, Some(er1.clone()), 10_000)?;
            assert_eq!(result, -1);

            let result = grow_table_with_ref.call(&mut store, Some(er1), 8)?;
            assert_eq!(result, 2);

            for i in 2..10 {
                let v = table1.get(&mut store, i);
                let e = v.as_ref().unwrap().unwrap_externref();
                let e_val: Option<&usize> = e.as_ref().unwrap().downcast(&mut store);
                assert_eq!(*e_val.unwrap(), 3);
            }
        }

        {
            fill_table_with_ref.call(&mut store, Some(er2), 0, 2)?;
        }

        {
            table2.set(&mut store, 0, Value::ExternRef(Some(er3.clone())))?;
            table2.set(&mut store, 1, Value::ExternRef(Some(er3.clone())))?;
            table2.set(&mut store, 2, Value::ExternRef(Some(er3.clone())))?;
            table2.set(&mut store, 3, Value::ExternRef(Some(er3.clone())))?;
            table2.set(&mut store, 4, Value::ExternRef(Some(er3)))?;
        }

        {
            copy_into_table2.call(&mut store)?;
            for i in 1..5 {
                let v = table2.get(&mut store, i);
                let e = v.as_ref().unwrap().unwrap_externref();
                let value: &usize = e.as_ref().unwrap().downcast(&mut store).unwrap();
                match i {
                    0 | 1 => assert_eq!(*value, 5),
                    4 => assert_eq!(*value, 7),
                    _ => assert_eq!(*value, 3),
                }
            }
        }

        {
            for i in 0..table1.size(&mut store) {
                table1.set(&mut store, i, Value::ExternRef(None))?;
            }
            for i in 0..table2.size(&mut store) {
                table2.set(&mut store, i, Value::ExternRef(None))?;
            }
        }

        Ok(())
    }
}
