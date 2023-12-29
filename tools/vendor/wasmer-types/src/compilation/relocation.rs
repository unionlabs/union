//! Relocation is the process of assigning load addresses for position-dependent
//! code and data of a program and adjusting the code and data to reflect the
//! assigned addresses.
//!
//! [Learn more](https://en.wikipedia.org/wiki/Relocation_(computing)).
//!
//! Each time a `Compiler` compiles a WebAssembly function (into machine code),
//! it also attaches if there are any relocations that need to be patched into
//! the generated machine code, so a given frontend (JIT or native) can
//! do the corresponding work to run it.

use super::section::SectionIndex;
use crate::entity::PrimaryMap;
use crate::lib::std::fmt;
use crate::lib::std::vec::Vec;
use crate::{Addend, CodeOffset};
use crate::{LibCall, LocalFunctionIndex};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Relocation kinds for every ISA.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[derive(
    RkyvSerialize, RkyvDeserialize, Archive, rkyv::CheckBytes, Copy, Clone, Debug, PartialEq, Eq,
)]
#[archive(as = "Self")]
#[repr(u8)]
pub enum RelocationKind {
    /// absolute 4-byte
    Abs4,
    /// absolute 8-byte
    Abs8,
    /// x86 PC-relative 4-byte
    X86PCRel4,
    /// x86 PC-relative 8-byte
    X86PCRel8,
    /// x86 call to PC-relative 4-byte
    X86CallPCRel4,
    /// x86 call to PLT-relative 4-byte
    X86CallPLTRel4,
    /// x86 GOT PC-relative 4-byte
    X86GOTPCRel4,
    /// Arm32 call target
    Arm32Call,
    /// Arm64 call target
    Arm64Call,
    /// Arm64 movk/z part 0
    Arm64Movw0,
    /// Arm64 movk/z part 1
    Arm64Movw1,
    /// Arm64 movk/z part 2
    Arm64Movw2,
    /// Arm64 movk/z part 3
    Arm64Movw3,
    /// RISC-V PC-relative high 20bit
    RiscvPCRelHi20,
    /// RISC-V PC-relative low 12bit, I-type
    RiscvPCRelLo12I,
    /// RISC-V call target
    RiscvCall,
    /// Elf x86_64 32 bit signed PC relative offset to two GOT entries for GD symbol.
    ElfX86_64TlsGd,
    // /// Mach-O x86_64 32 bit signed PC relative offset to a `__thread_vars` entry.
    // MachOX86_64Tlv,
}

impl fmt::Display for RelocationKind {
    /// Display trait implementation drops the arch, since its used in contexts where the arch is
    /// already unambiguous, e.g. clif syntax with isa specified. In other contexts, use Debug.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Abs4 => write!(f, "Abs4"),
            Self::Abs8 => write!(f, "Abs8"),
            Self::X86PCRel4 => write!(f, "PCRel4"),
            Self::X86PCRel8 => write!(f, "PCRel8"),
            Self::X86CallPCRel4 => write!(f, "CallPCRel4"),
            Self::X86CallPLTRel4 => write!(f, "CallPLTRel4"),
            Self::X86GOTPCRel4 => write!(f, "GOTPCRel4"),
            Self::Arm32Call | Self::Arm64Call | Self::RiscvCall => write!(f, "Call"),
            Self::Arm64Movw0 => write!(f, "Arm64MovwG0"),
            Self::Arm64Movw1 => write!(f, "Arm64MovwG1"),
            Self::Arm64Movw2 => write!(f, "Arm64MovwG2"),
            Self::Arm64Movw3 => write!(f, "Arm64MovwG3"),
            Self::ElfX86_64TlsGd => write!(f, "ElfX86_64TlsGd"),
            Self::RiscvPCRelHi20 => write!(f, "RiscvPCRelHi20"),
            Self::RiscvPCRelLo12I => write!(f, "RiscvPCRelLo12I"),
            // Self::MachOX86_64Tlv => write!(f, "MachOX86_64Tlv"),
        }
    }
}

/// A record of a relocation to perform.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[derive(RkyvSerialize, RkyvDeserialize, Archive, Debug, Clone, PartialEq, Eq)]
#[archive_attr(derive(rkyv::CheckBytes))]
pub struct Relocation {
    /// The relocation kind.
    pub kind: RelocationKind,
    /// Relocation target.
    pub reloc_target: RelocationTarget,
    /// The offset where to apply the relocation.
    pub offset: CodeOffset,
    /// The addend to add to the relocation value.
    pub addend: Addend,
}

