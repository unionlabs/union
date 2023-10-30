//! This file contains the low level operations that deal with switching between
//! stacks.
//!
//! The core operations are:
//! - `init_stack` to initialize a stack for the first resume.
//! - `switch_and_link` to switch control into a coroutine.
//! - `switch_yield` to return control from a coroutine back to its parent.
//! - `switch_and_reset` to return control for the last time.
//!
//! ## Linked stacks
//!
//! Stack linking allows a context switch to be automatically performed when the
//! initial function of a context returns or unwinds. This works by stashing a
//! copy of the parent context stack pointer near the stack base and updating it
//! every time we switch into the child context using `switch_and_link`.
//!
//! For unwinding and backtraces to work as expected (that is, to continue in
//! the parent after unwinding past the initial function of a child context),
//! we need to use special DWARF CFI instructions to tell the unwinder how to
//! find the parent frame.
//!
//! If you're curious a decent introduction to CFI things and unwinding is at
//! <https://www.imperialviolet.org/2017/01/18/cfi.html>.
//!
//! ## Frame pointers
//!
//! Some tools or OSes do not use DWARF for stack unwinding, prefering to use
//! the older (but simpler) frame pointer chain to capture a backtrace. This is
//! particularly common in performance profiling tools such as Linux's perf
//! callgraph profiler. These work by following a linked list of frame records
//! starting from the RBP register. Each record consists of 2 words: a pointer
//! to the previous frame (aka the previous RBP value) and the return address
//! for this frame (aka the saved RIP value).
//!
//! To support these tools, we also generate a valid stack frame record when
//! switching into a coroutine. This works by treating the parent link at the
//! root of the stack as a frame record which points to the top of the parent
//! stack. The top of the parent stack contains the saved RBP and RIP values in
//! the correct format for a frame record, which allows unwinding to continue on
//! the parent stack.
//!
//! The RIP value associated with the parent link is invalid since it points to
//! the start of the initial function, but this shouldn't block the unwinding
//! process.
//!
//! ## Stack layout
//!
//! Here is what the layout of the stack looks like when a coroutine is
//! suspended.
//!
//! ```text
//! +--------------+  <- Stack base
//! | Initial func |  <- Only used once when resuming for the first time.
//! +--------------+
//! | Parent link  |  <- The Yielder is a pointer to this address. When the
//! +--------------+     coroutine is running, it points to the top of the
//! |              |     parent stack which contains a saved RIP, RBP and RBX
//! ~     ...      ~     just like a suspened coroutine.
//! |              |
//! +--------------+
//! | Saved RBP    |
//! +--------------+
//! | Saved RBX    |
//! +--------------+
//! | Saved RIP    |
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
//! | Saved RBX   |
//! +-------------+
//! | Saved RIP   |  <- These 2 values form a valid entry in the frame pointer
//! +-------------+   | chain. The parent link itself is another entry in the
//! | Saved RBP   |  <- frame pointer chain since RBP points to it.
//! +-------------+  <- Parent link points here.
//! ```
//!
//! And finally, this is the stack layout of a coroutine that has just been
//! initialized:
//!
//! ```text
//! +--------------+  <- Stack base
//! | Initial func |
//! +--------------+
//! | Parent link  |  <- This hasn't been filled in yet at this point.
//! +--------------+
//! |              |
//! ~ Initial obj  ~  <- This has an unknown size, but the size is rounded up to
//! |              |     `STACK_ALIGNMENT`.
//! +--------------+
//! | Initial RIP  |  <- Points to stack_init_trampoline
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

