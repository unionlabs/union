//! In-memory representation of compiled machine code, with labels and fixups to
//! refer to those labels. Handles constant-pool island insertion and also
//! veneer insertion for out-of-range jumps.
//!
//! This code exists to solve three problems:
//!
//! - Branch targets for forward branches are not known until later, when we
//!   emit code in a single pass through the instruction structs.
//!
//! - On many architectures, address references or offsets have limited range.
//!   For example, on AArch64, conditional branches can only target code +/- 1MB
//!   from the branch itself.
//!
//! - The lowering of control flow from the CFG-with-edges produced by
//!   [BlockLoweringOrder](super::BlockLoweringOrder), combined with many empty
//!   edge blocks when the register allocator does not need to insert any
//!   spills/reloads/moves in edge blocks, results in many suboptimal branch
//!   patterns. The lowering also pays no attention to block order, and so
//!   two-target conditional forms (cond-br followed by uncond-br) can often by
//!   avoided because one of the targets is the fallthrough. There are several
//!   cases here where we can simplify to use fewer branches.
//!
//! This "buffer" implements a single-pass code emission strategy (with a later
//! "fixup" pass, but only through recorded fixups, not all instructions). The
//! basic idea is:
//!
//! - Emit branches as they are, including two-target (cond/uncond) compound
//!   forms, but with zero offsets and optimistically assuming the target will be
//!   in range. Record the "fixup" for later. Targets are denoted instead by
//!   symbolic "labels" that are then bound to certain offsets in the buffer as
//!   we emit code. (Nominally, there is a label at the start of every basic
//!   block.)
//!
//! - As we do this, track the offset in the buffer at which the first label
//!   reference "goes out of range". We call this the "deadline". If we reach the
//!   deadline and we still have not bound the label to which an unresolved branch
//!   refers, we have a problem!
//!
//! - To solve this problem, we emit "islands" full of "veneers". An island is
//!   simply a chunk of code inserted in the middle of the code actually produced
//!   by the emitter (e.g., vcode iterating over instruction structs). The emitter
//!   has some awareness of this: it either asks for an island between blocks, so
//!   it is not accidentally executed, or else it emits a branch around the island
//!   when all other options fail (see `Inst::EmitIsland` meta-instruction).
//!
//! - A "veneer" is an instruction (or sequence of instructions) in an "island"
//!   that implements a longer-range reference to a label. The idea is that, for
//!   example, a branch with a limited range can branch to a "veneer" instead,
//!   which is simply a branch in a form that can use a longer-range reference. On
//!   AArch64, for example, conditionals have a +/- 1 MB range, but a conditional
//!   can branch to an unconditional branch which has a +/- 128 MB range. Hence, a
//!   conditional branch's label reference can be fixed up with a "veneer" to
//!   achieve a longer range.
//!
//! - To implement all of this, we require the backend to provide a `LabelUse`
//!   type that implements a trait. This is nominally an enum that records one of
//!   several kinds of references to an offset in code -- basically, a relocation
//!   type -- and will usually correspond to different instruction formats. The
//!   `LabelUse` implementation specifies the maximum range, how to patch in the
//!   actual label location when known, and how to generate a veneer to extend the
//!   range.
//!
//! That satisfies label references, but we still may have suboptimal branch
//! patterns. To clean up the branches, we do a simple "peephole"-style
//! optimization on the fly. To do so, the emitter (e.g., `Inst::emit()`)
//! informs the buffer of branches in the code and, in the case of conditionals,
//! the code that would have been emitted to invert this branch's condition. We
//! track the "latest branches": these are branches that are contiguous up to
//! the current offset. (If any code is emitted after a branch, that branch or
//! run of contiguous branches is no longer "latest".) The latest branches are
//! those that we can edit by simply truncating the buffer and doing something
//! else instead.
//!
//! To optimize branches, we implement several simple rules, and try to apply
//! them to the "latest branches" when possible:
//!
//! - A branch with a label target, when that label is bound to the ending
//!   offset of the branch (the fallthrough location), can be removed altogether,
//!   because the branch would have no effect).
//!
//! - An unconditional branch that starts at a label location, and branches to
//!   another label, results in a "label alias": all references to the label bound
//!   *to* this branch instruction are instead resolved to the *target* of the
//!   branch instruction. This effectively removes empty blocks that just
//!   unconditionally branch to the next block. We call this "branch threading".
//!
//! - A conditional followed by an unconditional, when the conditional branches
//!   to the unconditional's fallthrough, results in (i) the truncation of the
//!   unconditional, (ii) the inversion of the condition's condition, and (iii)
//!   replacement of the conditional's target (using the original target of the
//!   unconditional). This is a fancy way of saying "we can flip a two-target
//!   conditional branch's taken/not-taken targets if it works better with our
//!   fallthrough". To make this work, the emitter actually gives the buffer
//!   *both* forms of every conditional branch: the true form is emitted into the
//!   buffer, and the "inverted" machine-code bytes are provided as part of the
//!   branch-fixup metadata.
//!
//! - An unconditional B preceded by another unconditional P, when B's label(s) have
//!   been redirected to target(B), can be removed entirely. This is an extension
//!   of the branch-threading optimization, and is valid because if we know there
//!   will be no fallthrough into this branch instruction (the prior instruction
//!   is an unconditional jump), and if we know we have successfully redirected
//!   all labels, then this branch instruction is unreachable. Note that this
//!   works because the redirection happens before the label is ever resolved
//!   (fixups happen at island emission time, at which point latest-branches are
//!   cleared, or at the end of emission), so we are sure to catch and redirect
//!   all possible paths to this instruction.
//!
//! # Branch-optimization Correctness
//!
//! The branch-optimization mechanism depends on a few data structures with
//! invariants, which are always held outside the scope of top-level public
//! methods:
//!
//! - The latest-branches list. Each entry describes a span of the buffer
//!   (start/end offsets), the label target, the corresponding fixup-list entry
//!   index, and the bytes (must be the same length) for the inverted form, if
//!   conditional. The list of labels that are bound to the start-offset of this
//!   branch is *complete* (if any label has a resolved offset equal to `start`
//!   and is not an alias, it must appear in this list) and *precise* (no label
//!   in this list can be bound to another offset). No label in this list should
//!   be an alias.  No two branch ranges can overlap, and branches are in
//!   ascending-offset order.
//!
//! - The labels-at-tail list. This contains all MachLabels that have been bound
//!   to (whose resolved offsets are equal to) the tail offset of the buffer.
//!   No label in this list should be an alias.
//!
//! - The label_offsets array, containing the bound offset of a label or
//!   UNKNOWN. No label can be bound at an offset greater than the current
//!   buffer tail.
//!
//! - The label_aliases array, containing another label to which a label is
//!   bound or UNKNOWN. A label's resolved offset is the resolved offset
//!   of the label it is aliased to, if this is set.
//!
//! We argue below, at each method, how the invariants in these data structures
//! are maintained (grep for "Post-invariant").
//!
//! Given these invariants, we argue why each optimization preserves execution
//! semantics below (grep for "Preserves execution semantics").

use crate::binemit::{Addend, CodeOffset, Reloc, StackMap};
use crate::ir::{ExternalName, Opcode, SourceLoc, TrapCode};
use crate::isa::unwind::UnwindInst;
use crate::machinst::{
    BlockIndex, MachInstLabelUse, TextSectionBuilder, VCodeConstant, VCodeConstants, VCodeInst,
};
use crate::timing;
use cranelift_entity::{entity_impl, SecondaryMap};
use log::trace;
use smallvec::SmallVec;
use std::convert::TryFrom;
use std::mem;
use std::string::String;
use std::vec::Vec;

