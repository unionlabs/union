//! This module exposes the machine-specific backend definition pieces.
//!
//! The MachInst infrastructure is the compiler backend, from CLIF
//! (ir::Function) to machine code. The purpose of this infrastructure is, at a
//! high level, to do instruction selection/lowering (to machine instructions),
//! register allocation, and then perform all the fixups to branches, constant
//! data references, etc., needed to actually generate machine code.
//!
//! The container for machine instructions, at various stages of construction,
//! is the `VCode` struct. We refer to a sequence of machine instructions organized
//! into basic blocks as "vcode". This is short for "virtual-register code".
//!
//! The compilation pipeline, from an `ir::Function` (already optimized as much as
//! you like by machine-independent optimization passes) onward, is as follows.
//!
//! ```plain
//!
//!     ir::Function                (SSA IR, machine-independent opcodes)
//!         |
//!         |  [lower]
//!         |
//!     VCode<arch_backend::Inst>   (machine instructions:
//!         |                        - mostly virtual registers.
//!         |                        - cond branches in two-target form.
//!         |                        - branch targets are block indices.
//!         |                        - in-memory constants held by insns,
//!         |                          with unknown offsets.
//!         |                        - critical edges (actually all edges)
//!         |                          are split.)
//!         |
//!         | [regalloc --> `regalloc2::Output`; VCode is unchanged]
//!         |
//!         | [binary emission via MachBuffer]
//!         |
//!     Vec<u8>                     (machine code:
//!         |                        - two-dest branches resolved via
//!         |                          streaming branch resolution/simplification.
//!         |                        - regalloc `Allocation` results used directly
//!         |                          by instruction emission code.
//!         |                        - prologue and epilogue(s) built and emitted
//!         |                          directly during emission.
//!         |                        - nominal-SP-relative offsets resolved
//!         |                          by tracking EmitState.)
//!
//! ```

use crate::binemit::{Addend, CodeInfo, CodeOffset, Reloc, StackMap};
use crate::ir::function::FunctionParameters;
use crate::ir::{DynamicStackSlot, RelSourceLoc, StackSlot, Type};
use crate::result::CodegenResult;
use crate::settings::Flags;
use crate::value_label::ValueLabelsRanges;
use alloc::vec::Vec;
use core::fmt::Debug;
use cranelift_entity::PrimaryMap;
use regalloc2::{Allocation, VReg};
use smallvec::{smallvec, SmallVec};
use std::string::String;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

#[macro_use]
pub mod isle;

pub mod lower;
pub use lower::*;
pub mod vcode;
pub use vcode::*;
pub mod compile;
pub use compile::*;
pub mod blockorder;
pub use blockorder::*;
pub mod abi;
pub use abi::*;
pub mod buffer;
pub use buffer::*;
pub mod helpers;
pub use helpers::*;
pub mod inst_common;
pub use inst_common::*;
pub mod valueregs;
pub use reg::*;
pub use valueregs::*;
pub mod reg;

/// A machine instruction.
pub trait MachInst: Clone + Debug {
    /// The ABI machine spec for this `MachInst`.
    type ABIMachineSpec: ABIMachineSpec<I = Self>;

    /// Return the registers referenced by this machine instruction along with
    /// the modes of reference (use, def, modify).
    fn get_operands<F: Fn(VReg) -> VReg>(&self, collector: &mut OperandCollector<'_, F>);

    /// If this is a simple move, return the (source, destination) tuple of registers.
    fn is_move(&self) -> Option<(Writable<Reg>, Reg)>;

    /// Is this a terminator (branch or ret)? If so, return its type
    /// (ret/uncond/cond) and target if applicable.
    fn is_term(&self) -> MachTerminator;

    /// Is this an "args" pseudoinst?
    fn is_args(&self) -> bool;

    /// Should this instruction be included in the clobber-set?
    fn is_included_in_clobbers(&self) -> bool;

    /// Generate a move.
    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Self;

    /// Generate a constant into a reg.
    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        alloc_tmp: F,
    ) -> SmallVec<[Self; 4]>;

    /// Generate a dummy instruction that will keep a value alive but
    /// has no other purpose.
    fn gen_dummy_use(reg: Reg) -> Self;

