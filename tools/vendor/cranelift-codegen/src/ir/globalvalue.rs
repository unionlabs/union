//! Global values.

use crate::ir::immediates::{Imm64, Offset32};
use crate::ir::{ExternalName, GlobalValue, Type};
use crate::isa::TargetIsa;
use crate::machinst::RelocDistance;
use core::fmt;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Information about a global value declaration.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum GlobalValueData {
    /// Value is the address of the VM context struct.
    VMContext,

    /// Value is pointed to by another global value.
    ///
    /// The `base` global value is assumed to contain a pointer. This global value is computed
    /// by loading from memory at that pointer value. The memory must be accessible, and
    /// naturally aligned to hold a value of the type. The data at this address is assumed
    /// to never change while the current function is executing.
    Load {
        /// The base pointer global value.
        base: GlobalValue,

        /// Offset added to the base pointer before doing the load.
        offset: Offset32,

        /// Type of the loaded value.
        global_type: Type,

        /// Specifies whether the memory that this refers to is readonly, allowing for the
        /// elimination of redundant loads.
        readonly: bool,
    },

    /// Value is an offset from another global value.
    IAddImm {
        /// The base pointer global value.
        base: GlobalValue,

        /// Byte offset to be added to the value.
        offset: Imm64,

        /// Type of the iadd.
        global_type: Type,
    },

    /// Value is symbolic, meaning it's a name which will be resolved to an
    /// actual value later (eg. by linking). Cranelift itself does not interpret
    /// this name; it's used by embedders to link with other data structures.
    ///
    /// For now, symbolic values always have pointer type, and represent
    /// addresses, however in the future they could be used to represent other
    /// things as well.
    Symbol {
        /// The symbolic name.
        name: ExternalName,

        /// Offset from the symbol. This can be used instead of IAddImm to represent folding an
        /// offset into a symbol.
        offset: Imm64,

        /// Will this symbol be defined nearby, such that it will always be a certain distance
        /// away, after linking? If so, references to it can avoid going through a GOT. Note that
        /// symbols meant to be preemptible cannot be colocated.
        ///
        /// If `true`, some backends may use relocation forms that have limited range: for example,
        /// a +/- 2^27-byte range on AArch64. See the documentation for
        /// [`RelocDistance`](crate::machinst::RelocDistance) for more details.
        colocated: bool,

        /// Does this symbol refer to a thread local storage value?
        tls: bool,
    },
}

impl GlobalValueData {
    /// Assume that `self` is an `GlobalValueData::Symbol` and return its name.
    pub fn symbol_name(&self) -> &ExternalName {
        match *self {
            Self::Symbol { ref name, .. } => name,
            _ => panic!("only symbols have names"),
        }
    }

    /// Return the type of this global.
    pub fn global_type(&self, isa: &dyn TargetIsa) -> Type {
        match *self {
            Self::VMContext { .. } | Self::Symbol { .. } => isa.pointer_type(),
            Self::IAddImm { global_type, .. } | Self::Load { global_type, .. } => global_type,
        }
    }

    /// If this global references a symbol, return an estimate of the relocation distance,
    /// based on the `colocated` flag.
    pub fn maybe_reloc_distance(&self) -> Option<RelocDistance> {
        match self {
            &GlobalValueData::Symbol {
                colocated: true, ..
            } => Some(RelocDistance::Near),
            &GlobalValueData::Symbol {
                colocated: false, ..
            } => Some(RelocDistance::Far),
            _ => None,
        }
    }
}

impl fmt::Display for GlobalValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::VMContext => write!(f, "vmctx"),
            Self::Load {
                base,
                offset,
                global_type,
                readonly,
            } => write!(
                f,
                "load.{} notrap aligned {}{}{}",
                global_type,
                if readonly { "readonly " } else { "" },
                base,
                offset
            ),
            Self::IAddImm {
                global_type,
                base,
                offset,
            } => write!(f, "iadd_imm.{} {}, {}", global_type, base, offset),
            Self::Symbol {
                ref name,
                offset,
                colocated,
                tls,
            } => {
                write!(
                    f,
                    "symbol {}{}{}",
                    if colocated { "colocated " } else { "" },
                    if tls { "tls " } else { "" },
                    name
                )?;
                let offset_val: i64 = offset.into();
                if offset_val > 0 {
                    write!(f, "+")?;
                }
                if offset_val != 0 {
                    write!(f, "{}", offset)?;
                }
                Ok(())
            }
        }
    }
}
