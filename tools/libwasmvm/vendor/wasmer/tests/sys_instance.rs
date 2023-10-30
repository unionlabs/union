#[cfg(feature = "sys")]
mod sys {
    use anyhow::Result;
    use wasmer::*;

    #[test]
    fn exports_work_after_multiple_instances_have_been_freed() -> Result<()> {
        let store = Store::default();
        let module = Module::new(
            &store,
            "
    (module
      (type $sum_t (func (param i32 i32) (result i32)))
      (func $sum_f (type $sum_t) (param $x i32) (param $y i32) (result i32)
        local.get $x
        local.get $y
        i32.add)
      (export \"sum\" (func $sum_f)))
",
        )?;

        let import_object = ImportObject::new();
        let instance = Instance::new(&module, &import_object)?;
        let instance2 = instance.clone();
        let instance3 = instance.clone();

        // The function is cloned to “break” the connection with `instance`.
        let sum = instance.exports.get_function("sum")?.clone();

        drop(instance);
        drop(instance2);
        drop(instance3);

        // All instances have been dropped, but `sum` continues to work!
        assert_eq!(
            sum.call(&[Value::I32(1), Value::I32(2)])?.into_vec(),
            vec![Value::I32(3)],
        );

        Ok(())
    }

    #[test]
    fn unit_native_function_env() -> Result<()> {
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
        let result = imported.call(&[Val::I32(4)])?;
        assert_eq!(result, expected);

        Ok(())
    }
}