/// Any struct that acts like a `Relocation`.
#[allow(missing_docs)]
pub trait RelocationLike {
    fn kind(&self) -> RelocationKind;
    fn reloc_target(&self) -> RelocationTarget;
    fn offset(&self) -> CodeOffset;
    fn addend(&self) -> Addend;

    /// Given a function start address, provide the relocation relative
    /// to that address.
    ///
    /// The function returns the relocation address and the delta.
    fn for_address(&self, start: usize, target_func_address: u64) -> (usize, u64) {
        match self.kind() {
            RelocationKind::Abs8
            | RelocationKind::Arm64Movw0
            | RelocationKind::Arm64Movw1
            | RelocationKind::Arm64Movw2
            | RelocationKind::Arm64Movw3
            | RelocationKind::RiscvPCRelLo12I => {
                let reloc_address = start + self.offset() as usize;
                let reloc_addend = self.addend() as isize;
                let reloc_abs = target_func_address
                    .checked_add(reloc_addend as u64)
                    .unwrap();
                (reloc_address, reloc_abs)
            }
            RelocationKind::X86PCRel4 => {
                let reloc_address = start + self.offset() as usize;
                let reloc_addend = self.addend() as isize;
                let reloc_delta_u32 = (target_func_address as u32)
                    .wrapping_sub(reloc_address as u32)
                    .checked_add(reloc_addend as u32)
                    .unwrap();
                (reloc_address, reloc_delta_u32 as u64)
            }
            RelocationKind::X86PCRel8 => {
                let reloc_address = start + self.offset() as usize;
                let reloc_addend = self.addend() as isize;
                let reloc_delta = target_func_address
                    .wrapping_sub(reloc_address as u64)
                    .checked_add(reloc_addend as u64)
                    .unwrap();
                (reloc_address, reloc_delta)
            }
            RelocationKind::X86CallPCRel4 | RelocationKind::X86CallPLTRel4 => {
                let reloc_address = start + self.offset() as usize;
                let reloc_addend = self.addend() as isize;
                let reloc_delta_u32 = (target_func_address as u32)
                    .wrapping_sub(reloc_address as u32)
                    .wrapping_add(reloc_addend as u32);
                (reloc_address, reloc_delta_u32 as u64)
            }
            RelocationKind::Arm64Call
            | RelocationKind::RiscvCall
            | RelocationKind::RiscvPCRelHi20 => {
                let reloc_address = start + self.offset() as usize;
                let reloc_addend = self.addend() as isize;
                let reloc_delta_u32 = target_func_address
                    .wrapping_sub(reloc_address as u64)
                    .wrapping_add(reloc_addend as u64);
                (reloc_address, reloc_delta_u32)
            }
            _ => panic!("Relocation kind unsupported"),
        }
    }
}

impl RelocationLike for Relocation {
    fn kind(&self) -> RelocationKind {
        self.kind
    }

    fn reloc_target(&self) -> RelocationTarget {
        self.reloc_target
    }

    fn offset(&self) -> CodeOffset {
        self.offset
    }

    fn addend(&self) -> Addend {
        self.addend
    }
}

impl RelocationLike for ArchivedRelocation {
    fn kind(&self) -> RelocationKind {
        self.kind
    }

    fn reloc_target(&self) -> RelocationTarget {
        self.reloc_target
    }

    fn offset(&self) -> CodeOffset {
        self.offset
    }

    fn addend(&self) -> Addend {
        self.addend
    }
}

/// Destination function. Can be either user function or some special one, like `memory.grow`.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[derive(
    RkyvSerialize, RkyvDeserialize, Archive, rkyv::CheckBytes, Debug, Copy, Clone, PartialEq, Eq,
)]
#[archive(as = "Self")]
#[repr(u8)]
pub enum RelocationTarget {
    /// A relocation to a function defined locally in the wasm (not an imported one).
    LocalFunc(LocalFunctionIndex),
    /// A compiler-generated libcall.
    LibCall(LibCall),
    /// Custom sections generated by the compiler
    CustomSection(SectionIndex),
}

/// Relocations to apply to function bodies.
pub type Relocations = PrimaryMap<LocalFunctionIndex, Vec<Relocation>>;