/// A buffer of output to be produced, fixed up, and then emitted to a CodeSink
/// in bulk.
///
/// This struct uses `SmallVec`s to support small-ish function bodies without
/// any heap allocation. As such, it will be several kilobytes large. This is
/// likely fine as long as it is stack-allocated for function emission then
/// thrown away; but beware if many buffer objects are retained persistently.
pub struct MachBuffer<I: VCodeInst> {
    /// The buffer contents, as raw bytes.
    data: SmallVec<[u8; 1024]>,
    /// Any relocations referring to this code. Note that only *external*
    /// relocations are tracked here; references to labels within the buffer are
    /// resolved before emission.
    relocs: SmallVec<[MachReloc; 16]>,
    /// Any trap records referring to this code.
    traps: SmallVec<[MachTrap; 16]>,
    /// Any call site records referring to this code.
    call_sites: SmallVec<[MachCallSite; 16]>,
    /// Any source location mappings referring to this code.
    srclocs: SmallVec<[MachSrcLoc; 64]>,
    /// Any stack maps referring to this code.
    stack_maps: SmallVec<[MachStackMap; 8]>,
    /// Any unwind info at a given location.
    unwind_info: SmallVec<[(CodeOffset, UnwindInst); 8]>,
    /// The current source location in progress (after `start_srcloc()` and
    /// before `end_srcloc()`).  This is a (start_offset, src_loc) tuple.
    cur_srcloc: Option<(CodeOffset, SourceLoc)>,
    /// Known label offsets; `UNKNOWN_LABEL_OFFSET` if unknown.
    label_offsets: SmallVec<[CodeOffset; 16]>,
    /// Label aliases: when one label points to an unconditional jump, and that
    /// jump points to another label, we can redirect references to the first
    /// label immediately to the second.
    ///
    /// Invariant: we don't have label-alias cycles. We ensure this by,
    /// before setting label A to alias label B, resolving B's alias
    /// target (iteratively until a non-aliased label); if B is already
    /// aliased to A, then we cannot alias A back to B.
    label_aliases: SmallVec<[MachLabel; 16]>,
    /// Constants that must be emitted at some point.
    pending_constants: SmallVec<[MachLabelConstant; 16]>,
    /// Fixups that must be performed after all code is emitted.
    fixup_records: SmallVec<[MachLabelFixup<I>; 16]>,
    /// Current deadline at which all constants are flushed and all code labels
    /// are extended by emitting long-range jumps in an island. This flush
    /// should be rare (e.g., on AArch64, the shortest-range PC-rel references
    /// are +/- 1MB for conditional jumps and load-literal instructions), so
    /// it's acceptable to track a minimum and flush-all rather than doing more
    /// detailed "current minimum" / sort-by-deadline trickery.
    island_deadline: CodeOffset,
    /// How many bytes are needed in the worst case for an island, given all
    /// pending constants and fixups.
    island_worst_case_size: CodeOffset,
    /// Latest branches, to facilitate in-place editing for better fallthrough
    /// behavior and empty-block removal.
    latest_branches: SmallVec<[MachBranch; 4]>,
    /// All labels at the current offset (emission tail). This is lazily
    /// cleared: it is actually accurate as long as the current offset is
    /// `labels_at_tail_off`, but if `cur_offset()` has grown larger, it should
    /// be considered as empty.
    ///
    /// For correctness, this *must* be complete (i.e., the vector must contain
    /// all labels whose offsets are resolved to the current tail), because we
    /// rely on it to update labels when we truncate branches.
    labels_at_tail: SmallVec<[MachLabel; 4]>,
    /// The last offset at which `labels_at_tail` is valid. It is conceptually
    /// always describing the tail of the buffer, but we do not clear
    /// `labels_at_tail` eagerly when the tail grows, rather we lazily clear it
    /// when the offset has grown past this (`labels_at_tail_off`) point.
    /// Always <= `cur_offset()`.
    labels_at_tail_off: CodeOffset,
    /// Map used constants to their [MachLabel].
    constant_labels: SecondaryMap<VCodeConstant, MachLabel>,
}

/// A `MachBuffer` once emission is completed: holds generated code and records,
/// without fixups. This allows the type to be independent of the backend.
pub struct MachBufferFinalized {
    /// The buffer contents, as raw bytes.
    data: SmallVec<[u8; 1024]>,
    /// Any relocations referring to this code. Note that only *external*
    /// relocations are tracked here; references to labels within the buffer are
    /// resolved before emission.
    relocs: SmallVec<[MachReloc; 16]>,
    /// Any trap records referring to this code.
    traps: SmallVec<[MachTrap; 16]>,
    /// Any call site records referring to this code.
    call_sites: SmallVec<[MachCallSite; 16]>,
    /// Any source location mappings referring to this code.
    srclocs: SmallVec<[MachSrcLoc; 64]>,
    /// Any stack maps referring to this code.
    stack_maps: SmallVec<[MachStackMap; 8]>,
    /// Any unwind info at a given location.
    pub unwind_info: SmallVec<[(CodeOffset, UnwindInst); 8]>,
}

const UNKNOWN_LABEL_OFFSET: CodeOffset = 0xffff_ffff;
const UNKNOWN_LABEL: MachLabel = MachLabel(0xffff_ffff);

/// Threshold on max length of `labels_at_this_branch` list to avoid
/// unbounded quadratic behavior (see comment below at use-site).
const LABEL_LIST_THRESHOLD: usize = 100;

/// A label refers to some offset in a `MachBuffer`. It may not be resolved at
/// the point at which it is used by emitted code; the buffer records "fixups"
/// for references to the label, and will come back and patch the code
/// appropriately when the label's location is eventually known.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachLabel(u32);
entity_impl!(MachLabel);

impl MachLabel {
    /// Get a label for a block. (The first N MachLabels are always reseved for
    /// the N blocks in the vcode.)
    pub fn from_block(bindex: BlockIndex) -> MachLabel {
        MachLabel(bindex)
    }

    /// Get the numeric label index.
    pub fn get(self) -> u32 {
        self.0
    }

    /// Creates a string representing this label, for convenience.
    pub fn to_string(&self) -> String {
        format!("label{}", self.0)
    }
}

impl Default for MachLabel {
    fn default() -> Self {
        UNKNOWN_LABEL
    }
}

/// A stack map extent, when creating a stack map.
pub enum StackMapExtent {
    /// The stack map starts at this instruction, and ends after the number of upcoming bytes
    /// (note: this is a code offset diff).
    UpcomingBytes(CodeOffset),

    /// The stack map started at the given offset and ends at the current one. This helps
    /// architectures where the instruction size has not a fixed length.
    StartedAtOffset(CodeOffset),
}

impl<I: VCodeInst> MachBuffer<I> {
    /// Create a new section, known to start at `start_offset` and with a size limited to
    /// `length_limit`.
    pub fn new() -> MachBuffer<I> {
        MachBuffer {
            data: SmallVec::new(),
            relocs: SmallVec::new(),
            traps: SmallVec::new(),
            call_sites: SmallVec::new(),
            srclocs: SmallVec::new(),
            stack_maps: SmallVec::new(),
            unwind_info: SmallVec::new(),
            cur_srcloc: None,
            label_offsets: SmallVec::new(),
            label_aliases: SmallVec::new(),
            pending_constants: SmallVec::new(),
            fixup_records: SmallVec::new(),
            island_deadline: UNKNOWN_LABEL_OFFSET,
            island_worst_case_size: 0,
            latest_branches: SmallVec::new(),
            labels_at_tail: SmallVec::new(),
            labels_at_tail_off: 0,
            constant_labels: SecondaryMap::new(),
        }
    }

    /// Debug-only: check invariants of labels and branch-records described
    /// under "Branch-optimization Correctness" above.
    ///
    /// These invariants are checked at branch-simplification
    /// time. Note that they may be temporarily violated at other
    /// times, e.g. after calling `add_{cond,uncond}_branch()` and
    /// before emitting branch bytes.
    fn check_label_branch_invariants(&self) {
        if !cfg!(debug_assertions) || cfg!(fuzzing) {
            return;
        }
        let cur_off = self.cur_offset();
        // Check that every entry in latest_branches has *correct*
        // labels_at_this_branch lists. We do not check completeness because
        // that would require building a reverse index, which is too slow even
        // for a debug invariant check.
        let mut last_end = 0;
        for b in &self.latest_branches {
            debug_assert!(b.start < b.end);
            debug_assert!(b.end <= cur_off);
            debug_assert!(b.start >= last_end);
            last_end = b.end;
            for &l in &b.labels_at_this_branch {
                debug_assert_eq!(self.resolve_label_offset(l), b.start);
                debug_assert_eq!(self.label_aliases[l.0 as usize], UNKNOWN_LABEL);
            }
        }

        // Check that every label is unresolved, or resolved at or
        // before cur_offset. If at cur_offset, must be in
        // `labels_at_tail`. We skip labels that are aliased to
        // others already.
        for (i, &off) in self.label_offsets.iter().enumerate() {
            let label = MachLabel(i as u32);
            if self.label_aliases[i] != UNKNOWN_LABEL {
                continue;
            }
            debug_assert!(off == UNKNOWN_LABEL_OFFSET || off <= cur_off);
            if off == cur_off {
                debug_assert!(
                    self.labels_at_tail_off == cur_off && self.labels_at_tail.contains(&label)
                );
            }
        }

        // Check that every label in `labels_at_tail_off` is precise, i.e.,
        // resolves to the cur offset.
        debug_assert!(self.labels_at_tail_off <= cur_off);
        if self.labels_at_tail_off == cur_off {
            for &l in &self.labels_at_tail {
                debug_assert_eq!(self.resolve_label_offset(l), cur_off);
                debug_assert_eq!(self.label_aliases[l.0 as usize], UNKNOWN_LABEL);
            }
        }
    }

    /// Current offset from start of buffer.
    pub fn cur_offset(&self) -> CodeOffset {
        self.data.len() as CodeOffset
    }

    /// Add a byte.
    pub fn put1(&mut self, value: u8) {
        trace!("MachBuffer: put byte @ {}: {:x}", self.cur_offset(), value);
        self.data.push(value);

        // Post-invariant: conceptual-labels_at_tail contains a complete and
        // precise list of labels bound at `cur_offset()`. We have advanced
        // `cur_offset()`, hence if it had been equal to `labels_at_tail_off`
        // before, it is not anymore (and it cannot become equal, because
        // `labels_at_tail_off` is always <= `cur_offset()`). Thus the list is
        // conceptually empty (even though it is only lazily cleared). No labels
        // can be bound at this new offset (by invariant on `label_offsets`).
        // Hence the invariant holds.
    }

