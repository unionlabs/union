//! ABI definitions.

use crate::binemit::StackMap;
use crate::ir::{Signature, StackSlot};
use crate::isa::CallConv;
use crate::machinst::*;
use crate::settings;
use regalloc::{Reg, Set, SpillSlot, Writable};
use smallvec::SmallVec;

/// A small vector of instructions (with some reasonable size); appropriate for
/// a small fixed sequence implementing one operation.
pub type SmallInstVec<I> = SmallVec<[I; 4]>;

/// Trait implemented by an object that tracks ABI-related state (e.g., stack
/// layout) and can generate code while emitting the *body* of a function.
pub trait ABICallee {
    /// The instruction type for the ISA associated with this ABI.
    type I: VCodeInst;

    /// Does the ABI-body code need a temp reg (and if so, of what type)? One
    /// will be provided to `init()` as the `maybe_tmp` arg if so.
    fn temp_needed(&self) -> Option<Type>;

    /// Initialize. This is called after the ABICallee is constructed because it
    /// may be provided with a temp vreg, which can only be allocated once the
    /// lowering context exists.
    fn init(&mut self, maybe_tmp: Option<Writable<Reg>>);

    /// Access the (possibly legalized) signature.
    fn signature(&self) -> &Signature;

    /// Accumulate outgoing arguments.  This ensures that at least SIZE bytes
    /// are allocated in the prologue to be available for use in function calls
    /// to hold arguments and/or return values.  If this function is called
    /// multiple times, the maximum of all SIZE values will be available.
    fn accumulate_outgoing_args_size(&mut self, size: u32);

    /// Get the settings controlling this function's compilation.
    fn flags(&self) -> &settings::Flags;

    /// Get the calling convention implemented by this ABI object.
    fn call_conv(&self) -> CallConv;

    /// Get the liveins of the function.
    fn liveins(&self) -> Set<RealReg>;

    /// Get the liveouts of the function.
    fn liveouts(&self) -> Set<RealReg>;

    /// Number of arguments.
    fn num_args(&self) -> usize;

    /// Number of return values.
    fn num_retvals(&self) -> usize;

    /// Number of stack slots (not spill slots).
    fn num_stackslots(&self) -> usize;

    /// The offsets of all stack slots (not spill slots) for debuginfo purposes.
    fn stackslot_offsets(&self) -> &PrimaryMap<StackSlot, u32>;

