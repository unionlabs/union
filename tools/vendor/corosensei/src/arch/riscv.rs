//! Low-level RISC-V support.
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
//! | Saved S1     |
//! +--------------+
//! | Saved S0     |
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
//! | Saved S1  |
//! +-----------+  <- The parent link points to here instead of pointing to the
//! | Saved PC  |     top of the stack. This matches the GCC/LLVM behavior of
//! +-----------+     having the frame pointer point to the address above the
//! | Saved S0  |     saved RA/FP.
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

// Helper macros to write assembly code that works on both RV32 and RV64.
cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        macro_rules! x {
            ($val32:expr, $val64:expr) => {
                $val64
            };
        }
    } else if #[cfg(target_pointer_width = "32")] {
        macro_rules! x {
            ($val32:expr, $val64:expr) => {
                $val32
            };
        }
    }
}
macro_rules! xlen_bytes {
    () => {
        x!("4", "8")
    };
    ($word_offset:expr) => {
        concat!("((", stringify!($word_offset), ") * ", xlen_bytes!(), ")")
    };
}
macro_rules! l {
    ($val:expr, $word_offset:expr, $addr:expr) => {
        concat!(
            x!("lw", "ld"),
            " ",
            $val,
            ", ",
            xlen_bytes!($word_offset),
            "(",
            $addr,
            ")"
        )
    };
}
macro_rules! s {
    ($val:expr, $word_offset:expr, $addr:expr) => {
        concat!(
            x!("sw", "sd"),
            " ",
            $val,
            ", ",
            xlen_bytes!($word_offset),
            "(",
            $addr,
            ")"
        )
    };
}
macro_rules! addi {
    ($dest:expr, $src:expr, $word_offset:expr) => {
        concat!("addi ", $dest, ", ", $src, ", ", xlen_bytes!($word_offset),)
    };
}

pub const STACK_ALIGNMENT: usize = 16;
pub const PARENT_LINK_OFFSET: usize = x!(8, 16);
pub type StackWord = usize;

