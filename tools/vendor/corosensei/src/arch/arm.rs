//! Low-level ARM support.
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
//! | Saved PC     |
//! +--------------+
//! | Saved FP     |  <- This is either R7 or R11 depending on the target.
//! +--------------+
//! | Saved R6     |
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
//! | Saved R6  |
//! +-----------+
//! | Saved PC  |
//! +-----------+
//! | Saved FP  |
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
use crate::unwind::{asm_may_unwind_root, InitialFunc, StackCallFunc, TrapHandler};
use crate::util::EncodedValue;

// The exact set of reserved registers can vary depending on the target and the
// compiler options used (e.g. relocation model) so we probe for this in
// build.rs and then generate an appropriate clobber list here depending on the
// results.
cfg_if::cfg_if! {
    if #[cfg(fp_is_r7)] {
        #[cfg(r9_reserved)]
        macro_rules! asm_clobbers {
            (options($($opt:ident),*), $($asm:tt)*) => {
                asm!(
                    $($asm)*
                    lateout("r11") _,
                    lateout("r4") _, lateout("r5") _, lateout("r8") _, lateout("r10") _,
                    lateout("q4") _, lateout("q5") _, lateout("q6") _, lateout("q7") _,
                    clobber_abi("C"),
                    options($($opt),*)
                )
            };
        }
        #[cfg(not(r9_reserved))]
        macro_rules! asm_clobbers {
            (options($($opt:ident),*), $($asm:tt)*) => {
                asm!(
                    $($asm)*
                    lateout("r11") _, lateout("r9") _,
                    lateout("r4") _, lateout("r5") _, lateout("r8") _, lateout("r10") _,
                    lateout("q4") _, lateout("q5") _, lateout("q6") _, lateout("q7") _,
                    clobber_abi("C"),
                    options($($opt),*)
                )
            };
        }
        macro_rules! fp {
            () => {
                "r7"
            };
        }
        macro_rules! fp_idx {
            () => {
                "7"
            };
        }
    } else {
        #[cfg(r9_reserved)]
        macro_rules! asm_clobbers {
            (options($($opt:ident),*), $($asm:tt)*) => {
                asm!(
                    $($asm)*
                    lateout("r7") _,
                    lateout("r4") _, lateout("r5") _, lateout("r8") _, lateout("r10") _,
                    lateout("q4") _, lateout("q5") _, lateout("q6") _, lateout("q7") _,
                    clobber_abi("C"),
                    options($($opt),*)
                )
            };
        }
        #[cfg(not(r9_reserved))]
        macro_rules! asm_clobbers {
            (options($($opt:ident),*), $($asm:tt)*) => {
                asm!(
                    $($asm)*
                    lateout("r7") _, lateout("r9") _,
                    lateout("r4") _, lateout("r5") _, lateout("r8") _, lateout("r10") _,
                    lateout("q4") _, lateout("q5") _, lateout("q6") _, lateout("q7") _,
                    clobber_abi("C"),
                    options($($opt),*)
                )
            };
        }
        macro_rules! fp {
            () => {
                "r11"
            };
        }
        macro_rules! fp_idx {
            () => {
                "11"
            };
        }
    }
}

// This approximately mirrors the logic in unwind.rs.
cfg_if::cfg_if! {
    if #[cfg(feature = "asm-unwind")] {
        macro_rules! asm_may_unwind {
            ($($asm:tt)*) => {
                asm_clobbers!(
                    options(may_unwind),
                    $($asm)*
                )
            };
        }
    } else {
        macro_rules! asm_may_unwind {
            ($($asm:tt)*) => {
                asm_clobbers!(
                    options(),
                    $($asm)*
                )
            };
        }
    }
}

// Thumb symbols have their lowest bit set, which requires some special
// handling. When a Thumb symbol is defined, we need to prefix it with
// .thumb_func so that the assembler sets the low bit in the symbol table.
//
// When computing the address of a Thumb label using ADR, we need to manually
// set the low bit of the address to ensure we get the correct address.
cfg_if::cfg_if! {
    if #[cfg(all(is_thumb, has_thumb2))] {
        macro_rules! thumb_symbol_def {
            () => {
                ".thumb\n.thumb_func"
            };
        }
        macro_rules! thumb_symbol_adr {
            ($reg:expr, $sym:expr) => {
                concat!("adr ", $reg, ", ", $sym, " + 1")
            };
        }
    } else if #[cfg(is_thumb)] {
        macro_rules! thumb_symbol_def {
            () => {
                ".thumb\n.thumb_func"
            };
        }
        // Thumb1 ADR can only represent addresses that are a multiple of 4.
        macro_rules! thumb_symbol_adr {
            ($reg:expr, $sym:expr) => {
                concat!("adr ", $reg, ", ", $sym, "\n", "adds ", $reg, ", ", $reg, ", #1")
            };
        }
    } else {
        macro_rules! thumb_symbol_def {
            () => {
                ""
            };
        }
        macro_rules! thumb_symbol_adr {
            ($reg:expr, $sym:expr) => {
                concat!("adr ", $reg, ", ", $sym)
            };
        }
    }
}