    /// Add 2 bytes.
    pub fn put2(&mut self, value: u16) {
        trace!(
            "MachBuffer: put 16-bit word @ {}: {:x}",
            self.cur_offset(),
            value
        );
        let bytes = value.to_le_bytes();
        self.data.extend_from_slice(&bytes[..]);

        // Post-invariant: as for `put1()`.
    }

    /// Add 4 bytes.
    pub fn put4(&mut self, value: u32) {
        trace!(
            "MachBuffer: put 32-bit word @ {}: {:x}",
            self.cur_offset(),
            value
        );
        let bytes = value.to_le_bytes();
        self.data.extend_from_slice(&bytes[..]);

        // Post-invariant: as for `put1()`.
    }

    /// Add 8 bytes.
    pub fn put8(&mut self, value: u64) {
        trace!(
            "MachBuffer: put 64-bit word @ {}: {:x}",
            self.cur_offset(),
            value
        );
        let bytes = value.to_le_bytes();
        self.data.extend_from_slice(&bytes[..]);

        // Post-invariant: as for `put1()`.
    }

    /// Add a slice of bytes.
    pub fn put_data(&mut self, data: &[u8]) {
        trace!(
            "MachBuffer: put data @ {}: len {}",
            self.cur_offset(),
            data.len()
        );
        self.data.extend_from_slice(data);

        // Post-invariant: as for `put1()`.
    }

    /// Reserve appended space and return a mutable slice referring to it.
    pub fn get_appended_space(&mut self, len: usize) -> &mut [u8] {
        trace!("MachBuffer: put data @ {}: len {}", self.cur_offset(), len);
        let off = self.data.len();
        let new_len = self.data.len() + len;
        self.data.resize(new_len, 0);
        &mut self.data[off..]

        // Post-invariant: as for `put1()`.
    }

    /// Align up to the given alignment.
    pub fn align_to(&mut self, align_to: CodeOffset) {
        trace!("MachBuffer: align to {}", align_to);
        assert!(align_to.is_power_of_two());
        while self.cur_offset() & (align_to - 1) != 0 {
            self.put1(0);
        }

        // Post-invariant: as for `put1()`.
    }

    /// Allocate a `Label` to refer to some offset. May not be bound to a fixed
    /// offset yet.
    pub fn get_label(&mut self) -> MachLabel {
        let l = self.label_offsets.len() as u32;
        self.label_offsets.push(UNKNOWN_LABEL_OFFSET);
        self.label_aliases.push(UNKNOWN_LABEL);
        trace!("MachBuffer: new label -> {:?}", MachLabel(l));
        MachLabel(l)

        // Post-invariant: the only mutation is to add a new label; it has no
        // bound offset yet, so it trivially satisfies all invariants.
    }

    /// Reserve the first N MachLabels for blocks.
    pub fn reserve_labels_for_blocks(&mut self, blocks: BlockIndex) {
        trace!("MachBuffer: first {} labels are for blocks", blocks);
        debug_assert!(self.label_offsets.is_empty());
        self.label_offsets
            .resize(blocks as usize, UNKNOWN_LABEL_OFFSET);
        self.label_aliases.resize(blocks as usize, UNKNOWN_LABEL);

        // Post-invariant: as for `get_label()`.
    }

    /// Reserve the next N MachLabels for constants.
    pub fn reserve_labels_for_constants(&mut self, constants: &VCodeConstants) {
        trace!(
            "MachBuffer: next {} labels are for constants",
            constants.len()
        );
        for c in constants.keys() {
            self.constant_labels[c] = self.get_label();
        }

        // Post-invariant: as for `get_label()`.
    }

    /// Retrieve the reserved label for a constant.
    pub fn get_label_for_constant(&self, constant: VCodeConstant) -> MachLabel {
        self.constant_labels[constant]
    }

    /// Bind a label to the current offset. A label can only be bound once.
    pub fn bind_label(&mut self, label: MachLabel) {
        trace!(
            "MachBuffer: bind label {:?} at offset {}",
            label,
            self.cur_offset()
        );
        debug_assert_eq!(self.label_offsets[label.0 as usize], UNKNOWN_LABEL_OFFSET);
        debug_assert_eq!(self.label_aliases[label.0 as usize], UNKNOWN_LABEL);
        let offset = self.cur_offset();
        self.label_offsets[label.0 as usize] = offset;
        self.lazily_clear_labels_at_tail();
        self.labels_at_tail.push(label);

        // Invariants hold: bound offset of label is <= cur_offset (in fact it
        // is equal). If the `labels_at_tail` list was complete and precise
        // before, it is still, because we have bound this label to the current
        // offset and added it to the list (which contains all labels at the
        // current offset).

        self.check_label_branch_invariants();
        self.optimize_branches();

        // Post-invariant: by `optimize_branches()` (see argument there).
    }

    /// Lazily clear `labels_at_tail` if the tail offset has moved beyond the
    /// offset that it applies to.
    fn lazily_clear_labels_at_tail(&mut self) {
        let offset = self.cur_offset();
        if offset > self.labels_at_tail_off {
            self.labels_at_tail_off = offset;
            self.labels_at_tail.clear();
        }

        // Post-invariant: either labels_at_tail_off was at cur_offset, and
        // state is untouched, or was less than cur_offset, in which case the
        // labels_at_tail list was conceptually empty, and is now actually
        // empty.
    }

    /// Resolve a label to an offset, if known. May return `UNKNOWN_LABEL_OFFSET`.
    pub(crate) fn resolve_label_offset(&self, mut label: MachLabel) -> CodeOffset {
        let mut iters = 0;
        while self.label_aliases[label.0 as usize] != UNKNOWN_LABEL {
            label = self.label_aliases[label.0 as usize];
            // To protect against an infinite loop (despite our assurances to
            // ourselves that the invariants make this impossible), assert out
            // after 1M iterations. The number of basic blocks is limited
            // in most contexts anyway so this should be impossible to hit with
            // a legitimate input.
            iters += 1;
            assert!(iters < 1_000_000, "Unexpected cycle in label aliases");
        }
        self.label_offsets[label.0 as usize]

        // Post-invariant: no mutations.
    }

    /// Emit a reference to the given label with the given reference type (i.e.,
    /// branch-instruction format) at the current offset.  This is like a
    /// relocation, but handled internally.
    ///
    /// This can be called before the branch is actually emitted; fixups will
    /// not happen until an island is emitted or the buffer is finished.
    pub fn use_label_at_offset(&mut self, offset: CodeOffset, label: MachLabel, kind: I::LabelUse) {
        trace!(
            "MachBuffer: use_label_at_offset: offset {} label {:?} kind {:?}",
            offset,
            label,
            kind
        );

        // Add the fixup, and update the worst-case island size based on a
        // veneer for this label use.
        self.fixup_records.push(MachLabelFixup {
            label,
            offset,
            kind,
        });
        if kind.supports_veneer() {
            self.island_worst_case_size += kind.veneer_size();
            self.island_worst_case_size &= !(I::LabelUse::ALIGN - 1);
        }
        let deadline = offset.saturating_add(kind.max_pos_range());
        if deadline < self.island_deadline {
            self.island_deadline = deadline;
        }

        // Post-invariant: no mutations to branches/labels data structures.
    }

    /// Inform the buffer of an unconditional branch at the given offset,
    /// targetting the given label. May be used to optimize branches.
    /// The last added label-use must correspond to this branch.
    /// This must be called when the current offset is equal to `start`; i.e.,
    /// before actually emitting the branch. This implies that for a branch that
    /// uses a label and is eligible for optimizations by the MachBuffer, the
    /// proper sequence is:
    ///
    /// - Call `use_label_at_offset()` to emit the fixup record.
    /// - Call `add_uncond_branch()` to make note of the branch.
    /// - Emit the bytes for the branch's machine code.
    ///
    /// Additional requirement: no labels may be bound between `start` and `end`
    /// (exclusive on both ends).
    pub fn add_uncond_branch(&mut self, start: CodeOffset, end: CodeOffset, target: MachLabel) {
        assert!(self.cur_offset() == start);
        debug_assert!(end > start);
        assert!(!self.fixup_records.is_empty());
        let fixup = self.fixup_records.len() - 1;
        self.lazily_clear_labels_at_tail();
        self.latest_branches.push(MachBranch {
            start,
            end,
            target,
            fixup,
            inverted: None,
            labels_at_this_branch: self.labels_at_tail.clone(),
        });

        // Post-invariant: we asserted branch start is current tail; the list of
        // labels at branch is cloned from list of labels at current tail.
    }

    /// Inform the buffer of a conditional branch at the given offset,
    /// targetting the given label. May be used to optimize branches.
    /// The last added label-use must correspond to this branch.
    ///
    /// Additional requirement: no labels may be bound between `start` and `end`
    /// (exclusive on both ends).
    pub fn add_cond_branch(
        &mut self,
        start: CodeOffset,
        end: CodeOffset,
        target: MachLabel,
        inverted: &[u8],
    ) {
        assert!(self.cur_offset() == start);
        debug_assert!(end > start);
        assert!(!self.fixup_records.is_empty());
        debug_assert!(inverted.len() == (end - start) as usize);
        let fixup = self.fixup_records.len() - 1;
        let inverted = Some(SmallVec::from(inverted));
        self.lazily_clear_labels_at_tail();
        self.latest_branches.push(MachBranch {
            start,
            end,
            target,
            fixup,
            inverted,
            labels_at_this_branch: self.labels_at_tail.clone(),
        });

        // Post-invariant: we asserted branch start is current tail; labels at
        // branch list is cloned from list of labels at current tail.
    }

