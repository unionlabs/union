//! Instruction Set Architectures.
//!
//! The `isa` module provides a `TargetIsa` trait which provides the behavior specialization needed
//! by the ISA-independent code generator. The sub-modules of this module provide definitions for
//! the instruction sets that Cranelift can target. Each sub-module has it's own implementation of
//! `TargetIsa`.
//!
//! # Constructing a `TargetIsa` instance
//!
//! The target ISA is built from the following information:
//!
//! - The name of the target ISA as a string. Cranelift is a cross-compiler, so the ISA to target
//!   can be selected dynamically. Individual ISAs can be left out when Cranelift is compiled, so a
//!   string is used to identify the proper sub-module.
//! - Values for settings that apply to all ISAs. This is represented by a `settings::Flags`
//!   instance.
//! - Values for ISA-specific settings.
//!
//! The `isa::lookup()` function is the main entry point which returns an `isa::Builder`
//! appropriate for the requested ISA:
//!
//! ```
//! # #[macro_use] extern crate target_lexicon;
//! use cranelift_codegen::isa;
//! use cranelift_codegen::settings::{self, Configurable};
//! use std::str::FromStr;
//! use target_lexicon::Triple;
//!
//! let shared_builder = settings::builder();
//! let shared_flags = settings::Flags::new(shared_builder);
//!
//! match isa::lookup(triple!("x86_64")) {
//!     Err(_) => {
//!         // The x86_64 target ISA is not available.
//!     }
//!     Ok(mut isa_builder) => {
//!         isa_builder.set("use_popcnt", "on");
//!         let isa = isa_builder.finish(shared_flags);
//!     }
//! }
//! ```
//!
//! The configured target ISA trait object is a `Box<TargetIsa>` which can be used for multiple
//! concurrent function compilations.

pub use crate::isa::call_conv::CallConv;

use crate::flowgraph;
use crate::ir::{self, Function};
#[cfg(feature = "unwind")]
use crate::isa::unwind::systemv::RegisterMappingError;
use crate::machinst::{MachCompileResult, TextSectionBuilder, UnwindInfoKind};
use crate::settings;
use crate::settings::SetResult;
use crate::CodegenResult;
use alloc::{boxed::Box, vec::Vec};
use core::fmt;
use core::fmt::{Debug, Formatter};
use target_lexicon::{triple, Architecture, OperatingSystem, PointerWidth, Triple};

// This module is made public here for benchmarking purposes. No guarantees are
// made regarding API stability.
#[cfg(feature = "x86")]
pub mod x64;

#[cfg(feature = "arm64")]
pub(crate) mod aarch64;

#[cfg(feature = "s390x")]
mod s390x;

pub mod unwind;

mod call_conv;

/// Returns a builder that can create a corresponding `TargetIsa`
/// or `Err(LookupError::SupportDisabled)` if not enabled.
macro_rules! isa_builder {
    ($name: ident, $cfg_terms: tt, $triple: ident) => {{
        #[cfg $cfg_terms]
        {
            Ok($name::isa_builder($triple))
        }
        #[cfg(not $cfg_terms)]
        {
            Err(LookupError::SupportDisabled)
        }
    }};
}

/// Look for an ISA for the given `triple`.
/// Return a builder that can create a corresponding `TargetIsa`.
pub fn lookup(triple: Triple) -> Result<Builder, LookupError> {
    match triple.architecture {
        Architecture::X86_64 => {
            isa_builder!(x64, (feature = "x86"), triple)
        }
        Architecture::Aarch64 { .. } => isa_builder!(aarch64, (feature = "arm64"), triple),
        Architecture::S390x { .. } => isa_builder!(s390x, (feature = "s390x"), triple),
        _ => Err(LookupError::Unsupported),
    }
}

/// Look for a supported ISA with the given `name`.
/// Return a builder that can create a corresponding `TargetIsa`.
pub fn lookup_by_name(name: &str) -> Result<Builder, LookupError> {
    use alloc::str::FromStr;
    lookup(triple!(name))
}