// Thumb1 support: in some cases we need to use longer instruction sequences
// when Thumb2 is not available. This doesn't apply when using the ARM ISA since
// that supports the full set of instructions.
cfg_if::cfg_if! {
    if #[cfg(any(not(is_thumb), has_thumb2))] {
        macro_rules! thumb1 {
            ($asm:expr) => { "" };
        }
        macro_rules! thumb2 {
            ($asm:expr) => { $asm }
        }
    } else {
        macro_rules! thumb1 {
            ($asm:expr) => { $asm };
        }
        macro_rules! thumb2 {
            ($asm:expr) => { "" };
        }
    }
}

pub const STACK_ALIGNMENT: usize = 8;
pub const PARENT_LINK_OFFSET: usize = 0;
pub type StackWord = u32;

global_asm!(
    ".balign 4",
    thumb_symbol_def!(),
    asm_function_begin!("stack_init_trampoline"),
    // ARM doesn't use DWARF CFI for unwinding, but we should still provide it
    // for use by debugging tools. In this case we only emit DWARF CFI in the
    // .debug_frame section which can be stripped unlike .eh_frame.
    ".cfi_sections .debug_frame",
    // Instead, ARM uses a special unwinding format specified by the Exception
    // Handling ABI (EHABI). We emit this unwinding information in parallel with
    // the DWARF CFI for debuggers. This unwinding information is used by the
    // runtime unwinder to generate backtraces and throw exceptions. For more
    // details see:
    // https://github.com/ARM-software/abi-aa/blob/main/ehabi32/ehabi32.rst
    ".fnstart",
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - SP points to the top of the parent stack.
    // - LR contains the return address in the parent context.
    // - FP contain its value from the parent context.
    // - R2 points to the top of our stack.
    // - R1 points to the base of our stack.
    // - R0 contains the argument passed from switch_and_link.
    //
    // Push the FP and PC values of the parent context to the parent stack.
    concat!("push {{", fp!(), ", lr}}"),
    // Write the parent stack pointer to the parent link and adjust R1 to point
    // to the parent link.
    thumb2!("str sp, [r1, #-8]!"),
    thumb1!("mov r3, sp"),
    thumb1!("subs r1, r1, #8"),
    thumb1!("str r3, [r1]"),
    // Switch to our stack while popping the padding and initial PC. This also
    // sets up r2 for the third argument to the initial function.
    "adds r2, r2, #12",
    "mov sp, r2",
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    concat!("mov ", fp!(), ", r1"),
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x77/0x7b 0x00: DW_OP_breg7/11 (r7/r11 + 0)
    // 0x06: DW_OP_deref
    // 0x23, 0x0c: DW_OP_plus_uconst 12
    concat!(
        ".cfi_escape 0x0f, 5, 0x70 + ",
        fp_idx!(),
        ", 0x00, 0x06, 0x23, 0x0c"
    ),
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    ".cfi_offset r6, -4",
    ".cfi_offset lr, -8",
    concat!(".cfi_offset ", fp!(), ", -12"),
    // For the EHABI unwinder we use a different set of unwinding directives
    // which have the same effect as the DWARF CFI directives. Similar to the
    // SEH opcodes used for x86_64 Windows, these are executed by the unwinder
    // in reverse to undo the effects of the function prologue.
    //
    // When the unwinder reaches this frame, it will have a virtual FP pointing
    // at the parent link. Then, the following EHABI unwinding operations are
    // performed in order:
    // - .movsp fp: This copies the virtual FP to the virtual SP so that all
    //   future stack references by the unwinder are done using that register.
    // - .save {sp}: This is where the magic happens. The parent link
    //   is read off the stack and placed in the virtual SP which now points to
    //   the top of the parent stack.
    // - .save {fp, pc}: This pops and restores the FP and PC from the parent
    //   context.
    // - .save {r6}: This pops and restores R6 from the parent context. This
    //   needs to be a separate operations due to the way r6 is positioned in
    //   the stack frame.
    //
    // We end up with the register state that we had prior to entering the
    // asm!() block in switch_and_link. Unwinding can then proceed on the parent
    // stack normally.
    ".save {{r6}}",
    concat!(".save {{", fp!(), ", pc}}"),
    ".save {{sp}}",
    concat!(".movsp ", fp!()),
    // As in the original x86_64 code, hand-write the call operation so that it
    // doesn't push an entry into the CPU's return prediction stack.
    thumb2!(thumb_symbol_adr!(
        "lr",
        asm_mangle!("stack_init_trampoline_return")
    )),
    thumb2!("ldr pc, [r1, #4]"),
    thumb1!(thumb_symbol_adr!(
        "r3",
        asm_mangle!("stack_init_trampoline_return")
    )),
    thumb1!("mov lr, r3"),
    thumb1!("ldr r3, [r1, #4]"),
    thumb1!("bx r3"),
    // The balign here seems to be necessary otherwise LLVM's assembler
    // generates invalid offsets to the label.
    ".balign 4",
    thumb_symbol_def!(),
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // This UDF is necessary because of our use of .cfi_signal_frame earlier.
    "udf #0",
    ".cfi_endproc",
    ".fnend",
    asm_function_end!("stack_init_trampoline"),
);

