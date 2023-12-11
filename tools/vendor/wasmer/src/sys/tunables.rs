pub use wasmer_compiler::BaseTunables;

// All BaseTunable definition now is in wasmer_compile crate
// Tests are still here

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused)]
    use crate::sys::NativeEngineExt;
    use crate::TableType;
    use std::cell::UnsafeCell;
    use std::ptr::NonNull;
    use wasmer_compiler::Tunables;
    use wasmer_types::{MemoryType, Pages, WASM_PAGE_SIZE};
    use wasmer_vm::{
        LinearMemory, MemoryError, MemoryStyle, TableStyle, VMConfig, VMMemory, VMMemoryDefinition,
        VMTable, VMTableDefinition,
    };

    #[test]
    fn memory_style() {
        let tunables = BaseTunables {
            static_memory_bound: Pages(2048),
            static_memory_offset_guard_size: 128,
            dynamic_memory_offset_guard_size: 256,
        };

        // No maximum
        let requested = MemoryType::new(3, None, true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Dynamic { offset_guard_size } => assert_eq!(offset_guard_size, 256),
            s => panic!("Unexpected memory style: {:?}", s),
        }

        // Large maximum
        let requested = MemoryType::new(3, Some(5_000_000), true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Dynamic { offset_guard_size } => assert_eq!(offset_guard_size, 256),
            s => panic!("Unexpected memory style: {:?}", s),
        }

        // Small maximum
        let requested = MemoryType::new(3, Some(16), true);
        let style = tunables.memory_style(&requested);
        match style {
            MemoryStyle::Static {
                bound,
                offset_guard_size,
            } => {
                assert_eq!(bound, Pages(2048));
                assert_eq!(offset_guard_size, 128);
            }
            s => panic!("Unexpected memory style: {:?}", s),
        }
    }

    #[derive(Debug)]
    struct VMTinyMemory {
        mem: Vec<u8>,
        memory_definition: Option<UnsafeCell<VMMemoryDefinition>>,
    }

    unsafe impl Send for VMTinyMemory {}
    unsafe impl Sync for VMTinyMemory {}

    impl VMTinyMemory {
        pub fn new() -> Result<Self, MemoryError> {
            let sz = 18 * WASM_PAGE_SIZE;
            let mut memory = Vec::new();
            memory.resize(sz, 0);
            let mut ret = Self {
                mem: memory,
                memory_definition: None,
            };
            ret.memory_definition = Some(UnsafeCell::new(VMMemoryDefinition {
                base: ret.mem.as_ptr() as _,
                current_length: sz,
            }));
            Ok(ret)
        }
    }

    impl LinearMemory for VMTinyMemory {
        fn ty(&self) -> MemoryType {
            MemoryType {
                minimum: Pages::from(18u32),
                maximum: Some(Pages::from(18u32)),
                shared: false,
            }
        }
        fn size(&self) -> Pages {
            Pages::from(18u32)
        }
        fn style(&self) -> MemoryStyle {
            MemoryStyle::Static {
                bound: Pages::from(18u32),
                offset_guard_size: 0,
            }
        }
        fn grow(&mut self, delta: Pages) -> Result<Pages, MemoryError> {
            Err(MemoryError::CouldNotGrow {
                current: Pages::from(100u32),
                attempted_delta: delta,
            })
        }
        fn vmmemory(&self) -> NonNull<VMMemoryDefinition> {
            unsafe {
                NonNull::new(
                    self.memory_definition
                        .as_ref()
                        .unwrap()
                        .get()
                        .as_mut()
                        .unwrap() as _,
                )
                .unwrap()
            }
        }

        fn try_clone(&self) -> Result<Box<dyn LinearMemory + 'static>, MemoryError> {
            Err(MemoryError::InvalidMemory {
                reason: "VMTinyMemory can not be cloned".to_string(),
            })
        }

        fn copy(&mut self) -> Result<Box<dyn LinearMemory + 'static>, MemoryError> {
            let mem = self.mem.clone();
            Ok(Box::new(Self {
                memory_definition: Some(UnsafeCell::new(VMMemoryDefinition {
                    base: mem.as_ptr() as _,
                    current_length: mem.len(),
                })),
                mem,
            }))
        }
        /*
        // this code allow custom memory to be ignoring init_memory
        use wasmer_vm::Trap;
        unsafe fn initialize_with_data(&self, _start: usize, _data: &[u8]) -> Result<(), Trap> {
            Ok(())
        }
        */
    }

    impl From<VMTinyMemory> for wasmer_vm::VMMemory {
        fn from(mem: VMTinyMemory) -> Self {
            Self(Box::new(mem))
        }
    }

    struct TinyTunables;
    impl Tunables for TinyTunables {
        fn memory_style(&self, _memory: &MemoryType) -> MemoryStyle {
            MemoryStyle::Static {
                bound: Pages::from(18u32),
                offset_guard_size: 0,
            }
        }

        /// Construct a `TableStyle` for the provided `TableType`
        fn table_style(&self, _table: &TableType) -> TableStyle {
            TableStyle::CallerChecksSignature
        }
        fn create_host_memory(
            &self,
            _ty: &MemoryType,
            _style: &MemoryStyle,
        ) -> Result<VMMemory, MemoryError> {
            let memory = VMTinyMemory::new().unwrap();
            Ok(VMMemory::from_custom(memory))
        }
        unsafe fn create_vm_memory(
            &self,
            _ty: &MemoryType,
            _style: &MemoryStyle,
            vm_definition_location: NonNull<VMMemoryDefinition>,
        ) -> Result<VMMemory, MemoryError> {
            let memory = VMTinyMemory::new().unwrap();
            // now, it's important to update vm_definition_location with the memory information!
            let mut ptr = vm_definition_location;
            let md = ptr.as_mut();
            let unsafecell = memory.memory_definition.as_ref().unwrap();
            let def = unsafecell.get().as_ref().unwrap();
            md.base = def.base;
            md.current_length = def.current_length;
            Ok(memory.into())
        }

        /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
        fn create_host_table(&self, ty: &TableType, style: &TableStyle) -> Result<VMTable, String> {
            VMTable::new(ty, style)
        }

        /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
        ///
        /// # Safety
        /// - `vm_definition_location` must point to a valid location in VM memory.
        unsafe fn create_vm_table(
            &self,
            ty: &TableType,
            style: &TableStyle,
            vm_definition_location: NonNull<VMTableDefinition>,
        ) -> Result<VMTable, String> {
            VMTable::from_definition(ty, style, vm_definition_location)
        }

        // Will use a minimum stack size of 8kb, not the 1Mb default
        fn vmconfig(&self) -> &crate::vm::VMConfig {
            &VMConfig {
                wasm_stack_size: Some(8 * 1024),
            }
        }
    }

    #[test]
    fn check_linearmemory() {
        let tunables = TinyTunables {};
        let vmmemory = tunables.create_host_memory(
            &MemoryType::new(1u32, Some(100u32), true),
            &MemoryStyle::Static {
                bound: Pages::from(18u32),
                offset_guard_size: 0u64,
            },
        );
        let mut vmmemory = vmmemory.unwrap();
        assert!(vmmemory.grow(Pages::from(50u32)).is_err());
        assert_eq!(vmmemory.size(), Pages::from(18u32));
        assert_eq!(
            vmmemory.grow(Pages::from(0u32)).err().unwrap(),
            MemoryError::CouldNotGrow {
                current: Pages::from(100u32),
                attempted_delta: Pages::from(0u32)
            }
        );
    }

    #[test]
    fn check_customtunables() -> Result<(), Box<dyn std::error::Error>> {
        use crate::{imports, wat2wasm, Engine, Instance, Memory, Module, Store};
        use wasmer_compiler_cranelift::Cranelift;

        let wasm_bytes = wat2wasm(
            br#"(module
            (memory (;0;) 18)
            (global (;0;) (mut i32) i32.const 1048576)
            (export "memory" (memory 0))
            (data (;0;) (i32.const 1048576) "*\00\00\00")
          )"#,
        )?;
        let compiler = Cranelift::default();

        let tunables = TinyTunables {};
        #[allow(deprecated)]
        let mut engine = Engine::new(compiler.into(), Default::default(), Default::default());
        engine.set_tunables(tunables);
        let mut store = Store::new(engine);
        //let mut store = Store::new(compiler);
        let module = Module::new(&store, wasm_bytes)?;
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;

        let mut memories: Vec<Memory> = instance
            .exports
            .iter()
            .memories()
            .map(|pair| pair.1.clone())
            .collect();
        assert_eq!(memories.len(), 1);
        let first_memory = memories.pop().unwrap();
        assert_eq!(first_memory.ty(&store).maximum.unwrap(), Pages(18));
        let view = first_memory.view(&store);
        let x = unsafe { view.data_unchecked_mut() }[0];
        assert_eq!(x, 0);

        Ok(())
    }

    #[test]
    #[cfg(all(
        feature = "singlepass",
        not(any(
            target_os = "windows",
            all(target_os = "macos", target_arch = "aarch64")
        ))
    ))]
    fn check_small_stack() -> Result<(), Box<dyn std::error::Error>> {
        use crate::{imports, wat2wasm, Engine, Instance, Module, Store};
        use wasmer_compiler_singlepass::Singlepass;
        // This test needs Singlepass compiler
        // because Cranelift will optimize the webassembly file
        // and remove all the unused local, even at optimization level "None"
        // But this test needs the huge amount of locals (1024 + a few)
        // so that the small stack is overflown (stack is only 8K, 1024 i64 local = 8K)
        // tWindows is disable as it seems Stack frame protection is not 100% efficient
        let wasm_bytes = wat2wasm(
            br#"(module
                (func (;0;) (result i64)
                  (local i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64)
                  i64.const 0
                  i64.const 5555
                  i64.add
                  local.set 8
                  i64.const 0
                  i64.const 5555
                  i64.add
                  local.set 9
                  i64.const 0
                  i64.const 5555
                  i64.add
                  local.set 10
                  local.get 10
                )
                (func $large_local (export "large_local") (result i64)
                  (local
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64

                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64
                   i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64 i64

                   i64
                  )
                  (local.set 0 (i64.const 1))
                  (local.set 1 (i64.const 1))
                  (local.set 2 (i64.const 1))
                  (local.set 3 (i64.const 1))
                  (local.set 1024 (i64.const 2))
                  (call 0)
                  local.set 1024
                  local.get 6
                  local.get 7
                  i64.add
                  local.get 8
                  i64.add
                  (call 0)
                  local.set 10
                  local.get 9
                  i64.add
                  local.get 10
                  i64.add
                  local.get 11
                  i64.add
                  local.get 12
                  i64.add
                  (call 0)
                  local.set 512
                  local.get 13
                  i64.add
                  local.get 14
                  i64.add
                  local.get 15
                  i64.add
                  local.get 1024
                  i64.add
                  local.get 0
                  i64.add
                )
              )
            "#,
        )?;
        let compiler = Singlepass::default();

        let tunables = TinyTunables {};
        #[allow(deprecated)]
        let mut engine = Engine::new(compiler.into(), Default::default(), Default::default());
        engine.set_tunables(tunables);
        let mut store = Store::new(engine);
        let module = Module::new(&store, wasm_bytes)?;
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)?;

        let result = instance
            .exports
            .get_function("large_local")?
            .call(&mut store, &[]);

        println!("result = {:?}", result);
        assert!(result.is_err());

        Ok(())
    }
}
