#[cfg(feature = "js")]
mod js {
    use js_sys::{Uint8Array, WebAssembly};
    use wasm_bindgen_test::*;
    use wasmer::*;

    #[wasm_bindgen_test]
    fn module_get_name() {
        let store = Store::default();
        let wat = r#"(module)"#;
        let module = Module::new(&store, wat).unwrap();
        assert_eq!(module.name(), None);
    }

    #[wasm_bindgen_test]
    fn module_set_name() {
        let store = Store::default();
        let wat = r#"(module $name)"#;
        let mut module = Module::new(&store, wat).unwrap();

        #[cfg(feature = "wasm-types-polyfill")]
        assert_eq!(module.name(), Some("name"));

        module.set_name("new_name");
        assert_eq!(module.name(), Some("new_name"));
    }

    #[wasm_bindgen_test]
    fn module_from_jsmodule() {
        let wat = br#"(module $name)"#;
        let binary = wat2wasm(wat).unwrap();
        let js_bytes = unsafe { Uint8Array::view(&binary) };
        let js_module = WebAssembly::Module::new(&js_bytes.into()).unwrap();
        let module: Module = js_module.into();
        assert_eq!(module.store(), &Store::default());
    }

    #[wasm_bindgen_test]
    fn imports() {
        let store = Store::default();
        let wat = r#"(module
    (import "host" "func" (func))
    (import "host" "memory" (memory 1))
    (import "host" "table" (table 1 anyfunc))
    (import "host" "global" (global i32))
)"#;
        let module = Module::new(&store, wat).unwrap();
        assert_eq!(
            module.imports().collect::<Vec<_>>(),
            vec![
                ImportType::new(
                    "host",
                    "func",
                    ExternType::Function(FunctionType::new(vec![], vec![]))
                ),
                ImportType::new(
                    "host",
                    "memory",
                    ExternType::Memory(MemoryType::new(Pages(1), None, false))
                ),
                ImportType::new(
                    "host",
                    "table",
                    ExternType::Table(TableType::new(Type::FuncRef, 1, None))
                ),
                ImportType::new(
                    "host",
                    "global",
                    ExternType::Global(GlobalType::new(Type::I32, Mutability::Const))
                )
            ]
        );