    /// Determine register class(es) to store the given Cranelift type, and the
    /// Cranelift type actually stored in the underlying register(s).  May return
    /// an error if the type isn't supported by this backend.
    ///
    /// If the type requires multiple registers, then the list of registers is
    /// returned in little-endian order.
    ///
    /// Note that the type actually stored in the register(s) may differ in the
    /// case that a value is split across registers: for example, on a 32-bit
    /// target, an I64 may be stored in two registers, each of which holds an
    /// I32. The actually-stored types are used only to inform the backend when
    /// generating spills and reloads for individual registers.
    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])>;

    /// Get an appropriate type that can fully hold a value in a given
    /// register class. This may not be the only type that maps to
    /// that class, but when used with `gen_move()` or the ABI trait's
    /// load/spill constructors, it should produce instruction(s) that
    /// move the entire register contents.
    fn canonical_type_for_rc(rc: RegClass) -> Type;

    /// Generate a jump to another target. Used during lowering of
    /// control flow.
    fn gen_jump(target: MachLabel) -> Self;

    /// Generate a NOP. The `preferred_size` parameter allows the caller to
    /// request a NOP of that size, or as close to it as possible. The machine
    /// backend may return a NOP whose binary encoding is smaller than the
    /// preferred size, but must not return a NOP that is larger. However,
    /// the instruction must have a nonzero size if preferred_size is nonzero.
    fn gen_nop(preferred_size: usize) -> Self;

    /// Align a basic block offset (from start of function).  By default, no
    /// alignment occurs.
    fn align_basic_block(offset: CodeOffset) -> CodeOffset {
        offset
    }

    /// What is the worst-case instruction size emitted by this instruction type?
    fn worst_case_size() -> CodeOffset;

    /// What is the register class used for reference types (GC-observable pointers)? Can
    /// be dependent on compilation flags.
    fn ref_type_regclass(_flags: &Flags) -> RegClass;

    /// Is this a safepoint?
    fn is_safepoint(&self) -> bool;

    /// Generate an instruction that must appear at the beginning of a basic
    /// block, if any. Note that the return value must not be subject to
    /// register allocation.
    fn gen_block_start(
        _is_indirect_branch_target: bool,
        _is_forward_edge_cfi_enabled: bool,
    ) -> Option<Self> {
        None
    }

    /// A label-use kind: a type that describes the types of label references that
    /// can occur in an instruction.
    type LabelUse: MachInstLabelUse;
}

/// A descriptor of a label reference (use) in an instruction set.
pub trait MachInstLabelUse: Clone + Copy + Debug + Eq {
    /// Required alignment for any veneer. Usually the required instruction
    /// alignment (e.g., 4 for a RISC with 32-bit instructions, or 1 for x86).
    const ALIGN: CodeOffset;

    /// What is the maximum PC-relative range (positive)? E.g., if `1024`, a
    /// label-reference fixup at offset `x` is valid if the label resolves to `x
    /// + 1024`.
    fn max_pos_range(self) -> CodeOffset;
    /// What is the maximum PC-relative range (negative)? This is the absolute
    /// value; i.e., if `1024`, then a label-reference fixup at offset `x` is
    /// valid if the label resolves to `x - 1024`.
    fn max_neg_range(self) -> CodeOffset;
    /// What is the size of code-buffer slice this label-use needs to patch in
    /// the label's value?
    fn patch_size(self) -> CodeOffset;
    /// Perform a code-patch, given the offset into the buffer of this label use
    /// and the offset into the buffer of the label's definition.
    /// It is guaranteed that, given `delta = offset - label_offset`, we will
    /// have `offset >= -self.max_neg_range()` and `offset <=
    /// self.max_pos_range()`.
    fn patch(self, buffer: &mut [u8], use_offset: CodeOffset, label_offset: CodeOffset);
    /// Can the label-use be patched to a veneer that supports a longer range?
    /// Usually valid for jumps (a short-range jump can jump to a longer-range
    /// jump), but not for e.g. constant pool references, because the constant
    /// load would require different code (one more level of indirection).
    fn supports_veneer(self) -> bool;
    /// How many bytes are needed for a veneer?
    fn veneer_size(self) -> CodeOffset;
    /// Generate a veneer. The given code-buffer slice is `self.veneer_size()`
    /// bytes long at offset `veneer_offset` in the buffer. The original
    /// label-use will be patched to refer to this veneer's offset.  A new
    /// (offset, LabelUse) is returned that allows the veneer to use the actual
    /// label. For veneers to work properly, it is expected that the new veneer
    /// has a larger range; on most platforms this probably means either a
    /// "long-range jump" (e.g., on ARM, the 26-bit form), or if already at that
    /// stage, a jump that supports a full 32-bit range, for example.
    fn generate_veneer(self, buffer: &mut [u8], veneer_offset: CodeOffset) -> (CodeOffset, Self);

