//! Custom resolution for external references.

use crate::LinkError;
use more_asserts::assert_ge;
use wasmer_types::entity::{BoxedSlice, EntityRef, PrimaryMap};
use wasmer_types::{
    ExternType, FunctionIndex, ImportError, ImportIndex, MemoryIndex, ModuleInfo, TableIndex,
};

use wasmer_vm::{
    FunctionBodyPtr, Imports, LinearMemory, MemoryStyle, StoreObjects, TableStyle, VMExtern,
    VMFunctionBody, VMFunctionImport, VMFunctionKind, VMGlobalImport, VMMemoryImport,
    VMTableImport,
};

/// Get an `ExternType` given a import index.
fn get_extern_from_import(module: &ModuleInfo, import_index: &ImportIndex) -> ExternType {
    match import_index {
        ImportIndex::Function(index) => {
            let func = module.signatures[module.functions[*index]].clone();
            ExternType::Function(func)
        }
        ImportIndex::Table(index) => {
            let table = module.tables[*index];
            ExternType::Table(table)
        }
        ImportIndex::Memory(index) => {
            let memory = module.memories[*index];
            ExternType::Memory(memory)
        }
        ImportIndex::Global(index) => {
            let global = module.globals[*index];
            ExternType::Global(global)
        }
    }
}

/// Get an `ExternType` given an export (and Engine signatures in case is a function).
fn get_extern_type(context: &StoreObjects, extern_: &VMExtern) -> ExternType {
    match extern_ {
        VMExtern::Function(f) => ExternType::Function(f.get(context).signature.clone()),
        VMExtern::Table(t) => ExternType::Table(*t.get(context).ty()),
        VMExtern::Memory(m) => ExternType::Memory(m.get(context).ty()),
        VMExtern::Global(g) => {
            let global = g.get(context).ty();
            ExternType::Global(*global)
        }
    }
}

fn get_runtime_size(context: &StoreObjects, extern_: &VMExtern) -> Option<u32> {
    match extern_ {
        VMExtern::Table(t) => Some(t.get(context).get_runtime_size()),
        VMExtern::Memory(m) => Some(m.get(context).get_runtime_size()),
        _ => None,
    }
}

/// This function allows to match all imports of a `ModuleInfo` with concrete definitions provided by
/// a `Resolver`.
///
/// If all imports are satisfied returns an `Imports` instance required for a module instantiation.
#[allow(clippy::result_large_err)]
pub fn resolve_imports(
    module: &ModuleInfo,
    imports: &[VMExtern],
    context: &StoreObjects,
    finished_dynamic_function_trampolines: &BoxedSlice<FunctionIndex, FunctionBodyPtr>,
    memory_styles: &PrimaryMap<MemoryIndex, MemoryStyle>,
    _table_styles: &PrimaryMap<TableIndex, TableStyle>,
) -> Result<Imports, LinkError> {
    let mut function_imports = PrimaryMap::with_capacity(module.num_imported_functions);
    let mut table_imports = PrimaryMap::with_capacity(module.num_imported_tables);
    let mut memory_imports = PrimaryMap::with_capacity(module.num_imported_memories);
    let mut global_imports = PrimaryMap::with_capacity(module.num_imported_globals);

    for (
        wasmer_types::ImportKey {
            module: module_name,
            field,
            import_idx,
        },
        import_index,
    ) in module.imports.iter()
    {
        let import_extern = get_extern_from_import(module, import_index);
        let resolved = if let Some(r) = imports.get(*import_idx as usize) {
            r
        } else {
            return Err(LinkError::Import(
                module_name.to_string(),
                field.to_string(),
                ImportError::UnknownImport(import_extern),
            ));
        };
        let extern_type = get_extern_type(context, resolved);
        let runtime_size = get_runtime_size(context, resolved);
        if !extern_type.is_compatible_with(&import_extern, runtime_size) {
            return Err(LinkError::Import(
                module_name.to_string(),
                field.to_string(),
                ImportError::IncompatibleType(import_extern, extern_type),
            ));
        }
        match *resolved {
            VMExtern::Function(handle) => {
                let f = handle.get(context);
                let address = match f.kind {
                    VMFunctionKind::Dynamic => {
                        // If this is a dynamic imported function,
                        // the address of the function is the address of the
                        // reverse trampoline.
                        let index = FunctionIndex::new(function_imports.len());
                        finished_dynamic_function_trampolines[index].0 as *mut VMFunctionBody as _
                    }
                    VMFunctionKind::Static => unsafe { f.anyfunc.as_ptr().as_ref().func_ptr },
                };

                function_imports.push(VMFunctionImport {
                    body: address,
                    environment: unsafe { f.anyfunc.as_ptr().as_ref().vmctx },
                    handle,
                });
            }
            VMExtern::Table(handle) => {
                let t = handle.get(context);
                match import_index {
                    ImportIndex::Table(index) => {
                        let import_table_ty = t.ty();
                        let expected_table_ty = &module.tables[*index];
                        if import_table_ty.ty != expected_table_ty.ty {
                            return Err(LinkError::Import(
                                module_name.to_string(),
                                field.to_string(),
                                ImportError::IncompatibleType(import_extern, extern_type),
                            ));
                        }

                        table_imports.push(VMTableImport {
                            definition: t.vmtable(),
                            handle,
                        });
                    }
                    _ => {
                        unreachable!("Table resolution did not match");
                    }
                }
            }
            VMExtern::Memory(handle) => {
                let m = handle.get(context);
                match import_index {
                    ImportIndex::Memory(index) => {
                        // Sanity-check: Ensure that the imported memory has at least
                        // guard-page protections the importing module expects it to have.
                        let export_memory_style = m.style();
                        let import_memory_style = &memory_styles[*index];
                        if let (
                            MemoryStyle::Static { bound, .. },
                            MemoryStyle::Static {
                                bound: import_bound,
                                ..
                            },
                        ) = (export_memory_style, &import_memory_style)
                        {
                            assert_ge!(bound, *import_bound);
                        }
                        assert_ge!(
                            export_memory_style.offset_guard_size(),
                            import_memory_style.offset_guard_size()
                        );
                    }
                    _ => {
                        // This should never be reached, as we did compatibility
                        // checks before
                        panic!("Memory resolution didn't matched");
                    }
                }

                memory_imports.push(VMMemoryImport {
                    definition: m.vmmemory(),
                    handle,
                });
            }

            VMExtern::Global(handle) => {
                let g = handle.get(context);
                global_imports.push(VMGlobalImport {
                    definition: g.vmglobal(),
                    handle,
                });
            }
        }
    }

    Ok(Imports::new(
        function_imports,
        table_imports,
        memory_imports,
        global_imports,
    ))
}
