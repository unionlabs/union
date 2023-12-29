use crate::js::export::Export;
use crate::js::resolver::Resolver;
use crate::js::store::Store;
use crate::js::types::{ExportType, ImportType};
// use crate::js::InstantiationError;
#[cfg(feature = "wat")]
use crate::js::error::WasmError;
use crate::js::error::{CompileError, DeserializeError, SerializeError};
use crate::js::RuntimeError;
use js_sys::{Reflect, Uint8Array, WebAssembly};
use std::fmt;
use std::io;
use std::path::Path;
#[cfg(feature = "std")]
use thiserror::Error;
use wasm_bindgen::JsValue;
use wasmer_types::{
    ExportsIterator, ExternType, FunctionType, GlobalType, ImportsIterator, MemoryType, Mutability,
    Pages, TableType, Type,
};

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum IoCompileError {
    /// An IO error
    #[cfg_attr(feature = "std", error(transparent))]
    Io(io::Error),
    /// A compilation error
    #[cfg_attr(feature = "std", error(transparent))]
    Compile(CompileError),
}

/// WebAssembly in the browser doesn't yet output the descriptor/types
/// corresponding to each extern (import and export).
///
/// This should be fixed once the JS-Types Wasm proposal is adopted
/// by the browsers:
/// https://github.com/WebAssembly/js-types/blob/master/proposals/js-types/Overview.md
///
/// Until that happens, we annotate the module with the expected
/// types so we can built on top of them at runtime.
#[derive(Clone)]
pub struct ModuleTypeHints {
    /// The type hints for the imported types
    pub imports: Vec<ExternType>,
    /// The type hints for the exported types
    pub exports: Vec<ExternType>,
}

/// A WebAssembly Module contains stateless WebAssembly
/// code that has already been compiled and can be instantiated
/// multiple times.
///
/// ## Cloning a module
///
/// Cloning a module is cheap: it does a shallow copy of the compiled
/// contents rather than a deep copy.
#[derive(Clone)]
pub struct Module {
    store: Store,
    module: WebAssembly::Module,
    name: Option<String>,
    // WebAssembly type hints
    type_hints: Option<ModuleTypeHints>,
    #[cfg(feature = "js-serializable-module")]
    raw_bytes: Option<Vec<u8>>,
}

impl Module {
    /// Creates a new WebAssembly Module given the configuration
    /// in the store.
    ///
    /// If the provided bytes are not WebAssembly-like (start with `b"\0asm"`),
    /// and the "wat" feature is enabled for this crate, this function will try to
    /// to convert the bytes assuming they correspond to the WebAssembly text
    /// format.
    ///
    /// ## Security
    ///
    /// Before the code is compiled, it will be validated using the store
    /// features.
    ///
    /// ## Errors
    ///
    /// Creating a WebAssembly module from bytecode can result in a
    /// [`CompileError`] since this operation requires to transorm the Wasm
    /// bytecode into code the machine can easily execute.
    ///
    /// ## Example
    ///
    /// Reading from a WAT file.
    ///
    /// ```
    /// use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// let wat = "(module)";
    /// let module = Module::new(&store, wat)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Reading from bytes:
    ///
    /// ```
    /// use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// // The following is the same as:
    /// // (module
    /// //   (type $t0 (func (param i32) (result i32)))
    /// //   (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
    /// //     get_local $p0
    /// //     i32.const 1
    /// //     i32.add)
    /// // )
    /// let bytes: Vec<u8> = vec![
    ///     0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60,
    ///     0x01, 0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0b, 0x01, 0x07,
    ///     0x61, 0x64, 0x64, 0x5f, 0x6f, 0x6e, 0x65, 0x00, 0x00, 0x0a, 0x09, 0x01,
    ///     0x07, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6a, 0x0b, 0x00, 0x1a, 0x04, 0x6e,
    ///     0x61, 0x6d, 0x65, 0x01, 0x0a, 0x01, 0x00, 0x07, 0x61, 0x64, 0x64, 0x5f,
    ///     0x6f, 0x6e, 0x65, 0x02, 0x07, 0x01, 0x00, 0x01, 0x00, 0x02, 0x70, 0x30,
    /// ];
    /// let module = Module::new(&store, bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(unreachable_code)]
    pub fn new(store: &Store, bytes: impl AsRef<[u8]>) -> Result<Self, CompileError> {
        #[cfg(feature = "wat")]
        let bytes = wat::parse_bytes(bytes.as_ref()).map_err(|e| {
            CompileError::Wasm(WasmError::Generic(format!(
                "Error when converting wat: {}",
                e
            )))
        })?;
        Self::from_binary(store, bytes.as_ref())
    }

