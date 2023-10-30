//! Support for 32-bit x86 Windows.
//!
//! This file is heavily based on the x86_64 Windows implementation.
//! Relevant differences are highlighted in comments, but otherwise most
//! comments have been removed to avoid duplication. Refer to x86_64_windows.rs
//! for detailed comments about what is happening in this file.
//!
//! ## Backtraces
//!
//! x86 Windows doesn't have a reliable mechanism for unwinding the stack to
//! produce a backtrace. Backtraces are produced by walking through a linked
//! list of frames starting at the EBP registers. Function prologues will set
//! up EBP to point to a pair of stack entries containing the previous EBP value
//! followed by the return address of the function.
//!
//! However this doesn't work if the Frame Pointer Omission optimization is
//! used: this allows the compiler to use EBP as an additional register at the
//! cost of no longer having a valid frame chain.
//!
//! The Windows unwinder is therefore very conservative when unwinding through
//! the frame pointer chain: it only accepts entries if they point within the
//! stack bounds. Unfortunately for us this means that cross-stack backtraces
//! are not possible.
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
//! | FinalExceptionHandler    |  <- Top-level SEH exception handler for this
//! +--------------------------+   | stack. SEHOP requires that this points to
//! | 0xffffffff               |  <- FinalExceptionHandler in ntdll.dll.
//! +--------------------------+
//! |                          |
//! ~     ...                  ~
//! |                          |
//! +--------------------------+
//! | TEB.GuaranteedStackBytes |  <-
//! +--------------------------+   |
//! | TEB.DeallocationStack    |  <- These are fields in the Thread Environement
//! +--------------------------+   | Block (TEB) which need to be saved and
//! | TEB.StackLimit           |  <- restored when switching stacks. The OS
//! +--------------------------+   | looks at these fields for various stack
//! | TEB.StackBase            |  <- related operations.
//! +--------------------------+   |
//! | TEB.ExceptionList        |  <-
//! +--------------------------+
//! | Saved EBP                |
//! +--------------------------+
//! | Saved ESI                |
//! +--------------------------+
//! | Saved EIP                |
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
//! | TEB.GuaranteedStackBytes |
//! +--------------------------+
//! | TEB.DeallocationStack    |
//! +--------------------------+
//! | TEB.StackLimit           |
//! +--------------------------+
//! | TEB.StackBase            |
//! +--------------------------+
//! | TEB.ExceptionList        |
//! +--------------------------+
//! | Saved ESI                |
//! +--------------------------+
//! | Saved EIP                |
//! +--------------------------+
//! | Saved EBP                |
//! +--------------------------+
//! ```
//!
//! And finally, this is the stack layout of a coroutine that has just been
//! initialized:
//!
//! ```text
//! +--------------------------+  <- Stack base
//! | Unused                   |  <- This space is used for returning
//! +--------------------------+   | TEB.StackLimit and
//! | Initial func             |  <- TEB.GuaranteedStackBytes on exit.
//! +--------------------------+
//! | Fake return address      |  <- 32-bit Windows doesn't have a reliable way
//! +--------------------------+     of unwinding the stack so we need to ensure
//! | Parent link              |     that we place a valid return address in
//! +--------------------------+     the frame pointer chain.
//! | FinalExceptionHandler    |
//! +--------------------------+
//! | 0xffffffff               |
//! +--------------------------+
//! |                          |
//! ~ Initial obj              ~
//! |                          |
//! +--------------------------+
//! | Address of initial obj   |  <- This is the 3rd argument of the initial
//! +--------------------------+     function, set up in advance.
//! | TEB.GuaranteedStackBytes |  <-
//! +--------------------------+   |
//! | TEB.DeallocationStack    |  <-
//! +--------------------------+   | Initial values for the TEB fields on the
//! | TEB.StackLimit           |  <- new stack.
//! +--------------------------+   |
//! | TEB.StackBase            |  <-
//! +--------------------------+   |
//! | TEB.ExceptionList        |  <-
//! +--------------------------+
//! | Initial EIP              |  <- Points to stack_init_trampoline
//! +--------------------------+
//! ```

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};