/// Describes reason for target lookup failure
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum LookupError {
    /// Support for this target was disabled in the current build.
    SupportDisabled,

    /// Support for this target has not yet been implemented.
    Unsupported,
}

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
impl std::error::Error for LookupError {}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LookupError::SupportDisabled => write!(f, "Support for this target is disabled"),
            LookupError::Unsupported => {
                write!(f, "Support for this target has not been implemented yet")
            }
        }
    }
}

/// Builder for a `TargetIsa`.
/// Modify the ISA-specific settings before creating the `TargetIsa` trait object with `finish`.
#[derive(Clone)]
pub struct Builder {
    triple: Triple,
    setup: settings::Builder,
    constructor:
        fn(Triple, settings::Flags, settings::Builder) -> CodegenResult<Box<dyn TargetIsa>>,
}

impl Builder {
    /// Gets the triple for the builder.
    pub fn triple(&self) -> &Triple {
        &self.triple
    }

    /// Iterates the available settings in the builder.
    pub fn iter(&self) -> impl Iterator<Item = settings::Setting> {
        self.setup.iter()
    }

    /// Combine the ISA-specific settings with the provided
    /// ISA-independent settings and allocate a fully configured
    /// `TargetIsa` trait object. May return an error if some of the
    /// flags are inconsistent or incompatible: for example, some
    /// platform-independent features, like general SIMD support, may
    /// need certain ISA extensions to be enabled.
    pub fn finish(self, shared_flags: settings::Flags) -> CodegenResult<Box<dyn TargetIsa>> {
        (self.constructor)(self.triple, shared_flags, self.setup)
    }
}

impl settings::Configurable for Builder {
    fn set(&mut self, name: &str, value: &str) -> SetResult<()> {
        self.setup.set(name, value)
    }

    fn enable(&mut self, name: &str) -> SetResult<()> {
        self.setup.enable(name)
    }
}

/// After determining that an instruction doesn't have an encoding, how should we proceed to
/// legalize it?
///
/// The `Encodings` iterator returns a legalization function to call.
pub type Legalize =
    fn(ir::Inst, &mut ir::Function, &mut flowgraph::ControlFlowGraph, &dyn TargetIsa) -> bool;

/// This struct provides information that a frontend may need to know about a target to
/// produce Cranelift IR for the target.
#[derive(Clone, Copy, Hash)]
pub struct TargetFrontendConfig {
    /// The default calling convention of the target.
    pub default_call_conv: CallConv,

    /// The pointer width of the target.
    pub pointer_width: PointerWidth,
}

impl TargetFrontendConfig {
    /// Get the pointer type of this target.
    pub fn pointer_type(self) -> ir::Type {
        ir::Type::int(u16::from(self.pointer_bits())).unwrap()
    }

    /// Get the width of pointers on this target, in units of bits.
    pub fn pointer_bits(self) -> u8 {
        self.pointer_width.bits()
    }

    /// Get the width of pointers on this target, in units of bytes.
    pub fn pointer_bytes(self) -> u8 {
        self.pointer_width.bytes()
    }
}

/// Methods that are specialized to a target ISA.
///
/// Implies a Display trait that shows the shared flags, as well as any ISA-specific flags.
pub trait TargetIsa: fmt::Display + Send + Sync {
    /// Get the name of this ISA.
    fn name(&self) -> &'static str;

    /// Get the target triple that was used to make this trait object.
    fn triple(&self) -> &Triple;

    /// Get the ISA-independent flags that were used to make this trait object.
    fn flags(&self) -> &settings::Flags;

    /// Get the ISA-dependent flag values that were used to make this trait object.
    fn isa_flags(&self) -> Vec<settings::Value>;

    /// Compile the given function.
    fn compile_function(
        &self,
        func: &Function,
        want_disasm: bool,
    ) -> CodegenResult<MachCompileResult>;

    #[cfg(feature = "unwind")]
    /// Map a regalloc::Reg to its corresponding DWARF register.
    fn map_regalloc_reg_to_dwarf(&self, _: ::regalloc::Reg) -> Result<u16, RegisterMappingError> {
        Err(RegisterMappingError::UnsupportedArchitecture)
    }