    /// Generate an instruction which copies an argument to a destination
    /// register.
    fn gen_copy_arg_to_regs(
        &self,
        idx: usize,
        into_reg: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I>;

    /// Is the given argument needed in the body (as opposed to, e.g., serving
    /// only as a special ABI-specific placeholder)? This controls whether
    /// lowering will copy it to a virtual reg use by CLIF instructions.
    fn arg_is_needed_in_body(&self, idx: usize) -> bool;

    /// Generate any setup instruction needed to save values to the
    /// return-value area. This is usually used when were are multiple return
    /// values or an otherwise large return value that must be passed on the
    /// stack; typically the ABI specifies an extra hidden argument that is a
    /// pointer to that memory.
    fn gen_retval_area_setup(&self) -> Option<Self::I>;

    /// Generate an instruction which copies a source register to a return value slot.
    fn gen_copy_regs_to_retval(
        &self,
        idx: usize,
        from_reg: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I>;

    /// Generate a return instruction.
    fn gen_ret(&self) -> Self::I;

    /// Generate an epilogue placeholder. The returned instruction should return `true` from
    /// `is_epilogue_placeholder()`; this is used to indicate to the lowering driver when
    /// the epilogue should be inserted.
    fn gen_epilogue_placeholder(&self) -> Self::I;

    // -----------------------------------------------------------------
    // Every function above this line may only be called pre-regalloc.
    // Every function below this line may only be called post-regalloc.
    // `spillslots()` must be called before any other post-regalloc
    // function.
    // ----------------------------------------------------------------

    /// Update with the number of spillslots, post-regalloc.
    fn set_num_spillslots(&mut self, slots: usize);

    /// Update with the clobbered registers, post-regalloc.
    fn set_clobbered(&mut self, clobbered: Set<Writable<RealReg>>);

    /// Get the address of a stackslot.
    fn stackslot_addr(&self, slot: StackSlot, offset: u32, into_reg: Writable<Reg>) -> Self::I;

    /// Load from a spillslot.
    fn load_spillslot(
        &self,
        slot: SpillSlot,
        ty: Type,
        into_reg: ValueRegs<Writable<Reg>>,
    ) -> SmallInstVec<Self::I>;

    /// Store to a spillslot.
    fn store_spillslot(
        &self,
        slot: SpillSlot,
        ty: Type,
        from_reg: ValueRegs<Reg>,
    ) -> SmallInstVec<Self::I>;

    /// Generate a stack map, given a list of spillslots and the emission state
    /// at a given program point (prior to emission fo the safepointing
    /// instruction).
    fn spillslots_to_stack_map(
        &self,
        slots: &[SpillSlot],
        state: &<Self::I as MachInstEmit>::State,
    ) -> StackMap;

    /// Generate a prologue, post-regalloc. This should include any stack
    /// frame or other setup necessary to use the other methods (`load_arg`,
    /// `store_retval`, and spillslot accesses.)  `self` is mutable so that we
    /// can store information in it which will be useful when creating the
    /// epilogue.
    fn gen_prologue(&mut self) -> SmallInstVec<Self::I>;

    /// Generate an epilogue, post-regalloc. Note that this must generate the
    /// actual return instruction (rather than emitting this in the lowering
    /// logic), because the epilogue code comes before the return and the two are
    /// likely closely related.
    fn gen_epilogue(&self) -> SmallInstVec<Self::I>;

    /// Returns the full frame size for the given function, after prologue
    /// emission has run. This comprises the spill slots and stack-storage slots
    /// (but not storage for clobbered callee-save registers, arguments pushed
    /// at callsites within this function, or other ephemeral pushes).  This is
    /// used for ABI variants where the client generates prologue/epilogue code,
    /// as in Baldrdash (SpiderMonkey integration).
    fn frame_size(&self) -> u32;

    /// Returns the size of arguments expected on the stack.
    fn stack_args_size(&self) -> u32;

    /// Get the spill-slot size.
    fn get_spillslot_size(&self, rc: RegClass) -> u32;

    /// Generate a spill.
    fn gen_spill(&self, to_slot: SpillSlot, from_reg: RealReg) -> Self::I;

    /// Generate a reload (fill).
    fn gen_reload(&self, to_reg: Writable<RealReg>, from_slot: SpillSlot) -> Self::I;
}

/// Trait implemented by an object that tracks ABI-related state and can
/// generate code while emitting a *call* to a function.
///
/// An instance of this trait returns information for a *particular*
/// callsite. It will usually be computed from the called function's
/// signature.
///
/// Unlike `ABICallee` above, methods on this trait are not invoked directly
/// by the machine-independent code. Rather, the machine-specific lowering
/// code will typically create an `ABICaller` when creating machine instructions
/// for an IR call instruction inside `lower()`, directly emit the arg and
/// and retval copies, and attach the register use/def info to the call.
///
/// This trait is thus provided for convenience to the backends.
pub trait ABICaller {
    /// The instruction type for the ISA associated with this ABI.
    type I: VCodeInst;

    /// Get the number of arguments expected.
    fn num_args(&self) -> usize;

    /// Access the (possibly legalized) signature.
    fn signature(&self) -> &Signature;

    /// Emit a copy of an argument value from a source register, prior to the call.
    fn emit_copy_regs_to_arg<C: LowerCtx<I = Self::I>>(
        &self,
        ctx: &mut C,
        idx: usize,
        from_reg: ValueRegs<Reg>,
    );

    /// Specific order for copying into arguments at callsites. We must be
    /// careful to copy into StructArgs first, because we need to be able
    /// to invoke memcpy() before we've loaded other arg regs (see above).
    fn get_copy_to_arg_order(&self) -> SmallVec<[usize; 8]>;

    /// Emit a copy a return value into a destination register, after the call returns.
    fn emit_copy_retval_to_regs<C: LowerCtx<I = Self::I>>(
        &self,
        ctx: &mut C,
        idx: usize,
        into_reg: ValueRegs<Writable<Reg>>,
    );

    /// Emit code to pre-adjust the stack, prior to argument copies and call.
    fn emit_stack_pre_adjust<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C);

    /// Emit code to post-adjust the satck, after call return and return-value copies.
    fn emit_stack_post_adjust<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C);

    /// Accumulate outgoing arguments.  This ensures that the caller (as
    /// identified via the CTX argument) allocates enough space in the
    /// prologue to hold all arguments and return values for this call.
    /// There is no code emitted at the call site, everything is done
    /// in the caller's function prologue.
    fn accumulate_outgoing_args_size<C: LowerCtx<I = Self::I>>(&self, ctx: &mut C);

    /// Emit the call itself.
    ///
    /// The returned instruction should have proper use- and def-sets according
    /// to the argument registers, return-value registers, and clobbered
    /// registers for this function signature in this ABI.
    ///
    /// (Arg registers are uses, and retval registers are defs. Clobbered
    /// registers are also logically defs, but should never be read; their
    /// values are "defined" (to the regalloc) but "undefined" in every other
    /// sense.)
    ///
    /// This function should only be called once, as it is allowed to re-use
    /// parts of the ABICaller object in emitting instructions.
    fn emit_call<C: LowerCtx<I = Self::I>>(&mut self, ctx: &mut C);
}