    fn truncate_last_branch(&mut self) {
        self.lazily_clear_labels_at_tail();
        // Invariants hold at this point.

        let b = self.latest_branches.pop().unwrap();
        assert!(b.end == self.cur_offset());

        // State:
        //    [PRE CODE]
        //  Offset b.start, b.labels_at_this_branch:
        //    [BRANCH CODE]
        //  cur_off, self.labels_at_tail -->
        //    (end of buffer)
        self.data.truncate(b.start as usize);
        self.fixup_records.truncate(b.fixup);
        while let Some(mut last_srcloc) = self.srclocs.last_mut() {
            if last_srcloc.end <= b.start {
                break;
            }
            if last_srcloc.start < b.start {
                last_srcloc.end = b.start;
                break;
            }
            self.srclocs.pop();
        }
        // State:
        //    [PRE CODE]
        //  cur_off, Offset b.start, b.labels_at_this_branch:
        //    (end of buffer)
        //
        //  self.labels_at_tail -->  (past end of buffer)
        let cur_off = self.cur_offset();
        self.labels_at_tail_off = cur_off;
        // State:
        //    [PRE CODE]
        //  cur_off, Offset b.start, b.labels_at_this_branch,
        //  self.labels_at_tail:
        //    (end of buffer)
        //
        // resolve_label_offset(l) for l in labels_at_tail:
        //    (past end of buffer)

        trace!(
            "truncate_last_branch: truncated {:?}; off now {}",
            b,
            cur_off
        );

        // Fix up resolved label offsets for labels at tail.
        for &l in &self.labels_at_tail {
            self.label_offsets[l.0 as usize] = cur_off;
        }
        // Old labels_at_this_branch are now at cur_off.
        self.labels_at_tail
            .extend(b.labels_at_this_branch.into_iter());

        // Post-invariant: this operation is defined to truncate the buffer,
        // which moves cur_off backward, and to move labels at the end of the
        // buffer back to the start-of-branch offset.
        //
        // latest_branches satisfies all invariants:
        // - it has no branches past the end of the buffer (branches are in
        //   order, we removed the last one, and we truncated the buffer to just
        //   before the start of that branch)
        // - no labels were moved to lower offsets than the (new) cur_off, so
        //   the labels_at_this_branch list for any other branch need not change.
        //
        // labels_at_tail satisfies all invariants:
        // - all labels that were at the tail after the truncated branch are
        //   moved backward to just before the branch, which becomes the new tail;
        //   thus every element in the list should remain (ensured by `.extend()`
        //   above).
        // - all labels that refer to the new tail, which is the start-offset of
        //   the truncated branch, must be present. The `labels_at_this_branch`
        //   list in the truncated branch's record is a complete and precise list
        //   of exactly these labels; we append these to labels_at_tail.
        // - labels_at_tail_off is at cur_off after truncation occurs, so the
        //   list is valid (not to be lazily cleared).
        //
        // The stated operation was performed:
        // - For each label at the end of the buffer prior to this method, it
        //   now resolves to the new (truncated) end of the buffer: it must have
        //   been in `labels_at_tail` (this list is precise and complete, and
        //   the tail was at the end of the truncated branch on entry), and we
        //   iterate over this list and set `label_offsets` to the new tail.
        //   None of these labels could have been an alias (by invariant), so
        //   `label_offsets` is authoritative for each.
        // - No other labels will be past the end of the buffer, because of the
        //   requirement that no labels be bound to the middle of branch ranges
        //   (see comments to `add_{cond,uncond}_branch()`).
        // - The buffer is truncated to just before the last branch, and the
        //   fixup record referring to that last branch is removed.
    }

    fn optimize_branches(&mut self) {
        self.lazily_clear_labels_at_tail();
        // Invariants valid at this point.

        trace!(
            "enter optimize_branches:\n b = {:?}\n l = {:?}\n f = {:?}",
            self.latest_branches,
            self.labels_at_tail,
            self.fixup_records
        );

        // We continue to munch on branches at the tail of the buffer until no
        // more rules apply. Note that the loop only continues if a branch is
        // actually truncated (or if labels are redirected away from a branch),
        // so this always makes progress.
        while let Some(b) = self.latest_branches.last() {
            let cur_off = self.cur_offset();
            trace!("optimize_branches: last branch {:?} at off {}", b, cur_off);
            // If there has been any code emission since the end of the last branch or
            // label definition, then there's nothing we can edit (because we
            // don't move code once placed, only back up and overwrite), so
            // clear the records and finish.
            if b.end < cur_off {
                break;
            }

            // If the "labels at this branch" list on this branch is
            // longer than a threshold, don't do any simplification,
            // and let the branch remain to separate those labels from
            // the current tail. This avoids quadratic behavior (see
            // #3468): otherwise, if a long string of "goto next;
            // next:" patterns are emitted, all of the labels will
            // coalesce into a long list of aliases for the current
            // buffer tail. We must track all aliases of the current
            // tail for correctness, but we are also allowed to skip
            // optimization (removal) of any branch, so we take the
            // escape hatch here and let it stand. In effect this
            // "spreads" the many thousands of labels in the
            // pathological case among an actual (harmless but
            // suboptimal) instruction once per N labels.
            if b.labels_at_this_branch.len() > LABEL_LIST_THRESHOLD {
                break;
            }

            // Invariant: we are looking at a branch that ends at the tail of
            // the buffer.

            // For any branch, conditional or unconditional:
            // - If the target is a label at the current offset, then remove
            //   the conditional branch, and reset all labels that targetted
            //   the current offset (end of branch) to the truncated
            //   end-of-code.
            //
            // Preserves execution semantics: a branch to its own fallthrough
            // address is equivalent to a no-op; in both cases, nextPC is the
            // fallthrough.
            if self.resolve_label_offset(b.target) == cur_off {
                trace!("branch with target == cur off; truncating");
                self.truncate_last_branch();
                continue;
            }

            // If latest is an unconditional branch:
            //
            // - If the branch's target is not its own start address, then for
            //   each label at the start of branch, make the label an alias of the
            //   branch target, and remove the label from the "labels at this
            //   branch" list.
            //
            //   - Preserves execution semantics: an unconditional branch's
            //     only effect is to set PC to a new PC; this change simply
            //     collapses one step in the step-semantics.
            //
            //   - Post-invariant: the labels that were bound to the start of
            //     this branch become aliases, so they must not be present in any
            //     labels-at-this-branch list or the labels-at-tail list. The
            //     labels are removed form the latest-branch record's
            //     labels-at-this-branch list, and are never placed in the
            //     labels-at-tail list. Furthermore, it is correct that they are
            //     not in either list, because they are now aliases, and labels
            //     that are aliases remain aliases forever.
            //
            // - If there is a prior unconditional branch that ends just before
            //   this one begins, and this branch has no labels bound to its
            //   start, then we can truncate this branch, because it is entirely
            //   unreachable (we have redirected all labels that make it
            //   reachable otherwise). Do so and continue around the loop.
            //
            //   - Preserves execution semantics: the branch is unreachable,
            //     because execution can only flow into an instruction from the
            //     prior instruction's fallthrough or from a branch bound to that
            //     instruction's start offset. Unconditional branches have no
            //     fallthrough, so if the prior instruction is an unconditional
            //     branch, no fallthrough entry can happen. The
            //     labels-at-this-branch list is complete (by invariant), so if it
            //     is empty, then the instruction is entirely unreachable. Thus,
            //     it can be removed.
            //
            //   - Post-invariant: ensured by truncate_last_branch().
            //
            // - If there is a prior conditional branch whose target label
            //   resolves to the current offset (branches around the
            //   unconditional branch), then remove the unconditional branch,
            //   and make the target of the unconditional the target of the
            //   conditional instead.
            //
            //   - Preserves execution semantics: previously we had:
            //
            //         L1:
            //            cond_br L2
            //            br L3
            //         L2:
            //            (end of buffer)
            //
            //     by removing the last branch, we have:
            //
            //         L1:
            //            cond_br L2
            //         L2:
            //            (end of buffer)
            //
            //     we then fix up the records for the conditional branch to
            //     have:
            //
            //         L1:
            //           cond_br.inverted L3
            //         L2:
            //
            //     In the original code, control flow reaches L2 when the
            //     conditional branch's predicate is true, and L3 otherwise. In
            //     the optimized code, the same is true.
            //
            //   - Post-invariant: all edits to latest_branches and
            //     labels_at_tail are performed by `truncate_last_branch()`,
            //     which maintains the invariants at each step.

            if b.is_uncond() {
                // Set any label equal to current branch's start as an alias of
                // the branch's target, if the target is not the branch itself
                // (i.e., an infinite loop).
                //
                // We cannot perform this aliasing if the target of this branch
                // ultimately aliases back here; if so, we need to keep this
                // branch, so break out of this loop entirely (and clear the
                // latest-branches list below).
                //
                // Note that this check is what prevents cycles from forming in
                // `self.label_aliases`. To see why, consider an arbitrary start
                // state:
                //
                // label_aliases[L1] = L2, label_aliases[L2] = L3, ..., up to
                // Ln, which is not aliased.
                //
                // We would create a cycle if we assigned label_aliases[Ln]
                // = L1.  Note that the below assignment is the only write
                // to label_aliases.
                //
                // By our other invariants, we have that Ln (`l` below)
                // resolves to the offset `b.start`, because it is in the
                // set `b.labels_at_this_branch`.
                //
                // If L1 were already aliased, through some arbitrarily deep
                // chain, to Ln, then it must also resolve to this offset
                // `b.start`.
                //
                // By checking the resolution of `L1` against this offset,
                // and aborting this branch-simplification if they are
                // equal, we prevent the below assignment from ever creating
                // a cycle.
                if self.resolve_label_offset(b.target) != b.start {
                    let redirected = b.labels_at_this_branch.len();
                    for &l in &b.labels_at_this_branch {
                        trace!(
                            " -> label at start of branch {:?} redirected to target {:?}",
                            l,
                            b.target
                        );
                        self.label_aliases[l.0 as usize] = b.target;
                        // NOTE: we continue to ensure the invariant that labels
                        // pointing to tail of buffer are in `labels_at_tail`
                        // because we already ensured above that the last branch
                        // cannot have a target of `cur_off`; so we never have
                        // to put the label into `labels_at_tail` when moving it
                        // here.
                    }
                    // Maintain invariant: all branches have been redirected
                    // and are no longer pointing at the start of this branch.
                    let mut_b = self.latest_branches.last_mut().unwrap();
                    mut_b.labels_at_this_branch.clear();

                    if redirected > 0 {
                        trace!(" -> after label redirects, restarting loop");
                        continue;
                    }
                } else {
                    break;
                }

                let b = self.latest_branches.last().unwrap();

                // Examine any immediately preceding branch.
                if self.latest_branches.len() > 1 {
                    let prev_b = &self.latest_branches[self.latest_branches.len() - 2];
                    trace!(" -> more than one branch; prev_b = {:?}", prev_b);
                    // This uncond is immediately after another uncond; we
                    // should have already redirected labels to this uncond away
                    // (but check to be sure); so we can truncate this uncond.
                    if prev_b.is_uncond()
                        && prev_b.end == b.start
                        && b.labels_at_this_branch.is_empty()
                    {
                        trace!(" -> uncond follows another uncond; truncating");
                        self.truncate_last_branch();
                        continue;
                    }

                    // This uncond is immediately after a conditional, and the
                    // conditional's target is the end of this uncond, and we've
                    // already redirected labels to this uncond away; so we can
                    // truncate this uncond, flip the sense of the conditional, and
                    // set the conditional's target (in `latest_branches` and in
                    // `fixup_records`) to the uncond's target.
                    if prev_b.is_cond()
                        && prev_b.end == b.start
                        && self.resolve_label_offset(prev_b.target) == cur_off
                    {
                        trace!(" -> uncond follows a conditional, and conditional's target resolves to current offset");
                        // Save the target of the uncond (this becomes the
                        // target of the cond), and truncate the uncond.
                        let target = b.target;
                        let data = prev_b.inverted.clone().unwrap();
                        self.truncate_last_branch();

                        // Mutate the code and cond branch.
                        let off_before_edit = self.cur_offset();
                        let prev_b = self.latest_branches.last_mut().unwrap();
                        let not_inverted = SmallVec::from(
                            &self.data[(prev_b.start as usize)..(prev_b.end as usize)],
                        );

                        // Low-level edit: replaces bytes of branch with
                        // inverted form. cur_off remains the same afterward, so
                        // we do not need to modify label data structures.
                        self.data.truncate(prev_b.start as usize);
                        self.data.extend_from_slice(&data[..]);

                        // Save the original code as the inversion of the
                        // inverted branch, in case we later edit this branch
                        // again.
                        prev_b.inverted = Some(not_inverted);
                        self.fixup_records[prev_b.fixup].label = target;
                        trace!(" -> reassigning target of condbr to {:?}", target);
                        prev_b.target = target;
                        debug_assert_eq!(off_before_edit, self.cur_offset());
                        continue;
                    }
                }
            }

            // If we couldn't do anything with the last branch, then break.
            break;
        }

        self.purge_latest_branches();

        trace!(
            "leave optimize_branches:\n b = {:?}\n l = {:?}\n f = {:?}",
            self.latest_branches,
            self.labels_at_tail,
            self.fixup_records
        );
    }