use super::{allocate_obj_on_stack, push};
use crate::stack::{Stack, StackPointer, StackTebFields};
use crate::unwind::{
    asm_may_unwind_root, asm_may_unwind_yield, cfi_reset_args_size_yield, InitialFunc,
    StackCallFunc, TrapHandler,
};
use crate::util::EncodedValue;

pub const STACK_ALIGNMENT: usize = 4;
pub const PARENT_LINK_OFFSET: usize = 0;
pub type StackWord = u32;

// On MinGW, we emit DWARF CFI information so that it can be used by GDB to
// reconstruct a backtrace. This isn't used for runtime unwinding (since we
// don't unwind through stack_init_trampoline on Windows) or backtrace
// generation (which uses the Windows StackWalk API).
cfg_if::cfg_if! {
    if #[cfg(target_env = "gnu")] {
        macro_rules! dwarf {
            ($asm:expr) => { $asm }
        }
    } else {
        macro_rules! dwarf {
            ($asm:expr) => { "" }
        }
    }
}

global_asm!(
    ".balign 16",
    asm_function_begin!("stack_init_trampoline"),
    dwarf!(".cfi_startproc"),
    dwarf!(".cfi_signal_frame"),
    // At this point our register state contains the following:
    // - ESP points to the top of the parent stack.
    // - EBP holds its value from the parent context.
    // - EAX points to the top of our stack.
    // - EDX points to the base of our stack.
    // - ECX contains the argument passed from switch_and_link.
    //
    // Save EBP from the parent context last to create a valid frame record.
    "push ebp",
    "mov ebp, esp",
    // Fill in the parent link.
    "mov [edx - 16], esp",
    // Adjust EDX to point to the parent link for the second argument.
    "sub edx, 16",
    // Switch to the coroutine stack, skipping the address of
    // stack_init_trampoline() at the top of the stack.
    "lea esp, [eax + 4]",
    // Pop the TEB fields for our new stack.
    "pop dword ptr fs:[0x0]",   // ExceptionList
    "pop dword ptr fs:[0x4]",   // StackBase
    "pop dword ptr fs:[0x8]",   // StackLimit
    "pop dword ptr fs:[0xe0c]", // DeallocationStack
    "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    "mov ebp, edx",
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x75 0x00: DW_OP_breg5 (ebp + 0)
    // 0x06: DW_OP_deref
    // 0x23, 0x20: DW_OP_plus_uconst 32
    dwarf!(".cfi_escape 0x0f, 5, 0x75, 0x00, 0x06, 0x23, 0x20"),
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    dwarf!(".cfi_offset esi, -24"),
    dwarf!(".cfi_offset eip, -28"),
    dwarf!(".cfi_offset ebp, -32"),
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
    "jmp [edx + 8]",
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // Although an INT3 isn't required here because the 32-bit SEH unwinder
    // doesn't look at the return address, we add it just in case to avoid
    // confusing debuggers.
    "int3",
    dwarf!(".cfi_endproc"),
    asm_function_end!("stack_init_trampoline"),
);

