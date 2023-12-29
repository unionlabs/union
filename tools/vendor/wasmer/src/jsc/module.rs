use crate::errors::InstantiationError;
use crate::errors::RuntimeError;
use crate::imports::Imports;
use crate::jsc::as_js::AsJs;
use crate::jsc::engine::JSC;
use crate::store::AsStoreMut;
use crate::store::AsStoreRef;
use crate::vm::VMInstance;
use crate::Extern;
use crate::IntoBytes;
use crate::{AsEngineRef, ExportType, ImportType};
use bytes::Bytes;
use rusty_jsc::{JSObject, JSString, JSValue};
use std::path::Path;
#[cfg(feature = "tracing")]
use tracing::{debug, warn};
use wasmer_types::{
    CompileError, DeserializeError, ExportsIterator, ExternType, FunctionType, GlobalType,
    ImportsIterator, MemoryType, ModuleInfo, Mutability, Pages, SerializeError, TableType, Type,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Module {
    module: JSObject,
    name: Option<String>,
    raw_bytes: Option<Bytes>,
    info: ModuleInfo,
}

// Module implements `structuredClone` in js, so it's safe it to make it Send.
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// ```js
// const module = new WebAssembly.Module(new Uint8Array([
//   0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00
// ]));
// structuredClone(module)
// ```
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl Module {
    pub(crate) fn from_binary(
        _engine: &impl AsEngineRef,
        binary: &[u8],
    ) -> Result<Self, CompileError> {
        unsafe { Self::from_binary_unchecked(_engine, binary) }
    }

    pub(crate) unsafe fn from_binary_unchecked(
        engine: &impl AsEngineRef,
        binary: &[u8],
    ) -> Result<Self, CompileError> {
        let mut binary = binary.to_vec();
        let engine = engine.as_engine_ref();
        let jsc = engine.jsc();
        let context = jsc.context();
        let bytes = JSObject::create_typed_array_with_bytes(&context, &mut binary).unwrap();
        let module_type = jsc.wasm_module_type();
        let global_wasm = jsc.global_wasm();
        let module = module_type
            .construct(&context, &[bytes.to_jsvalue()])
            .map_err(|e| CompileError::Validate(format!("{}", e.to_string(&context).unwrap())))?;

        Ok(Self::from_js_module(module, binary))
    }

    /// Creates a new WebAssembly module skipping any kind of validation from a javascript module
    ///
    pub(crate) unsafe fn from_js_module(module: JSObject, binary: impl IntoBytes) -> Self {
        let binary = binary.into_bytes();
        // The module is now validated, so we can safely parse it's types
        let info = crate::jsc::module_info_polyfill::translate_module(&binary[..])
            .unwrap()
            .info;

        Self {
            module,
            name: info.name.clone(),
            raw_bytes: Some(binary.into_bytes()),
            info,
        }
    }

    pub fn validate(engine: &impl AsEngineRef, binary: &[u8]) -> Result<(), CompileError> {
        let engine = engine.as_engine_ref();
        let jsc = engine.jsc();
        let context = jsc.context();
        let mut binary = binary.to_vec();
        let bytes = JSObject::create_typed_array_with_bytes(&context, &mut binary).unwrap();

        let global_wasm = jsc.global_wasm();
        let validate_type = jsc.wasm_validate_type();

        match validate_type.call(&context, Some(&global_wasm), &[bytes.to_jsvalue()]) {
            Ok(val) => {
                if val.to_bool(&context) {
                    Ok(())
                } else {
                    Err(CompileError::Validate(format!("Not a valid wasm binary")))
                }
            }
            Err(e) => Err(CompileError::Validate(format!(
                "Error while validating: {}",
                e.to_string(&context).unwrap()
            ))),
        }
    }

    pub(crate) fn instantiate(
        &self,
        store: &mut impl AsStoreMut,
        imports: &Imports,
    ) -> Result<VMInstance, RuntimeError> {
        // Ensure all imports come from the same store.
        if imports
            .into_iter()
            .any(|(_, import)| !import.is_from_store(store))
        {
            return Err(RuntimeError::user(Box::new(
                InstantiationError::DifferentStores,
            )));
        }

        let store = store.as_store_mut();
        let context = store.jsc().context();

        let mut imports_object = JSObject::new(&context);
        for import_type in self.imports() {
            let resolved_import = imports.get_export(import_type.module(), import_type.name());
            if let Some(import) = resolved_import {
                let val = imports_object.get_property(&context, import_type.module().to_string());
                if !val.is_undefined(&context) {
                    // If the namespace is already set
                    let mut obj_val = val.to_object(&context).unwrap();
                    obj_val.set_property(
                        &context,
                        import_type.name().to_string(),
                        import.as_jsvalue(&store.as_store_ref()),
                    );
                } else {
                    // If the namespace doesn't exist
                    let mut import_namespace = JSObject::new(&context);
                    import_namespace.set_property(
                        &context,
                        import_type.name().to_string(),
                        import.as_jsvalue(&store.as_store_ref()),
                    );
                    imports_object
                        .set_property(
                            &context,
                            import_type.module().to_string(),
                            import_namespace.to_jsvalue(),
                        )
                        .unwrap();
                }
            } else {
                #[cfg(feature = "tracing")]
                warn!(
                    "import not found {}:{}",
                    import_type.module(),
                    import_type.name()
                );
            }
            // in case the import is not found, the JS Wasm VM will handle
            // the error for us, so we don't need to handle it
        }

        let instance_type = store.jsc().wasm_instance_type();
        let instance = instance_type.construct(
            &context,
            &[self.module.to_jsvalue(), imports_object.to_jsvalue()],
        );
        Ok(instance.map_err(|e: JSValue| -> RuntimeError { e.into() })?)
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_ref())
    }

    pub fn serialize(&self) -> Result<Bytes, SerializeError> {
        return self.raw_bytes.clone().ok_or(SerializeError::Generic(
            "Not able to serialize module".to_string(),
        ));
    }

    pub unsafe fn deserialize_unchecked(
        engine: &impl AsEngineRef,
        bytes: impl IntoBytes,
    ) -> Result<Self, DeserializeError> {
        Self::deserialize(engine, bytes)
    }

    pub unsafe fn deserialize(
        engine: &impl AsEngineRef,
        bytes: impl IntoBytes,
    ) -> Result<Self, DeserializeError> {
        return Self::from_binary(engine, &bytes.into_bytes())
            .map_err(|e| DeserializeError::Compiler(e));
    }

    pub unsafe fn deserialize_from_file_unchecked(
        engine: &impl AsEngineRef,
        path: impl AsRef<Path>,
    ) -> Result<Self, DeserializeError> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::deserialize_unchecked(engine, bytes)
    }

    pub unsafe fn deserialize_from_file(
        engine: &impl AsEngineRef,
        path: impl AsRef<Path>,
    ) -> Result<Self, DeserializeError> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::deserialize(engine, bytes)
    }

    pub fn set_name(&mut self, name: &str) -> bool {
        self.name = Some(name.to_string());
        true
    }

    pub fn imports<'a>(&'a self) -> ImportsIterator<impl Iterator<Item = ImportType> + 'a> {
        self.info().imports()
    }

    pub fn exports<'a>(&'a self) -> ExportsIterator<impl Iterator<Item = ExportType> + 'a> {
        self.info().exports()
    }

    pub fn custom_sections<'a>(&'a self, name: &'a str) -> impl Iterator<Item = Box<[u8]>> + 'a {
        self.info().custom_sections(name)
    }

    pub(crate) fn info(&self) -> &ModuleInfo {
        &self.info
    }
}

// impl From<WebAssembly::Module> for Module {
//     fn from(module: WebAssembly::Module) -> Module {
//         Module {
//             module,
//             name: None,
//             type_hints: None,
//             #[cfg(feature = "js-serializable-module")]
//             raw_bytes: None,
//         }
//     }
// }

// impl<T: IntoBytes> From<(WebAssembly::Module, T)> for crate::module::Module {
//     fn from((module, binary): (WebAssembly::Module, T)) -> crate::module::Module {
//         unsafe { crate::module::Module(Module::from_js_module(module, binary.into_bytes())) }
//     }
// }

// impl From<WebAssembly::Module> for crate::module::Module {
//     fn from(module: WebAssembly::Module) -> crate::module::Module {
//         crate::module::Module(module.into())
//     }
// }
