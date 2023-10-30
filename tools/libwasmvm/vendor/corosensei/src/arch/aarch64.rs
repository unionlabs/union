//! Low-level AArch64 support.
//!
//! This file is heavily based on the x86_64 implementation.
//! Relevant differences are highlighted in comments, but otherwise most
//! comments have been removed to avoid duplication. Refer to x86_64.rs for
//! detailed comments about what is happening in this file.
//!
//! ## Stack layout
//!
//! Here is what the layout of the stack looks like when a coroutine is
//! suspended.
//!
//! ```text
//! +--------------+  <- Stack base
//! | Initial func |
//! +--------------+
//! | Parent link  |
//! +--------------+
//! |              |
//! ~     ...      ~
//! |              |
//! +--------------+
//! | Padding      |
//! +--------------+
//! | Saved PC     |
//! +--------------+
//! | Saved X29    |
//! +--------------+
//! | Saved X19    |
//! +--------------+
//! ```
//!
//! And this is the layout of the parent stack when a coroutine is running:
//!
//! ```text
//! |           |
//! ~    ...    ~
//! |           |
//! +-----------+
//! | Padding   |
//! +-----------+
//! | Saved X19 |
//! +-----------+
//! | Saved PC  |
//! +-----------+
//! | Saved X29 |
//! +-----------+
//! ```
//!
//! And finally, this is the stack layout of a coroutine that has just been
//! initialized:
//!
//! ```text
//! +--------------+  <- Stack base
//! | Initial func |
//! +--------------+
//! | Parent link  |
//! +--------------+
//! |              |
//! ~ Initial obj  ~
//! |              |
//! +--------------+
//! | Padding      |
//! +--------------+
//! | Initial PC   |
//! +--------------+
//! | Padding      |
//! +--------------+
//! | Padding      |
//! +--------------+  <- Initial stack pointer
//! ```

use core::arch::{asm, global_asm};

use super::{allocate_obj_on_stack, push};
use crate::stack::{Stack, StackPointer};
use crate::unwind::{
    asm_may_unwind_root, asm_may_unwind_yield, cfi_reset_args_size_root, cfi_reset_args_size_yield,
    InitialFunc, StackCallFunc, TrapHandler,
};
use crate::util::EncodedValue;

pub const STACK_ALIGNMENT: usize = 16;
pub const PARENT_LINK_OFFSET: usize = 0;
pub type StackWord = u64;

