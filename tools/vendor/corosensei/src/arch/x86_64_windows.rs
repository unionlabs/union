//! Windows is sufficiently different that it gets its own file. The code here
//! is heavily based off the generic x86_64 version, so you should read that
//! and fully understand it first.
//!
//! On x86_64, Windows differs from other platforms in a few ways:
//! - Information about the current stack is stored in the Thread Environment
//!   Block (TEB) which is accessed via the GS segment register. These TEB
//!   fields must be swapped when switching stacks. Some of these fields are
//!   mutable since Windows grows stacks dynamically.
//! - Windows uses a different calling convention than other platforms which use
//!   the standard System V x86_64 calling convention. However to keep the code
//!   simpler and avoid divergence we force the use of the System V calling
//!   convention on Windows.
//! - Windows uses Structured Exception Handling for unwinding instead of
//!   DWARF, which has a different (and more restrictive) set of unwind opcodes.
//!
//! ## Stack layout
//!
//! Here is what the layout of the stack looks like when a coroutine is
//! suspended.
//!
//! ```text
//! +--------------------------+  <- Stack base
//! | TEB.StackLimit           |  <- These are set by switch_and_reset from the
//! +--------------------------+   | the values in the TEB (see below) so that
//! | TEB.GuaranteedStackBytes |  <- they can be updated in the `Stack` if it
//! +--------------------------+     is later reused for another coroutine.
//! | Initial func             |     The other 2 TEB fields don't need to be
//! +--------------------------+     saved since they are constant for the
//! | Parent link              |     lifetime of the stack.
//! +--------------------------+
//! |                          |
//! ~     ...                  ~
//! |                          |
//! +--------------------------+
//! | TEB.GuaranteedStackBytes |  <-
//! +--------------------------+   | These are fields in the Thread Environement
//! | TEB.DeallocationStack    |  <- Block (TEB) which need to be saved and
//! +--------------------------+   | restored when switching stacks. The OS
//! | TEB.StackLimit           |  <- looks at these fields for various stack
//! +--------------------------+   | related operations.
//! | TEB.StackBase            |  <-
//! +--------------------------+
//! | TEB.ExceptionList        |  <- This is the legacy SEH exception handler
//! +--------------------------+     chain which is not used in Win64 but is
//! | Saved RBP                |     used in Wine.
//! +--------------------------+
//! | Saved RBX                |
//! +--------------------------+
//! | Saved RIP                |
//! +--------------------------+
//! ```
//!
//! And this is the layout of the parent stack when a coroutine is running:
//!
//! ```text
//! |                          |
//! ~     ...                  ~
//! |                          |
//! +--------------------------+
//! | Saved RIP                |  <- A second copy of the saved RIP is needed
//! +--------------------------+     here due to the limitation of SEH unwind
//! | TEB.GuaranteedStackBytes |     codes which are not as flexible as DWARF
//! +--------------------------+     CFI.
//! | TEB.DeallocationStack    |
//! +--------------------------+
//! | TEB.StackLimit           |
//! +--------------------------+
//! | TEB.StackBase            |
//! +--------------------------+
//! | TEB.ExceptionList        |
//! +--------------------------+
//! | Saved RBX                |
//! +--------------------------+
//! | Saved RIP                |
//! +--------------------------+
//! | Saved RBP                |
//! +--------------------------+
//! ```
//!
//! And finally, this is the stack layout of a coroutine that has just been
//! initialized:
//!
//! ```text
//! +--------------------------+  <- Stack base
//! | TEB.StackLimit           |  <- These haven't been initialized yet at this
//! +--------------------------+   | point.
//! | TEB.GuaranteedStackBytes |  <-
//! +--------------------------+
//! | Initial func             |
//! +--------------------------+
//! | Parent link              |  <- This hasn't been filled in yet at this
//! +--------------------------+     point.
//! |                          |
//! ~ Initial obj              ~  <- This has an unknown size, but the size is
//! |                          |     rounded up to `STACK_ALIGNMENT`.
//! +--------------------------+
//! | TEB.GuaranteedStackBytes |  <-
//! +--------------------------+   |
//! | TEB.DeallocationStack    |  <- Initial values for the TEB fields on the
//! +--------------------------+   | new stack.
//! | TEB.StackLimit           |  <-
//! +--------------------------+   |
//! | TEB.StackBase            |  <-
//! +--------------------------+
//! | Initial RIP              |  <- Points to stack_init_trampoline
//! +--------------------------+
//! ```