    fn purge_latest_branches(&mut self) {
        // All of our branch simplification rules work only if a branch ends at
        // the tail of the buffer, with no following code; and branches are in
        // order in latest_branches; so if the last entry ends prior to
        // cur_offset, then clear all entries.
        let cur_off = self.cur_offset();
        if let Some(l) = self.latest_branches.last() {
            if l.end < cur_off {
                trace!("purge_latest_branches: removing branch {:?}", l);
                self.latest_branches.clear();
            }
        }

        // Post-invariant: no invariant requires any branch to appear in
        // `latest_branches`; it is always optional. The list-clear above thus
        // preserves all semantics.
    }

    /// Emit a constant at some point in the future, binding the given label to
    /// its offset. The constant will be placed at most `max_distance` from the
    /// current offset.
    pub fn defer_constant(
        &mut self,
        label: MachLabel,
        align: CodeOffset,
        data: &[u8],
        max_distance: CodeOffset,
    ) {
        trace!(
            "defer_constant: eventually emit {} bytes aligned to {} at label {:?}",
            data.len(),
            align,
            label
        );
        let deadline = self.cur_offset().saturating_add(max_distance);
        self.island_worst_case_size += data.len() as CodeOffset;
        self.island_worst_case_size =
            (self.island_worst_case_size + I::LabelUse::ALIGN - 1) & !(I::LabelUse::ALIGN - 1);
        self.pending_constants.push(MachLabelConstant {
            label,
            align,
            data: SmallVec::from(data),
        });
        if deadline < self.island_deadline {
            self.island_deadline = deadline;
        }
    }

    /// Is an island needed within the next N bytes?
    pub fn island_needed(&self, distance: CodeOffset) -> bool {
        self.worst_case_end_of_island(distance) > self.island_deadline
    }

    /// Returns the maximal offset that islands can reach if `distance` more
    /// bytes are appended.
    ///
    /// This is used to determine if veneers need insertions since jumps that
    /// can't reach past this point must get a veneer of some form.
    fn worst_case_end_of_island(&self, distance: CodeOffset) -> CodeOffset {
        self.cur_offset()
            .saturating_add(distance)
            .saturating_add(self.island_worst_case_size)
    }

    /// Emit all pending constants and required pending veneers.
    ///
    /// Should only be called if `island_needed()` returns true, i.e., if we
    /// actually reach a deadline. It's not necessarily a problem to do so
    /// otherwise but it may result in unnecessary work during emission.
    pub fn emit_island(&mut self, distance: CodeOffset) {
        self.emit_island_maybe_forced(false, distance);
    }

    /// Same as `emit_island`, but an internal API with a `force_veneers`
    /// argument to force all veneers to always get emitted for debugging.
    fn emit_island_maybe_forced(&mut self, force_veneers: bool, distance: CodeOffset) {
        // We're going to purge fixups, so no latest-branch editing can happen
        // anymore.
        self.latest_branches.clear();

        // Reset internal calculations about islands since we're going to
        // change the calculus as we apply fixups. The `forced_threshold` is
        // used here to determine whether jumps to unknown labels will require
        // a veneer or not.
        let forced_threshold = self.worst_case_end_of_island(distance);
        self.island_deadline = UNKNOWN_LABEL_OFFSET;
        self.island_worst_case_size = 0;

        // First flush out all constants so we have more labels in case fixups
        // are applied against these labels.
        for MachLabelConstant { label, align, data } in mem::take(&mut self.pending_constants) {
            self.align_to(align);
            self.bind_label(label);
            self.put_data(&data[..]);
        }

        for fixup in mem::take(&mut self.fixup_records) {
            trace!("emit_island: fixup {:?}", fixup);
            let MachLabelFixup {
                label,
                offset,
                kind,
            } = fixup;
            let label_offset = self.resolve_label_offset(label);
            let start = offset as usize;
            let end = (offset + kind.patch_size()) as usize;

            if label_offset != UNKNOWN_LABEL_OFFSET {
                // If the offset of the label for this fixup is known then
                // we're going to do something here-and-now. We're either going
                // to patch the original offset because it's an in-bounds jump,
                // or we're going to generate a veneer, patch the fixup to jump
                // to the veneer, and then keep going.
                //
                // If the label comes after the original fixup, then we should
                // be guaranteed that the jump is in-bounds. Otherwise there's
                // a bug somewhere because this method wasn't called soon
                // enough. All forward-jumps are tracked and should get veneers
                // before their deadline comes and they're unable to jump
                // further.
                //
                // Otherwise if the label is before the fixup, then that's a
                // backwards jump. If it's past the maximum negative range
                // then we'll emit a veneer that to jump forward to which can
                // then jump backwards.
                let veneer_required = if label_offset >= offset {
                    assert!((label_offset - offset) <= kind.max_pos_range());
                    false
                } else {
                    (offset - label_offset) > kind.max_neg_range()
                };
                trace!(
                    " -> label_offset = {}, known, required = {} (pos {} neg {})",
                    label_offset,
                    veneer_required,
                    kind.max_pos_range(),
                    kind.max_neg_range()
                );

                if (force_veneers && kind.supports_veneer()) || veneer_required {
                    self.emit_veneer(label, offset, kind);
                } else {
                    let slice = &mut self.data[start..end];
                    trace!("patching in-range!");
                    kind.patch(slice, offset, label_offset);
                }
            } else {
                // If the offset of this label is not known at this time then
                // there's one of two possibilities:
                //
                // * First we may be about to exceed the maximum jump range of
                //   this fixup. In that case a veneer is inserted to buy some
                //   more budget for the forward-jump. It's guaranteed that the
                //   label will eventually come after where we're at, so we know
                //   that the forward jump is necessary.
                //
                // * Otherwise we're still within range of the forward jump but
                //   the precise target isn't known yet. In that case we
                //   enqueue the fixup to get processed later.
                if forced_threshold - offset > kind.max_pos_range() {
                    self.emit_veneer(label, offset, kind);
                } else {
                    self.use_label_at_offset(offset, label, kind);
                }
            }
        }
    }