global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 4",
    thumb_symbol_def!(),
    asm_function_begin!("stack_call_trampoline"),
    ".cfi_sections .debug_frame",
    ".fnstart",
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - SP points to the top of the parent stack.
    // - FP holds its value from the parent context.
    // - R2 is the function that should be called.
    // - R1 points to the top of our stack.
    // - R0 contains the argument to be passed to the function.
    //
    // Create a stack frame and point the frame pointer at it. We use both
    // DWARF and EHABI CFI directives just like stack_init_trampoline.
    concat!("push {{", fp!(), ", lr}}"),
    concat!("mov ", fp!(), ", sp"),
    concat!(".save {{", fp!(), ", lr}}"),
    concat!(".setfp ", fp!(), ", sp, #0"),
    concat!(".cfi_def_cfa ", fp!(), ", 8"),
    ".cfi_offset lr, -4",
    concat!(".cfi_offset ", fp!(), ", -8"),
    // Switch to the new stack.
    "mov sp, r1",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "blx r2",
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    concat!("mov sp, ", fp!()),
    concat!("pop {{", fp!(), ", pc}}"),
    ".cfi_endproc",
    ".fnend",
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
    allocate_obj_on_stack(&mut sp, 8, obj);

    // The stack is aligned to STACK_ALIGNMENT at this point.
    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Entry point called by switch_and_link().
    push(&mut sp, Some(stack_init_trampoline as StackWord));

    // Add an 8-byte offset because switch_and_link() looks for the target PC
    // 8 bytes above the stack pointer.
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

    asm_may_unwind!(
        // R6 is pushed separately because we want the FP/LR pair to be together
        // at the top of the stack so that it acts a frame record.
        "push {{r6}}",

        // Read the saved PC from the coroutine stack and call it.
        "ldr r3, [r2, #8]",
        "blx r3",

        // Upon returning, our register state contains the following:
        // - SP: Our stack, with the return address and FP popped.
        // - R1: The top of the coroutine stack, or 0 if coming from
        //       switch_and_reset.
        // - R0: The argument passed from the coroutine.

        // Restore R6.
        "pop {{r6}}",

        // Pass the argument in R0.
        inlateout("r0") arg => ret_val,

        // We get the coroutine stack pointer back in R1.
        lateout("r1") ret_sp,

        // We pass the stack base in R1.
        in("r1") stack_base.get(),

        // We pass the target stack pointer in R2.
        in("r2") sp.get(),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind!(
        thumb2!(thumb_symbol_adr!("lr", "0f")),
        thumb1!(thumb_symbol_adr!("r3", "0f")),
        thumb1!("mov lr, r3"),
        concat!("push {{r6, ", fp!(), ", lr}}"),

        // Save our stack pointer to R1.
        "mov r1, sp",

        // Get the parent stack pointer from the parent link.
        thumb2!("ldr sp, [r2]"),
        thumb1!("ldr r3, [r2]"),
        thumb1!("mov sp, r3"),

        // Pop the FP and PC to return into the parent context.
        concat!("pop {{", fp!(), ", pc}}"),

        // At this point our register state contains the following:
        // - SP points to the top of the parent stack.
        // - LR contains the return address in the parent context.
        // - FP contain its value from the parent context.
        // - R2 points to the top of our stack.
        // - R1 points to the base of our stack.
        // - R0 contains the argument passed from switch_and_link.
        //
        // The balign here seems to be necessary otherwise LLVM's assembler
        // generates invalid offsets to the label.
        ".balign 4",
        "0:",

        // Push the FP and PC values of the parent context to the parent stack.
        concat!("push {{", fp!(), ", lr}}"),

        // Write the parent stack pointer to the parent link.
        thumb2!("str sp, [r1, #-8]"),
        thumb1!("subs r1, r1, #8"),
        thumb1!("mov r3, sp"),
        thumb1!("str r3, [r1]"),

        // Switch to our stack
        "mov sp, r2",

        // Pop our R6 and FP values from the stack and implicitly pop the
        // saved PC as well.
        thumb2!(concat!("pop {{r6, ", fp!(), ", lr}}")),
        thumb1!(concat!("pop {{r6, ", fp!(), "}}")),
        thumb1!("add sp, sp, #4"),

        // Pass the argument in R0.
        inlateout("r0") arg => ret_val,

        // The parent link can be in any register, R2 is arbitrarily chosen
        // here.
        in("r2") parent_link,
    );

    ret_val
}

