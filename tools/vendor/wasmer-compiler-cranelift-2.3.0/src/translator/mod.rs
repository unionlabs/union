//! Tools for translating wasm function bytecode to Cranelift IR.

mod code_translator;
mod func_environ;
mod func_state;
mod func_translator;
mod translation_utils;
mod unwind;

pub use self::func_environ::{FuncEnvironment, GlobalVariable, ReturnMode, TargetEnvironment};
pub use self::func_state::FuncTranslationState;
pub use self::func_translator::FuncTranslator;
pub use self::translation_utils::{
    get_vmctx_value_label, irlibcall_to_libcall, irreloc_to_relocationkind,
    signature_to_cranelift_ir, type_to_irtype,
};
pub(crate) use self::unwind::{compiled_function_unwind_info, CraneliftUnwindInfo};