// This is a pretty special function that has no real signature. Its use is to
// be the "base" function of all coroutines. This entrypoint is used in
// init_stack() to bootstrap the execution of a new coroutine.
//
// We also use this function as a persistent frame on the stack to emit dwarf
// information to unwind into the caller. This allows us to unwind from the
// coroutines's stack back to the main stack that the coroutine was called from.
// We use special dwarf directives here to do so since this is a pretty
// nonstandard function.
global_asm!(
    ".balign 16",
    asm_function_begin!("stack_init_trampoline"),
    ".cfi_startproc",
    // GDB has a hard-coded check that rejects backtraces where the frame
    // addresses do not monotonically increase. This can unfortunately trigger
    // when the stack of a coroutine is located at a higher address than its
    // parent stack. See the backwards_stack_address test which specifically
    // triggers this (put a breakpoint on rust_panic).
    //
    // One way to work around this check is to mark this frame as a signal
    // frame. It is normal for signal frames to switch stack because of
    // sigaltstack, so GDB allows this. The only downside is that it makes the
    // frame appear as "<signal handler called>" in the GDB backtrace.
    //
    // Note that as a consequence of this, the unwinder will use the instruction
    // *after* the return address to search for unwind information. To avoid
    // issues, any asm! blocks containing a return address that may be unwound
    // into must not have that address at the end of the asm! block.
    cfi_signal_frame!(),
    // This gets called by switch_and_link() the first time a coroutine is
    // resumed, due to the initial state set up by init_stack().
    //
    // At this point our register state contains the following:
    // - RSP points to the top of the parent stack.
    // - RBP holds its value from the parent context.
    // - RDX points to the top of our stack.
    // - RSI points to the base of our stack.
    // - RDI contains the argument passed from switch_and_link.
    //
    // Save the RBP of the parent context to the parent stack. When combined
    // with the return address this forms a valid frame record (RBP & RIP) in
    // the frame pointer chain.
    "push rbp",
    // Fill in the parent link near the base of the stack. This is updated
    // every time we switch into a coroutine and allows the coroutine to
    // return to our context through the Yielder and when it unwinds.
    "mov [rsi - 16], rsp",
    // On entry RSI will be pointing to the stack base (see switch_and_link). We
    // need to adjust this to point to the parent link instead for the second
    // parameter of the entry function.
    "sub rsi, 16",
    // Switch to the coroutine stack, skipping the address of
    // stack_init_trampoline() at the top of the stack.
    "lea rsp, [rdx + 8]",
    // Set up the frame pointer to point at the parent link. This is needed for
    // the unwinding code below.
    "mov rbp, rsi",
    // This sequence of magic numbers deserves some explanation. We need to tell
    // the unwinder where to find the Canonical Frame Address (CFA) of the
    // parent context.
    //
    // The CFA is normally defined as the stack pointer value in the caller just
    // before executing the call instruction. In our case, this is the stack
    // pointer value that should be restored upon exiting the inline assembly
    // block inside switch_and_link().
    //
    // Once the unwinder reaches this function, it will have a virtual RBP value
    // pointing right at the parent link (see the diagram at the top of this
    // file). We need to use a custom DWARF expression to read this value off
    // the stack, and then add 24 bytes to skip over the 3 saved values on the
    // stack.
    //
    // The actual meanings of the magic bytes are:
    // 0x0f: DW_CFA_def_cfa_expression
    // 5: byte length of the following DWARF expression
    // 0x76 0x00: DW_OP_breg6 (rbp + 0) -- GDB doesn't like DW_OP_reg6
    // 0x06: DW_OP_deref
    // 0x23, 0x18: DW_OP_plus_uconst 24
    ".cfi_escape 0x0f, 5, 0x76, 0x00, 0x06, 0x23, 0x18",
    // Now we can tell the unwinder how to restore the 3 registers that were
    // pushed on the parent stack. These are described as offsets from the CFA
    // that we just calculated.
    ".cfi_offset rbx, -8",
    ".cfi_offset rip, -16",
    ".cfi_offset rbp, -24",
    // Set up the 3rd argument to the initial function to point to the object
    // that init_stack() set up on the stack.
    "mov rdx, rsp",
    // Rather than call the initial function with a CALL instruction, we
    // manually set up a return address and use JMP instead. This avoids a
    // misalignment of the CPU's return address predictor when a RET instruction
    // is later executed by a switch_yield() or switch_and_reset() in the
    // initial function. This is the reason why those functions are marked as
    // #[inline(always)].
    concat!(
        "lea rcx, [rip + ",
        asm_mangle!("stack_init_trampoline_return"),
        "]"
    ),
    "push rcx",
    // init_stack() placed the address of the initial function just above the
    // parent link on the stack.
    "jmp [rsi + 8]",
    // We don't need to do anything afterwards since the initial function will
    // never return. This is guaranteed by the ! return type.
    //
    // Export the return target of the initial trampoline. This is used when
    // setting up a trap handler.
    asm_function_alt_entry!("stack_init_trampoline_return"),
    // GDB uses some fairly complicated rules to determine whether to use the
    // symbol before or after the return address as the source for unwind
    // information. Normally an unwinder will use the address of the instruction
    // before the return address (usually the call instruction), but because
    // we marked our frame as a signal frame GDB will instead use the address
    // of the following instruction. See get_frame_address_in_block in GDB for
    // more details.
    //
    // We emit a dummy INT3 to ensure that the return address is still within
    // the bounds of the function. In any case, this instruction is never
    // executed since the function we are calling never returns.
    "int3",
    ".cfi_endproc",
    asm_function_end!("stack_init_trampoline"),
);

