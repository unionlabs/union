//! This module defines the parser and translator from wasmparser
//! to a common structure `ModuleInfo`.
//!
//! It's derived from [cranelift-wasm] but architected for multiple
//! compilers rather than just Cranelift.
//!
//! [cranelift-wasm]: https://crates.io/crates/cranelift-wasm/
mod environ;
mod middleware;
mod module;
mod state;
#[macro_use]
mod error;
mod sections;

pub use self::environ::{FunctionBinaryReader, FunctionBodyData, ModuleEnvironment};
pub use self::middleware::{
    FunctionMiddleware, MiddlewareBinaryReader, MiddlewareReaderState, ModuleMiddleware,
    ModuleMiddlewareChain,
};
pub use self::module::translate_module;
pub use self::sections::wptype_to_type;
pub use self::state::ModuleTranslationState;
pub use error::from_binaryreadererror_wasmerror;