global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 16",
    asm_function_begin!("stack_call_trampoline"),
    dwarf!(".cfi_startproc"),
    dwarf!(".cfi_signal_frame"),
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
    dwarf!(".cfi_def_cfa ebp, 8"),
    dwarf!(".cfi_offset ebp, -8"),
    // Save the TEB fields to the stack.
    "push dword ptr fs:[0xf78]", // GuaranteedStackBytes
    "push dword ptr fs:[0xe0c]", // DeallocationStack
    "push dword ptr fs:[0x8]",   // StackLimit
    "push dword ptr fs:[0x4]",   // StackBase
    "push dword ptr fs:[0x0]",   // ExceptionList
    // Switch to the new stack.
    "mov esp, edx",
    // Pop the TEB fields for our new stack.
    "pop dword ptr fs:[0x4]",   // StackBase
    "pop dword ptr fs:[0x8]",   // StackLimit
    "pop dword ptr fs:[0xe0c]", // DeallocationStack
    "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes
    // At this point ESP points to the initial SEH exception handler record.
    // Set it up in the TEB.
    "mov fs:[0x0], esp",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "call eax",
    // Save the two mutable TEB fields to the stack base so they can be
    // retrieved later.
    "push dword ptr fs:[0x8]",   // StackLimit
    "push dword ptr fs:[0xf78]", // GuaranteedStackBytes
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    "lea esp, [ebp - 20]",
    "pop dword ptr fs:[0x0]",   // ExceptionList
    "pop dword ptr fs:[0x4]",   // StackBase
    "pop dword ptr fs:[0x8]",   // StackLimit
    "pop dword ptr fs:[0xe0c]", // DeallocationStack
    "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes
    "pop ebp",
    "ret",
    dwarf!(".cfi_endproc"),
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

/// The end of the exception handler chain is marked with 0xffffffff.
const EXCEPTION_LIST_END: usize = !0;

/// Returns the address of FinalExceptionHandler in ntdll.dll. This is the last
/// handler in the linked list of exception handlers on the stack.
///
/// This is needed to support SEHOP (Structured Exception Handling Overwrite
/// Protection), an exploit mitigation technique that is sometimes activated by
/// default. Essentially, RaiseException will verify that the current chain of
/// exception handlers ends with the FinalExceptionHandler and termiante the
/// process if that is not the case. For more details see:
/// https://msrc-blog.microsoft.com/2009/02/02/preventing-the-exploitation-of-structured-exception-handler-seh-overwrites-with-sehop/
///
/// We use this to create an exception handler at the root of a coroutine which
/// points to FinalExceptionHandler. This handler is never called for Rust
/// panics since we catch those and pass them through manually.
fn ntdll_final_exception_handler() -> usize {
    // Use a cached value if possible.
    static CACHED: AtomicUsize = AtomicUsize::new(0);
    let cached = CACHED.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }

    // FinalExceptionHandler isn't part of the public API of ntdll.dll, so we
    // instead need to recover it by scanning through the linked list of SEH
    // exception handlers until we find the one at the root of our stack.
    //
    // We fetch the initial exception registration record from the TIB.
    let mut exception_list: *mut [usize; 2];
    unsafe {
        asm!("mov {}, fs:[0x0]", out(reg) exception_list, options(nostack, readonly, preserves_flags));
    }
    let mut exception_handler = 0;

    while exception_list as usize != EXCEPTION_LIST_END {
        let record = unsafe { *exception_list };
        exception_list = record[0] as *mut _;
        exception_handler = record[1];
    }

    // We should always find at least one SEH handler in the list.
    if exception_handler == 0 {
        panic!("FinalExceptionHandler not found");
    }
    exception_handler
}