// This function calls a function pointer on a new stack and restores the
// original stack upon returning. It is used by on_stack() and is much simpler
// than the full coroutine logic, but also more limited since yielding is not
// possible.
global_asm!(
    // See stack_init_trampoline for an explanation of the assembler directives
    // used here.
    ".balign 16",
    asm_function_begin!("stack_call_trampoline"),
    ".cfi_startproc",
    cfi_signal_frame!(),
    // At this point our register state contains the following:
    // - RSP points to the top of the parent stack.
    // - RBP holds its value from the parent context.
    // - RDX is the function that should be called.
    // - RSI points to the top of our stack.
    // - RDI contains the argument to be passed to the function.
    //
    // Create a stack frame and point the frame pointer at it.
    "push rbp",
    "mov rbp, rsp",
    ".cfi_def_cfa rbp, 16",
    ".cfi_offset rbp, -16",
    // Switch to the new stack.
    "mov rsp, rsi",
    // Call the function pointer. The argument is already in the correct
    // register for the function.
    "call rdx",
    // Switch back to the original stack by restoring from the frame pointer,
    // then return.
    "mov rsp, rbp",
    "pop rbp",
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

/// Sets up the initial state on a stack so that the given function is
/// executed on the first switch to this stack.
///
/// The given object is written to the stack and its address on the stack is
/// passed as the 3rd argument to the initial function.
#[inline]
pub unsafe fn init_stack<T>(stack: &impl Stack, func: InitialFunc<T>, obj: T) -> StackPointer {
    let mut sp = stack.base().get();

    // Place the address of the initial function to execute at the top of the
    // stack. This is read by stack_init_trampoline() and jumped to.
    push(&mut sp, Some(func as StackWord));

    // Placeholder for the stack pointer value of the parent context. This is
    // filled in every time switch_and_link() is called.
    push(&mut sp, None);

    // Allocate space on the stack for the initial object, rounding to
    // STACK_ALIGNMENT.
    allocate_obj_on_stack(&mut sp, 16, obj);

    // Set up an address at the top of the stack which is called by
    // switch_and_link() during the initial context switch.
    push(&mut sp, Some(stack_init_trampoline as StackWord));

    StackPointer::new_unchecked(sp)
}

/// This function is used to transfer control to a coroutine along with an
/// argument. A pointer back to our context is stored at a fixed offset from
/// the base of the target stack.
///
/// When another context switches back to us, we receive the argument they sent
/// as well as the stack pointer of the originating context. This can be `None`
/// if the caller used `switch_and_reset` and can't be returned to.
#[inline]
pub unsafe fn switch_and_link(
    arg: EncodedValue,
    sp: StackPointer,
    stack_base: StackPointer,
) -> (EncodedValue, Option<StackPointer>) {
    let (ret_val, ret_sp);

    asm_may_unwind_root!(
        // Save RBX. Ideally this would be done by specifying them as a clobber
        // but that is not possible since RBX is an LLVM reserved register.
        //
        // RBP is also reserved but it is pushed onto the stack later after the
        // call so that a valid frame pointer record is created.
        "push rbx",

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
        "call [rdx]",

        // Upon returning, our register state contains the following:
        // - RSP: Our stack, with the return address and RBP popped.
        // - RSI: The top of the coroutine stack, or 0 if coming from
        //        switch_and_reset.
        // - RDI: The argument passed from the coroutine.

        // The unwind state at this point is a bit tricky: the CFI instructions
        // in stack_init_trampoline will have already restored RBX even
        // though the program counter looks like it is pointing before the POP
        // instruction. However this doesn't cause any issues in practice.

        // Restore RBX.
        "pop rbx",

        // The RDI register is specifically chosen to hold the argument since
        // the ABI uses it for the first argument of a function call.
        //
        // This register is not modified in the assembly code, it is passed
        // straight through to the new context.
        inlateout("rdi") arg => ret_val,

        // The returned stack pointer can be in any register, RSI is arbitrarily
        // chosen here. This must match the register used in switch_yield() and
        // switch_and_reset().
        lateout("rsi") ret_sp,

        // Pass the stack base in RSI so that on the first switch it is passed
        // as the second argument of the initial function. In
        // stack_init_trampoline this is adjusted to point to the parent
        // link directly.
        in("rsi") stack_base.get() as u64,

        // The target stack pointer can be in any register, RDX is arbitrarily
        // chosen here. This needs to match with the register expected by
        // switch_yield().
        in("rdx") sp.get() as u64,

        // Mark all registers as clobbered. Most of the work is done by
        // clobber_abi, we just add the remaining callee-saved registers here.
        // RBX and RBP are LLVM reserved registers and have to be manually
        // saved and restored in the assembly code.
        //
        // Doing this here is more efficient than manually saving all the
        // callee-saved registers: the compiler can avoid repeated saves and
        // restores when multiple context switches are called from the same
        // function.
        lateout("r12") _, lateout("r13") _, lateout("r14") _, lateout("r15") _,
        clobber_abi("sysv64"),
    );

    (ret_val, StackPointer::new(ret_sp))
}

/// This function performs the inverse of `switch_and_link` by returning
/// control to the parent context.
///
/// This function does not return a stack pointer value for the parent context
/// when it switches back to us. Instead, the stack pointer value for the parent
/// context is available in the parent link on the stack.
// This function must always be inlined because it is very sensitive to the
// CPU's return address predictor. See stack_init_trampoline for more details.
#[inline(always)]
pub unsafe fn switch_yield(arg: EncodedValue, parent_link: *mut StackPointer) -> EncodedValue {
    let ret_val;

    asm_may_unwind_yield!(
        // Save RBP and RBX. Ideally this would be done by specifying them as
        // clobbers but that is not possible since they are LLVM reserved
        // registers.
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

        // Restore the parent's RBP register which is at the top of the stack.
        "pop rbp",

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

        // This gets called by switch_and_link(). At this point our register
        // state contains the following:
        // - RSP points to the top of the parent stack.
        // - RBP holds its value from the parent context.
        // - RDX points to the top of our stack, including the return address.
        // - RSI points to the base of our stack.
        // - RDI contains the argument passed from switch_and_link.
        "0:",

        // Save the RBP of the parent context to the parent stack. When combined
        // with the return address this forms a valid frame record (RBP & RIP)
        // in the frame pointer chain.
        "push rbp",

        // Update the parent link near the base of the stack. This is updated
        // every time we switch into a coroutine and allows the coroutine to
        // return to our context through the Yielder and when it unwinds.
        "mov [rsi - 16], rsp",

        // Switch back to our stack, skipping the return address.
        "lea rsp, [rdx + 8]",

        // Restore RBP and RBX.
        "pop rbx",
        "pop rbp",

        // RDI is used by switch_and_link to pass the argument in/out.
        inlateout("rdi") arg => ret_val,

        // The parent link can be in any register, RDX is arbitrarily chosen
        // here.
        in("rdx") parent_link as u64,

        // See switch_and_link() for an explanation of the clobbers.
        lateout("r12") _, lateout("r13") _, lateout("r14") _, lateout("r15") _,
        clobber_abi("sysv64"),
    );

    ret_val
}

/// Variant of `switch_yield` used when returning from the initial function in a
/// context.
///
/// This works by returning a stack pointer value of 0 which prevents the
/// current context from being resumed. There must not be any object left on the
/// stack with pending destructors when this is called.
///
/// Since the stack is still available at this point, `arg` can safely point to
/// memory on the stack until the parent context frees or reuses the stack.
// This function must always be inlined because it is very sensitive to the
// CPU's return address predictor. See stack_init_trampoline for more details.
#[inline(always)]
pub unsafe fn switch_and_reset(arg: EncodedValue, parent_link: *mut StackPointer) -> ! {
    // Most of this code is identical to switch_yield(), refer to the
    // comments there. Only the differences are commented.
    asm!(
        // Load the parent context's stack pointer.
        "mov rsp, [{parent_link}]",

        // Restore the parent's RBP register which is at the top of the stack.
        "pop rbp",

        // Return into the parent context. The top of the parent stack contains
        // a return address generated by the CALL instruction in
        // switch_and_link().
        "ret",

        parent_link = in(reg) parent_link as u64,

        in("rdi") arg,

        // Hard-code the returned stack pointer value to 0 to indicate that this
        // coroutine is done.
        in("rsi") 0,

        options(noreturn),
    );
}

/// Variant of `switch_and_link` which runs a function on the coroutine stack
/// instead of resuming the coroutine. This function will throw an exception
/// which will unwind the coroutine stack to its root.
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
        // Save RBX just like the first half of switch_and_link().
        "push rbx",

        // Push a return address to the stack.
        "lea rax, [rip + 0f]",
        "push rax",

        // Save RBP of the parent context.
        "push rbp",

        // Update the parent link near the base of the coroutine stack.
        "mov [rsi - 16], rsp",

        // Switch to the coroutine stack.
        "mov rsp, rdx",

        // Pop the return address of the target context.
        "pop rax",

        // Restore RBP and RBX from the target context.
        "pop rbx",
        "pop rbp",

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
        "push rax",
        "jmp {throw}",

        // Upon returning, our register state is just like a normal return into
        // switch_and_link().
        "0:",

        // Restore registers just like the second half of switch_and_link.
        "pop rbx",

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

/// Drops the initial object on a coroutine that has not started yet.
#[inline]
pub unsafe fn drop_initial_obj(
    _stack_base: StackPointer,
    stack_ptr: StackPointer,
    drop_fn: unsafe fn(ptr: *mut u8),
) {
    let ptr = (stack_ptr.get() as *mut u8).add(8);
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

/// This function sets up the stack and register state of a trap handler so that
/// upon resuming execution control will immediately jump to the given function
/// with the given argument.
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

    // Set up a return address which returns to stack_init_trampoline. This has
    // the necessary unwinding metadata to switch back to the primary stack when
    // unwinding.
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
        in("rdi") arg,
        in("rsi") stack.base().get(),
        in("rdx") f,
        clobber_abi("sysv64"),
    )
}