global_asm!(
    ".balign 4",
    asm_function_begin!("stack_init_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - SP points to the top of the parent stack.
    // - RA contains the return address in the parent context.
    // - S0 and S1 contain their value from the parent context.
    // - A2 points to the top of our stack.
    // - A1 points to the base of our stack.
    // - A0 contains the argument passed from switch_and_link.
    //
    // Push the S0, S1 and PC values of the parent context onto the parent
    // stack.
    addi!("sp", "sp", -4),
    s!("s1", 2, "sp"),
    s!("ra", 1, "sp"),
    s!("s0", 0, "sp"),
    // Write the parent stack pointer to the parent link. This is adjusted to
    // point just above the saved PC/RA to match the GCC/LLVM ABI.
    addi!("t0", "sp", 2),
    s!("t0", -2, "a1"),
    // Set up the frame pointer to point at the stack base. This is needed for
    // the unwinding code below.
    "mv s0, a1",
    // Adjust A1 to point to the parent link.
    addi!("a1", "a1", -2),
    // Pop the padding and initial PC from the coroutine stack. This also sets
    // up the 3rd argument to the initial function to point to the object that
    // init_stack() set up on the stack.
    addi!("a2", "a2", 4),
    // Switch to the coroutine stack.
    "mv sp, a2",
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x78 0x78/0x70: DW_OP_breg8 (s0 - 8/16)
    // 0x06: DW_OP_deref
    // 0x23, 0x08/0x10: DW_OP_plus_uconst 8/16
    concat!(
        ".cfi_escape 0x0f, 5, 0x78, ",
        x!("0x78", "0x70"),
        ", 0x06, 0x23, ",
        xlen_bytes!(2)
    ),
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    concat!(".cfi_offset s1, ", xlen_bytes!(-2)),
    concat!(".cfi_offset ra, ", xlen_bytes!(-3)),
    concat!(".cfi_offset s0, ", xlen_bytes!(-4)),
    // As in the original x86_64 code, hand-write the call operation so that it
    // doesn't push an entry into the CPU's return prediction stack.
    concat!("lla ra, ", asm_mangle!("stack_init_trampoline_return")),
    l!("t0", 1, "a1"),
    "jr t0",
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // This UNIMP is necessary because of our use of .cfi_signal_frame earlier.
    "unimp",
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
    // - S0 holds its value from the parent context.
    // - A2 is the function that should be called.
    // - A1 points to the top of our stack.
    // - A0 contains the argument to be passed to the function.
    //
    // Create a stack frame and point the frame pointer at it. This is a bit
    // tricky because we need to ensure 16-byte stack alignment even on RV32.
    "addi sp, sp, -16",
    ".cfi_def_cfa_offset 16",
    s!("ra", 1, "sp"),
    s!("s0", 0, "sp"),
    addi!("s0", "sp", 2),
    concat!(".cfi_def_cfa s0, ", x!("8", "0")),
    concat!(".cfi_offset ra, ", x!("-12", "-8")),
    concat!(".cfi_offset s0, ", x!("-16", "-16")),
    // Switch to the new stack.
    "mv sp, a1",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "jalr a2",
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    addi!("sp", "s0", -2),
    l!("ra", 1, "sp"),
    l!("s0", 0, "sp"),
    "addi sp, sp, 16",
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
    allocate_obj_on_stack(&mut sp, x!(8, 16), obj);

    // The stack is aligned to STACK_ALIGNMENT at this point.
    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Padding so the final stack pointer value is properly aligned.
    push(&mut sp, None);

    // Entry point called by switch_and_link().
    push(&mut sp, Some(stack_init_trampoline as StackWord));

    // Add a 2-word offset because switch_and_link() looks for the target PC
    // 2 words above the stack pointer.
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
        l!("t0", 2, "a2"),
        "jalr t0",

        // Upon returning, our register state contains the following:
        // - A2: Our stack pointer + 2 words.
        // - A1: The top of the coroutine stack, or 0 if coming from
        //       switch_and_reset.
        // - A0: The argument passed from the coroutine.

        // Switch back to our stack and free the saved registers.
        addi!("sp", "a2", 2),

        // Pass the argument in A0.
        inlateout("a0") arg => ret_val,

        // We get the coroutine stack pointer back in A1.
        lateout("a1") ret_sp,

        // We pass the stack base in A1.
        in("a1") stack_base.get(),

        // We pass the target stack pointer in A2.
        in("a2") sp.get(),

        // Mark all registers as clobbered.
        lateout("s2") _, lateout("s3") _, lateout("s4") _, lateout("s5") _,
        lateout("s6") _, lateout("s7") _, lateout("s8") _, lateout("s9") _,
        lateout("s10") _, lateout("s11") _,
        lateout("fs0") _, lateout("fs1") _, lateout("fs2") _, lateout("fs3") _,
        lateout("fs4") _, lateout("fs5") _, lateout("fs6") _, lateout("fs7") _,
        lateout("fs8") _, lateout("fs9") _, lateout("fs10") _, lateout("fs11") _,
        clobber_abi("C"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind_yield!(
        // Save S0 and S1 while also reserving space on the stack for our
        // saved PC.
        addi!("sp", "sp", -4),
        s!("s0", 0, "sp"),
        s!("s1", 1, "sp"),

        // Write our return address to its expected position on the stack.
        "lla ra, 0f",
        s!("ra", 2, "sp"),

        // Get the parent stack pointer from the parent link.
        l!("a2", 0, "t0"),

        // Save our stack pointer to A1.
        "mv a1, sp",

        // Restore S0, S1 and RA from the parent stack.
        l!("s1", 0, "a2"),
        l!("ra", -1, "a2"),
        l!("s0", -2, "a2"),

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
        // - RA contains the return address in the parent context.
        // - S0 and S1 contain their value from the parent context.
        // - A2 points to the top of our stack.
        // - A1 points to the base of our stack.
        // - A0 contains the argument passed from switch_and_link.
        "0:",

        // Push the S0, S1 and PC values of the parent context onto the parent
        // stack.
        addi!("sp", "sp", -4),
        s!("s1", 2, "sp"),
        s!("ra", 1, "sp"),
        s!("s0", 0, "sp"),

        // Write the parent stack pointer to the parent link. This is adjusted
        // to point just above the saved PC/RA to match the GCC/LLVM ABI.
        addi!("t0", "sp", 2),
        s!("t0", -2, "a1"),

        // Load our S0 and S1 values from the coroutine stack.
        l!("s1", 1, "a2"),
        l!("s0", 0, "a2"),

        // Switch to the coroutine stack while popping the saved registers and
        // padding.
        addi!("sp", "a2", 4),

        // Pass the argument in A0.
        inlateout("a0") arg => ret_val,

        // The parent link can be in any register, T0 is arbitrarily chosen
        // here.
        in("t0") parent_link,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("s2") _, lateout("s3") _, lateout("s4") _, lateout("s5") _,
        lateout("s6") _, lateout("s7") _, lateout("s8") _, lateout("s9") _,
        lateout("s10") _, lateout("s11") _,
        lateout("fs0") _, lateout("fs1") _, lateout("fs2") _, lateout("fs3") _,
        lateout("fs4") _, lateout("fs5") _, lateout("fs6") _, lateout("fs7") _,
        lateout("fs8") _, lateout("fs9") _, lateout("fs10") _, lateout("fs11") _,
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
        l!("a2", 0, "{parent_link}"),

        // Restore S0, S1 and RA from the parent stack.
        l!("s1", 0, "a2"),
        l!("ra", -1, "a2"),
        l!("s0", -2, "a2"),

        // Return into the parent context
        "ret",

        parent_link = in(reg) parent_link,

        in("a0") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("a1") 0,

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
        "lla ra, 0f",

        // Save the registers of the parent context.
        addi!("sp", "sp", -4),
        s!("s1", 2, "sp"),
        s!("ra", 1, "sp"),
        s!("s0", 0, "sp"),

        // Write the parent stack pointer to the parent link. This is adjusted
        // to point just above the saved PC/RA to match the GCC/LLVM ABI.
        addi!("t1", "sp", 2),
        s!("t1", -2, "a1"),

        // Load the coroutine registers, with the saved PC into RA.
        l!("ra", 2, "t0"),
        l!("s1", 1, "t0"),
        l!("s0", 0, "t0"),

        // Switch to the coroutine stack while popping the saved registers and
        // padding.
        addi!("sp", "t0", 4),

        // Simulate a call with an artificial return address so that the throw
        // function will unwind straight into the switch_and_yield() call with
        // the register state expected outside the asm! block.
        "tail {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // Switch back to our stack and free the saved registers.
        addi!("sp", "a2", 2),

        // Helper function to trigger stack unwinding.
        throw = sym throw,

        // Argument to pass to the throw function.
        in("a0") forced_unwind.0.get(),

        // Same output registers as switch_and_link().
        lateout("a0") ret_val,
        lateout("a1") ret_sp,

        // Stack pointer and stack base inputs for stack switching.
        in("a1") stack_base.get(),
        in("t0") sp.get(),

        // See switch_and_link() for an explanation of the clobbers.
        lateout("s2") _, lateout("s3") _, lateout("s4") _, lateout("s5") _,
        lateout("s6") _, lateout("s7") _, lateout("s8") _, lateout("s9") _,
        lateout("s10") _, lateout("s11") _,
        lateout("fs0") _, lateout("fs1") _, lateout("fs2") _, lateout("fs3") _,
        lateout("fs4") _, lateout("fs5") _, lateout("fs6") _, lateout("fs7") _,
        lateout("fs8") _, lateout("fs9") _, lateout("fs10") _, lateout("fs11") _,
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
    let ptr = (stack_ptr.get() as *mut u8).add(x!(16, 32));
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
/// # let regs = TrapHandlerRegs { pc: 0, ra: 0, sp: 0, a0: 0, a1: 0, s0: 0 };
/// let TrapHandlerRegs { pc, ra, sp, a0, a1, s0 } = regs;
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct TrapHandlerRegs {
    pub pc: usize,
    pub ra: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub s0: usize,
}

pub unsafe fn setup_trap_trampoline<T>(
    stack_base: StackPointer,
    val: T,
    handler: TrapHandler<T>,
) -> TrapHandlerRegs {
    // Preserve the top 8/16 bytes of the stack since they contain the parent
    // link.
    let parent_link = stack_base.get() - x!(8, 16);

    // Everything below this can be overwritten. Write the object to the stack.
    let mut sp = parent_link;
    allocate_obj_on_stack(&mut sp, x!(8, 16), val);
    let val_ptr = sp;

    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Set up registers for entry into the function.
    TrapHandlerRegs {
        pc: handler as usize,
        ra: stack_init_trampoline_return as usize,
        sp,
        a0: val_ptr,
        a1: parent_link,
        s0: stack_base.get(),
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
        concat!("call ", asm_mangle!("stack_call_trampoline")),
        "nop",
        in("a0") arg,
        in("a1") stack.base().get(),
        in("a2") f,
        clobber_abi("C"),
    )
}