    /// Emits a "veneer" the `kind` code at `offset` to jump to `label`.
    ///
    /// This will generate extra machine code, using `kind`, to get a
    /// larger-jump-kind than `kind` allows. The code at `offset` is then
    /// patched to jump to our new code, and then the new code is enqueued for
    /// a fixup to get processed at some later time.
    fn emit_veneer(&mut self, label: MachLabel, offset: CodeOffset, kind: I::LabelUse) {
        // If this `kind` doesn't support a veneer then that's a bug in the
        // backend because we need to implement support for such a veneer.
        assert!(
            kind.supports_veneer(),
            "jump beyond the range of {:?} but a veneer isn't supported",
            kind,
        );

        // Allocate space for a veneer in the island.
        self.align_to(I::LabelUse::ALIGN);
        let veneer_offset = self.cur_offset();
        trace!("making a veneer at {}", veneer_offset);
        let start = offset as usize;
        let end = (offset + kind.patch_size()) as usize;
        let slice = &mut self.data[start..end];
        // Patch the original label use to refer to the veneer.
        trace!(
            "patching original at offset {} to veneer offset {}",
            offset,
            veneer_offset
        );
        kind.patch(slice, offset, veneer_offset);
        // Generate the veneer.
        let veneer_slice = self.get_appended_space(kind.veneer_size() as usize);
        let (veneer_fixup_off, veneer_label_use) =
            kind.generate_veneer(veneer_slice, veneer_offset);
        trace!(
            "generated veneer; fixup offset {}, label_use {:?}",
            veneer_fixup_off,
            veneer_label_use
        );
        // Register a new use of `label` with our new veneer fixup and offset.
        // This'll recalculate deadlines accordingly and enqueue this fixup to
        // get processed at some later time.
        self.use_label_at_offset(veneer_fixup_off, label, veneer_label_use);
    }

    fn finish_emission_maybe_forcing_veneers(&mut self, force_veneers: bool) {
        while !self.pending_constants.is_empty() || !self.fixup_records.is_empty() {
            // `emit_island()` will emit any pending veneers and constants, and
            // as a side-effect, will also take care of any fixups with resolved
            // labels eagerly.
            self.emit_island_maybe_forced(force_veneers, u32::MAX);
        }

        // Ensure that all labels have been fixed up after the last island is emitted. This is a
        // full (release-mode) assert because an unresolved label means the emitted code is
        // incorrect.
        assert!(self.fixup_records.is_empty());
    }

    /// Finish any deferred emissions and/or fixups.
    pub fn finish(mut self) -> MachBufferFinalized {
        let _tt = timing::vcode_emit_finish();

        self.finish_emission_maybe_forcing_veneers(false);

        let mut srclocs = self.srclocs;
        srclocs.sort_by_key(|entry| entry.start);

        MachBufferFinalized {
            data: self.data,
            relocs: self.relocs,
            traps: self.traps,
            call_sites: self.call_sites,
            srclocs,
            stack_maps: self.stack_maps,
            unwind_info: self.unwind_info,
        }
    }

    /// Add an external relocation at the current offset.
    pub fn add_reloc(
        &mut self,
        srcloc: SourceLoc,
        kind: Reloc,
        name: &ExternalName,
        addend: Addend,
    ) {
        let name = name.clone();
        // FIXME(#3277): This should use `I::LabelUse::from_reloc` to optionally
        // generate a label-use statement to track whether an island is possibly
        // needed to escape this function to actually get to the external name.
        // This is most likely to come up on AArch64 where calls between
        // functions use a 26-bit signed offset which gives +/- 64MB. This means
        // that if a function is 128MB in size and there's a call in the middle
        // it's impossible to reach the actual target. Also, while it's
        // technically possible to jump to the start of a function and then jump
        // further, island insertion below always inserts islands after
        // previously appended code so for Cranelift's own implementation this
        // is also a problem for 64MB functions on AArch64 which start with a
        // call instruction, those won't be able to escape.
        //
        // Ideally what needs to happen here is that a `LabelUse` is
        // transparently generated (or call-sites of this function are audited
        // to generate a `LabelUse` instead) and tracked internally. The actual
        // relocation would then change over time if and when a veneer is
        // inserted, where the relocation here would be patched by this
        // `MachBuffer` to jump to the veneer. The problem, though, is that all
        // this still needs to end up, in the case of a singular function,
        // generating a final relocation pointing either to this particular
        // relocation or to the veneer inserted. Additionally
        // `MachBuffer` needs the concept of a label which will never be
        // resolved, so `emit_island` doesn't trip over not actually ever
        // knowning what some labels are. Currently the loop in
        // `finish_emission_maybe_forcing_veneers` would otherwise infinitely
        // loop.
        //
        // For now this means that because relocs aren't tracked at all that
        // AArch64 functions have a rough size limits of 64MB. For now that's
        // somewhat reasonable and the failure mode is a panic in `MachBuffer`
        // when a relocation can't otherwise be resolved later, so it shouldn't
        // actually result in any memory unsafety or anything like that.
        self.relocs.push(MachReloc {
            offset: self.data.len() as CodeOffset,
            srcloc,
            kind,
            name,
            addend,
        });
    }

    /// Add a trap record at the current offset.
    pub fn add_trap(&mut self, srcloc: SourceLoc, code: TrapCode) {
        self.traps.push(MachTrap {
            offset: self.data.len() as CodeOffset,
            srcloc,
            code,
        });
    }

    /// Add a call-site record at the current offset.
    pub fn add_call_site(&mut self, srcloc: SourceLoc, opcode: Opcode) {
        debug_assert!(
            opcode.is_call(),
            "adding call site info for a non-call instruction."
        );
        self.call_sites.push(MachCallSite {
            ret_addr: self.data.len() as CodeOffset,
            srcloc,
            opcode,
        });
    }

    /// Add an unwind record at the current offset.
    pub fn add_unwind(&mut self, unwind: UnwindInst) {
        self.unwind_info.push((self.cur_offset(), unwind));
    }

    /// Set the `SourceLoc` for code from this offset until the offset at the
    /// next call to `end_srcloc()`.
    pub fn start_srcloc(&mut self, loc: SourceLoc) {
        self.cur_srcloc = Some((self.cur_offset(), loc));
    }

    /// Mark the end of the `SourceLoc` segment started at the last
    /// `start_srcloc()` call.
    pub fn end_srcloc(&mut self) {
        let (start, loc) = self
            .cur_srcloc
            .take()
            .expect("end_srcloc() called without start_srcloc()");
        let end = self.cur_offset();
        // Skip zero-length extends.
        debug_assert!(end >= start);
        if end > start {
            self.srclocs.push(MachSrcLoc { start, end, loc });
        }
    }

    /// Add stack map metadata for this program point: a set of stack offsets
    /// (from SP upward) that contain live references.
    ///
    /// The `offset_to_fp` value is the offset from the nominal SP (at which the `stack_offsets`
    /// are based) and the FP value. By subtracting `offset_to_fp` from each `stack_offsets`
    /// element, one can obtain live-reference offsets from FP instead.
    pub fn add_stack_map(&mut self, extent: StackMapExtent, stack_map: StackMap) {
        let (start, end) = match extent {
            StackMapExtent::UpcomingBytes(insn_len) => {
                let start_offset = self.cur_offset();
                (start_offset, start_offset + insn_len)
            }
            StackMapExtent::StartedAtOffset(start_offset) => {
                let end_offset = self.cur_offset();
                debug_assert!(end_offset >= start_offset);
                (start_offset, end_offset)
            }
        };
        self.stack_maps.push(MachStackMap {
            offset: start,
            offset_end: end,
            stack_map,
        });
    }
}

