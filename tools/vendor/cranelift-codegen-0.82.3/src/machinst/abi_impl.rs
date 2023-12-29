//! Implementation of a vanilla ABI, shared between several machines. The
//! implementation here assumes that arguments will be passed in registers
//! first, then additional args on the stack; that the stack grows downward,
//! contains a standard frame (return address and frame pointer), and the
//! compiler is otherwise free to allocate space below that with its choice of
//! layout; and that the machine has some notion of caller- and callee-save
//! registers. Most modern machines, e.g. x86-64 and AArch64, should fit this
//! mold and thus both of these backends use this shared implementation.
//!
//! See the documentation in specific machine backends for the "instantiation"
//! of this generic ABI, i.e., which registers are caller/callee-save, arguments
//! and return values, and any other special requirements.
//!
//! For now the implementation here assumes a 64-bit machine, but we intend to
//! make this 32/64-bit-generic shortly.
//!
//! # Vanilla ABI
//!
//! First, arguments and return values are passed in registers up to a certain
//! fixed count, after which they overflow onto the stack. Multiple return
//! values either fit in registers, or are returned in a separate return-value
//! area on the stack, given by a hidden extra parameter.
//!
//! Note that the exact stack layout is up to us. We settled on the
//! below design based on several requirements. In particular, we need
//! to be able to generate instructions (or instruction sequences) to
//! access arguments, stack slots, and spill slots before we know how
//! many spill slots or clobber-saves there will be, because of our
//! pass structure. We also prefer positive offsets to negative
//! offsets because of an asymmetry in some machines' addressing modes
//! (e.g., on AArch64, positive offsets have a larger possible range
//! without a long-form sequence to synthesize an arbitrary
//! offset). We also need clobber-save registers to be "near" the
//! frame pointer: Windows unwind information requires it to be within
//! 240 bytes of RBP. Finally, it is not allowed to access memory
//! below the current SP value.
//!
//! We assume that a prologue first pushes the frame pointer (and
//! return address above that, if the machine does not do that in
//! hardware). We set FP to point to this two-word frame record. We
//! store all other frame slots below this two-word frame record, with
//! the stack pointer remaining at or below this fixed frame storage
//! for the rest of the function. We can then access frame storage
//! slots using positive offsets from SP. In order to allow codegen
//! for the latter before knowing how SP might be adjusted around
//! callsites, we implement a "nominal SP" tracking feature by which a
//! fixup (distance between actual SP and a "nominal" SP) is known at
//! each instruction.
//!
//! Note that if we ever support dynamic stack-space allocation (for
//! `alloca`), we will need a way to reference spill slots and stack
//! slots without "nominal SP", because we will no longer be able to
//! know a static offset from SP to the slots at any particular
//! program point. Probably the best solution at that point will be to
//! revert to using the frame pointer as the reference for all slots,
//! and creating a "nominal FP" synthetic addressing mode (analogous
//! to "nominal SP" today) to allow generating spill/reload and
//! stackslot accesses before we know how large the clobber-saves will
//! be.
//!
//! # Stack Layout
//!
//! The stack looks like:
//!
//! ```plain
//!   (high address)
//!
//!                              +---------------------------+
//!                              |          ...              |
//!                              | stack args                |
//!                              | (accessed via FP)         |
//!                              +---------------------------+
//! SP at function entry ----->  | return address            |
//!                              +---------------------------+
//! FP after prologue -------->  | FP (pushed by prologue)   |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | clobbered callee-saves    |
//! unwind-frame base     ---->  | (pushed by prologue)      |
//!                              +---------------------------+
//!                              |          ...              |
//!                              | spill slots               |
//!                              | (accessed via nominal SP) |
//!                              |          ...              |
//!                              | stack slots               |
//!                              | (accessed via nominal SP) |
//! nominal SP --------------->  | (alloc'd by prologue)     |
//! (SP at end of prologue)      +---------------------------+
//!                              | [alignment as needed]     |
//!                              |          ...              |
//!                              | args for call             |
//! SP before making a call -->  | (pushed at callsite)      |
//!                              +---------------------------+
//!
//!   (low address)
//! ```
//!
//! # Multi-value Returns
//!
//! Note that we support multi-value returns in two ways. First, we allow for
//! multiple return-value registers. Second, if teh appropriate flag is set, we
//! support the SpiderMonkey Wasm ABI.  For details of the multi-value return
//! ABI, see:
//!
//! <https://searchfox.org/mozilla-central/rev/bc3600def806859c31b2c7ac06e3d69271052a89/js/src/wasm/WasmStubs.h#134>
//!
//! In brief:
//! - Return values are processed in *reverse* order.
//! - The first return value in this order (so the last return) goes into the
//!   ordinary return register.
//! - Any further returns go in a struct-return area, allocated upwards (in
//!   address order) during the reverse traversal.
//! - This struct-return area is provided by the caller, and a pointer to its
//!   start is passed as an invisible last (extra) argument. Normally the caller
//!   will allocate this area on the stack. When we generate calls, we place it
//!   just above the on-stack argument area.
//! - So, for example, a function returning 4 i64's (v0, v1, v2, v3), with no
//!   formal arguments, would:
//!   - Accept a pointer `P` to the struct return area as a hidden argument in the
//!     first argument register on entry.
//!   - Return v3 in the one and only return-value register.
//!   - Return v2 in memory at `[P]`.
//!   - Return v1 in memory at `[P+8]`.
//!   - Return v0 in memory at `[P+16]`.

use super::abi::*;
use crate::binemit::StackMap;
use crate::ir::types::*;
use crate::ir::{ArgumentExtension, ArgumentPurpose, StackSlot};
use crate::machinst::*;
use crate::settings;
use crate::CodegenResult;
use crate::{ir, isa};
use alloc::vec::Vec;
use regalloc::{RealReg, Reg, RegClass, Set, SpillSlot, Writable};
use smallvec::{smallvec, SmallVec};
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem;

/// A location for (part of) an argument or return value. These "storage slots"
/// are specified for each register-sized part of an argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ABIArgSlot {
    /// In a real register.
    Reg {
        /// Register that holds this arg.
        reg: RealReg,
        /// Value type of this arg.
        ty: ir::Type,
        /// Should this arg be zero- or sign-extended?
        extension: ir::ArgumentExtension,
    },
    /// Arguments only: on stack, at given offset from SP at entry.
    Stack {
        /// Offset of this arg relative to the base of stack args.
        offset: i64,
        /// Value type of this arg.
        ty: ir::Type,
        /// Should this arg be zero- or sign-extended?
        extension: ir::ArgumentExtension,
    },
}

/// An ABIArg is composed of one or more parts. This allows for a CLIF-level
/// Value to be passed with its parts in more than one location at the ABI
/// level. For example, a 128-bit integer may be passed in two 64-bit registers,
/// or even a 64-bit register and a 64-bit stack slot, on a 64-bit machine. The
/// number of "parts" should correspond to the number of registers used to store
/// this type according to the machine backend.
///
/// As an invariant, the `purpose` for every part must match. As a further
/// invariant, a `StructArg` part cannot appear with any other part.
#[derive(Clone, Debug)]
pub enum ABIArg {
    /// Storage slots (registers or stack locations) for each part of the
    /// argument value. The number of slots must equal the number of register
    /// parts used to store a value of this type.
    Slots {
        /// Slots, one per register part.
        slots: Vec<ABIArgSlot>,
        /// Purpose of this arg.
        purpose: ir::ArgumentPurpose,
    },
    /// Structure argument. We reserve stack space for it, but the CLIF-level
    /// semantics are a little weird: the value passed to the call instruction,
    /// and received in the corresponding block param, is a *pointer*. On the
    /// caller side, we memcpy the data from the passed-in pointer to the stack
    /// area; on the callee side, we compute a pointer to this stack area and
    /// provide that as the argument's value.
    StructArg {
        /// Offset of this arg relative to base of stack args.
        offset: i64,
        /// Size of this arg on the stack.
        size: u64,
        /// Purpose of this arg.
        purpose: ir::ArgumentPurpose,
    },
}

