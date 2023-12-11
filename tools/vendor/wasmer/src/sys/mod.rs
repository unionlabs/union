pub(crate) mod engine;
pub(crate) mod errors;
pub(crate) mod extern_ref;
pub(crate) mod externals;
pub(crate) mod instance;
pub(crate) mod mem_access;
pub(crate) mod module;
mod tunables;
pub(crate) mod typed_function;

pub use crate::sys::engine::NativeEngineExt;
pub use crate::sys::tunables::BaseTunables;
#[cfg(feature = "compiler")]
pub use wasmer_compiler::{
    wasmparser, CompilerConfig, FunctionMiddleware, MiddlewareReaderState, ModuleMiddleware,
};
pub use wasmer_compiler::{Artifact, EngineBuilder, Features, Tunables};
#[cfg(feature = "cranelift")]
pub use wasmer_compiler_cranelift::{Cranelift, CraneliftOptLevel};
#[cfg(feature = "llvm")]
pub use wasmer_compiler_llvm::{LLVMOptLevel, LLVM};
#[cfg(feature = "singlepass")]
pub use wasmer_compiler_singlepass::Singlepass;

pub use wasmer_vm::VMConfig;

pub(crate) mod vm {
    //! The `vm` module re-exports wasmer-vm types.
    use wasmer_vm::InternalStoreHandle;
    pub(crate) use wasmer_vm::{
        VMExtern, VMExternRef, VMFuncRef, VMFunction, VMFunctionBody, VMFunctionEnvironment,
        VMGlobal, VMInstance, VMMemory, VMTable, VMTrampoline,
    };

    pub(crate) type VMExternTable = InternalStoreHandle<VMTable>;
    pub(crate) type VMExternMemory = InternalStoreHandle<VMMemory>;
    pub(crate) type VMExternGlobal = InternalStoreHandle<VMGlobal>;
    pub(crate) type VMExternFunction = InternalStoreHandle<VMFunction>;

    pub type VMFunctionCallback = *const VMFunctionBody;
}