        // Now we test the iterators
        assert_eq!(
            module.imports().functions().collect::<Vec<_>>(),
            vec![ImportType::new(
                "host",
                "func",
                FunctionType::new(vec![], vec![])
            ),]
        );
        assert_eq!(
            module.imports().memories().collect::<Vec<_>>(),
            vec![ImportType::new(
                "host",
                "memory",
                MemoryType::new(Pages(1), None, false)
            ),]
        );
        assert_eq!(
            module.imports().tables().collect::<Vec<_>>(),
            vec![ImportType::new(
                "host",
                "table",
                TableType::new(Type::FuncRef, 1, None)
            ),]
        );
        assert_eq!(
            module.imports().globals().collect::<Vec<_>>(),
            vec![ImportType::new(
                "host",
                "global",
                GlobalType::new(Type::I32, Mutability::Const)
            ),]
        );
    }

    #[wasm_bindgen_test]
    fn exports() {
        let store = Store::default();
        let wat = r#"(module
    (func (export "func") nop)
    (memory (export "memory") 2)
    (table (export "table") 2 funcref)
    (global (export "global") i32 (i32.const 0))
)"#;
        let mut module = Module::new(&store, wat).unwrap();
        module
            .set_type_hints(ModuleTypeHints {
                exports: vec![
                    ExternType::Function(FunctionType::new(vec![], vec![])),
                    ExternType::Memory(MemoryType::new(Pages(2), None, false)),
                    ExternType::Table(TableType::new(Type::FuncRef, 2, None)),
                    ExternType::Global(GlobalType::new(Type::I32, Mutability::Const)),
                ],
                imports: vec![],
            })
            .unwrap();
        assert_eq!(
            module.exports().collect::<Vec<_>>(),
            vec![
                ExportType::new(
                    "func",
                    ExternType::Function(FunctionType::new(vec![], vec![]))
                ),
                ExportType::new(
                    "memory",
                    ExternType::Memory(MemoryType::new(Pages(2), None, false))
                ),
                ExportType::new(
                    "table",
                    ExternType::Table(TableType::new(Type::FuncRef, 2, None))
                ),
                ExportType::new(
                    "global",
                    ExternType::Global(GlobalType::new(Type::I32, Mutability::Const))
                )
            ]
        );

        // Now we test the iterators
        assert_eq!(
            module.exports().functions().collect::<Vec<_>>(),
            vec![ExportType::new("func", FunctionType::new(vec![], vec![])),]
        );
        assert_eq!(
            module.exports().memories().collect::<Vec<_>>(),
            vec![ExportType::new(
                "memory",
                MemoryType::new(Pages(2), None, false)
            ),]
        );
        assert_eq!(
            module.exports().tables().collect::<Vec<_>>(),
            vec![ExportType::new(
                "table",
                TableType::new(Type::FuncRef, 2, None)
            ),]
        );
        assert_eq!(
            module.exports().globals().collect::<Vec<_>>(),
            vec![ExportType::new(
                "global",
                GlobalType::new(Type::I32, Mutability::Const)
            ),]
        );
    }

    // Test commented because it doesn't work in old versions of Node
    // which makes the CI to fail.

    // #[wasm_bindgen_test]
    // fn calling_host_functions_with_negative_values_works() {
    //     let store = Store::default();
    //     let wat = r#"(module
    //     (import "host" "host_func1" (func (param i64)))
    //     (import "host" "host_func2" (func (param i32)))
    //     (import "host" "host_func3" (func (param i64)))
    //     (import "host" "host_func4" (func (param i32)))
    //     (import "host" "host_func5" (func (param i32)))
    //     (import "host" "host_func6" (func (param i32)))
    //     (import "host" "host_func7" (func (param i32)))
    //     (import "host" "host_func8" (func (param i32)))

    //     (func (export "call_host_func1")
    //           (call 0 (i64.const -1)))
    //     (func (export "call_host_func2")
    //           (call 1 (i32.const -1)))
    //     (func (export "call_host_func3")
    //           (call 2 (i64.const -1)))
    //     (func (export "call_host_func4")
    //           (call 3 (i32.const -1)))
    //     (func (export "call_host_func5")
    //           (call 4 (i32.const -1)))
    //     (func (export "call_host_func6")
    //           (call 5 (i32.const -1)))
    //     (func (export "call_host_func7")
    //           (call 6 (i32.const -1)))
    //     (func (export "call_host_func8")
    //           (call 7 (i32.const -1)))
    // )"#;
    //     let module = Module::new(&store, wat).unwrap();
    //     let imports = imports! {
    //         "host" => {
    //             "host_func1" => Function::new_native(&store, |p: u64| {
    //                 println!("host_func1: Found number {}", p);
    //                 assert_eq!(p, u64::max_value());
    //             }),
    //             "host_func2" => Function::new_native(&store, |p: u32| {
    //                 println!("host_func2: Found number {}", p);
    //                 assert_eq!(p, u32::max_value());
    //             }),
    //             "host_func3" => Function::new_native(&store, |p: i64| {
    //                 println!("host_func3: Found number {}", p);
    //                 assert_eq!(p, -1);
    //             }),
    //             "host_func4" => Function::new_native(&store, |p: i32| {
    //                 println!("host_func4: Found number {}", p);
    //                 assert_eq!(p, -1);
    //             }),
    //             "host_func5" => Function::new_native(&store, |p: i16| {
    //                 println!("host_func5: Found number {}", p);
    //                 assert_eq!(p, -1);
    //             }),
    //             "host_func6" => Function::new_native(&store, |p: u16| {
    //                 println!("host_func6: Found number {}", p);
    //                 assert_eq!(p, u16::max_value());
    //             }),
    //             "host_func7" => Function::new_native(&store, |p: i8| {
    //                 println!("host_func7: Found number {}", p);
    //                 assert_eq!(p, -1);
    //             }),
    //             "host_func8" => Function::new_native(&store, |p: u8| {
    //                 println!("host_func8: Found number {}", p);
    //                 assert_eq!(p, u8::max_value());
    //             }),
    //         }
    //     };
    //     let instance = Instance::new(&module, &imports).unwrap();

    //     let f1: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func1")
    //         .unwrap();
    //     let f2: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func2")
    //         .unwrap();
    //     let f3: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func3")
    //         .unwrap();
    //     let f4: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func4")
    //         .unwrap();
    //     let f5: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func5")
    //         .unwrap();
    //     let f6: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func6")
    //         .unwrap();
    //     let f7: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func7")
    //         .unwrap();
    //     let f8: NativeFunc<(), ()> = instance
    //         .exports
    //         .get_native_function("call_host_func8")
    //         .unwrap();

    //     f1.call().unwrap();
    //     f2.call().unwrap();
    //     f3.call().unwrap();
    //     f4.call().unwrap();
    //     f5.call().unwrap();
    //     f6.call().unwrap();
    //     f7.call().unwrap();
    //     f8.call().unwrap();
    // }
}