global_asm!(
    ".balign 4",
    asm_function_begin!("stack_init_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - SP points to the top of the parent stack.
    // - LR contains the return address in the parent context.
    // - X19 and X29 contain their value from the parent context.
    // - X2 points to the top of the coroutine stack.
    // - X1 points to the base of our stack.
    // - X0 contains the argument passed from switch_and_link.
    //
    // Push the X19, X29 and PC values of the parent context onto the parent
    // stack.
    "stp x29, lr, [sp, #-32]!",
    "str x19, [sp, #16]",
    // Write the parent stack pointer to the parent link and adjust X1 to point
    // to the parent link.
    "mov x3, sp",
    "str x3, [x1, #-16]!",
    // Switch to the coroutine stack and pop the padding and initial PC.
    "add sp, x2, #32",
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    "mov x29, x1",
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x8d 0x00: DW_OP_breg29 (x29 + 0)
    // 0x06: DW_OP_deref
    // 0x23, 0x20: DW_OP_plus_uconst 32
    ".cfi_escape 0x0f, 5, 0x8d, 0x00, 0x06, 0x23, 0x20",
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    ".cfi_offset x19, -16",
    ".cfi_offset lr, -24",
    ".cfi_offset x29, -32",
    // Set up the 3rd argument to the initial function to point to the object
    // that init_stack() set up on the stack.
    "mov x2, sp",
    // As in the original x86_64 code, hand-write the call operation so that it
    // doesn't push an entry into the CPU's return prediction stack.
    "adr lr, 0f",
    "ldr x3, [x1, #8]",
    "br x3",
    // AArch64 Mach-O does not have a way of representing relocations on an ADR
    // instruction so we have to use a local label that the assembler can fully
    // resolve ahead of time.
    "0:",
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // This BRK is necessary because of our use of .cfi_signal_frame earlier.
    "brk #0",
    ".cfi_endproc",
    asm_function_end!("stack_init_trampoline"),
);

global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 4",
    asm_function_begin!("stack_call_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - SP points to the top of the parent stack.
    // - X29 holds its value from the parent context.
    // - X2 is the function that should be called.
    // - X1 points to the top of our stack.
    // - X0 contains the argument to be passed to the function.
    //
    // Create a stack frame and point the frame pointer at it.
    "stp x29, x30, [sp, #-16]!",
    "mov x29, sp",
    ".cfi_def_cfa x29, 16",
    ".cfi_offset x30, -8",
    ".cfi_offset x29, -16",
    // Switch to the new stack.
    "mov sp, x1",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "blr x2",
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    "mov sp, x29",
    "ldp x29, x30, [sp], #16",
    "ret",
    ".cfi_endproc",
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

    // Initial function.
    push(&mut sp, Some(func as StackWord));

    // Placeholder for parent link.
    push(&mut sp, None);

    // Allocate space on the stack for the initial object, rounding to
    // STACK_ALIGNMENT.
    allocate_obj_on_stack(&mut sp, 16, obj);

    // The stack is aligned to STACK_ALIGNMENT at this point.
    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Padding so the final stack pointer value is properly aligned.
    push(&mut sp, None);

    // Entry point called by switch_and_link().
    push(&mut sp, Some(stack_init_trampoline as StackWord));

    // Add a 16-byte offset because switch_and_link() looks for the target PC
    // 16 bytes above the stack pointer.
    push(&mut sp, None);
    push(&mut sp, None);

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
        // DW_CFA_GNU_args_size 0
        //
        // Indicate to the unwinder that this "call" does not take any arguments
        // and no stack space needs to be popped before executing a landing pad.
        // This is mainly here to undo the effect of any previous
        // DW_CFA_GNU_args_size that may have been set in the current function.
        cfi_reset_args_size_root!(),

        // Read the saved PC from the coroutine stack and call it.
        "ldr x3, [x2, #16]",
        "blr x3",

        // Upon returning, our register state contains the following:
        // - X2: Our stack pointer.
        // - X1: The top of the coroutine stack, or 0 if coming from
        //       switch_and_reset.
        // - X0: The argument passed from the coroutine.

        // Switch back to our stack and free the saved registers.
        "add sp, x2, #32",

        // Pass the argument in X0.
        inlateout("x0") arg => ret_val,

        // We get the coroutine stack pointer back in X1.
        lateout("x1") ret_sp,

        // We pass the stack base in X1.
        in("x1") stack_base.get() as u64,

        // We pass the target stack pointer in X3.
        in("x2") sp.get() as u64,

        // Mark all registers as clobbered. The clobber_abi() will automatically
        // mark X18 as clobbered if it is not reserved by the platform.
        lateout("x20") _, lateout("x21") _, lateout("x22") _, lateout("x23") _,
        lateout("x24") _, lateout("x25") _, lateout("x26") _, lateout("x27") _,
        lateout("x28") _,
        clobber_abi("C"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind_yield!(
        // Save X19 and X29 while also reserving space on the stack for our
        // saved PC.
        "stp x19, x29, [sp, #-32]!",

        // Write our return address to its expected position on the stack.
        "adr lr, 0f",
        "str lr, [sp, #16]",

        // Get the parent stack pointer from the parent link.
        "ldr x2, [x2]",

        // Save our stack pointer to X1.
        "mov x1, sp",

        // Restore X19, X29 and LR from the parent stack.
        "ldr x19, [x2, #16]",
        "ldp x29, lr, [x2]",

        // DW_CFA_GNU_args_size 0
        //
        // Indicate to the unwinder that this "call" does not take any arguments
        // and no stack space needs to be popped before executing a landing pad.
        // This is mainly here to undo the effect of any previous
        // DW_CFA_GNU_args_size that may have been set in the current function.
        //
        // This is needed here even though we don't call anything because
        // switch_and_throw may inject a call which returns to this point.
        cfi_reset_args_size_yield!(),

        // Return into the parent context
        "ret",

        // This gets called by switch_and_link(). At this point our register
        // state contains the following:
        // - SP points to the top of the parent stack.
        // - LR contains the return address in the parent context.
        // - X19 and X29 contain their value from the parent context.
        // - X2 points to the top of the coroutine stack.
        // - X1 points to the base of our stack.
        // - X0 contains the argument passed from switch_and_link.
        "0:",

        // Push the X19, X29 and PC values of the parent context onto the parent
        // stack.
        "stp x29, lr, [sp, #-32]!",
        "str x19, [sp, #16]",

        // Write the parent stack pointer to the parent link.
        "mov x3, sp",
        "str x3, [x1, #-16]",

        // Load our X19 and X29 values from the coroutine stack.
        "ldp x19, x29, [x2]",

        // Switch to the coroutine stack while popping the saved registers and
        // padding.
        "add sp, x2, #32",

        // Pass the argument in X0.
        inlateout("x0") arg => ret_val,

        // The parent link can be in any register, X2 is arbitrarily chosen
        // here.
        in("x2") parent_link as u64,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("x20") _, lateout("x21") _, lateout("x22") _, lateout("x23") _,
        lateout("x24") _, lateout("x25") _, lateout("x26") _, lateout("x27") _,
        lateout("x28") _,
        clobber_abi("C"),
    );

    ret_val
}

#[inline(always)]
pub unsafe fn switch_and_reset(arg: EncodedValue, parent_link: *mut StackPointer) -> ! {
    // Most of this code is identical to switch_yield(), refer to the
    // comments there. Only the differences are commented.
    asm!(
        // Load the parent context's stack pointer.
        "ldr x2, [{parent_link}]",

        // Restore X19, X29 and LR from the parent stack.
        "ldr x19, [x2, #16]",
        "ldp x29, lr, [x2]",

        // Return into the parent context
        "ret",

        parent_link = in(reg) parent_link as u64,

        in("x0") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("x1") 0,

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
    extern "C-unwind" fn throw(forced_unwind: crate::unwind::ForcedUnwind) -> ! {
        extern crate std;
        use std::boxed::Box;
        std::panic::resume_unwind(Box::new(forced_unwind));
    }

    let (ret_val, ret_sp);

    asm_may_unwind_root!(
        // Set up a return address.
        "adr lr, 0f",

        // Save the registers of the parent context.
        "stp x29, lr, [sp, #-32]!",
        "str x19, [sp, #16]",

        // Update the parent link near the base of the coroutine stack.
        "mov x3, sp",
        "str x3, [x1, #-16]",

        // Load the coroutine registers, with the saved PC into LR.
        "ldr lr, [x2, #16]",
        "ldp x19, x29, [x2]",

        // Switch to the coroutine stack while popping the saved registers and
        // padding.
        "add sp, x2, #32",

        // DW_CFA_GNU_args_size 0
        //
        // Indicate to the unwinder that this "call" does not take any arguments
        // and no stack space needs to be popped before executing a landing pad.
        // This is mainly here to undo the effect of any previous
        // DW_CFA_GNU_args_size that may have been set in the current function.
        cfi_reset_args_size_root!(),

        // Simulate a call with an artificial return address so that the throw
        // function will unwind straight into the switch_and_yield() call with
        // the register state expected outside the asm! block.
        "b {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // Switch back to our stack and free the saved registers.
        "add sp, x2, #32",

        // Helper function to trigger stack unwinding.
        throw = sym throw,

        // Argument to pass to the throw function.
        in("x0") forced_unwind.0.get(),

        // Same output registers as switch_and_link().
        lateout("x0") ret_val,
        lateout("x1") ret_sp,

        // Stack pointer and stack base inputs for stack switching.
        in("x1") stack_base.get() as u64,
        in("x2") sp.get() as u64,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("x20") _, lateout("x21") _, lateout("x22") _, lateout("x23") _,
        lateout("x24") _, lateout("x25") _, lateout("x26") _, lateout("x27") _,
        lateout("x28") _,
        clobber_abi("C"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline]
pub unsafe fn drop_initial_obj(
    _stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(32);
    drop_fn(ptr);
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
/// # let regs = TrapHandlerRegs { pc: 0, sp: 0, x0: 0, x1: 0, x29: 0, lr: 0 };
/// let TrapHandlerRegs { pc, sp, x0, x1, x29, lr } = regs;
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct TrapHandlerRegs {
    pub pc: u64,
    pub sp: u64,
    pub x0: u64,
    pub x1: u64,
    pub x29: u64,
    pub lr: u64,
}

pub unsafe fn setup_trap_trampoline<T>(
    stack_base: StackPointer,
    val: T,
    handler: TrapHandler<T>,
) -> TrapHandlerRegs {
    // Preserve the top 16 bytes of the stack since they contain the parent
    // link.
    let parent_link = stack_base.get() - 16;

    // Everything below this can be overwritten. Write the object to the stack.
    let mut sp = parent_link;
    allocate_obj_on_stack(&mut sp, 16, val);
    let val_ptr = sp;

    // Set up registers for entry into the function.
    TrapHandlerRegs {
        pc: handler as u64,
        sp: sp as u64,
        x0: val_ptr as u64,
        x1: parent_link as u64,
        x29: parent_link as u64,
        lr: stack_init_trampoline_return as u64,
    }
}

/// This function executes a function on the given stack. The argument is passed
/// through to the called function.
#[inline]
pub unsafe fn on_stack(arg: *mut u8, stack: impl Stack, f: StackCallFunc) {
    // This is a bit subtle: because we use .cfi_signal_frame in the trampoline,
    // the unwinder will look for unwinding information at the instruction
    // after the return address. Normal compiler code generation does not
    // expect this and may generate incorrect entries in the exception handling
    // table. We work around this by adding a NOP instruction after the call.
    asm_may_unwind_root!(
        // DW_CFA_GNU_args_size 0
        cfi_reset_args_size_root!(),
        concat!("bl ", asm_mangle!("stack_call_trampoline")),
        "nop",
        in("x0") arg,
        in("x1") stack.base().get(),
        in("x2") f,
        clobber_abi("C"),
    );
}
