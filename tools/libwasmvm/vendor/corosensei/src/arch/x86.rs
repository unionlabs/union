//! Low level 32-bit x86 support.
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
//! +--------------|
//! | Parent link  |
//! +--------------+
//! |              |
//! ~     ...      ~
//! |              |
//! +--------------+
//! | Saved EBP    |
//! +--------------+
//! | Saved ESI    |
//! +--------------+
//! | Saved EIP    |
//! +--------------+
//! ```
//!
//! And this is the layout of the parent stack when a coroutine is running:
//!
//! ```text
//! |             |
//! ~     ...     ~
//! |             |
//! +-------------+
//! | Saved ESI   |
//! +-------------+
//! | Saved EIP   |
//! +-------------+
//! | Saved EBP   |
//! +-------------+
//! ```
//!
//! And finally, this is the stack layout of a coroutine that has just been
//! initialized:
//!
//! ```text
//! +------------------------+  <- Stack base
//! | Initial func           |
//! +------------------------+
//! | Parent link            |
//! +------------------------+
//! |                        |
//! ~ Initial obj            ~
//! |                        |
//! +------------------------+
//! | Padding                |  <- This padding is required to ensure the stack
//! +------------------------+   | is properly aligned to 16 bytes in the
//! | Padding                |  <- initial function.
//! +------------------------+   |
//! | Padding                |  <-
//! +------------------------+
//! | Address of initial obj |  <- This is the 3rd argument of the initial
//! +------------------------+     function, set up in advance.
//! | Initial EIP            |
//! +------------------------+  <- Initial stack pointer
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
pub type StackWord = u32;

