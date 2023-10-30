use syn::parse;
use proc_macro_error::emit_error;

use crate::common::{Size, Stmt, Jump};
use crate::State;

use std::fmt::Debug;

pub mod x64;
pub mod aarch64;

pub(crate) trait Arch : Debug + Send {
    fn name(&self) -> &str;
    fn set_features(&mut self, features: &[syn::Ident]);
    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size);
    fn default_align(&self) -> u8;
    fn compile_instruction(&self, state: &mut State, input: parse::ParseStream) -> parse::Result<()>;
}

#[derive(Clone, Debug)]
pub struct DummyArch {
    name: &'static str
}

impl DummyArch {
    fn new(name: &'static str) -> DummyArch {
        DummyArch { name }
    }
}

impl Arch for DummyArch {
    fn name(&self) -> &str {
        self.name
    }

    fn set_features(&mut self, features: &[syn::Ident]) {
        if let Some(feature) = features.first() {
            emit_error!(feature, "Cannot set features when the assembling architecture is undefined. Define it using a .arch directive");
        }
    }

    fn handle_static_reloc(&self, _stmts: &mut Vec<Stmt>, reloc: Jump, _size: Size) {
        let span = reloc.span();
        emit_error!(span, "Current assembling architecture is undefined. Define it using a .arch directive");
    }

    fn default_align(&self) -> u8 {
        0
    }

    fn compile_instruction(&self, _state: &mut State, input: parse::ParseStream) -> parse::Result<()> {
        emit_error!(input.cursor().span(), "Current assembling architecture is undefined. Define it using a .arch directive");
        Ok(())
    }
}

pub(crate) fn from_str(s: &str) -> Option<Box<dyn Arch>> {
    match s {
        "x64" => Some(Box::new(x64::Archx64::default())),
        "x86" => Some(Box::new(x64::Archx86::default())),
        "aarch64" => Some(Box::new(aarch64::ArchAarch64::default())),
        "unknown" => Some(Box::new(DummyArch::new("unknown"))),
        _ => None
    }
}

#[cfg(target_arch="x86_64")]
pub const CURRENT_ARCH: &str = "x64";
#[cfg(target_arch="x86")]
pub const CURRENT_ARCH: &str = "x86";
#[cfg(target_arch="aarch64")]
pub const CURRENT_ARCH: &str = "aarch64";
#[cfg(not(any(target_arch="x86", target_arch="x86_64", target_arch="aarch64")))]
pub const CURRENT_ARCH: &str = "unknown";
