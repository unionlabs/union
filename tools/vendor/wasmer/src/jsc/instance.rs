use crate::errors::InstantiationError;
use crate::exports::Exports;
use crate::imports::Imports;
use crate::jsc::as_js::AsJs;
use crate::jsc::engine::JSC;
use crate::jsc::vm::VMInstance;
use crate::module::Module;
use crate::store::AsStoreMut;
use crate::Extern;

#[derive(Clone, PartialEq, Eq)]
pub struct Instance {
    pub(crate) _handle: VMInstance,
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

    pub(crate) fn from_module_and_instance(
        mut store: &mut impl AsStoreMut,
        module: &Module,
        instance: VMInstance,
    ) -> Result<(Self, Exports), InstantiationError> {
        let engine = store.as_store_mut();
        let context = engine.jsc().context();

        let mut instance_exports = instance
            .get_property(&context, "exports".to_string())
            .to_object(&context)
            .unwrap();

        let exports_ty = module.exports().collect::<Vec<_>>();

        let exports = exports_ty
            .iter()
            .map(|export_type| {
                let name = export_type.name();
                let mut store = store.as_store_mut();
                let context = store.jsc().context();
                let extern_type = export_type.ty();
                // Annotation is here to prevent spurious IDE warnings.
                let js_export = instance_exports.get_property(&context, name.to_string());
                let extern_ = Extern::from_jsvalue(&mut store, extern_type, &js_export).unwrap();
                Ok((name.to_string(), extern_))
            })
            .collect::<Result<Exports, InstantiationError>>()?;

        Ok((Self { _handle: instance }, exports))
    }
}