use core::arch::{asm, global_asm};

use super::{allocate_obj_on_stack, push};
use crate::stack::{Stack, StackPointer, StackTebFields};
use crate::unwind::{
    asm_may_unwind_root, asm_may_unwind_yield, InitialFunc, StackCallFunc, TrapHandler,
};
use crate::util::EncodedValue;

pub const STACK_ALIGNMENT: usize = 16;
pub const PARENT_LINK_OFFSET: usize = 0;
pub type StackWord = u64;

global_asm!(
    ".balign 16",
    asm_function_begin!("stack_init_trampoline"),
    ".seh_proc stack_init_trampoline",
    // At this point our register state contains the following:
    // - RSP points to the top of the parent stack.
    // - RBP holds its value from the parent context.
    // - RDX points to the top of our stack.
    // - RSI points to the base of our stack.
    // - RDI contains the argument passed from switch_and_link.
    //
    // Save RBP from the parent context last to create a valid frame record.
    "push rbp",
    // Fill in the parent link.
    "mov [rsi - 32], rsp",
    // Adjust RSI to point to the parent link for the second argument.
    "sub rsi, 32",
    // Switch to the coroutine stack, skipping the address of
    // stack_init_trampoline() at the top of the stack.
    "lea rsp, [rdx + 8]",
    // Reset the ExceptionList field of the TEB. This is not used on Win64 but
    // it *is* used by Wine. The end-of-chain value is !0.
    "mov qword ptr gs:[0x0], 0xffffffffffffffff",
    // Pop the TEB fields for our new stack.
    "pop qword ptr gs:[0x8]",    // StackBase
    "pop qword ptr gs:[0x10]",   // StackLimit
    "pop qword ptr gs:[0x1478]", // DeallocationStack
    "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    "mov rbp, rsi",
    // These directives tell the unwinder how to restore the register state to
    // that of the parent call frame. These are executed in reverse by the
    // unwinder as it undoes the effects of the function prologue.
    //
    // When the unwinder reaches this frame, it will have a virtual RBP pointing
    // at the parent link. Then, the following SEH unwinding operations are
    // performed in order:
    // - .seh_setframe rbp, 0: This copies the virtual RBP to the virtual RSP so
    //   that all future stack references by the unwinder are done using that
    //   register.
    // - .seh_savereg rsp, 0: This is where the magic happens. The parent link
    //   is read off the stack and placed in the virtual RSP which now points to
    //   the top of the parent stack.
    // - .seh_pushreg rbp: This pops and restores RBP from the parent context.
    // - .seh_stackalloc 8: This increases RSP by 8 to pop the saved RIP.
    // - .seh_pushreg rbx: This pops and restores RBX from the parent context.
    // - .seh_stackalloc 40: This increases RSP by 40 to pop the saved TEB
    //   fields.
    //
    // After all these operations, the unwinder will pop a return address off
    // the stack. This is the secondary copy of the return address created in
    // switch_and_link.
    //
    // We end up with the register state that we had prior to entering the
    // asm!() block in switch_and_link. Unwinding can then proceed on the parent
    // stack normally.
    //
    // Note that we only use these unwind opcodes for computing backtraces. We
    // can't actually use this to throw an exception across stacks because the
    // unwinder will not update the TEB fields when switching stacks.
    ".seh_stackalloc 40",
    ".seh_pushreg rbx",
    ".seh_stackalloc 8",
    ".seh_pushreg rbp",
    ".seh_savereg rsp, 0",
    ".seh_setframe rbp, 0",
    ".seh_endprologue",
    // Set up the 3rd argument to the initial function to point to the object
    // that init_stack() set up on the stack.
    "mov rdx, rsp",
    // As in the original x86_64 code, hand-write the call operation so that it
    // doesn't push an entry into the CPU's return prediction stack.
    concat!(
        "lea rcx, [rip + ",
        asm_mangle!("stack_init_trampoline_return"),
        "]"
    ),
    "push rcx",
    "jmp [rsi + 8]",
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // The SEH unwinder works by looking at a return target and scanning forward
    // to look for an epilog code sequence. We add an int3 instruction to avoid
    // this scan running off the end of the function. This works since int3 is
    // not part of any valid epilog code sequence.
    //
    // See LLVM's X86AvoidTrailingCall pass for more details:
    // https://github.com/llvm/llvm-project/blob/5aa24558cfa67e2a2e99c4e9c6d6b68bf372e00e/lib/Target/X86/X86AvoidTrailingCall.cpp
    "int3",
    ".seh_endproc",
    asm_function_end!("stack_init_trampoline"),
);