    /// Creates a new WebAssembly module from a file path.
    pub fn from_file(_store: &Store, _file: impl AsRef<Path>) -> Result<Self, IoCompileError> {
        unimplemented!();
    }

    /// Creates a new WebAssembly module from a binary.
    ///
    /// Opposed to [`Module::new`], this function is not compatible with
    /// the WebAssembly text format (if the "wat" feature is enabled for
    /// this crate).
    pub fn from_binary(store: &Store, binary: &[u8]) -> Result<Self, CompileError> {
        //
        // Self::validate(store, binary)?;
        unsafe { Self::from_binary_unchecked(store, binary) }
    }

    /// Creates a new WebAssembly module skipping any kind of validation.
    ///
    /// # Safety
    ///
    /// This is safe since the JS vm should be safe already.
    /// We maintain the `unsafe` to preserve the same API as Wasmer
    pub unsafe fn from_binary_unchecked(
        store: &Store,
        binary: &[u8],
    ) -> Result<Self, CompileError> {
        let js_bytes = Uint8Array::view(binary);
        let module = WebAssembly::Module::new(&js_bytes.into()).unwrap();

        // The module is now validated, so we can safely parse it's types
        #[cfg(feature = "wasm-types-polyfill")]
        let (type_hints, name) = {
            let info = crate::js::module_info_polyfill::translate_module(binary).unwrap();

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

        Ok(Self {
            store: store.clone(),
            module,
            type_hints,
            name,
            #[cfg(feature = "js-serializable-module")]
            raw_bytes: Some(binary.to_vec()),
        })
    }

    /// Validates a new WebAssembly Module given the configuration
    /// in the Store.
    ///
    /// This validation is normally pretty fast and checks the enabled
    /// WebAssembly features in the Store Engine to assure deterministic
    /// validation of the Module.
    pub fn validate(_store: &Store, binary: &[u8]) -> Result<(), CompileError> {
        let js_bytes = unsafe { Uint8Array::view(binary) };
        match WebAssembly::validate(&js_bytes.into()) {
            Ok(true) => Ok(()),
            _ => Err(CompileError::Validate("Invalid Wasm file".to_owned())),
        }
    }

    pub(crate) fn instantiate(
        &self,
        resolver: &dyn Resolver,
    ) -> Result<(WebAssembly::Instance, Vec<Export>), RuntimeError> {
        let imports = js_sys::Object::new();
        let mut import_externs: Vec<Export> = vec![];
        for (i, import_type) in self.imports().enumerate() {
            let resolved_import =
                resolver.resolve(i as u32, import_type.module(), import_type.name());
            if let Some(import) = resolved_import {
                let val = js_sys::Reflect::get(&imports, &import_type.module().into())?;
                if !val.is_undefined() {
                    // If the namespace is already set
                    js_sys::Reflect::set(&val, &import_type.name().into(), import.as_jsvalue())?;
                } else {
                    // If the namespace doesn't exist
                    let import_namespace = js_sys::Object::new();
                    js_sys::Reflect::set(
                        &import_namespace,
                        &import_type.name().into(),
                        import.as_jsvalue(),
                    )?;
                    js_sys::Reflect::set(
                        &imports,
                        &import_type.module().into(),
                        &import_namespace.into(),
                    )?;
                }
                import_externs.push(import);
            }
            // in case the import is not found, the JS Wasm VM will handle
            // the error for us, so we don't need to handle it
        }
        Ok((
            WebAssembly::Instance::new(&self.module, &imports)
                .map_err(|e: JsValue| -> RuntimeError { e.into() })?,
            import_externs,
        ))
    }

    /// Returns the name of the current module.
    ///
    /// This name is normally set in the WebAssembly bytecode by some
    /// compilers, but can be also overwritten using the [`Module::set_name`] method.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// let wat = "(module $moduleName)";
    /// let module = Module::new(&store, wat)?;
    /// assert_eq!(module.name(), Some("moduleName"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_ref())
        // self.artifact.module_ref().name.as_deref()
    }