    /// Returns the corresponding label-use for the relocation specified.
    ///
    /// This returns `None` if the relocation doesn't have a corresponding
    /// representation for the target architecture.
    fn from_reloc(reloc: Reloc, addend: Addend) -> Option<Self>;
}

/// Describes a block terminator (not call) in the vcode, when its branches
/// have not yet been finalized (so a branch may have two targets).
///
/// Actual targets are not included: the single-source-of-truth for
/// those is the VCode itself, which holds, for each block, successors
/// and outgoing branch args per successor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachTerminator {
    /// Not a terminator.
    None,
    /// A return instruction.
    Ret,
    /// An unconditional branch to another block.
    Uncond,
    /// A conditional branch to one of two other blocks.
    Cond,
    /// An indirect branch with known possible targets.
    Indirect,
}

/// A trait describing the ability to encode a MachInst into binary machine code.
pub trait MachInstEmit: MachInst {
    /// Persistent state carried across `emit` invocations.
    type State: MachInstEmitState<Self>;
    /// Constant information used in `emit` invocations.
    type Info;
    /// Emit the instruction.
    fn emit(
        &self,
        allocs: &[Allocation],
        code: &mut MachBuffer<Self>,
        info: &Self::Info,
        state: &mut Self::State,
    );
    /// Pretty-print the instruction.
    fn pretty_print_inst(&self, allocs: &[Allocation], state: &mut Self::State) -> String;
}

/// A trait describing the emission state carried between MachInsts when
/// emitting a function body.
pub trait MachInstEmitState<I: VCodeInst>: Default + Clone + Debug {
    /// Create a new emission state given the ABI object.
    fn new(abi: &Callee<I::ABIMachineSpec>) -> Self;
    /// Update the emission state before emitting an instruction that is a
    /// safepoint.
    fn pre_safepoint(&mut self, _stack_map: StackMap) {}
    /// Update the emission state to indicate instructions are associated with a
    /// particular RelSourceLoc.
    fn pre_sourceloc(&mut self, _srcloc: RelSourceLoc) {}
}

/// The result of a `MachBackend::compile_function()` call. Contains machine
/// code (as bytes) and a disassembly, if requested.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct CompiledCodeBase<T: CompilePhase> {
    /// Machine code.
    pub buffer: MachBufferFinalized<T>,
    /// Size of stack frame, in bytes.
    pub frame_size: u32,
    /// Disassembly, if requested.
    pub disasm: Option<String>,
    /// Debug info: value labels to registers/stackslots at code offsets.
    pub value_labels_ranges: ValueLabelsRanges,
    /// Debug info: stackslots to stack pointer offsets.
    pub sized_stackslot_offsets: PrimaryMap<StackSlot, u32>,
    /// Debug info: stackslots to stack pointer offsets.
    pub dynamic_stackslot_offsets: PrimaryMap<DynamicStackSlot, u32>,
    /// Basic-block layout info: block start offsets.
    ///
    /// This info is generated only if the `machine_code_cfg_info`
    /// flag is set.
    pub bb_starts: Vec<CodeOffset>,
    /// Basic-block layout info: block edges. Each edge is `(from,
    /// to)`, where `from` and `to` are basic-block start offsets of
    /// the respective blocks.
    ///
    /// This info is generated only if the `machine_code_cfg_info`
    /// flag is set.
    pub bb_edges: Vec<(CodeOffset, CodeOffset)>,
    /// Minimum alignment for the function, derived from the use of any
    /// pc-relative loads.
    pub alignment: u32,
}