#[inline]
pub unsafe fn init_stack<T>(stack: &impl Stack, func: InitialFunc<T>, obj: T) -> StackPointer {
    let mut sp = stack.base().get();

    // Placeholder for returning TEB.StackLimit.
    push(&mut sp, None);

    // Initial function.
    push(&mut sp, Some(func as StackWord));

    // Set up a proper "fake" return address for the frame pointer chain. We
    // just point it back at stack_init_trampoline_return to ensure that the
    // unwinder doesn't try to look for arbitrary unwind information from
    // somewhere else.
    //
    // This is only needed on x86 Windows because this is the only platform
    // where the stack walker will alternate between looking at the frame
    // pointer chain and looking up FPO unwinding information in a PDB.
    push(&mut sp, Some(stack_init_trampoline_return as StackWord));

    // Placeholder for parent link.
    push(&mut sp, None);

    // Set up the initial SEH exception handler record. This must point to
    // FinalExceptionHandler in ntdll.dll for compatibility with SEHOP.
    push(&mut sp, Some(ntdll_final_exception_handler() as StackWord));
    push(&mut sp, Some(EXCEPTION_LIST_END as StackWord));
    let initial_seh_handler = sp;

    // Allocate space on the stack for the initial object, rounding to
    // STACK_ALIGNMENT.
    allocate_obj_on_stack(&mut sp, 24, obj);
    let initial_obj = sp;

    // Set up the stack for the 3rd argument to the initial function.
    push(&mut sp, Some(initial_obj as StackWord));

    // Write the TEB fields for the target stack.
    let teb = stack.teb_fields();
    push(&mut sp, Some(teb.GuaranteedStackBytes as StackWord));
    push(&mut sp, Some(teb.DeallocationStack as StackWord));
    push(&mut sp, Some(teb.StackLimit as StackWord));
    push(&mut sp, Some(teb.StackBase as StackWord));
    push(&mut sp, Some(initial_seh_handler as StackWord));

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
        // Save the TEB fields to the stack.
        "push dword ptr fs:[0xf78]", // GuaranteedStackBytes
        "push dword ptr fs:[0xe0c]", // DeallocationStack
        "push dword ptr fs:[0x8]", // StackLimit
        "push dword ptr fs:[0x4]", // StackBase
        "push dword ptr fs:[0x0]", // ExceptionList

        "push esi",

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
        "0:",

        "pop esi",

        // Restore the TEB fields.
        "pop dword ptr fs:[0x0]", // ExceptionList
        "pop dword ptr fs:[0x4]", // StackBase
        "pop dword ptr fs:[0x8]", // StackLimit
        "pop dword ptr fs:[0xe0c]", // DeallocationStack
        "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes

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
        // Save the TEB fields to the stack.
        "push dword ptr fs:[0xf78]", // GuaranteedStackBytes
        "push dword ptr fs:[0xe0c]", // DeallocationStack
        "push dword ptr fs:[0x8]", // StackLimit
        "push dword ptr fs:[0x4]", // StackBase
        "push dword ptr fs:[0x0]", // ExceptionList

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
        "mov [edx - 16], esp",

        // Switch back to our stack, skipping the return address.
        "lea esp, [eax + 4]",

        // Restore EBP and EBX.
        "pop esi",
        "pop ebp",

        // Restore the TEB fields.
        "pop dword ptr fs:[0x0]", // ExceptionList
        "pop dword ptr fs:[0x4]", // StackBase
        "pop dword ptr fs:[0x8]", // StackLimit
        "pop dword ptr fs:[0xe0c]", // DeallocationStack
        "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes

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
        // Write the 2 TEB fields which can change during corountine execution
        // to the base of the stack. This is later recovered by
        // update_teb_from_stack().
        "mov eax, fs:[0x8]", // StackLimit
        "mov [edi + 12], eax",
        "mov eax, fs:[0xf78]", // GuaranteedStackBytes
        "mov [edi + 8], eax",

        // Load the parent context's stack pointer.
        "mov esp, [edi]",

        // Restore the parent context's RBP.
        "pop ebp",

        // Return into the parent context.
        "ret",

        in("edi") parent_link,
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
        "mov ebx, FS:[0x18]",
        // Save state just like the first half of switch_and_link().
        "push dword ptr fs:[0xf78]", // GuaranteedStackBytes
        "push dword ptr fs:[0xe0c]", // DeallocationStack
        "push dword ptr fs:[0x8]", // StackLimit
        "push dword ptr fs:[0x4]", // StackBase
        "push dword ptr fs:[0x0]", // ExceptionList
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
        "mov [edx - 16], esp",

        // Switch to the coroutine stack.
        "mov esp, eax",

        // Pop the return address of the target context.
        "pop eax",

        // Restore EBP and ESI from the target context.
        "pop esi",
        "pop ebp",

        // Restore the TEB fields of the target context.
        "pop dword ptr fs:[0x0]", // ExceptionList
        "pop dword ptr fs:[0x4]", // StackBase
        "pop dword ptr fs:[0x8]", // StackLimit
        "pop dword ptr fs:[0xe0c]", // DeallocationStack
        "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes

        // Simulate a call with an artificial return address so that the throw
        // function will unwind straight into the switch_and_yield() call with
        // the register state expected outside the asm! block.
        "push eax",
        "jmp {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // This is copied from the second half of switch_and_link().
        "pop esi",
        "pop dword ptr fs:[0x0]", // ExceptionList
        "pop dword ptr fs:[0x4]", // StackBase
        "pop dword ptr fs:[0x8]", // StackLimit
        "pop dword ptr fs:[0xe0c]", // DeallocationStack
        "pop dword ptr fs:[0xf78]", // GuaranteedStackBytes

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

#[inline]
pub unsafe fn drop_initial_obj(
    stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(28);
    drop_fn(ptr);

    // Also copy the TEB fields to the base of the stack so that they can be
    // retreived by update_stack_teb_fields().
    let base = stack_base.get() as *mut StackWord;
    let stack = stack_ptr.get() as *const StackWord;
    *base.sub(1) = *stack.add(3); // StackLimit
    *base.sub(2) = *stack.add(5); // GuaranteedStackBytes
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
    // Preserve the top 24 bytes of the stack since they contain:
    // - Space for returning the TEB fields, filled by switch_and_reset().
    // - The parent link.
    // - The initial SEH exception handler record.
    let parent_link = stack_base.get() - 16;

    // Everything below this can be overwritten. Write the object to the stack.
    let mut sp = parent_link - 8;
    allocate_obj_on_stack(&mut sp, 24, val);
    let val_ptr = sp;

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
pub unsafe fn on_stack<S: Stack>(arg: *mut u8, stack: S, f: StackCallFunc) {
    let stack = scopeguard::guard(stack, |mut stack| {
        let base = stack.base().get() as *const usize;
        let stack_limit = *base.sub(4);
        let guaranteed_stack_bytes = *base.sub(5);
        stack.update_teb_fields(stack_limit, guaranteed_stack_bytes);
    });

    // Set up the values on the new stack.
    let mut sp = stack.base().get();

    // Add 4 bytes of padding for SAFESEH which requires that the exception
    // handler record does not extend all the way to the stack base.
    push(&mut sp, None);

    // Set up the initial SEH exception handler record. This must point to
    // FinalExceptionHandler in ntdll.dll for compatibility with SEHOP.
    push(&mut sp, Some(ntdll_final_exception_handler() as StackWord));
    push(&mut sp, Some(EXCEPTION_LIST_END as StackWord));

    // Write the TEB fields for the target stack.
    let teb = stack.teb_fields();
    push(&mut sp, Some(teb.GuaranteedStackBytes as StackWord));
    push(&mut sp, Some(teb.DeallocationStack as StackWord));
    push(&mut sp, Some(teb.StackLimit as StackWord));
    push(&mut sp, Some(teb.StackBase as StackWord));

    asm_may_unwind_root!(
        concat!("call ", asm_mangle!("stack_call_trampoline")),
        in("ecx") arg,
        in("edx") sp,
        in("eax") f,
        clobber_abi("fastcall"),
    );
}

/// The trap handler will have reset our stack offset back to the base of the
/// coroutine stack, but it won't have reset the SEH ExceptionList chain in the
/// TIB. We need to manually reset it here before executing any user code which
/// might raise an exception.
pub unsafe fn reset_seh_handler(parent_link: *mut StackPointer) {
    // The initial exception record is conveniently located just below the
    // parent link.
    let exception_record = parent_link as usize - 8;

    // Write to the ExceptionList field in the TIB, just like on entry to the
    // coroutine.
    asm!("mov fs:[0x0], {}", in(reg) exception_record, options(nostack, preserves_flags));
}
