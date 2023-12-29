use crate::js::env::HostEnvInitError;
use crate::js::export::Export;
use crate::js::exports::Exports;
use crate::js::externals::Extern;
use crate::js::module::Module;
use crate::js::resolver::Resolver;
use crate::js::store::Store;
use crate::js::trap::RuntimeError;
use js_sys::WebAssembly;
use std::fmt;
#[cfg(feature = "std")]
use thiserror::Error;

/// A WebAssembly Instance is a stateful, executable
/// instance of a WebAssembly [`Module`].
///
/// Instance objects contain all the exported WebAssembly
/// functions, memories, tables and globals that allow
/// interacting with WebAssembly.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#module-instances>
#[derive(Clone)]
pub struct Instance {
    instance: WebAssembly::Instance,
    module: Module,
    /// The exports for an instance.
    pub exports: Exports,
}

/// An error while instantiating a module.
///
/// This is not a common WebAssembly error, however
/// we need to differentiate from a `LinkError` (an error
/// that happens while linking, on instantiation), a
/// Trap that occurs when calling the WebAssembly module
/// start function, and an error when initializing the user's
/// host environments.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum InstantiationError {
    /// A linking ocurred during instantiation.
    #[cfg_attr(feature = "std", error("Link error: {0}"))]
    Link(String),

    /// A runtime error occured while invoking the start function
    #[cfg_attr(feature = "std", error(transparent))]
    Start(RuntimeError),

    /// Error occurred when initializing the host environment.
    #[cfg_attr(feature = "std", error(transparent))]
    HostEnvInitialization(HostEnvInitError),
}

#[cfg(feature = "core")]
impl std::fmt::Display for InstantiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InstantiationError")
    }
}

impl Instance {
    /// Creates a new `Instance` from a WebAssembly [`Module`] and a
    /// set of imports resolved by the [`Resolver`].
    ///
    /// The resolver can be anything that implements the [`Resolver`] trait,
    /// so you can plug custom resolution for the imports, if you wish not
    /// to use [`ImportObject`].
    ///
    /// The [`ImportObject`] is the easiest way to provide imports to the instance.
    ///
    /// [`ImportObject`]: crate::js::ImportObject
    ///
    /// ```
    /// # use wasmer::{imports, Store, Module, Global, Value, Instance};
    /// # fn main() -> anyhow::Result<()> {
    /// let store = Store::default();
    /// let module = Module::new(&store, "(module)")?;
    /// let imports = imports!{
    ///   "host" => {
    ///     "var" => Global::new(&store, Value::I32(2))
    ///   }
    /// };
    /// let instance = Instance::new(&module, &imports)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Errors
    ///
    /// The function can return [`InstantiationError`]s.
    ///
    /// Those are, as defined by the spec:
    ///  * Link errors that happen when plugging the imports into the instance
    ///  * Runtime errors that happen when running the module `start` function.
    pub fn new(
        module: &Module,
        resolver: &(dyn Resolver + Send + Sync),
    ) -> Result<Self, InstantiationError> {
        let (instance, imports) = module
            .instantiate(resolver)
            .map_err(|e| InstantiationError::Start(e))?;

        let self_instance = Self::from_module_and_instance(module, instance)?;
        self_instance.init_envs(&imports)?;
        Ok(self_instance)
    }

    /// Creates a Wasmer `Instance` from a Wasmer `Module` and a WebAssembly Instance
    ///
    /// # Important
    ///
    /// Is expected that the function [`Instance::init_envs`] is run manually
    /// by the user in case the instance has any Wasmer imports, so the function
    /// environments are properly initiated.
    ///
    /// *This method is only available when targeting JS environments*
    pub fn from_module_and_instance(
        module: &Module,
        instance: WebAssembly::Instance,
    ) -> Result<Self, InstantiationError> {
        let store = module.store();
        let instance_exports = instance.exports();
        let exports = module
            .exports()
            .map(|export_type| {
                let name = export_type.name();
                let extern_type = export_type.ty().clone();
                let js_export =
                    js_sys::Reflect::get(&instance_exports, &name.into()).map_err(|_e| {
                        InstantiationError::Link(format!(
                            "Can't get {} from the instance exports",
                            &name
                        ))
                    })?;
                let export: Export = (js_export, extern_type).into();
                let extern_ = Extern::from_vm_export(store, export);
                Ok((name.to_string(), extern_))
            })
            .collect::<Result<Exports, InstantiationError>>()?;

        Ok(Self {
            instance,
            module: module.clone(),
            exports,
        })
    }

    /// Initialize the given extern imports with the `Instance`.
    ///
    /// # Important
    ///
    /// This method should be called if the Wasmer `Instance` is initialized
    /// from Javascript with an already existing `WebAssembly.Instance` but with
    /// a imports from the Rust side.
    ///
    /// *This method is only available when targeting JS environments*
    pub fn init_envs(&self, imports: &[Export]) -> Result<(), InstantiationError> {
        for import in imports {
            if let Export::Function(func) = import {
                func.init_envs(&self)
                    .map_err(|e| InstantiationError::HostEnvInitialization(e))?;
            }
        }
        Ok(())
    }

    /// Gets the [`Module`] associated with this instance.
    pub fn module(&self) -> &Module {
        &self.module
    }

    /// Returns the [`Store`] where the `Instance` belongs.
    pub fn store(&self) -> &Store {
        self.module.store()
    }

    /// Returns the inner WebAssembly Instance
    #[doc(hidden)]
    pub fn raw(&self) -> &WebAssembly::Instance {
        &self.instance
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Instance")
            .field("exports", &self.exports)
            .finish()
    }
}
