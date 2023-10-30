use syn::parse;
use proc_macro_error::emit_error;

mod ast;
mod compiler;
mod parser;
mod debug;
mod x64data;

use crate::State;
use crate::arch::Arch;
use crate::common::{Size, Stmt, Jump};

#[cfg(feature = "dynasm_opmap")]
pub use debug::create_opmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86Mode {
    Long,
    Protected
}

struct Context<'a, 'b: 'a> {
    pub state: &'a mut State<'b>,
    pub mode: X86Mode,
    pub features: x64data::Features
}

#[derive(Clone, Debug)]
pub struct Archx64 {
    features: x64data::Features
}

impl Default for Archx64 {
    fn default() -> Archx64 {
        Archx64 { features: x64data::Features::all() }
    }
}

impl Arch for Archx64 {
    fn name(&self) -> &str {
        "x64"
    }

    fn set_features(&mut self, features: &[syn::Ident]) {
        let mut new_features = x64data::Features::empty();
        for ident in features {
            new_features |= match x64data::Features::from_str(&ident.to_string()) {
                Some(feature) => feature,
                None => {
                    emit_error!(ident, "Architecture x64 does not support feature '{}'", ident);
                    continue;
                }
            }
        }
        self.features = new_features;
    }

    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size) {
        stmts.push(Stmt::Const(0, size));
        // field_offset of size. Relative to the start of the field so matching ref_offset. Type is size and relative
        stmts.push(reloc.encode(size.in_bytes(), size.in_bytes(), &[size.in_bytes(), 0]));
    }

    fn default_align(&self) -> u8 {
        0x90
    }

    fn compile_instruction(&self, state: &mut State, input: parse::ParseStream) -> parse::Result<()> {
        let mut ctx = Context {
            state,
            mode: X86Mode::Long,
            features: self.features
        };
        let (instruction, args) = parser::parse_instruction(&mut ctx, input)?;
        let span = instruction.span;

        if let Err(Some(e)) = compiler::compile_instruction(&mut ctx, instruction, args) {
            emit_error!(span, e);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Archx86 {
    features: x64data::Features
}

impl Default for Archx86 {
    fn default() -> Archx86 {
        Archx86 { features: x64data::Features::all() }
    }
}

impl Arch for Archx86 {
    fn name(&self) -> &str {
        "x86"
    }

    fn set_features(&mut self, features: &[syn::Ident]) {
        let mut new_features = x64data::Features::empty();
        for ident in features {
            new_features |= match x64data::Features::from_str(&ident.to_string()) {
                Some(feature) => feature,
                None => {
                    emit_error!(ident, "Architecture x86 does not support feature '{}'", ident);
                    continue;
                }
            }
        }
        self.features = new_features;
    }

    fn handle_static_reloc(&self, stmts: &mut Vec<Stmt>, reloc: Jump, size: Size) {
        stmts.push(Stmt::Const(0, size));
        // field_offset of size. Relative to the start of the field so matching ref_offset. Type is simply the size.
        stmts.push(reloc.encode(size.in_bytes(), size.in_bytes(), &[size.in_bytes()]));
    }

    fn default_align(&self) -> u8 {
        0x90
    }

    fn compile_instruction(&self, state: &mut State, input: parse::ParseStream) -> parse::Result<()> {
        let mut ctx = Context {
            state,
            mode: X86Mode::Protected,
            features: self.features
        };
        let (instruction, args) = parser::parse_instruction(&mut ctx, input)?;
        let span = instruction.span;

        if let Err(Some(e)) = compiler::compile_instruction(&mut ctx, instruction, args) {
            emit_error!(span, e);
        }
        Ok(())
    }
}