global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 16",
    asm_function_begin!("stack_call_trampoline"),
    ".seh_proc stack_call_trampoline",
    // At this point our register state contains the following:
    // - RSP points to the top of the parent stack.
    // - RBP holds its value from the parent context.
    // - RDX is the function that should be called.
    // - RSI points to the top of our stack.
    // - RDI contains the argument to be passed to the function.
    // - R8-R11 contains the TEB fields for the new stack.
    //
    // Save RBP.
    "push rbp",
    ".seh_pushreg rbp",
    // Save the TEB fields to the stack.
    "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x1478]", // DeallocationStack
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x10]", // StackLimit
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x8]", // StackBase
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x0]", // ExceptionList
    ".seh_stackalloc 8",
    // Set up a stack frame.
    "mov rbp, rsp",
    ".seh_setframe rbp, 0",
    ".seh_endprologue",
    // Switch to the new stack.
    "mov rsp, rsi",
    // Reset the ExceptionList field of the TEB. This is not used on Win64 but
    // it *is* used by Wine. The end-of-chain value is !0.
    "mov qword ptr gs:[0x0], 0xffffffffffffffff",
    // Set the TEB fields for the new stack.
    "mov gs:[0x8], r8",     // StackBase
    "mov gs:[0x10], r9",    // StackLimit
    "mov gs:[0x1478], r10", // DeallocationStack
    "mov gs:[0x1748], r11", // GuaranteedStackBytes
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "call rdx",
    // Save the two mutable TEB fields to the stack base so they can be
    // retrieved later.
    "push qword ptr gs:[0x10]",   // StackLimit
    "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    "mov rsp, rbp",
    "pop qword ptr gs:[0x0]",    // ExceptionList
    "pop qword ptr gs:[0x8]",    // StackBase
    "pop qword ptr gs:[0x10]",   // StackLimit
    "pop qword ptr gs:[0x1478]", // DeallocationStack
    "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes
    "pop rbp",
    "ret",
    ".seh_endproc",
    asm_function_end!("stack_call_trampoline"),
);

// These trampolines use a custom calling convention and should only be called
// with inline assembly.
extern "C" {
    fn stack_init_trampoline(arg: EncodedValue, stack_base: StackPointer, stack_ptr: StackPointer);
    fn stack_init_trampoline_return();
    #[allow(dead_code)]
    fn stack_call_trampoline(arg: *mut u8, sp: StackPointer, f: StackCallFunc);
}

#[inline]
pub unsafe fn init_stack<T>(stack: &impl Stack, func: InitialFunc<T>, obj: T) -> StackPointer {
    let mut sp = stack.base().get();

    // Placeholders for returning TEB.StackLimit and TEB.GuaranteedStackBytes.
    push(&mut sp, None);
    push(&mut sp, None);

    // Initial function.
    push(&mut sp, Some(func as StackWord));

    // Placeholder for parent link.
    push(&mut sp, None);

    // Allocate space on the stack for the initial object, rounding to
    // STACK_ALIGNMENT.
    allocate_obj_on_stack(&mut sp, 32, obj);

    // Write the TEB fields for the target stack.
    let teb = stack.teb_fields();
    push(&mut sp, Some(teb.GuaranteedStackBytes as StackWord));
    push(&mut sp, Some(teb.DeallocationStack as StackWord));
    push(&mut sp, Some(teb.StackLimit as StackWord));
    push(&mut sp, Some(teb.StackBase as StackWord));

    // The stack is aligned to STACK_ALIGNMENT at this point.
    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Entry point called by switch_and_link().
    push(&mut sp, Some(stack_init_trampoline as StackWord));

    StackPointer::new_unchecked(sp)
}

