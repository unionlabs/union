//! The Wasmer Engine.

mod error;
#[cfg(not(target_arch = "wasm32"))]
mod resolver;
#[cfg(not(target_arch = "wasm32"))]
mod trap;
#[cfg(not(target_arch = "wasm32"))]
mod tunables;

#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
mod artifact;
#[cfg(feature = "translator")]
mod builder;
#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
mod code_memory;
#[cfg(feature = "translator")]
mod inner;
#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
mod link;
#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
mod unwind;

pub use self::error::{InstantiationError, LinkError};
#[cfg(not(target_arch = "wasm32"))]
pub use self::resolver::resolve_imports;
#[cfg(not(target_arch = "wasm32"))]
pub use self::trap::*;
#[cfg(not(target_arch = "wasm32"))]
pub use self::tunables::{BaseTunables, Tunables};

#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
pub use self::artifact::Artifact;
#[cfg(feature = "translator")]
pub use self::builder::EngineBuilder;
#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
pub use self::code_memory::CodeMemory;
#[cfg(feature = "translator")]
pub use self::inner::{Engine, EngineInner};
#[cfg(feature = "translator")]
#[cfg(not(target_arch = "wasm32"))]
pub use self::link::link_module;
