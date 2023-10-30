#![allow(missing_docs)]

mod dynamic_function;
mod function_call;

pub use self::dynamic_function::make_trampoline_dynamic_function;
pub use self::function_call::make_trampoline_function_call;

pub use cranelift_codegen::print_errors::pretty_error;
pub use cranelift_codegen::Context;
pub use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