impl MachBufferFinalized {
    /// Get a list of source location mapping tuples in sorted-by-start-offset order.
    pub fn get_srclocs_sorted(&self) -> &[MachSrcLoc] {
        &self.srclocs[..]
    }

    /// Get the total required size for the code.
    pub fn total_size(&self) -> CodeOffset {
        self.data.len() as CodeOffset
    }

    /// Return the code in this mach buffer as a hex string for testing purposes.
    pub fn stringify_code_bytes(&self) -> String {
        // This is pretty lame, but whatever ..
        use std::fmt::Write;
        let mut s = String::with_capacity(self.data.len() * 2);
        for b in &self.data {
            write!(&mut s, "{:02X}", b).unwrap();
        }
        s
    }

    /// Get the code bytes.
    pub fn data(&self) -> &[u8] {
        // N.B.: we emit every section into the .text section as far as
        // the `CodeSink` is concerned; we do not bother to segregate
        // the contents into the actual program text, the jumptable and the
        // rodata (constant pool). This allows us to generate code assuming
        // that these will not be relocated relative to each other, and avoids
        // having to designate each section as belonging in one of the three
        // fixed categories defined by `CodeSink`. If this becomes a problem
        // later (e.g. because of memory permissions or similar), we can
        // add this designation and segregate the output; take care, however,
        // to add the appropriate relocations in this case.

        &self.data[..]
    }

    /// Get the list of external relocations for this code.
    pub fn relocs(&self) -> &[MachReloc] {
        &self.relocs[..]
    }

    /// Get the list of trap records for this code.
    pub fn traps(&self) -> &[MachTrap] {
        &self.traps[..]
    }

    /// Get the stack map metadata for this code.
    pub fn stack_maps(&self) -> &[MachStackMap] {
        &self.stack_maps[..]
    }

    /// Get the list of call sites for this code.
    pub fn call_sites(&self) -> &[MachCallSite] {
        &self.call_sites[..]
    }
}

/// A constant that is deferred to the next constant-pool opportunity.
struct MachLabelConstant {
    /// This label will refer to the constant's offset.
    label: MachLabel,
    /// Required alignment.
    align: CodeOffset,
    /// This data will be emitted when able.
    data: SmallVec<[u8; 16]>,
}

/// A fixup to perform on the buffer once code is emitted. Fixups always refer
/// to labels and patch the code based on label offsets. Hence, they are like
/// relocations, but internal to one buffer.
#[derive(Debug)]
struct MachLabelFixup<I: VCodeInst> {
    /// The label whose offset controls this fixup.
    label: MachLabel,
    /// The offset to fix up / patch to refer to this label.
    offset: CodeOffset,
    /// The kind of fixup. This is architecture-specific; each architecture may have,
    /// e.g., several types of branch instructions, each with differently-sized
    /// offset fields and different places within the instruction to place the
    /// bits.
    kind: I::LabelUse,
}

/// A relocation resulting from a compilation.
#[derive(Clone, Debug)]
pub struct MachReloc {
    /// The offset at which the relocation applies, *relative to the
    /// containing section*.
    pub offset: CodeOffset,
    /// The original source location.
    pub srcloc: SourceLoc,
    /// The kind of relocation.
    pub kind: Reloc,
    /// The external symbol / name to which this relocation refers.
    pub name: ExternalName,
    /// The addend to add to the symbol value.
    pub addend: i64,
}

/// A trap record resulting from a compilation.
#[derive(Clone, Debug)]
pub struct MachTrap {
    /// The offset at which the trap instruction occurs, *relative to the
    /// containing section*.
    pub offset: CodeOffset,
    /// The original source location.
    pub srcloc: SourceLoc,
    /// The trap code.
    pub code: TrapCode,
}

/// A call site record resulting from a compilation.
#[derive(Clone, Debug)]
pub struct MachCallSite {
    /// The offset of the call's return address, *relative to the containing section*.
    pub ret_addr: CodeOffset,
    /// The original source location.
    pub srcloc: SourceLoc,
    /// The call's opcode.
    pub opcode: Opcode,
}

/// A source-location mapping resulting from a compilation.
#[derive(Clone, Debug)]
pub struct MachSrcLoc {
    /// The start of the region of code corresponding to a source location.
    /// This is relative to the start of the function, not to the start of the
    /// section.
    pub start: CodeOffset,
    /// The end of the region of code corresponding to a source location.
    /// This is relative to the start of the section, not to the start of the
    /// section.
    pub end: CodeOffset,
    /// The source location.
    pub loc: SourceLoc,
}

/// Record of stack map metadata: stack offsets containing references.
#[derive(Clone, Debug)]
pub struct MachStackMap {
    /// The code offset at which this stack map applies.
    pub offset: CodeOffset,
    /// The code offset just past the "end" of the instruction: that is, the
    /// offset of the first byte of the following instruction, or equivalently,
    /// the start offset plus the instruction length.
    pub offset_end: CodeOffset,
    /// The stack map itself.
    pub stack_map: StackMap,
}

/// Record of branch instruction in the buffer, to facilitate editing.
#[derive(Clone, Debug)]
struct MachBranch {
    start: CodeOffset,
    end: CodeOffset,
    target: MachLabel,
    fixup: usize,
    inverted: Option<SmallVec<[u8; 8]>>,
    /// All labels pointing to the start of this branch. For correctness, this
    /// *must* be complete (i.e., must contain all labels whose resolved offsets
    /// are at the start of this branch): we rely on being able to redirect all
    /// labels that could jump to this branch before removing it, if it is
    /// otherwise unreachable.
    labels_at_this_branch: SmallVec<[MachLabel; 4]>,
}

impl MachBranch {
    fn is_cond(&self) -> bool {
        self.inverted.is_some()
    }
    fn is_uncond(&self) -> bool {
        self.inverted.is_none()
    }
}

/// Implementation of the `TextSectionBuilder` trait backed by `MachBuffer`.
///
/// Note that `MachBuffer` was primarily written for intra-function references
/// of jumps between basic blocks, but it's also quite usable for entire text
/// sections and resolving references between functions themselves. This
/// builder interprets "blocks" as labeled functions for the purposes of
/// resolving labels internally in the buffer.
pub struct MachTextSectionBuilder<I: VCodeInst> {
    buf: MachBuffer<I>,
    next_func: u32,
    force_veneers: bool,
}

impl<I: VCodeInst> MachTextSectionBuilder<I> {
    pub fn new(num_funcs: u32) -> MachTextSectionBuilder<I> {
        let mut buf = MachBuffer::new();
        buf.reserve_labels_for_blocks(num_funcs);
        MachTextSectionBuilder {
            buf,
            next_func: 0,
            force_veneers: false,
        }
    }
}

impl<I: VCodeInst> TextSectionBuilder for MachTextSectionBuilder<I> {
    fn append(&mut self, named: bool, func: &[u8], align: u32) -> u64 {
        // Conditionally emit an island if it's necessary to resolve jumps
        // between functions which are too far away.
        let size = func.len() as u32;
        if self.force_veneers || self.buf.island_needed(size) {
            self.buf.emit_island_maybe_forced(self.force_veneers, size);
        }

        self.buf.align_to(align);
        let pos = self.buf.cur_offset();
        if named {
            self.buf.bind_label(MachLabel::from_block(self.next_func));
            self.next_func += 1;
        }
        self.buf.put_data(func);
        u64::from(pos)
    }

    fn resolve_reloc(&mut self, offset: u64, reloc: Reloc, addend: Addend, target: u32) -> bool {
        let label = MachLabel::from_block(target);
        let offset = u32::try_from(offset).unwrap();
        match I::LabelUse::from_reloc(reloc, addend) {
            Some(label_use) => {
                self.buf.use_label_at_offset(offset, label, label_use);
                true
            }
            None => false,
        }
    }

    fn force_veneers(&mut self) {
        self.force_veneers = true;
    }

    fn finish(&mut self) -> Vec<u8> {
        // Double-check all functions were pushed.
        assert_eq!(self.next_func, self.buf.label_offsets.len() as u32);

        // Finish up any veneers, if necessary.
        self.buf
            .finish_emission_maybe_forcing_veneers(self.force_veneers);

        // We don't need the data any more, so return it to the caller.
        mem::take(&mut self.buf.data).into_vec()
    }
}

// We use an actual instruction definition to do tests, so we depend on the `arm64` feature here.
#[cfg(all(test, feature = "arm64"))]
mod test {
    use super::*;
    use crate::isa::aarch64::inst::xreg;
    use crate::isa::aarch64::inst::{BranchTarget, CondBrKind, EmitInfo, Inst};
    use crate::machinst::MachInstEmit;
    use crate::settings;
    use std::default::Default;
    use std::vec::Vec;

    fn label(n: u32) -> MachLabel {
        MachLabel::from_block(n)
    }
    fn target(n: u32) -> BranchTarget {
        BranchTarget::Label(label(n))
    }