impl ABIArg {
    /// Get the purpose of this arg.
    fn get_purpose(&self) -> ir::ArgumentPurpose {
        match self {
            &ABIArg::Slots { purpose, .. } => purpose,
            &ABIArg::StructArg { purpose, .. } => purpose,
        }
    }

    /// Is this a StructArg?
    fn is_struct_arg(&self) -> bool {
        match self {
            &ABIArg::StructArg { .. } => true,
            _ => false,
        }
    }

    /// Create an ABIArg from one register.
    pub fn reg(
        reg: RealReg,
        ty: ir::Type,
        extension: ir::ArgumentExtension,
        purpose: ir::ArgumentPurpose,
    ) -> ABIArg {
        ABIArg::Slots {
            slots: vec![ABIArgSlot::Reg { reg, ty, extension }],
            purpose,
        }
    }

    /// Create an ABIArg from one stack slot.
    pub fn stack(
        offset: i64,
        ty: ir::Type,
        extension: ir::ArgumentExtension,
        purpose: ir::ArgumentPurpose,
    ) -> ABIArg {
        ABIArg::Slots {
            slots: vec![ABIArgSlot::Stack {
                offset,
                ty,
                extension,
            }],
            purpose,
        }
    }
}

/// Are we computing information about arguments or return values? Much of the
/// handling is factored out into common routines; this enum allows us to
/// distinguish which case we're handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgsOrRets {
    /// Arguments.
    Args,
    /// Return values.
    Rets,
}

/// Is an instruction returned by an ABI machine-specific backend a safepoint,
/// or not?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstIsSafepoint {
    /// The instruction is a safepoint.
    Yes,
    /// The instruction is not a safepoint.
    No,
}

/// Abstract location for a machine-specific ABI impl to translate into the
/// appropriate addressing mode.
#[derive(Clone, Copy, Debug)]
pub enum StackAMode {
    /// Offset from the frame pointer, possibly making use of a specific type
    /// for a scaled indexing operation.
    FPOffset(i64, ir::Type),
    /// Offset from the nominal stack pointer, possibly making use of a specific
    /// type for a scaled indexing operation.
    NominalSPOffset(i64, ir::Type),
    /// Offset from the real stack pointer, possibly making use of a specific
    /// type for a scaled indexing operation.
    SPOffset(i64, ir::Type),
}

impl StackAMode {
    /// Offset by an addend.
    pub fn offset(self, addend: i64) -> Self {
        match self {
            StackAMode::FPOffset(off, ty) => StackAMode::FPOffset(off + addend, ty),
            StackAMode::NominalSPOffset(off, ty) => StackAMode::NominalSPOffset(off + addend, ty),
            StackAMode::SPOffset(off, ty) => StackAMode::SPOffset(off + addend, ty),
        }
    }
}

/// Trait implemented by machine-specific backend to provide information about
/// register assignments and to allow generating the specific instructions for
/// stack loads/saves, prologues/epilogues, etc.
pub trait ABIMachineSpec {
    /// The instruction type.
    type I: VCodeInst;

    /// Returns the number of bits in a word, that is 32/64 for 32/64-bit architecture.
    fn word_bits() -> u32;

    /// Returns the number of bytes in a word.
    fn word_bytes() -> u32 {
        return Self::word_bits() / 8;
    }

    /// Returns word-size integer type.
    fn word_type() -> Type {
        match Self::word_bits() {
            32 => I32,
            64 => I64,
            _ => unreachable!(),
        }
    }

    /// Returns word register class.
    fn word_reg_class() -> RegClass {
        match Self::word_bits() {
            32 => RegClass::I32,
            64 => RegClass::I64,
            _ => unreachable!(),
        }
    }

    /// Returns required stack alignment in bytes.
    fn stack_align(call_conv: isa::CallConv) -> u32;

    /// Process a list of parameters or return values and allocate them to registers
    /// and stack slots.
    ///
    /// Returns the list of argument locations, the stack-space used (rounded up
    /// to as alignment requires), and if `add_ret_area_ptr` was passed, the
    /// index of the extra synthetic arg that was added.
    fn compute_arg_locs(
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        params: &[ir::AbiParam],
        args_or_rets: ArgsOrRets,
        add_ret_area_ptr: bool,
    ) -> CodegenResult<(Vec<ABIArg>, i64, Option<usize>)>;

    /// Returns the offset from FP to the argument area, i.e., jumping over the saved FP, return
    /// address, and maybe other standard elements depending on ABI (e.g. Wasm TLS reg).
    fn fp_to_arg_offset(call_conv: isa::CallConv, flags: &settings::Flags) -> i64;