global_asm!(
    ".balign 16",
    asm_function_begin!("stack_init_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - ESP points to the top of the parent stack.
    // - EBP holds its value from the parent context.
    // - EAX points to the top of our stack.
    // - EDX points to the base of our stack.
    // - ECX contains the argument passed from switch_and_link.
    //
    // Save EBP from the parent context last to create a valid frame record.
    "push ebp",
    // Fill in the parent link.
    "mov [edx - 8], esp",
    // Adjust EDX to point to the parent link for the second argument.
    "sub edx, 8",
    // Switch to the coroutine stack, skipping the address of
    // stack_init_trampoline() at the top of the stack.
    "lea esp, [eax + 4]",
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    "mov ebp, edx",
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x75 0x00: DW_OP_breg5 (ebp + 0)
    // 0x06: DW_OP_deref
    // 0x23, 0x0c: DW_OP_plus_uconst 12
    ".cfi_escape 0x0f, 5, 0x75, 0x00, 0x06, 0x23, 0x0c",
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    ".cfi_offset esi, -4",
    ".cfi_offset eip, -8",
    ".cfi_offset ebp, -12",
    // Calculate the address of stack_init_trampoline_return in a way that works
    // with position-independent code. This is tricky on 32-bit x86 since we
    // cannot access EIP directly. Instead, we use a CALL to the next
    // instruction so that the return address is pushed to the stack. The CPU is
    // smart enough to realize that this is not a real call and doesn't mess up
    // the return prediction stack.
    "call 2f",
    "2:",
    // We now have the address of "2" on the stack. Ideally we would adjust it
    // so that it points to stack_init_trampoline_return but this isn't actually
    // necessary since the initial function never returns. The unwinding
    // information at 2: and stack_init_trampoline_return is identical so we
    // can just leave the return address as it is.
    //
    // As in the original x86_64 code, use a jmp instead of a call so that it
    // doesn't push an entry into the CPU's return prediction stack.
    "jmp [edx + 4]",
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // This INT3 is necessary because of our use of .cfi_signal_frame earlier.
    "int3",
    ".cfi_endproc",
    asm_function_end!("stack_init_trampoline"),
);

global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 16",
    asm_function_begin!("stack_call_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - ESP points to the top of the parent stack.
    // - EBP holds its value from the parent context.
    // - EAX is the function that should be called.
    // - EDX points to the top of our stack.
    // - ECX contains the argument to be passed to the function.
    //
    // Create a stack frame and point the frame pointer at it.
    "push ebp",
    "mov ebp, esp",
    ".cfi_def_cfa ebp, 8",
    ".cfi_offset ebp, -8",
    // Switch to the new stack.
    "mov esp, edx",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "call eax",
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    "mov esp, ebp",
    "pop ebp",
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
    allocate_obj_on_stack(&mut sp, 8, obj);
    let initial_obj = sp;

    // Set up the stack for the 3rd argument to the initial function. This also
    // involves some padding to ensure stack alignment.
    push(&mut sp, None);
    push(&mut sp, None);
    push(&mut sp, None);
    push(&mut sp, Some(initial_obj as StackWord));

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
        // On x86, LLVM uses ESI as a reserved register instead of RBX.
        "push esi",

        // DW_CFA_GNU_args_size 0
        //
        // Indicate to the unwinder that this "call" does not take any arguments
        // and no stack space needs to be popped before executing a landing pad.
        // This is mainly here to undo the effect of any previous
        // DW_CFA_GNU_args_size that may have been set in the current function.
        cfi_reset_args_size_root!(),

        // Push a return address onto our stack and then jump to the return
        // address at the top of the coroutine stack.
        //
        // From here on execution continues in stack_init_trampoline or the 0:
        // label in switch_yield.
        "call [eax]",

        // Upon returning, our register state contains the following:
        // - ESP: Our stack, with the return address and EBP popped.
        // - EDX: The top of the coroutine stack, or 0 if coming from
        //        switch_and_reset.
        // - ECX: The argument passed from the coroutine.

        // Restore ESI.
        "pop esi",

        // Pass the argument in ECX.
        inlateout("ecx") arg => ret_val,

        // We get the coroutine stack pointer back in EDX.
        lateout("edx") ret_sp,

        // We pass the stack base in EDX.
        in("edx") stack_base.get(),

        // We pass the target stack pointer in EAX.
        in("eax") sp.get(),

        // Mark all registers as clobbered.
        lateout("ebx") _, lateout("edi") _,
        clobber_abi("C"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind_yield!(
        // Save EBP and ESI.
        "push ebp",
        "push esi",

        // Calculate the address of stack_init_trampoline_return in a way that
        // works with position-independent code. This is tricky on 32-bit x86
        // since we cannot access EIP directly. Instead, we use a CALL to the
        // next instruction so that the return address is pushed to the stack.
        // The CPU is smart enough to realize that this is not a real call and
        // doesn't mess up the return prediction stack.
        "call 2f",
        "2:",

        // We now have the address of "2" on the stack, we need to adjust it to
        // point to "0". We use an intermediate constant here to work around a
        // limitation of LLVM's Intel syntax parser which doesn't support 2
        // symbols in an expression.
        ".equ .Loffset_yield, 0f - 2b",
        "add dword ptr [esp], offset .Loffset_yield",

        // Save our stack pointer to EDX, which is then returned out of
        // switch_and_link().
        "mov edx, esp",

        // Load the parent context's stack pointer.
        "mov esp, [eax]",

        // Restore the parent's EBP register which is at the top of the stack.
        "pop ebp",

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

        // Return into the parent context. This returns control to
        // switch_and_link() after the call instruction.
        "ret",

        // At this point our register state contains the following:
        // - ESP points to the top of the parent stack.
        // - EBP holds its value from the parent context.
        // - EAX points to the top of our stack.
        // - EDX points to the base of our stack.
        // - ECX contains the argument passed from switch_and_link.
        "0:",

        // Save the EBP of the parent context to the parent stack.
        "push ebp",

        // Update the parent link near the base of the stack.
        "mov [edx - 8], esp",

        // Switch back to our stack, skipping the return address.
        "lea esp, [eax + 4]",

        // Restore EBP and EBX.
        "pop esi",
        "pop ebp",

        // ECX is used by switch_and_link to pass the argument in/out.
        inlateout("ecx") arg => ret_val,

        // The parent link can be in any register, EAX is arbitrarily chosen
        // here.
        in("eax") parent_link,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("ebx") _, lateout("edi") _,
        clobber_abi("C"),
    );

    ret_val
}

#[inline(always)]
pub unsafe fn switch_and_reset(arg: EncodedValue, parent_link: *mut StackPointer) -> ! {
    asm!(
        // Load the parent context's stack pointer.
        "mov esp, [{parent_link}]",

        // Restore the parent context's RBP.
        "pop ebp",

        // Return into the parent context.
        "ret",

        parent_link = in(reg) parent_link,

        in("ecx") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("edx") 0,

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
    extern "fastcall-unwind" fn throw(forced_unwind: crate::unwind::ForcedUnwind) -> ! {
        extern crate std;
        use std::boxed::Box;
        std::panic::resume_unwind(Box::new(forced_unwind));
    }

    let (ret_val, ret_sp);

    asm_may_unwind_root!(
        // Save ESI just like the first half of switch_and_link().
        "push esi",

        // Push a return address to the stack. See switch_and_yield for details
        // about how this code works.
        "call 2f",
        "2:",
        ".equ .Loffset_throw, 0f - 2b",
        "add dword ptr [esp], offset .Loffset_throw",

        // Save EBP of the parent context.
        "push ebp",

        // Update the parent link near the base of the coroutine stack.
        "mov [edx - 8], esp",

        // Switch to the coroutine stack.
        "mov esp, eax",

        // Pop the return address of the target context.
        "pop eax",

        // Restore EBP and ESI from the target context.
        "pop esi",
        "pop ebp",

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
        "push eax",
        "jmp {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // Restore registers just like the second half of switch_and_link.
        "pop esi",

        // Helper function to trigger stack unwinding.
        throw = sym throw,

        // Argument to pass to the throw function.
        in("ecx") forced_unwind.0.get(),

        // Same output registers as switch_and_link().
        lateout("ecx") ret_val,
        lateout("edx") ret_sp,

        // Stack pointer and stack base inputs for stack switching.
        in("edx") stack_base.get(),
        in("eax") sp.get(),

        // See switch_and_link() for an explanation of the clobbers.
        lateout("ebx") _, lateout("edi") _,
        clobber_abi("C"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

/// Drops the initial object on a coroutine that has not started yet.
#[inline]
pub unsafe fn drop_initial_obj(
    _stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(20);
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
/// # let regs = TrapHandlerRegs { eip: 0, esp: 0, ebp: 0, ecx: 0, edx: 0 };
/// let TrapHandlerRegs { eip, esp, ebp, ecx, edx } = regs;
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub struct TrapHandlerRegs {
    pub eip: u32,
    pub esp: u32,
    pub ebp: u32,
    pub ecx: u32,
    pub edx: u32,
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

    debug_assert_eq!(sp % STACK_ALIGNMENT, 0);

    // Set up a return address which returns to stack_init_trampoline.
    push(&mut sp, Some(stack_init_trampoline_return as StackWord));

    // Set up registers for entry into the function.
    TrapHandlerRegs {
        eip: handler as u32,
        esp: sp as u32,
        ecx: val_ptr as u32,
        edx: parent_link as u32,
        ebp: parent_link as u32,
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
        in("ecx") arg,
        in("edx") stack.base().get(),
        in("eax") f,
        clobber_abi("fastcall"),
    )
}