#[inline(always)]
pub unsafe fn switch_and_reset(arg: EncodedValue, parent_link: *mut StackPointer) -> ! {
    // Most of this code is identical to switch_yield(), refer to the
    // comments there. Only the differences are commented.
    asm!(
        // Load the parent context's stack pointer.
        thumb2!("ldr sp, [{parent_link}]"),
        thumb1!("ldr r3, [{parent_link}]"),
        thumb1!("mov sp, r3"),

        // Pop FP and PC off the parent stack and return into the parent
        // context.
        concat!("pop {{", fp!(), ", pc}}"),

        parent_link = in(reg) parent_link,

        in("r0") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("r1") 0,

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

    asm_may_unwind!(
        // Set up a return address.
        thumb2!(thumb_symbol_adr!("lr", "0f")),
        thumb1!(thumb_symbol_adr!("r3", "0f")),
        thumb1!("mov lr, r3"),

        // Save the registers of the parent context.
        "push {{r6}}",
        concat!("push {{", fp!(), ", lr}}"),

        // Update the parent link near the base of the coroutine stack.
        "mov r3, sp",
        "str r3, [r1, #-8]",

        // Switch to the coroutine stack.
        "mov sp, r2",

        // Pop the coroutine registers, with the saved PC into LR.
        thumb2!(concat!("pop {{r6, ", fp!(), ", lr}}")),
        thumb1!(concat!("pop {{r6, ", fp!(), "}}")),
        thumb1!("pop {{r3}}"),
        thumb1!("mov lr, r3"),

        // Simulate a call with an artificial return address so that the throw
        // function will unwind straight into the switch_and_yield() call with
        // the register state expected outside the asm! block.
        "b {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        //
        // The balign here seems to be necessary otherwise LLVM's assembler
        // generates invalid offsets to the label.
        ".balign 4",
        "0:",

        // Restore R6. Note that we need at least one instruction
        // here after the return due to the use of .cfi_signal_frame.
        "pop {{r6}}",

        // Helper function to trigger stack unwinding.
        throw = sym throw,

        // Argument to pass to the throw function.
        in("r0") forced_unwind.0.get(),

        // Same output registers as switch_and_link().
        lateout("r0") ret_val,
        lateout("r1") ret_sp,

        // Stack pointer and stack base inputs for stack switching.
        in("r1") stack_base.get(),
        in("r2") sp.get(),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline]
pub unsafe fn drop_initial_obj(
    _stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(12);
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
/// # let regs = TrapHandlerRegs {
/// #   pc: 0, r0: 0, r1: 0, r7: 0, r11: 0, r13: 0, r14: 0,
/// #   cpsr_thumb: false, cpsr_endian: false
/// # };
/// let TrapHandlerRegs { pc, r0, r1, r7, r11, r13, r14, cpsr_thumb, cpsr_endian } = regs;
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct TrapHandlerRegs {
    pub pc: u32,
    pub r0: u32,
    pub r1: u32,
    pub r7: u32,
    pub r11: u32,
    pub r13: u32,
    pub r14: u32,
    pub cpsr_thumb: bool,
    pub cpsr_endian: bool,
}

pub unsafe fn setup_trap_trampoline<T>(
    stack_base: StackPointer,
    val: T,
    handler: TrapHandler<T>,
) -> TrapHandlerRegs {
    // Preserve the top 8 bytes of the stack since they contain the parent link.
    let parent_link = stack_base.get() - 8;

    // Everything below this can be overwritten. Write the object to the stack.
    let mut sp = parent_link;
    allocate_obj_on_stack(&mut sp, 8, val);
    let val_ptr = sp;

    // Set up registers for entry into the function.
    TrapHandlerRegs {
        pc: handler as u32 & !1,
        r0: val_ptr as u32,
        r1: parent_link as u32,
        r7: parent_link as u32,
        r11: parent_link as u32,
        r13: sp as u32,
        r14: stack_init_trampoline_return as u32,
        cpsr_thumb: handler as u32 & 1 != 0,
        cpsr_endian: cfg!(target_endian = "big"),
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
        concat!("blx ", asm_mangle!("stack_call_trampoline")),
        "nop",
        in("r0") arg,
        in("r1") stack.base().get(),
        in("r2") f,
        clobber_abi("C"),
    )
}
