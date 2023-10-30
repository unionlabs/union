use loupe::MemoryUsage;
use std::sync::Arc;
use wasmer_vm::{ImportInitializerFuncPtr, VMExtern, VMFunction, VMGlobal, VMMemory, VMTable};

/// The value of an export passed from one instance to another.
#[derive(Debug, Clone)]
pub enum Export {
    /// A function export value.
    Function(ExportFunction),

    /// A table export value.
    Table(VMTable),

    /// A memory export value.
    Memory(VMMemory),

    /// A global export value.
    Global(VMGlobal),
}

impl From<Export> for VMExtern {
    fn from(other: Export) -> Self {
        match other {
            Export::Function(ExportFunction { vm_function, .. }) => Self::Function(vm_function),
            Export::Memory(vm_memory) => Self::Memory(vm_memory),
            Export::Table(vm_table) => Self::Table(vm_table),
            Export::Global(vm_global) => Self::Global(vm_global),
        }
    }
}

impl From<VMExtern> for Export {
    fn from(other: VMExtern) -> Self {
        match other {
            VMExtern::Function(vm_function) => Self::Function(ExportFunction {
                vm_function,
                metadata: None,
            }),
            VMExtern::Memory(vm_memory) => Self::Memory(vm_memory),
            VMExtern::Table(vm_table) => Self::Table(vm_table),
            VMExtern::Global(vm_global) => Self::Global(vm_global),
        }
    }
}

/// Extra metadata about `ExportFunction`s.
///
/// The metadata acts as a kind of manual virtual dispatch. We store the
/// user-supplied `WasmerEnv` as a void pointer and have methods on it
/// that have been adapted to accept a void pointer.
///
/// This struct owns the original `host_env`, thus when it gets dropped
/// it calls the `drop` function on it.
#[derive(Debug, PartialEq, MemoryUsage)]
pub struct ExportFunctionMetadata {
    /// This field is stored here to be accessible by `Drop`.
    ///
    /// At the time it was added, it's not accessed anywhere outside of
    /// the `Drop` implementation. This field is the "master copy" of the env,
    /// that is, the original env passed in by the user. Every time we create
    /// an `Instance` we clone this with the `host_env_clone_fn` field.
    ///
    /// Thus, we only bother to store the master copy at all here so that
    /// we can free it.
    ///
    /// See `wasmer_vm::export::VMFunction::vmctx` for the version of
    /// this pointer that is used by the VM when creating an `Instance`.
    pub(crate) host_env: *mut std::ffi::c_void,

    /// Function pointer to `WasmerEnv::init_with_instance(&mut self, instance: &Instance)`.
    ///
    /// This function is called to finish setting up the environment after
    /// we create the `api::Instance`.
    // This one is optional for now because dynamic host envs need the rest
    // of this without the init fn
    #[loupe(skip)]
    pub(crate) import_init_function_ptr: Option<ImportInitializerFuncPtr>,

    /// A function analogous to `Clone::clone` that returns a leaked `Box`.
    #[loupe(skip)]
    pub(crate) host_env_clone_fn: fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,

    /// The destructor to free the host environment.
    ///
    /// # Safety
    /// - This function should only be called in when properly synchronized.
    /// For example, in the `Drop` implementation of this type.
    #[loupe(skip)]
    pub(crate) host_env_drop_fn: unsafe fn(*mut std::ffi::c_void),
}

/// This can be `Send` because `host_env` comes from `WasmerEnv` which is
/// `Send`. Therefore all operations should work on any thread.
unsafe impl Send for ExportFunctionMetadata {}
/// This data may be shared across threads, `drop` is an unsafe function
/// pointer, so care must be taken when calling it.
unsafe impl Sync for ExportFunctionMetadata {}

impl ExportFunctionMetadata {
    /// Create an `ExportFunctionMetadata` type with information about
    /// the exported function.
    ///
    /// # Safety
    /// - the `host_env` must be `Send`.
    /// - all function pointers must work on any thread.
    pub unsafe fn new(
        host_env: *mut std::ffi::c_void,
        import_init_function_ptr: Option<ImportInitializerFuncPtr>,
        host_env_clone_fn: fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,
        host_env_drop_fn: fn(*mut std::ffi::c_void),
    ) -> Self {
        Self {
            host_env,
            import_init_function_ptr,
            host_env_clone_fn,
            host_env_drop_fn,
        }
    }
}

// We have to free `host_env` here because we always clone it before using it
// so all the `host_env`s freed at the `Instance` level won't touch the original.
impl Drop for ExportFunctionMetadata {
    fn drop(&mut self) {
        if !self.host_env.is_null() {
            // # Safety
            // - This is correct because we know no other references
            //   to this data can exist if we're dropping it.
            unsafe {
                (self.host_env_drop_fn)(self.host_env);
            }
        }
    }
}

/// A function export value with an extra function pointer to initialize
/// host environments.
#[derive(Debug, Clone, PartialEq, MemoryUsage)]
pub struct ExportFunction {
    /// The VM function, containing most of the data.
    pub vm_function: VMFunction,
    /// Contains functions necessary to create and initialize host envs
    /// with each `Instance` as well as being responsible for the
    /// underlying memory of the host env.
    pub metadata: Option<Arc<ExportFunctionMetadata>>,
}

impl From<ExportFunction> for Export {
    fn from(func: ExportFunction) -> Self {
        Self::Function(func)
    }
}

impl From<VMTable> for Export {
    fn from(table: VMTable) -> Self {
        Self::Table(table)
    }
}

impl From<VMMemory> for Export {
    fn from(memory: VMMemory) -> Self {
        Self::Memory(memory)
    }
}

impl From<VMGlobal> for Export {
    fn from(global: VMGlobal) -> Self {
        Self::Global(global)
    }
}