    /// Generate a load from the stack.
    fn gen_load_stack(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Self::I;

    /// Generate a store to the stack.
    fn gen_store_stack(mem: StackAMode, from_reg: Reg, ty: Type) -> Self::I;

    /// Generate a move.
    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Self::I;

    /// Generate an integer-extend operation.
    fn gen_extend(
        to_reg: Writable<Reg>,
        from_reg: Reg,
        is_signed: bool,
        from_bits: u8,
        to_bits: u8,
    ) -> Self::I;

    /// Generate a return instruction.
    fn gen_ret() -> Self::I;

    /// Generate an "epilogue placeholder" instruction, recognized by lowering
    /// when using the Baldrdash ABI.
    fn gen_epilogue_placeholder() -> Self::I;

    /// Generate an add-with-immediate. Note that even if this uses a scratch
    /// register, it must satisfy two requirements:
    ///
    /// - The add-imm sequence must only clobber caller-save registers, because
    ///   it will be placed in the prologue before the clobbered callee-save
    ///   registers are saved.
    ///
    /// - The add-imm sequence must work correctly when `from_reg` and/or
    ///   `into_reg` are the register returned by `get_stacklimit_reg()`.
    fn gen_add_imm(into_reg: Writable<Reg>, from_reg: Reg, imm: u32) -> SmallInstVec<Self::I>;

    /// Generate a sequence that traps with a `TrapCode::StackOverflow` code if
    /// the stack pointer is less than the given limit register (assuming the
    /// stack grows downward).
    fn gen_stack_lower_bound_trap(limit_reg: Reg) -> SmallInstVec<Self::I>;

    /// Generate an instruction to compute an address of a stack slot (FP- or
    /// SP-based offset).
    fn gen_get_stack_addr(mem: StackAMode, into_reg: Writable<Reg>, ty: Type) -> Self::I;

    /// Get a fixed register to use to compute a stack limit. This is needed for
    /// certain sequences generated after the register allocator has already
    /// run. This must satisfy two requirements:
    ///
    /// - It must be a caller-save register, because it will be clobbered in the
    ///   prologue before the clobbered callee-save registers are saved.
    ///
    /// - It must be safe to pass as an argument and/or destination to
    ///   `gen_add_imm()`. This is relevant when an addition with a large
    ///   immediate needs its own temporary; it cannot use the same fixed
    ///   temporary as this one.
    fn get_stacklimit_reg() -> Reg;

    /// Generate a store to the given [base+offset] address.
    fn gen_load_base_offset(into_reg: Writable<Reg>, base: Reg, offset: i32, ty: Type) -> Self::I;

    /// Generate a load from the given [base+offset] address.
    fn gen_store_base_offset(base: Reg, offset: i32, from_reg: Reg, ty: Type) -> Self::I;

    /// Adjust the stack pointer up or down.
    fn gen_sp_reg_adjust(amount: i32) -> SmallInstVec<Self::I>;

    /// Generate a meta-instruction that adjusts the nominal SP offset.
    fn gen_nominal_sp_adj(amount: i32) -> Self::I;

    /// Generates extra unwind instructions for a new frame  for this
    /// architecture, whether the frame has a prologue sequence or not.
    fn gen_debug_frame_info(
        _flags: &settings::Flags,
        _isa_flags: &Vec<settings::Value>,
    ) -> SmallInstVec<Self::I> {
        // By default, generates nothing.
        smallvec![]
    }

    /// Generate the usual frame-setup sequence for this architecture: e.g.,
    /// `push rbp / mov rbp, rsp` on x86-64, or `stp fp, lr, [sp, #-16]!` on
    /// AArch64.
    fn gen_prologue_frame_setup(flags: &settings::Flags) -> SmallInstVec<Self::I>;

    /// Generate the usual frame-restore sequence for this architecture.
    fn gen_epilogue_frame_restore(flags: &settings::Flags) -> SmallInstVec<Self::I>;

    /// Generate a probestack call.
    fn gen_probestack(_frame_size: u32) -> SmallInstVec<Self::I>;

    /// Get all clobbered registers that are callee-saved according to the ABI; the result
    /// contains the registers in a sorted order.
    fn get_clobbered_callee_saves(
        call_conv: isa::CallConv,
        regs: &Set<Writable<RealReg>>,
    ) -> Vec<Writable<RealReg>>;

    /// Determine whether it is necessary to generate the usual frame-setup
    /// sequence (refer to gen_prologue_frame_setup()).
    fn is_frame_setup_needed(
        is_leaf: bool,
        stack_args_size: u32,
        num_clobbered_callee_saves: usize,
        fixed_frame_storage_size: u32,
    ) -> bool;

    /// Generate a clobber-save sequence. The implementation here should return
    /// a sequence of instructions that "push" or otherwise save to the stack all
    /// registers written/modified by the function body that are callee-saved.
    /// The sequence of instructions should adjust the stack pointer downward,
    /// and should align as necessary according to ABI requirements.
    ///
    /// Returns stack bytes used as well as instructions. Does not adjust
    /// nominal SP offset; caller will do that.
    fn gen_clobber_save(
        call_conv: isa::CallConv,
        setup_frame: bool,
        flags: &settings::Flags,
        clobbered_callee_saves: &Vec<Writable<RealReg>>,
        fixed_frame_storage_size: u32,
        outgoing_args_size: u32,
    ) -> (u64, SmallVec<[Self::I; 16]>);

    /// Generate a clobber-restore sequence. This sequence should perform the
    /// opposite of the clobber-save sequence generated above, assuming that SP
    /// going into the sequence is at the same point that it was left when the
    /// clobber-save sequence finished.
    fn gen_clobber_restore(
        call_conv: isa::CallConv,
        flags: &settings::Flags,
        clobbers: &Set<Writable<RealReg>>,
        fixed_frame_storage_size: u32,
        outgoing_args_size: u32,
    ) -> SmallVec<[Self::I; 16]>;

    /// Generate a call instruction/sequence. This method is provided one
    /// temporary register to use to synthesize the called address, if needed.
    fn gen_call(
        dest: &CallDest,
        uses: Vec<Reg>,
        defs: Vec<Writable<Reg>>,
        opcode: ir::Opcode,
        tmp: Writable<Reg>,
        callee_conv: isa::CallConv,
        callee_conv: isa::CallConv,
    ) -> SmallVec<[(InstIsSafepoint, Self::I); 2]>;

    /// Generate a memcpy invocation. Used to set up struct args. May clobber
    /// caller-save registers; we only memcpy before we start to set up args for
    /// a call.
    fn gen_memcpy(
        call_conv: isa::CallConv,
        dst: Reg,
        src: Reg,
        size: usize,
    ) -> SmallVec<[Self::I; 8]>;

    /// Get the number of spillslots required for the given register-class.
    fn get_number_of_spillslots_for_value(rc: RegClass) -> u32;

    /// Get the current virtual-SP offset from an instruction-emission state.
    fn get_virtual_sp_offset_from_state(s: &<Self::I as MachInstEmit>::State) -> i64;

    /// Get the "nominal SP to FP" offset from an instruction-emission state.
    fn get_nominal_sp_to_fp(s: &<Self::I as MachInstEmit>::State) -> i64;

    /// Get all caller-save registers, that is, registers that we expect
    /// not to be saved across a call to a callee with the given ABI.
    fn get_regs_clobbered_by_call(call_conv_of_callee: isa::CallConv) -> Vec<Writable<Reg>>;

    /// Get the needed extension mode, given the mode attached to the argument
    /// in the signature and the calling convention. The input (the attribute in
    /// the signature) specifies what extension type should be done *if* the ABI
    /// requires extension to the full register; this method's return value
    /// indicates whether the extension actually *will* be done.
    fn get_ext_mode(
        call_conv: isa::CallConv,
        specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension;
}

/// ABI information shared between body (callee) and caller.
struct ABISig {
    /// Argument locations (regs or stack slots). Stack offsets are relative to
    /// SP on entry to function.
    args: Vec<ABIArg>,
    /// Return-value locations. Stack offsets are relative to the return-area
    /// pointer.
    rets: Vec<ABIArg>,
    /// Space on stack used to store arguments.
    stack_arg_space: i64,
    /// Space on stack used to store return values.
    stack_ret_space: i64,
    /// Index in `args` of the stack-return-value-area argument.
    stack_ret_arg: Option<usize>,
    /// Calling convention used.
    call_conv: isa::CallConv,
}

impl ABISig {
    fn from_func_sig<M: ABIMachineSpec>(
        sig: &ir::Signature,
        flags: &settings::Flags,
    ) -> CodegenResult<ABISig> {
        // Compute args and retvals from signature. Handle retvals first,
        // because we may need to add a return-area arg to the args.
        let (rets, stack_ret_space, _) = M::compute_arg_locs(
            sig.call_conv,
            flags,
            &sig.returns,
            ArgsOrRets::Rets,
            /* extra ret-area ptr = */ false,
        )?;
        let need_stack_return_area = stack_ret_space > 0;
        let (args, stack_arg_space, stack_ret_arg) = M::compute_arg_locs(
            sig.call_conv,
            flags,
            &sig.params,
            ArgsOrRets::Args,
            need_stack_return_area,
        )?;

        log::trace!(
            "ABISig: sig {:?} => args = {:?} rets = {:?} arg stack = {} ret stack = {} stack_ret_arg = {:?}",
            sig,
            args,
            rets,
            stack_arg_space,
            stack_ret_space,
            stack_ret_arg
        );

        Ok(ABISig {
            args,
            rets,
            stack_arg_space,
            stack_ret_space,
            stack_ret_arg,
            call_conv: sig.call_conv,
        })
    }
}

/// ABI object for a function body.
pub struct ABICalleeImpl<M: ABIMachineSpec> {
    /// CLIF-level signature, possibly normalized.
    ir_sig: ir::Signature,
    /// Signature: arg and retval regs.
    sig: ABISig,
    /// Offsets to each stackslot.
    stackslots: PrimaryMap<StackSlot, u32>,
    /// Total stack size of all stackslots.
    stackslots_size: u32,
    /// Stack size to be reserved for outgoing arguments.
    outgoing_args_size: u32,
    /// Clobbered registers, from regalloc.
    clobbered: Set<Writable<RealReg>>,
    /// Total number of spillslots, from regalloc.
    spillslots: Option<usize>,
    /// Storage allocated for the fixed part of the stack frame.  This is
    /// usually the same as the total frame size below, except in the case
    /// of the baldrdash calling convention.
    fixed_frame_storage_size: u32,
    /// "Total frame size", as defined by "distance between FP and nominal SP".
    /// Some items are pushed below nominal SP, so the function may actually use
    /// more stack than this would otherwise imply. It is simply the initial
    /// frame/allocation size needed for stackslots and spillslots.
    total_frame_size: Option<u32>,
    /// The register holding the return-area pointer, if needed.
    ret_area_ptr: Option<Writable<Reg>>,
    /// Calling convention this function expects.
    call_conv: isa::CallConv,
    /// The settings controlling this function's compilation.
    flags: settings::Flags,
    /// The ISA-specific flag values controlling this function's compilation.
    isa_flags: Vec<settings::Value>,
    /// Whether or not this function is a "leaf", meaning it calls no other
    /// functions
    is_leaf: bool,
    /// If this function has a stack limit specified, then `Reg` is where the
    /// stack limit will be located after the instructions specified have been
    /// executed.
    ///
    /// Note that this is intended for insertion into the prologue, if
    /// present. Also note that because the instructions here execute in the
    /// prologue this happens after legalization/register allocation/etc so we
    /// need to be extremely careful with each instruction. The instructions are
    /// manually register-allocated and carefully only use caller-saved
    /// registers and keep nothing live after this sequence of instructions.
    stack_limit: Option<(Reg, SmallInstVec<M::I>)>,
    /// Are we to invoke the probestack function in the prologue? If so,
    /// what is the minimum size at which we must invoke it?
    probestack_min_frame: Option<u32>,
    /// Whether it is necessary to generate the usual frame-setup sequence.
    setup_frame: bool,

    _mach: PhantomData<M>,
}

fn get_special_purpose_param_register(
    f: &ir::Function,
    abi: &ABISig,
    purpose: ir::ArgumentPurpose,
) -> Option<Reg> {
    let idx = f.signature.special_param_index(purpose)?;
    match &abi.args[idx] {
        &ABIArg::Slots { ref slots, .. } => match &slots[0] {
            &ABIArgSlot::Reg { reg, .. } => Some(reg.to_reg()),
            _ => None,
        },
        _ => None,
    }
}

fn ty_from_class(class: RegClass) -> Type {
    match class {
        RegClass::I32 => I32,
        RegClass::I64 => I64,
        RegClass::F32 => F32,
        RegClass::F64 => F64,
        RegClass::V128 => I8X16,
        _ => panic!("Unknown regclass: {:?}", class),
    }
}

impl<M: ABIMachineSpec> ABICalleeImpl<M> {
    /// Create a new body ABI instance.
    pub fn new(
        f: &ir::Function,
        flags: settings::Flags,
        isa_flags: Vec<settings::Value>,
    ) -> CodegenResult<Self> {
        log::trace!("ABI: func signature {:?}", f.signature);

        let ir_sig = ensure_struct_return_ptr_is_returned(&f.signature);
        let sig = ABISig::from_func_sig::<M>(&ir_sig, &flags)?;

        let call_conv = f.signature.call_conv;
        // Only these calling conventions are supported.
        debug_assert!(
            call_conv == isa::CallConv::SystemV
                || call_conv == isa::CallConv::Fast
                || call_conv == isa::CallConv::Cold
                || call_conv.extends_baldrdash()
                || call_conv.extends_windows_fastcall()
                || call_conv == isa::CallConv::AppleAarch64
                || call_conv == isa::CallConv::WasmtimeSystemV
                || call_conv == isa::CallConv::WasmtimeAppleAarch64,
            "Unsupported calling convention: {:?}",
            call_conv
        );

        // Compute stackslot locations and total stackslot size.
        let mut stack_offset: u32 = 0;
        let mut stackslots = PrimaryMap::new();
        for (stackslot, data) in f.stack_slots.iter() {
            let off = stack_offset;
            stack_offset += data.size;
            let mask = M::word_bytes() - 1;
            stack_offset = (stack_offset + mask) & !mask;
            debug_assert_eq!(stackslot.as_u32() as usize, stackslots.len());
            stackslots.push(off);
        }

        // Figure out what instructions, if any, will be needed to check the
        // stack limit. This can either be specified as a special-purpose
        // argument or as a global value which often calculates the stack limit
        // from the arguments.
        let stack_limit =
            get_special_purpose_param_register(f, &sig, ir::ArgumentPurpose::StackLimit)
                .map(|reg| (reg, smallvec![]))
                .or_else(|| f.stack_limit.map(|gv| gen_stack_limit::<M>(f, &sig, gv)));

        // Determine whether a probestack call is required for large enough
        // frames (and the minimum frame size if so).
        let probestack_min_frame = if flags.enable_probestack() {
            assert!(
                !flags.probestack_func_adjusts_sp(),
                "SP-adjusting probestack not supported in new backends"
            );
            Some(1 << flags.probestack_size_log2())
        } else {
            None
        };

        Ok(Self {
            ir_sig,
            sig,
            stackslots,
            stackslots_size: stack_offset,
            outgoing_args_size: 0,
            clobbered: Set::empty(),
            spillslots: None,
            fixed_frame_storage_size: 0,
            total_frame_size: None,
            ret_area_ptr: None,
            call_conv,
            flags,
            isa_flags,
            is_leaf: f.is_leaf(),
            stack_limit,
            probestack_min_frame,
            setup_frame: true,
            _mach: PhantomData,
        })
    }

    /// Inserts instructions necessary for checking the stack limit into the
    /// prologue.
    ///
    /// This function will generate instructions necessary for perform a stack
    /// check at the header of a function. The stack check is intended to trap
    /// if the stack pointer goes below a particular threshold, preventing stack
    /// overflow in wasm or other code. The `stack_limit` argument here is the
    /// register which holds the threshold below which we're supposed to trap.
    /// This function is known to allocate `stack_size` bytes and we'll push
    /// instructions onto `insts`.
    ///
    /// Note that the instructions generated here are special because this is
    /// happening so late in the pipeline (e.g. after register allocation). This
    /// means that we need to do manual register allocation here and also be
    /// careful to not clobber any callee-saved or argument registers. For now
    /// this routine makes do with the `spilltmp_reg` as one temporary
    /// register, and a second register of `tmp2` which is caller-saved. This
    /// should be fine for us since no spills should happen in this sequence of
    /// instructions, so our register won't get accidentally clobbered.
    ///
    /// No values can be live after the prologue, but in this case that's ok
    /// because we just need to perform a stack check before progressing with
    /// the rest of the function.
    fn insert_stack_check(
        &self,
        stack_limit: Reg,
        stack_size: u32,
        insts: &mut SmallInstVec<M::I>,
    ) {
        // With no explicit stack allocated we can just emit the simple check of
        // the stack registers against the stack limit register, and trap if
        // it's out of bounds.
        if stack_size == 0 {
            insts.extend(M::gen_stack_lower_bound_trap(stack_limit));
            return;
        }

        // Note that the 32k stack size here is pretty special. See the
        // documentation in x86/abi.rs for why this is here. The general idea is
        // that we're protecting against overflow in the addition that happens
        // below.
        if stack_size >= 32 * 1024 {
            insts.extend(M::gen_stack_lower_bound_trap(stack_limit));
        }

        // Add the `stack_size` to `stack_limit`, placing the result in
        // `scratch`.
        //
        // Note though that `stack_limit`'s register may be the same as
        // `scratch`. If our stack size doesn't fit into an immediate this
        // means we need a second scratch register for loading the stack size
        // into a register.
        let scratch = Writable::from_reg(M::get_stacklimit_reg());
        insts.extend(M::gen_add_imm(scratch, stack_limit, stack_size).into_iter());
        insts.extend(M::gen_stack_lower_bound_trap(scratch.to_reg()));
    }
}

/// Generates the instructions necessary for the `gv` to be materialized into a
/// register.
///
/// This function will return a register that will contain the result of
/// evaluating `gv`. It will also return any instructions necessary to calculate
/// the value of the register.
///
/// Note that global values are typically lowered to instructions via the
/// standard legalization pass. Unfortunately though prologue generation happens
/// so late in the pipeline that we can't use these legalization passes to
/// generate the instructions for `gv`. As a result we duplicate some lowering
/// of `gv` here and support only some global values. This is similar to what
/// the x86 backend does for now, and hopefully this can be somewhat cleaned up
/// in the future too!
///
/// Also note that this function will make use of `writable_spilltmp_reg()` as a
/// temporary register to store values in if necessary. Currently after we write
/// to this register there's guaranteed to be no spilled values between where
/// it's used, because we're not participating in register allocation anyway!
fn gen_stack_limit<M: ABIMachineSpec>(
    f: &ir::Function,
    abi: &ABISig,
    gv: ir::GlobalValue,
) -> (Reg, SmallInstVec<M::I>) {
    let mut insts = smallvec![];
    let reg = generate_gv::<M>(f, abi, gv, &mut insts);
    return (reg, insts);
}

fn generate_gv<M: ABIMachineSpec>(
    f: &ir::Function,
    abi: &ABISig,
    gv: ir::GlobalValue,
    insts: &mut SmallInstVec<M::I>,
) -> Reg {
    match f.global_values[gv] {
        // Return the direct register the vmcontext is in
        ir::GlobalValueData::VMContext => {
            get_special_purpose_param_register(f, abi, ir::ArgumentPurpose::VMContext)
                .expect("no vmcontext parameter found")
        }
        // Load our base value into a register, then load from that register
        // in to a temporary register.
        ir::GlobalValueData::Load {
            base,
            offset,
            global_type: _,
            readonly: _,
        } => {
            let base = generate_gv::<M>(f, abi, base, insts);
            let into_reg = Writable::from_reg(M::get_stacklimit_reg());
            insts.push(M::gen_load_base_offset(
                into_reg,
                base,
                offset.into(),
                M::word_type(),
            ));
            return into_reg.to_reg();
        }
        ref other => panic!("global value for stack limit not supported: {}", other),
    }
}

fn gen_load_stack_multi<M: ABIMachineSpec>(
    from: StackAMode,
    dst: ValueRegs<Writable<Reg>>,
    ty: Type,
) -> SmallInstVec<M::I> {
    let mut ret = smallvec![];
    let (_, tys) = M::I::rc_for_type(ty).unwrap();
    let mut offset = 0;
    // N.B.: registers are given in the `ValueRegs` in target endian order.
    for (&dst, &ty) in dst.regs().iter().zip(tys.iter()) {
        ret.push(M::gen_load_stack(from.offset(offset), dst, ty));
        offset += ty.bytes() as i64;
    }
    ret
}

fn gen_store_stack_multi<M: ABIMachineSpec>(
    from: StackAMode,
    src: ValueRegs<Reg>,
    ty: Type,
) -> SmallInstVec<M::I> {
    let mut ret = smallvec![];
    let (_, tys) = M::I::rc_for_type(ty).unwrap();
    let mut offset = 0;
    // N.B.: registers are given in the `ValueRegs` in target endian order.
    for (&src, &ty) in src.regs().iter().zip(tys.iter()) {
        ret.push(M::gen_store_stack(from.offset(offset), src, ty));
        offset += ty.bytes() as i64;
    }
    ret
}

fn ensure_struct_return_ptr_is_returned(sig: &ir::Signature) -> ir::Signature {
    let params_structret = sig
        .params
        .iter()
        .find(|p| p.purpose == ArgumentPurpose::StructReturn);
    let rets_have_structret = sig.returns.len() > 0
        && sig
            .returns
            .iter()
            .any(|arg| arg.purpose == ArgumentPurpose::StructReturn);
    let mut sig = sig.clone();
    if params_structret.is_some() && !rets_have_structret {
        sig.returns.insert(0, params_structret.unwrap().clone());
    }
    sig
}

impl<M: ABIMachineSpec> ABICallee for ABICalleeImpl<M> {
    type I = M::I;

    fn signature(&self) -> &ir::Signature {
        &self.ir_sig
    }

    fn temp_needed(&self) -> Option<Type> {
        if self.sig.stack_ret_arg.is_some() {
            Some(M::word_type())
        } else {
            None
        }
    }

    fn init(&mut self, maybe_tmp: Option<Writable<Reg>>) {
        if self.sig.stack_ret_arg.is_some() {
            assert!(maybe_tmp.is_some());
            self.ret_area_ptr = maybe_tmp;
        }
    }

    fn accumulate_outgoing_args_size(&mut self, size: u32) {
        if size > self.outgoing_args_size {
            self.outgoing_args_size = size;
        }
    }

    fn flags(&self) -> &settings::Flags {
        &self.flags
    }

    fn call_conv(&self) -> isa::CallConv {
        self.sig.call_conv
    }

    fn liveins(&self) -> Set<RealReg> {
        let mut set: Set<RealReg> = Set::empty();
        for arg in &self.sig.args {
            if let &ABIArg::Slots { ref slots, .. } = arg {
                for slot in slots {
                    if let ABIArgSlot::Reg { reg, .. } = slot {
                        set.insert(*reg);
                    }
                }
            }
        }
        set
    }

    fn liveouts(&self) -> Set<RealReg> {
        let mut set: Set<RealReg> = Set::empty();
        for ret in &self.sig.rets {
            if let &ABIArg::Slots { ref slots, .. } = ret {
                for slot in slots {
                    if let ABIArgSlot::Reg { reg, .. } = slot {
                        set.insert(*reg);
                    }
                }
            }
        }
        set
    }

    fn num_args(&self) -> usize {
        self.sig.args.len()
    }

    fn num_retvals(&self) -> usize {
        self.sig.rets.len()
    }

    fn num_stackslots(&self) -> usize {
        self.stackslots.len()
    }

    fn stackslot_offsets(&self) -> &PrimaryMap<StackSlot, u32> {
        &self.stackslots
    }

    fn gen_copy_arg_to_regs(
        &self,
        idx: usize,
        into_regs: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I> {
        let mut insts = smallvec![];
        match &self.sig.args[idx] {
            &ABIArg::Slots { ref slots, .. } => {
                assert_eq!(into_regs.len(), slots.len());
                for (slot, into_reg) in slots.iter().zip(into_regs.regs().iter()) {
                    match slot {
                        // Extension mode doesn't matter (we're copying out, not in; we
                        // ignore high bits by convention).
                        &ABIArgSlot::Reg { reg, ty, .. } => {
                            insts.push(M::gen_move(*into_reg, reg.to_reg(), ty));
                        }
                        &ABIArgSlot::Stack { offset, ty, .. } => {
                            insts.push(M::gen_load_stack(
                                StackAMode::FPOffset(
                                    M::fp_to_arg_offset(self.call_conv, &self.flags) + offset,
                                    ty,
                                ),
                                *into_reg,
                                ty,
                            ));
                        }
                    }
                }
            }
            &ABIArg::StructArg { offset, .. } => {
                let into_reg = into_regs.only_reg().unwrap();
                insts.push(M::gen_get_stack_addr(
                    StackAMode::FPOffset(
                        M::fp_to_arg_offset(self.call_conv, &self.flags) + offset,
                        I8,
                    ),
                    into_reg,
                    I8,
                ));
            }
        }
        insts
    }

    fn arg_is_needed_in_body(&self, idx: usize) -> bool {
        match self.sig.args[idx].get_purpose() {
            // Special Baldrdash-specific pseudo-args that are present only to
            // fill stack slots.  Won't ever be used as ordinary values in the
            // body.
            ir::ArgumentPurpose::CalleeTLS | ir::ArgumentPurpose::CallerTLS => false,
            _ => true,
        }
    }

    fn gen_copy_regs_to_retval(
        &self,
        idx: usize,
        from_regs: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I> {
        let mut ret = smallvec![];
        let word_bits = M::word_bits() as u8;
        match &self.sig.rets[idx] {
            &ABIArg::Slots { ref slots, .. } => {
                assert_eq!(from_regs.len(), slots.len());
                for (slot, from_reg) in slots.iter().zip(from_regs.regs().iter()) {
                    match slot {
                        &ABIArgSlot::Reg {
                            reg, ty, extension, ..
                        } => {
                            let from_bits = ty_bits(ty) as u8;
                            let ext = M::get_ext_mode(self.sig.call_conv, extension);
                            match (ext, from_bits) {
                                (ArgumentExtension::Uext, n) | (ArgumentExtension::Sext, n)
                                    if n < word_bits =>
                                {
                                    let signed = ext == ArgumentExtension::Sext;
                                    ret.push(M::gen_extend(
                                        Writable::from_reg(reg.to_reg()),
                                        from_reg.to_reg(),
                                        signed,
                                        from_bits,
                                        /* to_bits = */ word_bits,
                                    ));
                                }
                                _ => {
                                    ret.push(M::gen_move(
                                        Writable::from_reg(reg.to_reg()),
                                        from_reg.to_reg(),
                                        ty,
                                    ));
                                }
                            };
                        }
                        &ABIArgSlot::Stack {
                            offset,
                            ty,
                            extension,
                            ..
                        } => {
                            let mut ty = ty;
                            let from_bits = ty_bits(ty) as u8;
                            // A machine ABI implementation should ensure that stack frames
                            // have "reasonable" size. All current ABIs for machinst
                            // backends (aarch64 and x64) enforce a 128MB limit.
                            let off = i32::try_from(offset).expect(
                                "Argument stack offset greater than 2GB; should hit impl limit first",
                                );
                            let ext = M::get_ext_mode(self.sig.call_conv, extension);
                            // Trash the from_reg; it should be its last use.
                            match (ext, from_bits) {
                                (ArgumentExtension::Uext, n) | (ArgumentExtension::Sext, n)
                                    if n < word_bits =>
                                {
                                    assert_eq!(M::word_reg_class(), from_reg.to_reg().get_class());
                                    let signed = ext == ArgumentExtension::Sext;
                                    ret.push(M::gen_extend(
                                        Writable::from_reg(from_reg.to_reg()),
                                        from_reg.to_reg(),
                                        signed,
                                        from_bits,
                                        /* to_bits = */ word_bits,
                                    ));
                                    // Store the extended version.
                                    ty = M::word_type();
                                }
                                _ => {}
                            };
                            ret.push(M::gen_store_base_offset(
                                self.ret_area_ptr.unwrap().to_reg(),
                                off,
                                from_reg.to_reg(),
                                ty,
                            ));
                        }
                    }
                }
            }
            &ABIArg::StructArg { .. } => {
                panic!("StructArg in return position is unsupported");
            }
        }
        ret
    }

    fn gen_retval_area_setup(&self) -> Option<Self::I> {
        if let Some(i) = self.sig.stack_ret_arg {
            let insts = self.gen_copy_arg_to_regs(i, ValueRegs::one(self.ret_area_ptr.unwrap()));
            let inst = insts.into_iter().next().unwrap();
            log::trace!(
                "gen_retval_area_setup: inst {:?}; ptr reg is {:?}",
                inst,
                self.ret_area_ptr.unwrap().to_reg()
            );
            Some(inst)
        } else {
            log::trace!("gen_retval_area_setup: not needed");
            None
        }
    }

    fn gen_ret(&self) -> Self::I {
        M::gen_ret()
    }

    fn gen_epilogue_placeholder(&self) -> Self::I {
        M::gen_epilogue_placeholder()
    }

    fn set_num_spillslots(&mut self, slots: usize) {
        self.spillslots = Some(slots);
    }

    fn set_clobbered(&mut self, clobbered: Set<Writable<RealReg>>) {
        self.clobbered = clobbered;
    }

    /// Produce an instruction that computes a stackslot address.
    fn stackslot_addr(&self, slot: StackSlot, offset: u32, into_reg: Writable<Reg>) -> Self::I {
        // Offset from beginning of stackslot area, which is at nominal SP (see
        // [MemArg::NominalSPOffset] for more details on nominal SP tracking).
        let stack_off = self.stackslots[slot] as i64;
        let sp_off: i64 = stack_off + (offset as i64);
        M::gen_get_stack_addr(StackAMode::NominalSPOffset(sp_off, I8), into_reg, I8)
    }

    /// Load from a spillslot.
    fn load_spillslot(
        &self,
        slot: SpillSlot,
        ty: Type,
        into_regs: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I> {
        // Offset from beginning of spillslot area, which is at nominal SP + stackslots_size.
        let islot = slot.get() as i64;
        let spill_off = islot * M::word_bytes() as i64;
        let sp_off = self.stackslots_size as i64 + spill_off;
        log::trace!("load_spillslot: slot {:?} -> sp_off {}", slot, sp_off);

        gen_load_stack_multi::<M>(StackAMode::NominalSPOffset(sp_off, ty), into_regs, ty)
    }

    /// Store to a spillslot.
    fn store_spillslot(
        &self,
        slot: SpillSlot,
        ty: Type,
        from_regs: ValueRegs<Reg>,
    ) -> SmallInstVec<Self::I> {
        // Offset from beginning of spillslot area, which is at nominal SP + stackslots_size.
        let islot = slot.get() as i64;
        let spill_off = islot * M::word_bytes() as i64;
        let sp_off = self.stackslots_size as i64 + spill_off;
        log::trace!("store_spillslot: slot {:?} -> sp_off {}", slot, sp_off);

        gen_store_stack_multi::<M>(StackAMode::NominalSPOffset(sp_off, ty), from_regs, ty)
    }

    fn spillslots_to_stack_map(
        &self,
        slots: &[SpillSlot],
        state: &<Self::I as MachInstEmit>::State,
    ) -> StackMap {
        let virtual_sp_offset = M::get_virtual_sp_offset_from_state(state);
        let nominal_sp_to_fp = M::get_nominal_sp_to_fp(state);
        assert!(virtual_sp_offset >= 0);
        log::trace!(
            "spillslots_to_stackmap: slots = {:?}, state = {:?}",
            slots,
            state
        );
        let map_size = (virtual_sp_offset + nominal_sp_to_fp) as u32;
        let bytes = M::word_bytes();
        let map_words = (map_size + bytes - 1) / bytes;
        let mut bits = std::iter::repeat(false)
            .take(map_words as usize)
            .collect::<Vec<bool>>();

        let first_spillslot_word =
            ((self.stackslots_size + virtual_sp_offset as u32) / bytes) as usize;
        for &slot in slots {
            let slot = slot.get() as usize;
            bits[first_spillslot_word + slot] = true;
        }

        StackMap::from_slice(&bits[..])
    }

    fn gen_prologue(&mut self) -> SmallInstVec<Self::I> {
        let bytes = M::word_bytes();
        let mut total_stacksize = self.stackslots_size + bytes * self.spillslots.unwrap() as u32;
        if self.call_conv.extends_baldrdash() {
            debug_assert!(
                !self.flags.enable_probestack(),
                "baldrdash does not expect cranelift to emit stack probes"
            );
            total_stacksize += self.flags.baldrdash_prologue_words() as u32 * bytes;
        }
        let mask = M::stack_align(self.call_conv) - 1;
        let total_stacksize = (total_stacksize + mask) & !mask; // 16-align the stack.
        let clobbered_callee_saves = M::get_clobbered_callee_saves(self.call_conv, &self.clobbered);
        let mut insts = smallvec![];

        if !self.call_conv.extends_baldrdash() {
            self.fixed_frame_storage_size += total_stacksize;
            self.setup_frame = M::is_frame_setup_needed(
                self.is_leaf,
                self.stack_args_size(),
                clobbered_callee_saves.len(),
                self.fixed_frame_storage_size,
            );

            insts.extend(M::gen_debug_frame_info(&self.flags, &self.isa_flags).into_iter());

            if self.setup_frame {
                // set up frame
                insts.extend(M::gen_prologue_frame_setup(&self.flags).into_iter());
            }

            // Leaf functions with zero stack don't need a stack check if one's
            // specified, otherwise always insert the stack check.
            if total_stacksize > 0 || !self.is_leaf {
                if let Some((reg, stack_limit_load)) = &self.stack_limit {
                    insts.extend(stack_limit_load.clone());
                    self.insert_stack_check(*reg, total_stacksize, &mut insts);
                }
                if let Some(min_frame) = &self.probestack_min_frame {
                    if total_stacksize >= *min_frame {
                        insts.extend(M::gen_probestack(total_stacksize));
                    }
                }
            }
        }

        // Save clobbered registers.
        let (clobber_size, clobber_insts) = M::gen_clobber_save(
            self.call_conv,
            self.setup_frame,
            &self.flags,
            &clobbered_callee_saves,
            self.fixed_frame_storage_size,
            self.outgoing_args_size,
        );
        insts.extend(clobber_insts);

        // N.B.: "nominal SP", which we use to refer to stackslots and
        // spillslots, is defined to be equal to the stack pointer at this point
        // in the prologue.
        //
        // If we push any further data onto the stack in the function
        // body, we emit a virtual-SP adjustment meta-instruction so
        // that the nominal SP references behave as if SP were still
        // at this point. See documentation for
        // [crate::machinst::abi_impl](this module) for more details
        // on stackframe layout and nominal SP maintenance.

        self.total_frame_size = Some(total_stacksize + clobber_size as u32);
        insts
    }

    fn gen_epilogue(&self) -> SmallInstVec<M::I> {
        let mut insts = smallvec![];

        // Restore clobbered registers.
        insts.extend(M::gen_clobber_restore(
            self.call_conv,
            &self.flags,
            &self.clobbered,
            self.fixed_frame_storage_size,
            self.outgoing_args_size,
        ));

        // N.B.: we do *not* emit a nominal SP adjustment here, because (i) there will be no
        // references to nominal SP offsets before the return below, and (ii) the instruction
        // emission tracks running SP offset linearly (in straight-line order), not according to
        // the CFG, so early returns in the middle of function bodies would cause an incorrect
        // offset for the rest of the body.

        if !self.call_conv.extends_baldrdash() {
            if self.setup_frame {
                insts.extend(M::gen_epilogue_frame_restore(&self.flags));
            }

            insts.push(M::gen_ret());
        }

        log::trace!("Epilogue: {:?}", insts);
        insts
    }

    fn frame_size(&self) -> u32 {
        self.total_frame_size
            .expect("frame size not computed before prologue generation")
    }

    fn stack_args_size(&self) -> u32 {
        self.sig.stack_arg_space as u32
    }

    fn get_spillslot_size(&self, rc: RegClass) -> u32 {
        M::get_number_of_spillslots_for_value(rc)
    }

    fn gen_spill(&self, to_slot: SpillSlot, from_reg: RealReg) -> Self::I {
        let ty = ty_from_class(from_reg.to_reg().get_class());
        self.store_spillslot(to_slot, ty, ValueRegs::one(from_reg.to_reg()))
            .into_iter()
            .next()
            .unwrap()
    }

    fn gen_reload(&self, to_reg: Writable<RealReg>, from_slot: SpillSlot) -> Self::I {
        let ty = ty_from_class(to_reg.to_reg().get_class());
        self.load_spillslot(
            from_slot,
            ty,
            writable_value_regs(ValueRegs::one(to_reg.to_reg().to_reg())),
        )
        .into_iter()
        .next()
        .unwrap()
    }
}

fn abisig_to_uses_and_defs<M: ABIMachineSpec>(sig: &ABISig) -> (Vec<Reg>, Vec<Writable<Reg>>) {
    // Compute uses: all arg regs.
    let mut uses = Vec::new();
    for arg in &sig.args {
        if let &ABIArg::Slots { ref slots, .. } = arg {
            for slot in slots {
                match slot {
                    &ABIArgSlot::Reg { reg, .. } => {
                        uses.push(reg.to_reg());
                    }
                    _ => {}
                }
            }
        }
    }

    // Compute defs: all retval regs, and all caller-save (clobbered) regs.
    let mut defs = M::get_regs_clobbered_by_call(sig.call_conv);
    for ret in &sig.rets {
        if let &ABIArg::Slots { ref slots, .. } = ret {
            for slot in slots {
                match slot {
                    &ABIArgSlot::Reg { reg, .. } => {
                        defs.push(Writable::from_reg(reg.to_reg()));
                    }
                    _ => {}
                }
            }
        }
    }

    (uses, defs)
}

/// ABI object for a callsite.
pub struct ABICallerImpl<M: ABIMachineSpec> {
    /// CLIF-level signature, possibly normalized.
    ir_sig: ir::Signature,
    /// The called function's signature.
    sig: ABISig,
    /// All uses for the callsite, i.e., function args.
    uses: Vec<Reg>,
    /// All defs for the callsite, i.e., return values and caller-saves.
    defs: Vec<Writable<Reg>>,
    /// Call destination.
    dest: CallDest,
    /// Actual call opcode; used to distinguish various types of calls.
    opcode: ir::Opcode,
    /// Caller's calling convention.
    caller_conv: isa::CallConv,
    /// The settings controlling this compilation.
    flags: settings::Flags,

    _mach: PhantomData<M>,
}

/// Destination for a call.
#[derive(Debug, Clone)]
pub enum CallDest {
    /// Call to an ExtName (named function symbol).
    ExtName(ir::ExternalName, RelocDistance),
    /// Indirect call to a function pointer in a register.
    Reg(Reg),
}

impl<M: ABIMachineSpec> ABICallerImpl<M> {
    /// Create a callsite ABI object for a call directly to the specified function.
    pub fn from_func(
        sig: &ir::Signature,
        extname: &ir::ExternalName,
        dist: RelocDistance,
        caller_conv: isa::CallConv,
        flags: &settings::Flags,
    ) -> CodegenResult<ABICallerImpl<M>> {
        let ir_sig = ensure_struct_return_ptr_is_returned(sig);
        let sig = ABISig::from_func_sig::<M>(&ir_sig, flags)?;
        let (uses, defs) = abisig_to_uses_and_defs::<M>(&sig);
        Ok(ABICallerImpl {
            ir_sig,
            sig,
            uses,
            defs,
            dest: CallDest::ExtName(extname.clone(), dist),
            opcode: ir::Opcode::Call,
            caller_conv,
            flags: flags.clone(),
            _mach: PhantomData,
        })
    }

    /// Create a callsite ABI object for a call to a function pointer with the
    /// given signature.
    pub fn from_ptr(
        sig: &ir::Signature,
        ptr: Reg,
        opcode: ir::Opcode,
        caller_conv: isa::CallConv,
        flags: &settings::Flags,
    ) -> CodegenResult<ABICallerImpl<M>> {
        let ir_sig = ensure_struct_return_ptr_is_returned(sig);
        let sig = ABISig::from_func_sig::<M>(&ir_sig, flags)?;
        let (uses, defs) = abisig_to_uses_and_defs::<M>(&sig);
        Ok(ABICallerImpl {
            ir_sig,
            sig,
            uses,
            defs,
            dest: CallDest::Reg(ptr),
            opcode,
            caller_conv,
            flags: flags.clone(),
            _mach: PhantomData,
        })
    }
}

fn adjust_stack_and_nominal_sp<M: ABIMachineSpec, C: LowerCtx<I = M::I>>(
    ctx: &mut C,
    off: i32,
    is_sub: bool,
) {
    if off == 0 {
        return;
    }
    let amt = if is_sub { -off } else { off };
    for inst in M::gen_sp_reg_adjust(amt) {
        ctx.emit(inst);
    }
    ctx.emit(M::gen_nominal_sp_adj(-amt));
}

impl<M: ABIMachineSpec> ABICaller for ABICallerImpl<M> {
    type I = M::I;

    fn signature(&self) -> &ir::Signature {
        &self.ir_sig
    }

    fn num_args(&self) -> usize {
        if self.sig.stack_ret_arg.is_some() {
            self.sig.args.len() - 1
        } else {
            self.sig.args.len()
        }
    }

    fn accumulate_outgoing_args_size<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C) {
        let off = self.sig.stack_arg_space + self.sig.stack_ret_space;
        ctx.abi().accumulate_outgoing_args_size(off as u32);
    }

    fn emit_stack_pre_adjust<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C) {
        let off = self.sig.stack_arg_space + self.sig.stack_ret_space;
        adjust_stack_and_nominal_sp::<M, C>(ctx, off as i32, /* is_sub = */ true)
    }

    fn emit_stack_post_adjust<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C) {
        let off = self.sig.stack_arg_space + self.sig.stack_ret_space;
        adjust_stack_and_nominal_sp::<M, C>(ctx, off as i32, /* is_sub = */ false)
    }

    fn emit_copy_regs_to_arg<C: LowerCtx<I = Self::I>>(
        &self,
        ctx: &mut C,
        idx: usize,
        from_regs: ValueRegs<Reg>,
    ) {
        let word_rc = M::word_reg_class();
        let word_bits = M::word_bits() as usize;
        match &self.sig.args[idx] {
            &ABIArg::Slots { ref slots, .. } => {
                assert_eq!(from_regs.len(), slots.len());
                for (slot, from_reg) in slots.iter().zip(from_regs.regs().iter()) {
                    match slot {
                        &ABIArgSlot::Reg {
                            reg, ty, extension, ..
                        } => {
                            let ext = M::get_ext_mode(self.sig.call_conv, extension);
                            if ext != ir::ArgumentExtension::None && ty_bits(ty) < word_bits {
                                assert_eq!(word_rc, reg.get_class());
                                let signed = match ext {
                                    ir::ArgumentExtension::Uext => false,
                                    ir::ArgumentExtension::Sext => true,
                                    _ => unreachable!(),
                                };
                                ctx.emit(M::gen_extend(
                                    Writable::from_reg(reg.to_reg()),
                                    *from_reg,
                                    signed,
                                    ty_bits(ty) as u8,
                                    word_bits as u8,
                                ));
                            } else {
                                ctx.emit(M::gen_move(
                                    Writable::from_reg(reg.to_reg()),
                                    *from_reg,
                                    ty,
                                ));
                            }
                        }
                        &ABIArgSlot::Stack {
                            offset,
                            ty,
                            extension,
                            ..
                        } => {
                            let mut ty = ty;
                            let ext = M::get_ext_mode(self.sig.call_conv, extension);
                            if ext != ir::ArgumentExtension::None && ty_bits(ty) < word_bits {
                                assert_eq!(word_rc, from_reg.get_class());
                                let signed = match ext {
                                    ir::ArgumentExtension::Uext => false,
                                    ir::ArgumentExtension::Sext => true,
                                    _ => unreachable!(),
                                };
                                // Extend in place in the source register. Our convention is to
                                // treat high bits as undefined for values in registers, so this
                                // is safe, even for an argument that is nominally read-only.
                                ctx.emit(M::gen_extend(
                                    Writable::from_reg(*from_reg),
                                    *from_reg,
                                    signed,
                                    ty_bits(ty) as u8,
                                    word_bits as u8,
                                ));
                                // Store the extended version.
                                ty = M::word_type();
                            }
                            ctx.emit(M::gen_store_stack(
                                StackAMode::SPOffset(offset, ty),
                                *from_reg,
                                ty,
                            ));
                        }
                    }
                }
            }
            &ABIArg::StructArg { offset, size, .. } => {
                let src_ptr = from_regs.only_reg().unwrap();
                let dst_ptr = ctx.alloc_tmp(M::word_type()).only_reg().unwrap();
                ctx.emit(M::gen_get_stack_addr(
                    StackAMode::SPOffset(offset, I8),
                    dst_ptr,
                    I8,
                ));
                // Emit a memcpy from `src_ptr` to `dst_ptr` of `size` bytes.
                // N.B.: because we process StructArg params *first*, this is
                // safe w.r.t. clobbers: we have not yet filled in any other
                // arg regs.
                let memcpy_call_conv = isa::CallConv::for_libcall(&self.flags, self.sig.call_conv);
                for insn in
                    M::gen_memcpy(memcpy_call_conv, dst_ptr.to_reg(), src_ptr, size as usize)
                        .into_iter()
                {
                    ctx.emit(insn);
                }
            }
        }
    }

    fn get_copy_to_arg_order(&self) -> SmallVec<[usize; 8]> {
        let mut ret = SmallVec::new();
        for (i, arg) in self.sig.args.iter().enumerate() {
            // Struct args.
            if arg.is_struct_arg() {
                ret.push(i);
            }
        }
        for (i, arg) in self.sig.args.iter().enumerate() {
            // Non-struct args. Skip an appended return-area arg for multivalue
            // returns, if any.
            if !arg.is_struct_arg() && i < self.ir_sig.params.len() {
                ret.push(i);
            }
        }
        ret
    }

    fn emit_copy_retval_to_regs<C: LowerCtx<I = Self::I>>(
        &self,
        ctx: &mut C,
        idx: usize,
        into_regs: ValueRegs<Writable<Reg>>,
    ) {
        match &self.sig.rets[idx] {
            &ABIArg::Slots { ref slots, .. } => {
                assert_eq!(into_regs.len(), slots.len());
                for (slot, into_reg) in slots.iter().zip(into_regs.regs().iter()) {
                    match slot {
                        // Extension mode doesn't matter because we're copying out, not in,
                        // and we ignore high bits in our own registers by convention.
                        &ABIArgSlot::Reg { reg, ty, .. } => {
                            ctx.emit(M::gen_move(*into_reg, reg.to_reg(), ty));
                        }
                        &ABIArgSlot::Stack { offset, ty, .. } => {
                            let ret_area_base = self.sig.stack_arg_space;
                            ctx.emit(M::gen_load_stack(
                                StackAMode::SPOffset(offset + ret_area_base, ty),
                                *into_reg,
                                ty,
                            ));
                        }
                    }
                }
            }
            &ABIArg::StructArg { .. } => {
                panic!("StructArg not supported in return position");
            }
        }
    }

    fn emit_call<C: LowerCtx<I = Self::I>>(&mut self, ctx: &mut C) {
        let (uses, defs) = (
            mem::replace(&mut self.uses, Default::default()),
            mem::replace(&mut self.defs, Default::default()),
        );
        let word_type = M::word_type();
        if let Some(i) = self.sig.stack_ret_arg {
            let rd = ctx.alloc_tmp(word_type).only_reg().unwrap();
            let ret_area_base = self.sig.stack_arg_space;
            ctx.emit(M::gen_get_stack_addr(
                StackAMode::SPOffset(ret_area_base, I8),
                rd,
                I8,
            ));
            self.emit_copy_regs_to_arg(ctx, i, ValueRegs::one(rd.to_reg()));
        }
        let tmp = ctx.alloc_tmp(word_type).only_reg().unwrap();
        for (is_safepoint, inst) in M::gen_call(
            &self.dest,
            uses,
            defs,
            self.opcode,
            tmp,
            self.sig.call_conv,
            self.caller_conv,
        )
        .into_iter()
        {
            match is_safepoint {
                InstIsSafepoint::Yes => ctx.emit_safepoint(inst),
                InstIsSafepoint::No => ctx.emit(inst),
            }
        }
    }
}
