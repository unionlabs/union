use crate::errors::RuntimeError;
use crate::imports::Imports;
use crate::js::AsJs;
use crate::store::AsStoreMut;
use crate::vm::VMInstance;
use crate::Extern;
use crate::IntoBytes;
use crate::{errors::InstantiationError, js::js_handle::JsHandle};
use crate::{AsEngineRef, ExportType, ImportType};
use bytes::Bytes;
use js_sys::{Reflect, Uint8Array, WebAssembly};
use std::path::Path;
#[cfg(feature = "tracing")]
use tracing::{debug, warn};
use wasm_bindgen::JsValue;
use wasmer_types::{
    CompileError, DeserializeError, ExportsIterator, ExternType, FunctionType, GlobalType,
    ImportsIterator, MemoryType, ModuleInfo, Mutability, Pages, SerializeError, TableType, Type,
};

/// WebAssembly in the browser doesn't yet output the descriptor/types
/// corresponding to each extern (import and export).
///
/// This should be fixed once the JS-Types Wasm proposal is adopted
/// by the browsers:
/// https://github.com/WebAssembly/js-types/blob/master/proposals/js-types/Overview.md
///
/// Until that happens, we annotate the module with the expected
/// types so we can built on top of them at runtime.
#[derive(Clone, PartialEq, Eq)]
pub struct ModuleTypeHints {
    /// The type hints for the imported types
    pub imports: Vec<ExternType>,
    /// The type hints for the exported types
    pub exports: Vec<ExternType>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Module {
    module: JsHandle<WebAssembly::Module>,
    name: Option<String>,
    // WebAssembly type hints
    type_hints: Option<ModuleTypeHints>,
    #[cfg(feature = "js-serializable-module")]
    raw_bytes: Option<Bytes>,
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

impl From<Module> for JsValue {
    fn from(val: Module) -> Self {
        Self::from(val.module)
    }
}

impl Module {
    pub(crate) fn from_binary(
        _engine: &impl AsEngineRef,
        binary: &[u8],
    ) -> Result<Self, CompileError> {
        unsafe { Self::from_binary_unchecked(_engine, binary) }
    }

    pub(crate) unsafe fn from_binary_unchecked(
        _engine: &impl AsEngineRef,
        binary: &[u8],
    ) -> Result<Self, CompileError> {
        let js_bytes = Uint8Array::view(binary);
        let module = WebAssembly::Module::new(&js_bytes.into())
            .map_err(|e| CompileError::Validate(format!("{}", e.as_string().unwrap())))?;
        Ok(Self::from_js_module(module, binary))
    }

    /// Creates a new WebAssembly module skipping any kind of validation from a javascript module
    pub(crate) unsafe fn from_js_module(
        module: WebAssembly::Module,
        binary: impl IntoBytes,
    ) -> Self {
        let binary = binary.into_bytes();

        // The module is now validated, so we can safely parse it's types
        #[cfg(feature = "wasm-types-polyfill")]
        let (type_hints, name) = {
            let info = crate::js::module_info_polyfill::translate_module(&binary[..]).unwrap();

            (
                Some(ModuleTypeHints {
                    imports: info
                        .info
                        .imports()
                        .map(|import| import.ty().clone())
                        .collect::<Vec<_>>(),
                    exports: info
                        .info
                        .exports()
                        .map(|export| export.ty().clone())
                        .collect::<Vec<_>>(),
                }),
                info.info.name,
            )
        };
        #[cfg(not(feature = "wasm-types-polyfill"))]
        let (type_hints, name) = (None, None);

        Self {
            module: JsHandle::new(module),
            type_hints,
            name,
            #[cfg(feature = "js-serializable-module")]
            raw_bytes: Some(binary),
        }
    }

    pub fn validate(_engine: &impl AsEngineRef, binary: &[u8]) -> Result<(), CompileError> {
        let js_bytes = unsafe { Uint8Array::view(binary) };
        // Annotation is here to prevent spurious IDE warnings.
        #[allow(unused_unsafe)]
        unsafe {
            match WebAssembly::validate(&js_bytes.into()) {
                Ok(true) => Ok(()),
                _ => Err(CompileError::Validate("Invalid Wasm file".to_owned())),
            }
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

        let imports_object = js_sys::Object::new();
        let mut import_externs: Vec<Extern> = vec![];
        for import_type in self.imports() {
            let resolved_import = imports.get_export(import_type.module(), import_type.name());
            // Annotation is here to prevent spurious IDE warnings.
            #[allow(unused_variables)]
            if let wasmer_types::ExternType::Memory(mem_ty) = import_type.ty() {
                if resolved_import.is_some() {
                    #[cfg(feature = "tracing")]
                    debug!("imported shared memory {:?}", &mem_ty);
                } else {
                    #[cfg(feature = "tracing")]
                    warn!(
                        "Error while importing {0:?}.{1:?}: memory. Expected {2:?}",
                        import_type.module(),
                        import_type.name(),
                        import_type.ty(),
                    );
                }
            }
            // Annotation is here to prevent spurious IDE warnings.
            #[allow(unused_unsafe)]
            unsafe {
                if let Some(import) = resolved_import {
                    let val = js_sys::Reflect::get(&imports_object, &import_type.module().into())?;
                    if !val.is_undefined() {
                        // If the namespace is already set
                        js_sys::Reflect::set(
                            &val,
                            &import_type.name().into(),
                            &import.as_jsvalue(&store.as_store_ref()),
                        )?;
                    } else {
                        // If the namespace doesn't exist
                        let import_namespace = js_sys::Object::new();
                        js_sys::Reflect::set(
                            &import_namespace,
                            &import_type.name().into(),
                            &import.as_jsvalue(&store.as_store_ref()),
                        )?;
                        js_sys::Reflect::set(
                            &imports_object,
                            &import_type.module().into(),
                            &import_namespace.into(),
                        )?;
                    }
                    import_externs.push(import);
                } else {
                    #[cfg(feature = "tracing")]
                    warn!(
                        "import not found {}:{}",
                        import_type.module(),
                        import_type.name()
                    );
                }
            }
            // in case the import is not found, the JS Wasm VM will handle
            // the error for us, so we don't need to handle it
        }
        Ok(WebAssembly::Instance::new(&self.module, &imports_object)
            .map_err(|e: JsValue| -> RuntimeError { e.into() })?)
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_ref())
    }

    pub fn serialize(&self) -> Result<Bytes, SerializeError> {
        #[cfg(feature = "js-serializable-module")]
        return self.raw_bytes.clone().ok_or(SerializeError::Generic(
            "Not able to serialize module".to_string(),
        ));

        #[cfg(not(feature = "js-serializable-module"))]
        return Err(SerializeError::Generic(
            "You need to enable the `js-serializable-module` feature flag to serialize a `Module`"
                .to_string(),
        ));
    }

    pub unsafe fn deserialize_unchecked(
        _engine: &impl AsEngineRef,
        _bytes: impl IntoBytes,
    ) -> Result<Self, DeserializeError> {
        #[cfg(feature = "js-serializable-module")]
        return Self::from_binary(_engine, &_bytes.into_bytes())
            .map_err(|e| DeserializeError::Compiler(e));

        #[cfg(not(feature = "js-serializable-module"))]
        return Err(DeserializeError::Generic("You need to enable the `js-serializable-module` feature flag to deserialize a `Module`".to_string()));
    }

    pub unsafe fn deserialize(
        _engine: &impl AsEngineRef,
        _bytes: impl IntoBytes,
    ) -> Result<Self, DeserializeError> {
        #[cfg(feature = "js-serializable-module")]
        return Self::from_binary(_engine, &_bytes.into_bytes())
            .map_err(|e| DeserializeError::Compiler(e));

        #[cfg(not(feature = "js-serializable-module"))]
        return Err(DeserializeError::Generic("You need to enable the `js-serializable-module` feature flag to deserialize a `Module`".to_string()));
    }

    pub unsafe fn deserialize_from_file_unchecked(
        engine: &impl AsEngineRef,
        path: impl AsRef<Path>,
    ) -> Result<Self, DeserializeError> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::deserialize(engine, bytes)
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
        // match Reflect::set(self.module.as_ref(), &"wasmer_name".into(), &name.into()) {
        //     Ok(_) => true,
        //     _ => false
        // }
        // Arc::get_mut(&mut self.artifact)
        //     .and_then(|artifact| artifact.module_mut())
        //     .map(|mut module_info| {
        //         module_info.info.name = Some(name.to_string());
        //         true
        //     })
        //     .unwrap_or(false)
    }

    pub fn imports<'a>(&'a self) -> ImportsIterator<impl Iterator<Item = ImportType> + 'a> {
        let imports = WebAssembly::Module::imports(&self.module);
        let iter = imports
            .iter()
            .enumerate()
            .map(move |(i, val)| {
                // Annotation is here to prevent spurious IDE warnings.
                #[allow(unused_unsafe)]
                unsafe {
                    let module = Reflect::get(val.as_ref(), &"module".into())
                        .unwrap()
                        .as_string()
                        .unwrap();
                    let field = Reflect::get(val.as_ref(), &"name".into())
                        .unwrap()
                        .as_string()
                        .unwrap();
                    let kind = Reflect::get(val.as_ref(), &"kind".into())
                        .unwrap()
                        .as_string()
                        .unwrap();
                    let type_hint = self
                        .type_hints
                        .as_ref()
                        .map(|hints| hints.imports.get(i).unwrap().clone());
                    let extern_type = if let Some(hint) = type_hint {
                        hint
                    } else {
                        match kind.as_str() {
                            "function" => {
                                let func_type = FunctionType::new(vec![], vec![]);
                                ExternType::Function(func_type)
                            }
                            "global" => {
                                let global_type = GlobalType::new(Type::I32, Mutability::Const);
                                ExternType::Global(global_type)
                            }
                            "memory" => {
                                // The javascript API does not yet expose these properties so without
                                // the type_hints we don't know what memory to import.
                                let memory_type = MemoryType::new(Pages(1), None, false);
                                ExternType::Memory(memory_type)
                            }
                            "table" => {
                                let table_type = TableType::new(Type::FuncRef, 1, None);
                                ExternType::Table(table_type)
                            }
                            _ => unimplemented!(),
                        }
                    };
                    ImportType::new(&module, &field, extern_type)
                }
            })
            .collect::<Vec<_>>()
            .into_iter();
        ImportsIterator::new(iter, imports.length() as usize)
    }

    /// Set the type hints for this module.
    ///
    /// Returns an error if the hints doesn't match the shape of
    /// import or export types of the module.
    #[allow(unused)]
    pub fn set_type_hints(&mut self, type_hints: ModuleTypeHints) -> Result<(), String> {
        let exports = WebAssembly::Module::exports(&self.module);
        // Check exports
        if exports.length() as usize != type_hints.exports.len() {
            return Err("The exports length must match the type hints lenght".to_owned());
        }
        for (i, val) in exports.iter().enumerate() {
            // Annotation is here to prevent spurious IDE warnings.
            #[allow(unused_unsafe)]
            let kind = unsafe {
                Reflect::get(val.as_ref(), &"kind".into())
                    .unwrap()
                    .as_string()
                    .unwrap()
            };
            // It is safe to unwrap as we have already checked for the exports length
            let type_hint = type_hints.exports.get(i).unwrap();
            let expected_kind = match type_hint {
                ExternType::Function(_) => "function",
                ExternType::Global(_) => "global",
                ExternType::Memory(_) => "memory",
                ExternType::Table(_) => "table",
            };
            if expected_kind != kind.as_str() {
                return Err(format!("The provided type hint for the export {} is {} which doesn't match the expected kind: {}", i, kind.as_str(), expected_kind));
            }
        }
        self.type_hints = Some(type_hints);
        Ok(())
    }

    pub fn exports<'a>(&'a self) -> ExportsIterator<impl Iterator<Item = ExportType> + 'a> {
        let exports = WebAssembly::Module::exports(&self.module);
        let iter = exports
            .iter()
            .enumerate()
            .map(move |(i, val)| {
                // Annotation is here to prevent spurious IDE warnings.
                #[allow(unused_unsafe)]
                let field = unsafe {
                    Reflect::get(val.as_ref(), &"name".into())
                        .unwrap()
                        .as_string()
                        .unwrap()
                };
                // Annotation is here to prevent spurious IDE warnings.
                #[allow(unused_unsafe)]
                let kind = unsafe {
                    Reflect::get(val.as_ref(), &"kind".into())
                        .unwrap()
                        .as_string()
                        .unwrap()
                };
                let type_hint = self
                    .type_hints
                    .as_ref()
                    .map(|hints| hints.exports.get(i).unwrap().clone());
                let extern_type = if let Some(hint) = type_hint {
                    hint
                } else {
                    // The default types
                    match kind.as_str() {
                        "function" => {
                            let func_type = FunctionType::new(vec![], vec![]);
                            ExternType::Function(func_type)
                        }
                        "global" => {
                            let global_type = GlobalType::new(Type::I32, Mutability::Const);
                            ExternType::Global(global_type)
                        }
                        "memory" => {
                            let memory_type = MemoryType::new(Pages(1), None, false);
                            ExternType::Memory(memory_type)
                        }
                        "table" => {
                            let table_type = TableType::new(Type::FuncRef, 1, None);
                            ExternType::Table(table_type)
                        }
                        _ => unimplemented!(),
                    }
                };
                ExportType::new(&field, extern_type)
            })
            .collect::<Vec<_>>()
            .into_iter();
        ExportsIterator::new(iter, exports.length() as usize)
    }

    pub fn custom_sections<'a>(&'a self, name: &'a str) -> impl Iterator<Item = Box<[u8]>> + 'a {
        WebAssembly::Module::custom_sections(&self.module, name)
            .iter()
            .map(move |buf_val| {
                let typebuf: js_sys::Uint8Array = js_sys::Uint8Array::new(&buf_val);
                typebuf.to_vec().into_boxed_slice()
            })
            .collect::<Vec<Box<[u8]>>>()
            .into_iter()
    }

    pub(crate) fn info(&self) -> &ModuleInfo {
        unimplemented!()
    }
}

impl From<WebAssembly::Module> for Module {
    #[track_caller]
    fn from(module: WebAssembly::Module) -> Module {
        Module {
            module: JsHandle::new(module),
            name: None,
            type_hints: None,
            #[cfg(feature = "js-serializable-module")]
            raw_bytes: None,
        }
    }
}

impl<T: IntoBytes> From<(WebAssembly::Module, T)> for crate::module::Module {
    fn from((module, binary): (WebAssembly::Module, T)) -> crate::module::Module {
        unsafe { crate::module::Module(Module::from_js_module(module, binary.into_bytes())) }
    }
}

impl From<WebAssembly::Module> for crate::module::Module {
    fn from(module: WebAssembly::Module) -> crate::module::Module {
        crate::module::Module(module.into())
    }
}