#[inline]
pub unsafe fn switch_and_link(
    arg: EncodedValue,
    sp: StackPointer,
    stack_base: StackPointer,
) -> (EncodedValue, Option<StackPointer>) {
    let (ret_val, ret_sp);

    asm_may_unwind_root!(
        // Set up a secondary copy of the return address. This is only used by
        // the unwinder, not by actual returns.
        "lea rax, [rip + 0f]",
        "push rax",

        // Save the TEB fields to the stack.
        "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
        "push qword ptr gs:[0x1478]", // DeallocationStack
        "push qword ptr gs:[0x10]", // StackLimit
        "push qword ptr gs:[0x8]", // StackBase
        "push qword ptr gs:[0x0]", // ExceptionList

        "push rbx",

        // Push a return address onto our stack and then jump to the return
        // address at the top of the coroutine stack.
        //
        // From here on execution continues in stack_init_trampoline or the 0:
        // label in switch_yield.
        "call [rdx]",

        // Upon returning, our register state contains the following:
        // - RSP: Our stack, with the return address and RBP popped.
        // - RSI: The top of the coroutine stack, or 0 if coming from
        //        switch_and_reset.
        // - RDI: The argument passed from the coroutine.
        "0:",

        "pop rbx",

        // Restore the TEB fields.
        "pop qword ptr gs:[0x0]", // ExceptionList
        "pop qword ptr gs:[0x8]", // StackBase
        "pop qword ptr gs:[0x10]", // StackLimit
        "pop qword ptr gs:[0x1478]", // DeallocationStack
        "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes

        // Pop the secondary return address.
        "add rsp, 8",

        // Pass the argument in RDI.
        inlateout("rdi") arg => ret_val,

        // We get the coroutine stack pointer back in RSI.
        lateout("rsi") ret_sp,

        // We pass the stack base in RSI.
        in("rsi") stack_base.get() as u64,

        // We pass the target stack pointer in RDX.
        in("rdx") sp.get() as u64,

        // Mark all registers as clobbered.
        lateout("r12") _, lateout("r13") _, lateout("r14") _, lateout("r15") _,
        clobber_abi("sysv64"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind_yield!(
        // Save the TEB fields to the stack.
        "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
        "push qword ptr gs:[0x1478]", // DeallocationStack
        "push qword ptr gs:[0x10]", // StackLimit
        "push qword ptr gs:[0x8]", // StackBase
        "push qword ptr gs:[0x0]", // ExceptionList

        "push rbp",
        "push rbx",

        // Push a return address on the stack. This is the address that will be
        // called by switch_and_link() the next time this context is resumed.
        "lea rax, [rip + 0f]",
        "push rax",

        // Save our stack pointer to RSI, which is then returned out of
        // switch_and_link().
        "mov rsi, rsp",

        // Load the parent context's stack pointer.
        "mov rsp, [rdx]",

        // Restore the parent context's RBP.
        "pop rbp",

        // Return into the parent context. This returns control to
        // switch_and_link() after the call instruction.
        "ret",

        // This gets called by switch_and_link(). At this point our register
        // state contains the following:
        // - RSP points to the top of the parent stack.
        // - RBP holds its value from the parent context.
        // - RDX points to the top of our stack, including the return address.
        // - RSI points to the base of our stack.
        // - RDI contains the argument passed from switch_and_link.
        "0:",

        // Save RBP from the parent context last to create a valid frame record.
        "push rbp",

        // Update the parent link near the base of the stack.
        "mov [rsi - 32], rsp",

        // Switch back to our stack, skipping the return address.
        "lea rsp, [rdx + 8]",

        "pop rbx",
        "pop rbp",

        // Restore the TEB fields.
        "pop qword ptr gs:[0x0]", // ExceptionList
        "pop qword ptr gs:[0x8]", // StackBase
        "pop qword ptr gs:[0x10]", // StackLimit
        "pop qword ptr gs:[0x1478]", // DeallocationStack
        "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes

        // Pass the argument in RDI.
        inlateout("rdi") arg => ret_val,

        // The parent link can be in any register, RDX is arbitrarily chosen
        // here.
        in("rdx") parent_link as u64,

        // Mark all registers as clobbered.
        lateout("r12") _, lateout("r13") _, lateout("r14") _, lateout("r15") _,
        clobber_abi("sysv64"),
    );

    ret_val
}

#[inline(always)]
pub unsafe fn switch_and_reset(arg: EncodedValue, parent_link: *mut StackPointer) -> ! {
    asm!(
        // Write the 2 TEB fields which can change during corountine execution
        // to the base of the stack. This is later recovered by
        // update_teb_from_stack().
        "mov rax, gs:[0x10]", // StackLimit
        "mov [rdx + 24], rax",
        "mov rax, gs:[0x1748]", // GuaranteedStackBytes
        "mov [rdx + 16], rax",

        // Load the parent context's stack pointer.
        "mov rsp, [rdx]",

        // Restore the parent context's RBP.
        "pop rbp",

        // Return into the parent context.
        "ret",

        in("rdx") parent_link as u64,
        in("rdi") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("rsi") 0,

        options(noreturn),
    );
}

#[inline]
#[cfg(feature = "asm-unwind")]
pub unsafe fn switch_and_throw(
    forced_unwind: crate::unwind::ForcedUnwind,
    sp: StackPointer,
    stack_base: StackPointer,
) -> (EncodedValue, Option<StackPointer>) {
    extern "sysv64-unwind" fn throw(forced_unwind: crate::unwind::ForcedUnwind) -> ! {
        extern crate std;
        use std::boxed::Box;
        std::panic::resume_unwind(Box::new(forced_unwind));
    }

    let (ret_val, ret_sp);

    asm_may_unwind_root!(
        // Save state just like the first half of switch_and_link().
        "lea rax, [rip + 0f]",
        "push rax",
        "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
        "push qword ptr gs:[0x1478]", // DeallocationStack
        "push qword ptr gs:[0x10]", // StackLimit
        "push qword ptr gs:[0x8]", // StackBase
        "push qword ptr gs:[0x0]", // ExceptionList
        "push rbx",

        // Push a second copy of the return address to the stack.
        "push rax",

        // Save RBP of the parent context.
        "push rbp",

        // Update the parent link near the base of the coroutine stack.
        "mov [rsi - 32], rsp",

        // Switch to the coroutine stack.
        "mov rsp, rdx",

        // Pop the return address of the target context.
        "pop rax",

        // Restore RBP and RBX from the target context.
        "pop rbx",
        "pop rbp",

        // Restore the TEB fields of the target context.
        "pop qword ptr gs:[0x0]", // ExceptionList
        "pop qword ptr gs:[0x8]", // StackBase
        "pop qword ptr gs:[0x10]", // StackLimit
        "pop qword ptr gs:[0x1478]", // DeallocationStack
        "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes

        // Simulate a call with an artificial return address so that the throw
        // function will unwind straight into the switch_and_yield() call with
        // the register state expected outside the asm! block.
        "push rax",
        "jmp {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // This is copied from the second half of switch_and_link().
        "pop rbx",
        "pop qword ptr gs:[0x0]", // ExceptionList
        "pop qword ptr gs:[0x8]", // StackBase
        "pop qword ptr gs:[0x10]", // StackLimit
        "pop qword ptr gs:[0x1478]", // DeallocationStack
        "pop qword ptr gs:[0x1748]", // GuaranteedStackBytes
        "add rsp, 8",

        // Helper function to trigger stack unwinding.
        throw = sym throw,

        // Argument to pass to the throw function.
        in("rdi") forced_unwind.0.get(),

        // Same output registers as switch_and_link().
        lateout("rdi") ret_val,
        lateout("rsi") ret_sp,

        // Stack pointer and stack base inputs for stack switching.
        in("rsi") stack_base.get() as u64,
        in("rdx") sp.get() as u64,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("r12") _, lateout("r13") _, lateout("r14") _, lateout("r15") _,
        clobber_abi("sysv64"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline]
pub unsafe fn drop_initial_obj(
    stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(40);
    drop_fn(ptr);

    // Also copy the TEB fields to the base of the stack so that they can be
    // retreived by update_stack_teb_fields().
    let base = stack_base.get() as *mut StackWord;
    let stack = stack_ptr.get() as *const StackWord;
    *base.sub(1) = *stack.add(2); // StackLimit
    *base.sub(2) = *stack.add(4); // GuaranteedStackBytes
}

/// This function must be called after a stack has finished running a coroutine
/// so that the `StackLimit` and `GuaranteedStackBytes` fields from the TEB can
/// be updated in the stack. This is necessary if the stack is reused for
/// another coroutine.
#[inline]
pub unsafe fn update_stack_teb_fields(stack: &mut impl Stack) {
    let base = stack.base().get() as *const StackWord;
    let stack_limit = *base.sub(1) as usize;
    let guaranteed_stack_bytes = *base.sub(2) as usize;
    stack.update_teb_fields(stack_limit, guaranteed_stack_bytes);
}

/// This function is called by `on_parent_stack` to read the saved TEB fields
/// saved on the parent stack.
#[inline]
pub unsafe fn read_parent_stack_teb_fields(stack_ptr: StackPointer) -> StackTebFields {
    let stack_ptr = stack_ptr.get() as *const StackWord;
    StackTebFields {
        StackBase: *stack_ptr.add(4) as usize,
        StackLimit: *stack_ptr.add(5) as usize,
        DeallocationStack: *stack_ptr.add(6) as usize,
        GuaranteedStackBytes: *stack_ptr.add(7) as usize,
    }
}

/// This function is called by `on_parent_stack` to update the saved TEB fields
/// saved on the parent stack at the end of execution.
#[inline]
pub unsafe fn update_parent_stack_teb_fields(
    stack_ptr: StackPointer,
    stack_limit: usize,
    guaranteed_stack_bytes: usize,
) {
    let stack_ptr = stack_ptr.get() as *mut StackWord;
    *stack_ptr.add(5) = stack_limit as StackWord;
    *stack_ptr.add(7) = guaranteed_stack_bytes as StackWord;
}

/// Registers which must be updated upon return from a trap handler.
///
/// The exact set of registers that need to be updated varies depending on the
/// target. Note that *all* registers must be updated to the specified values,
/// otherwise behavior is undefined.
///
/// To catch any issues at compilation time, it is recommended to use Rust's
/// pattern matching syntax to extract the individual registers from this
/// struct.
///
/// ```
/// # use corosensei::trap::TrapHandlerRegs;
/// # let regs = TrapHandlerRegs { rip: 0, rsp: 0, rbp: 0, rdi: 0, rsi: 0 };
/// let TrapHandlerRegs { rip, rsp, rbp, rdi, rsi } = regs;
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct TrapHandlerRegs {
    pub rip: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
}

pub unsafe fn setup_trap_trampoline<T>(
    stack_base: StackPointer,
    val: T,
    handler: TrapHandler<T>,
) -> TrapHandlerRegs {
    // Preserve the top 32 bytes of the stack since they contain the parent
    // link. The top 16 bytes are filled by switch_and_reset() with TEB fields.
    let parent_link = stack_base.get() - 32;

    // Everything below this can be overwritten. Write the object to the stack.
    let mut sp = parent_link;
    allocate_obj_on_stack(&mut sp, 32, val);
    let val_ptr = sp;

    // Set up a return address which returns to stack_init_trampoline.
    push(&mut sp, Some(stack_init_trampoline_return as StackWord));

    // Set up registers for entry into the function.
    TrapHandlerRegs {
        rip: handler as u64,
        rsp: sp as u64,
        rdi: val_ptr as u64,
        rsi: parent_link as u64,
        rbp: parent_link as u64,
    }
}

/// This function executes a function on the given stack. The argument is passed
/// through to the called function.
#[inline]
pub unsafe fn on_stack<S: Stack>(arg: *mut u8, stack: S, f: StackCallFunc) {
    let stack = scopeguard::guard(stack, |mut stack| {
        let base = stack.base().get() as *const u64;
        let stack_limit = *base.sub(1) as usize;
        let guaranteed_stack_bytes = *base.sub(2) as usize;
        stack.update_teb_fields(stack_limit, guaranteed_stack_bytes);
    });

    let teb_fields = stack.teb_fields();

    asm_may_unwind_root!(
        concat!("call ", asm_mangle!("stack_call_trampoline")),
        in("rdi") arg,
        in("rsi") stack.base().get(),
        in("rdx") f,
        in("r8") teb_fields.StackBase,
        in("r9") teb_fields.StackLimit,
        in("r10") teb_fields.DeallocationStack,
        in("r11") teb_fields.GuaranteedStackBytes,
        clobber_abi("sysv64"),
    );
}