impl CompiledCodeStencil {
    /// Apply function parameters to finalize a stencil into its final form.
    pub fn apply_params(self, params: &FunctionParameters) -> CompiledCode {
        CompiledCode {
            buffer: self.buffer.apply_params(params),
            frame_size: self.frame_size,
            disasm: self.disasm,
            value_labels_ranges: self.value_labels_ranges,
            sized_stackslot_offsets: self.sized_stackslot_offsets,
            dynamic_stackslot_offsets: self.dynamic_stackslot_offsets,
            bb_starts: self.bb_starts,
            bb_edges: self.bb_edges,
            alignment: self.alignment,
        }
    }
}

impl<T: CompilePhase> CompiledCodeBase<T> {
    /// Get a `CodeInfo` describing section sizes from this compilation result.
    pub fn code_info(&self) -> CodeInfo {
        CodeInfo {
            total_size: self.buffer.total_size(),
        }
    }

    /// Returns a reference to the machine code generated for this function compilation.
    pub fn code_buffer(&self) -> &[u8] {
        self.buffer.data()
    }
}

/// Result of compiling a `FunctionStencil`, before applying `FunctionParameters` onto it.
///
/// Only used internally, in a transient manner, for the incremental compilation cache.
pub type CompiledCodeStencil = CompiledCodeBase<Stencil>;

/// `CompiledCode` in its final form (i.e. after `FunctionParameters` have been applied), ready for
/// consumption.
pub type CompiledCode = CompiledCodeBase<Final>;

impl CompiledCode {
    /// If available, return information about the code layout in the
    /// final machine code: the offsets (in bytes) of each basic-block
    /// start, and all basic-block edges.
    pub fn get_code_bb_layout(&self) -> (Vec<usize>, Vec<(usize, usize)>) {
        (
            self.bb_starts.iter().map(|&off| off as usize).collect(),
            self.bb_edges
                .iter()
                .map(|&(from, to)| (from as usize, to as usize))
                .collect(),
        )
    }

    /// Creates unwind information for the function.
    ///
    /// Returns `None` if the function has no unwind information.
    #[cfg(feature = "unwind")]
    pub fn create_unwind_info(
        &self,
        isa: &dyn crate::isa::TargetIsa,
    ) -> CodegenResult<Option<crate::isa::unwind::UnwindInfo>> {
        let unwind_info_kind = match isa.triple().operating_system {
            target_lexicon::OperatingSystem::Windows => UnwindInfoKind::Windows,
            _ => UnwindInfoKind::SystemV,
        };
        isa.emit_unwind_info(self, unwind_info_kind)
    }
}

/// An object that can be used to create the text section of an executable.
///
/// This primarily handles resolving relative relocations at
/// text-section-assembly time rather than at load/link time. This
/// architecture-specific logic is sort of like a linker, but only for one
/// object file at a time.
pub trait TextSectionBuilder {
    /// Appends `data` to the text section with the `align` specified.
    ///
    /// If `labeled` is `true` then this also binds the appended data to the
    /// `n`th label for how many times this has been called with `labeled:
    /// true`. The label target can be passed as the `target` argument to
    /// `resolve_reloc`.
    ///
    /// This function returns the offset at which the data was placed in the
    /// text section.
    fn append(&mut self, labeled: bool, data: &[u8], align: u32) -> u64;

    /// Attempts to resolve a relocation for this function.
    ///
    /// The `offset` is the offset of the relocation, within the text section.
    /// The `reloc` is the kind of relocation.
    /// The `addend` is the value to add to the relocation.
    /// The `target` is the labeled function that is the target of this
    /// relocation.
    ///
    /// Labeled functions are created with the `append` function above by
    /// setting the `labeled` parameter to `true`.
    ///
    /// If this builder does not know how to handle `reloc` then this function
    /// will return `false`. Otherwise this function will return `true` and this
    /// relocation will be resolved in the final bytes returned by `finish`.
    fn resolve_reloc(&mut self, offset: u64, reloc: Reloc, addend: Addend, target: usize) -> bool;

    /// A debug-only option which is used to for
    fn force_veneers(&mut self);

    /// Completes this text section, filling out any final details, and returns
    /// the bytes of the text section.
    fn finish(&mut self) -> Vec<u8>;
}

/// Expected unwind info type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnwindInfoKind {
    /// No unwind info.
    None,
    /// SystemV CIE/FDE unwind info.
    #[cfg(feature = "unwind")]
    SystemV,
    /// Windows X64 Unwind info
    #[cfg(feature = "unwind")]
    Windows,
}