    #[test]
    fn test_elide_jump_to_next() {
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(2);
        buf.bind_label(label(0));
        let inst = Inst::Jump { dest: target(1) };
        inst.emit(&mut buf, &info, &mut state);
        buf.bind_label(label(1));
        let buf = buf.finish();
        assert_eq!(0, buf.total_size());
    }

    #[test]
    fn test_elide_trivial_jump_blocks() {
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(4);

        buf.bind_label(label(0));
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),
            taken: target(1),
            not_taken: target(2),
        };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        let inst = Inst::Jump { dest: target(3) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(2));
        let inst = Inst::Jump { dest: target(3) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(3));

        let buf = buf.finish();
        assert_eq!(0, buf.total_size());
    }

    #[test]
    fn test_flip_cond() {
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(4);

        buf.bind_label(label(0));
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),
            taken: target(1),
            not_taken: target(2),
        };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        let inst = Inst::Udf {
            trap_code: TrapCode::Interrupt,
        };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(2));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(3));

        let buf = buf.finish();

        let mut buf2 = MachBuffer::new();
        let mut state = Default::default();
        let inst = Inst::TrapIf {
            kind: CondBrKind::NotZero(xreg(0)),
            trap_code: TrapCode::Interrupt,
        };
        inst.emit(&mut buf2, &info, &mut state);
        let inst = Inst::Nop4;
        inst.emit(&mut buf2, &info, &mut state);

        let buf2 = buf2.finish();

        assert_eq!(buf.data, buf2.data);
    }

    #[test]
    fn test_island() {
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(4);

        buf.bind_label(label(0));
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),
            taken: target(2),
            not_taken: target(3),
        };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        while buf.cur_offset() < 2000000 {
            if buf.island_needed(0) {
                buf.emit_island(0);
            }
            let inst = Inst::Nop4;
            inst.emit(&mut buf, &info, &mut state);
        }

        buf.bind_label(label(2));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(3));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        let buf = buf.finish();

        assert_eq!(2000000 + 8, buf.total_size());

        let mut buf2 = MachBuffer::new();
        let mut state = Default::default();
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),

            // This conditionally taken branch has a 19-bit constant, shifted
            // to the left by two, giving us a 21-bit range in total. Half of
            // this range positive so the we should be around 1 << 20 bytes
            // away for our jump target.
            //
            // There are two pending fixups by the time we reach this point,
            // one for this 19-bit jump and one for the unconditional 26-bit
            // jump below. A 19-bit veneer is 4 bytes large and the 26-bit
            // veneer is 20 bytes large, which means that pessimistically
            // assuming we'll need two veneers we need 24 bytes of extra
            // space, meaning that the actual island should come 24-bytes
            // before the deadline.
            taken: BranchTarget::ResolvedOffset((1 << 20) - 4 - 20),

            // This branch is in-range so no veneers should be needed, it should
            // go directly to the target.
            not_taken: BranchTarget::ResolvedOffset(2000000 + 4 - 4),
        };
        inst.emit(&mut buf2, &info, &mut state);

        let buf2 = buf2.finish();

        assert_eq!(&buf.data[0..8], &buf2.data[..]);
    }

    #[test]
    fn test_island_backward() {
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(4);

        buf.bind_label(label(0));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(2));
        while buf.cur_offset() < 2000000 {
            let inst = Inst::Nop4;
            inst.emit(&mut buf, &info, &mut state);
        }

        buf.bind_label(label(3));
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),
            taken: target(0),
            not_taken: target(1),
        };
        inst.emit(&mut buf, &info, &mut state);

        let buf = buf.finish();

        assert_eq!(2000000 + 12, buf.total_size());

        let mut buf2 = MachBuffer::new();
        let mut state = Default::default();
        let inst = Inst::CondBr {
            kind: CondBrKind::NotZero(xreg(0)),
            taken: BranchTarget::ResolvedOffset(8),
            not_taken: BranchTarget::ResolvedOffset(4 - (2000000 + 4)),
        };
        inst.emit(&mut buf2, &info, &mut state);
        let inst = Inst::Jump {
            dest: BranchTarget::ResolvedOffset(-(2000000 + 8)),
        };
        inst.emit(&mut buf2, &info, &mut state);

        let buf2 = buf2.finish();

        assert_eq!(&buf.data[2000000..], &buf2.data[..]);
    }

    #[test]
    fn test_multiple_redirect() {
        // label0:
        //   cbz x0, label1
        //   b label2
        // label1:
        //   b label3
        // label2:
        //   nop
        //   nop
        //   b label0
        // label3:
        //   b label4
        // label4:
        //   b label5
        // label5:
        //   b label7
        // label6:
        //   nop
        // label7:
        //   ret
        //
        // -- should become:
        //
        // label0:
        //   cbz x0, label7
        // label2:
        //   nop
        //   nop
        //   b label0
        // label6:
        //   nop
        // label7:
        //   ret

        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(8);

        buf.bind_label(label(0));
        let inst = Inst::CondBr {
            kind: CondBrKind::Zero(xreg(0)),
            taken: target(1),
            not_taken: target(2),
        };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        let inst = Inst::Jump { dest: target(3) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(2));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);
        inst.emit(&mut buf, &info, &mut state);
        let inst = Inst::Jump { dest: target(0) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(3));
        let inst = Inst::Jump { dest: target(4) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(4));
        let inst = Inst::Jump { dest: target(5) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(5));
        let inst = Inst::Jump { dest: target(7) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(6));
        let inst = Inst::Nop4;
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(7));
        let inst = Inst::Ret;
        inst.emit(&mut buf, &info, &mut state);

        let buf = buf.finish();

        let golden_data = vec![
            0xa0, 0x00, 0x00, 0xb4, // cbz x0, 0x14
            0x1f, 0x20, 0x03, 0xd5, // nop
            0x1f, 0x20, 0x03, 0xd5, // nop
            0xfd, 0xff, 0xff, 0x17, // b 0
            0x1f, 0x20, 0x03, 0xd5, // nop
            0xc0, 0x03, 0x5f, 0xd6, // ret
        ];

        assert_eq!(&golden_data[..], &buf.data[..]);
    }

    #[test]
    fn test_handle_branch_cycle() {
        // label0:
        //   b label1
        // label1:
        //   b label2
        // label2:
        //   b label3
        // label3:
        //   b label4
        // label4:
        //   b label1  // note: not label0 (to make it interesting).
        //
        // -- should become:
        //
        // label0, label1, ..., label4:
        //   b label0
        let info = EmitInfo::new(settings::Flags::new(settings::builder()));
        let mut buf = MachBuffer::new();
        let mut state = Default::default();

        buf.reserve_labels_for_blocks(5);

        buf.bind_label(label(0));
        let inst = Inst::Jump { dest: target(1) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(1));
        let inst = Inst::Jump { dest: target(2) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(2));
        let inst = Inst::Jump { dest: target(3) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(3));
        let inst = Inst::Jump { dest: target(4) };
        inst.emit(&mut buf, &info, &mut state);

        buf.bind_label(label(4));
        let inst = Inst::Jump { dest: target(1) };
        inst.emit(&mut buf, &info, &mut state);

        let buf = buf.finish();

        let golden_data = vec![
            0x00, 0x00, 0x00, 0x14, // b 0
        ];

        assert_eq!(&golden_data[..], &buf.data[..]);
    }

    #[test]
    fn metadata_records() {
        let mut buf = MachBuffer::<Inst>::new();

        buf.reserve_labels_for_blocks(1);

        buf.bind_label(label(0));
        buf.put1(1);
        buf.add_trap(SourceLoc::default(), TrapCode::HeapOutOfBounds);
        buf.put1(2);
        buf.add_trap(SourceLoc::default(), TrapCode::IntegerOverflow);
        buf.add_trap(SourceLoc::default(), TrapCode::IntegerDivisionByZero);
        buf.add_call_site(SourceLoc::default(), Opcode::Call);
        buf.add_reloc(
            SourceLoc::default(),
            Reloc::Abs4,
            &ExternalName::user(0, 0),
            0,
        );
        buf.put1(3);
        buf.add_reloc(
            SourceLoc::default(),
            Reloc::Abs8,
            &ExternalName::user(1, 1),
            1,
        );
        buf.put1(4);

        let buf = buf.finish();

        assert_eq!(buf.data(), &[1, 2, 3, 4]);
        assert_eq!(
            buf.traps()
                .iter()
                .map(|trap| (trap.offset, trap.code))
                .collect::<Vec<_>>(),
            vec![
                (1, TrapCode::HeapOutOfBounds),
                (2, TrapCode::IntegerOverflow),
                (2, TrapCode::IntegerDivisionByZero)
            ]
        );
        assert_eq!(
            buf.call_sites()
                .iter()
                .map(|call_site| (call_site.ret_addr, call_site.opcode))
                .collect::<Vec<_>>(),
            vec![(2, Opcode::Call)]
        );
        assert_eq!(
            buf.relocs()
                .iter()
                .map(|reloc| (reloc.offset, reloc.kind))
                .collect::<Vec<_>>(),
            vec![(2, Reloc::Abs4), (3, Reloc::Abs8)]
        );
    }
}
