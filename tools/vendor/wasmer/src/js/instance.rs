use crate::exports::Exports;
use crate::imports::Imports;
use crate::js::as_js::AsJs;
use crate::js::vm::VMInstance;
use crate::module::Module;
use crate::store::AsStoreMut;
use crate::Extern;
use crate::{errors::InstantiationError, js::js_handle::JsHandle};
use js_sys::WebAssembly;

#[derive(Clone, PartialEq, Eq)]
pub struct Instance {
    pub(crate) _handle: JsHandle<VMInstance>,
}

// Instance can't be Send in js because it dosen't support `structuredClone`
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// unsafe impl Send for Instance {}

impl Instance {
    pub(crate) fn new(
        mut store: &mut impl AsStoreMut,
        module: &Module,
        imports: &Imports,
    ) -> Result<(Self, Exports), InstantiationError> {
        let instance = module
            .0
            .instantiate(&mut store, imports)
            .map_err(|e| InstantiationError::Start(e))?;

        Self::from_module_and_instance(store, &module, instance)
    }

    pub(crate) fn new_by_index(
        store: &mut impl AsStoreMut,
        module: &Module,
        externs: &[Extern],
    ) -> Result<(Self, Exports), InstantiationError> {
        let mut imports = Imports::new();
        for (import_ty, extern_ty) in module.imports().zip(externs.iter()) {
            imports.define(import_ty.module(), import_ty.name(), extern_ty.clone());
        }
        Self::new(store, module, &imports)
    }

    /// Creates a Wasmer `Instance` from a Wasmer `Module` and a WebAssembly Instance
    pub(crate) fn from_module_and_instance(
        mut store: &mut impl AsStoreMut,
        module: &Module,
        instance: WebAssembly::Instance,
    ) -> Result<(Self, Exports), InstantiationError> {
        let instance_exports = instance.exports();

        let exports = module
            .exports()
            .map(|export_type| {
                let name = export_type.name();
                let extern_type = export_type.ty();
                // Annotation is here to prevent spurious IDE warnings.
                #[allow(unused_unsafe)]
                let js_export =
                    unsafe { js_sys::Reflect::get(&instance_exports, &name.into()).unwrap() };
                let extern_ = Extern::from_jsvalue(&mut store, extern_type, &js_export)
                    .map_err(|e| wasm_bindgen::JsValue::from(e))
                    .unwrap();
                Ok((name.to_string(), extern_))
            })
            .collect::<Result<Exports, InstantiationError>>()?;

        let instance = Instance {
            _handle: JsHandle::new(instance),
        };

        Ok((instance, exports))
    }
}