    /// Serializes a module into a binary representation that the `Engine`
    /// can later process via [`Module::deserialize`].
    ///
    #[cfg(feature = "js-serializable-module")]
    pub fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        self.raw_bytes.clone().ok_or(SerializeError::Generic(
            "Not able to serialize module".to_string(),
        ))
    }

    /// Deserializes a serialized Module binary into a `Module`.
    ///
    /// This is safe since deserialization under `js` is essentially same as reconstructing `Module`.
    /// We maintain the `unsafe` to preserve the same API as Wasmer
    #[cfg(feature = "js-serializable-module")]
    pub unsafe fn deserialize(store: &Store, bytes: &[u8]) -> Result<Self, DeserializeError> {
        Self::new(store, bytes).map_err(|e| DeserializeError::Compiler(e))
    }

    /// Sets the name of the current module.
    /// This is normally useful for stacktraces and debugging.
    ///
    /// It will return `true` if the module name was changed successfully,
    /// and return `false` otherwise (in case the module is already
    /// instantiated).
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// let wat = "(module)";
    /// let mut module = Module::new(&store, wat)?;
    /// assert_eq!(module.name(), None);
    /// module.set_name("foo");
    /// assert_eq!(module.name(), Some("foo"));
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns an iterator over the imported types in the Module.
    ///
    /// The order of the imports is guaranteed to be the same as in the
    /// WebAssembly bytecode.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// let wat = r#"(module
    ///     (import "host" "func1" (func))
    ///     (import "host" "func2" (func))
    /// )"#;
    /// let module = Module::new(&store, wat)?;
    /// for import in module.imports() {
    ///     assert_eq!(import.module(), "host");
    ///     assert!(import.name().contains("func"));
    ///     import.ty();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn imports<'a>(&'a self) -> ImportsIterator<impl Iterator<Item = ImportType> + 'a> {
        let imports = WebAssembly::Module::imports(&self.module);
        let iter = imports
            .iter()
            .map(move |val| {
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
                let extern_type = match kind.as_str() {
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
                };
                ImportType::new(&module, &field, extern_type)
            })
            .collect::<Vec<_>>()
            .into_iter();
        ImportsIterator::new(iter, imports.length() as usize)
    }

    /// Set the type hints for this module.
    ///
    /// Returns an error if the hints doesn't match the shape of
    /// import or export types of the module.
    pub fn set_type_hints(&mut self, type_hints: ModuleTypeHints) -> Result<(), String> {
        let exports = WebAssembly::Module::exports(&self.module);
        // Check exports
        if exports.length() as usize != type_hints.exports.len() {
            return Err("The exports length must match the type hints lenght".to_owned());
        }
        for (i, val) in exports.iter().enumerate() {
            let kind = Reflect::get(val.as_ref(), &"kind".into())
                .unwrap()
                .as_string()
                .unwrap();
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

    /// Returns an iterator over the exported types in the Module.
    ///
    /// The order of the exports is guaranteed to be the same as in the
    /// WebAssembly bytecode.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::*;
    /// # fn main() -> anyhow::Result<()> {
    /// # let store = Store::default();
    /// let wat = r#"(module
    ///     (func (export "namedfunc"))
    ///     (memory (export "namedmemory") 1)
    /// )"#;
    /// let module = Module::new(&store, wat)?;
    /// for export_ in module.exports() {
    ///     assert!(export_.name().contains("named"));
    ///     export_.ty();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn exports<'a>(&'a self) -> ExportsIterator<impl Iterator<Item = ExportType> + 'a> {
        let exports = WebAssembly::Module::exports(&self.module);
        let iter = exports
            .iter()
            .enumerate()
            .map(move |(i, val)| {
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

    // /// Get the custom sections of the module given a `name`.
    // ///
    // /// # Important
    // ///
    // /// Following the WebAssembly spec, one name can have multiple
    // /// custom sections. That's why an iterator (rather than one element)
    // /// is returned.
    // pub fn custom_sections<'a>(&'a self, name: &'a str) -> impl Iterator<Item = Arc<[u8]>> + 'a {
    //     unimplemented!();
    // }

    /// Returns the [`Store`] where the `Instance` belongs.
    pub fn store(&self) -> &Store {
        &self.store
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name())
            .finish()
    }
}

impl From<WebAssembly::Module> for Module {
    fn from(module: WebAssembly::Module) -> Module {
        Module {
            store: Store::default(),
            module,
            name: None,
            type_hints: None,
            #[cfg(feature = "js-serializable-module")]
            raw_bytes: None,
        }
    }
}