    /// IntCC condition for Unsigned Addition Overflow (Carry).
    fn unsigned_add_overflow_condition(&self) -> ir::condcodes::IntCC;

    /// Creates unwind information for the function.
    ///
    /// Returns `None` if there is no unwind information for the function.
    #[cfg(feature = "unwind")]
    fn emit_unwind_info(
        &self,
        result: &MachCompileResult,
        kind: UnwindInfoKind,
    ) -> CodegenResult<Option<crate::isa::unwind::UnwindInfo>>;

    /// Creates a new System V Common Information Entry for the ISA.
    ///
    /// Returns `None` if the ISA does not support System V unwind information.
    #[cfg(feature = "unwind")]
    fn create_systemv_cie(&self) -> Option<gimli::write::CommonInformationEntry> {
        // By default, an ISA cannot create a System V CIE
        None
    }

    /// Returns an object that can be used to build the text section of an
    /// executable.
    ///
    /// This object will internally attempt to handle as many relocations as
    /// possible using relative calls/jumps/etc between functions.
    ///
    /// The `num_labeled_funcs` argument here is the number of functions which
    /// will be "labeled" or might have calls between them, typically the number
    /// of defined functions in the object file.
    fn text_section_builder(&self, num_labeled_funcs: u32) -> Box<dyn TextSectionBuilder>;
}

/// Methods implemented for free for target ISA!
impl<'a> dyn TargetIsa + 'a {
    /// Get the default calling convention of this target.
    pub fn default_call_conv(&self) -> CallConv {
        CallConv::triple_default(self.triple())
    }

    /// Get the endianness of this ISA.
    pub fn endianness(&self) -> ir::Endianness {
        match self.triple().endianness().unwrap() {
            target_lexicon::Endianness::Little => ir::Endianness::Little,
            target_lexicon::Endianness::Big => ir::Endianness::Big,
        }
    }

    /// Returns the code (text) section alignment for this ISA.
    pub fn code_section_alignment(&self) -> u64 {
        use target_lexicon::*;
        match (self.triple().operating_system, self.triple().architecture) {
            (
                OperatingSystem::MacOSX { .. }
                | OperatingSystem::Darwin
                | OperatingSystem::Ios
                | OperatingSystem::Tvos,
                Architecture::Aarch64(..),
            ) => 0x4000,
            // 64 KB is the maximal page size (i.e. memory translation granule size)
            // supported by the architecture and is used on some platforms.
            (_, Architecture::Aarch64(..)) => 0x10000,
            _ => 0x1000,
        }
    }

    /// Get the pointer type of this ISA.
    pub fn pointer_type(&self) -> ir::Type {
        ir::Type::int(u16::from(self.pointer_bits())).unwrap()
    }

    /// Get the width of pointers on this ISA.
    pub(crate) fn pointer_width(&self) -> PointerWidth {
        self.triple().pointer_width().unwrap()
    }

    /// Get the width of pointers on this ISA, in units of bits.
    pub fn pointer_bits(&self) -> u8 {
        self.pointer_width().bits()
    }

    /// Get the width of pointers on this ISA, in units of bytes.
    pub fn pointer_bytes(&self) -> u8 {
        self.pointer_width().bytes()
    }

    /// Get the information needed by frontends producing Cranelift IR.
    pub fn frontend_config(&self) -> TargetFrontendConfig {
        TargetFrontendConfig {
            default_call_conv: self.default_call_conv(),
            pointer_width: self.pointer_width(),
        }
    }

    /// Returns the flavor of unwind information emitted for this target.
    pub(crate) fn unwind_info_kind(&self) -> UnwindInfoKind {
        match self.triple().operating_system {
            #[cfg(feature = "unwind")]
            OperatingSystem::Windows => UnwindInfoKind::Windows,
            #[cfg(feature = "unwind")]
            _ => UnwindInfoKind::SystemV,
            #[cfg(not(feature = "unwind"))]
            _ => UnwindInfoKind::None,
        }
    }
}

impl Debug for &dyn TargetIsa {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TargetIsa {{ triple: {:?}, pointer_width: {:?}}}",
            self.triple(),
            self.pointer_width()
        )
    }
}
