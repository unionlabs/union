extern crate std;

use std::cell::Cell;
use std::rc::Rc;
use std::string::ToString;
use std::{println, ptr};

use crate::coroutine::Coroutine;
use crate::{CoroutineResult, ScopedCoroutine};

#[test]
fn smoke() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let mut coroutine = Coroutine::<(), (), ()>::new(move |_, _| {
        hit2.set(true);
    });
    assert!(!hit.get());
    assert!(!coroutine.started());
    assert!(!coroutine.done());
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(()));
    assert!(hit.get());
    assert!(coroutine.started());
    assert!(coroutine.done());
}

#[test]
fn suspend_and_resume() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let mut coroutine = Coroutine::<(), (), ()>::new(move |y, _| {
        y.suspend(());
        hit2.set(true);
        y.suspend(());
    });
    assert!(!hit.get());
    assert!(!coroutine.started());
    assert!(!coroutine.done());
    assert_eq!(coroutine.resume(()), CoroutineResult::Yield(()));
    assert!(!hit.get());
    assert!(coroutine.started());
    assert!(!coroutine.done());
    assert_eq!(coroutine.resume(()), CoroutineResult::Yield(()));
    assert!(hit.get());
    assert!(coroutine.started());
    assert!(!coroutine.done());
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(()));
    assert!(hit.get());
    assert!(coroutine.started());
    assert!(coroutine.done());
}

// Linked backtraces are not supported on x86 Windows.
#[cfg_attr(all(windows, target_arch = "x86"), ignore)]
#[test]
fn backtrace_traces_to_host() {
    #[inline(never)] // try to get this to show up in backtraces
    fn look_for_me() {
        run_test();
    }
    fn assert_contains_host() {
        let trace = backtrace::Backtrace::new();
        println!("{:?}", trace);
        assert!(trace
            .frames()
            .iter()
            .flat_map(|f| f.symbols())
            .filter_map(|s| Some(s.name()?.to_string()))
            .any(|s| s.contains("look_for_me")));
    }

    fn run_test() {
        assert_contains_host();
        let mut coroutine = Coroutine::<(), (), ()>::new(move |y, ()| {
            assert_contains_host();
            y.suspend(());
            y.on_parent_stack(|| assert_contains_host());
            y.suspend(());
            assert_contains_host();
        });
        assert_eq!(coroutine.resume(()), CoroutineResult::Yield(()));
        assert_eq!(coroutine.resume(()), CoroutineResult::Yield(()));
        assert_eq!(coroutine.resume(()), CoroutineResult::Return(()));
    }

    look_for_me();
}

#[cfg(feature = "unwind")]
#[test]
#[should_panic(expected = "foobar")]
fn panics_propagated() {
    use std::panic::{self, AssertUnwindSafe};

    let a = Rc::new(Cell::new(false));
    let b = SetOnDrop(a.clone());
    let mut coroutine = Coroutine::<(), (), ()>::new(move |_, ()| {
        drop(&b);
        panic!("foobar");
    });
    let result = panic::catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
    assert!(result.is_err());
    assert!(a.get());
    panic::resume_unwind(result.unwrap_err());

    struct SetOnDrop(Rc<Cell<bool>>);

    impl Drop for SetOnDrop {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
}

#[cfg(feature = "unwind")]
#[test]
#[should_panic(expected = "foobar")]
fn panics_propagated_via_parent() {
    use std::panic::{self, AssertUnwindSafe};

    let a = Rc::new(Cell::new(false));
    let b = SetOnDrop(a.clone());
    let mut coroutine = Coroutine::<(), (), ()>::new(move |y, ()| {
        drop(&b);
        y.on_parent_stack(|| {
            panic!("foobar");
        });
    });
    let result = panic::catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
    assert!(result.is_err());
    assert!(a.get());
    panic::resume_unwind(result.unwrap_err());

    struct SetOnDrop(Rc<Cell<bool>>);

    impl Drop for SetOnDrop {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
}

#[test]
fn suspend_and_resume_values() {
    let mut coroutine = Coroutine::new(move |y, first| {
        assert_eq!(first, 2.0);
        assert_eq!(y.suspend(4), 3.0);
        "hello".to_string()
    });
    assert_eq!(coroutine.resume(2.0), CoroutineResult::Yield(4));
    assert_eq!(
        coroutine.resume(3.0),
        CoroutineResult::Return("hello".to_string())
    );
}

#[test]
fn stateful() {
    #[repr(align(128))]
    struct Aligned(u8);
    let state = [41, 42, 43, 44, 45];
    let aligned = Aligned(100);
    let mut coroutine = Coroutine::new(move |y, _| {
        assert_eq!(&aligned as *const _ as usize % 128, 0);
        for i in state {
            y.suspend(i);
        }
    });
    for i in state {
        assert_eq!(coroutine.resume(()), CoroutineResult::Yield(i));
    }
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(()));
}

#[test]
fn force_unwind() {
    struct SetOnDrop<'a>(&'a mut bool);
    impl<'a> Drop for SetOnDrop<'a> {
        fn drop(&mut self) {
            *self.0 = true;
        }
    }

    let mut a = false;
    let mut b = false;
    let a_drop = SetOnDrop(&mut a);
    let b_drop = SetOnDrop(&mut b);
    let mut coroutine = ScopedCoroutine::<(), (), (), _>::new(move |y, ()| {
        drop(a_drop);
        y.suspend(());
        drop(b_drop);
    });
    assert!(!coroutine.started());
    assert!(!coroutine.done());
    coroutine.force_unwind();
    assert!(coroutine.started());
    assert!(coroutine.done());
    drop(coroutine);
    assert!(a);
    assert!(b);

    #[cfg(feature = "unwind")]
    {
        let mut a = false;
        let mut b = false;
        let a_drop = SetOnDrop(&mut a);
        let b_drop = SetOnDrop(&mut b);
        let mut coroutine = ScopedCoroutine::<(), (), (), _>::new(move |y, ()| {
            drop(a_drop);
            y.suspend(());
            drop(b_drop);
        });
        coroutine.resume(());
        assert!(coroutine.started());
        assert!(!coroutine.done());
        coroutine.force_unwind();
        assert!(coroutine.started());
        assert!(coroutine.done());
        drop(coroutine);
        assert!(a);
        assert!(b);
    }

    let mut a = false;
    let mut b = false;
    let a_drop = SetOnDrop(&mut a);
    let b_drop = SetOnDrop(&mut b);
    let mut coroutine = ScopedCoroutine::<(), (), (), _>::new(move |y, ()| {
        drop(a_drop);
        y.suspend(());
        drop(b_drop);
    });
    coroutine.resume(());
    coroutine.resume(());
    assert!(coroutine.started());
    assert!(coroutine.done());
    coroutine.force_unwind();
    assert!(coroutine.started());
    assert!(coroutine.done());
    drop(coroutine);
    assert!(a);
    assert!(b);
}

// The Windows stack starts out small with only one page comitted. Check that it
// gets properly grown by the kernel as needed.
#[test]
fn stack_growth() {
    let mut coroutine = Coroutine::<(), (), ()>::new(|_, ()| {
        fn recurse(i: u32, p: &mut [u8; 10000]) {
            unsafe {
                // Ensure the stack allocation isn't optimized away.
                ptr::read_volatile(&p);
            }
            if i > 0 {
                recurse(i - 1, &mut [0; 10000]);
            }
        }

        // Use ~500KB of stack.
        recurse(50, &mut [0; 10000]);
    });
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(()));
}

#[cfg(feature = "unwind")]
#[test]
#[should_panic = "foobar"]
fn backward_stack_address() {
    use crate::stack::DefaultStack;

    // If we create the thread after the stack then (at least on Linux) we
    // should have the thread stack at a lower address than the coroutine
    // stack.
    let stack = DefaultStack::new(1024 * 1024).unwrap();
    let result = std::thread::spawn(move || {
        let mut coroutine = Coroutine::<(), (), ()>::with_stack(stack, move |_, ()| {
            panic!("foobar");
        });

        coroutine.resume(());
        unreachable!();
    })
    .join();
    std::panic::resume_unwind(result.unwrap_err());
}

#[cfg(feature = "unwind")]
#[test]
#[should_panic = "foobar"]
fn forward_stack_address() {
    use crate::stack::DefaultStack;

    // Just in case, also test allocating the stack after thread creation. This
    // should cover all cases.
    let result = std::thread::spawn(move || {
        let stack = DefaultStack::new(1024 * 1024).unwrap();
        let mut coroutine = Coroutine::<(), (), ()>::with_stack(stack, move |_, ()| {
            panic!("foobar");
        });

        coroutine.resume(());
        unreachable!();
    })
    .join();
    std::panic::resume_unwind(result.unwrap_err());
}

#[test]
fn trap_handler() {
    trap_handler::setup_handler();
    let mut coroutine = Coroutine::new(|y, ()| {
        unsafe {
            println!("Before trap");
            ptr::write_volatile(1 as *mut u8, 0);
            println!("After trap");
        }
        y.suspend(1);
        2
    });
    trap_handler::HANDLER.with(|x| {
        x.set(Some((coroutine.trap_handler(), |h| unsafe {
            h.setup_trap_handler(|| 42)
        })))
    });
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(42));
}

#[cfg(feature = "unwind")]
#[test]
#[should_panic = "foobar"]
fn trap_handler_panic() {
    trap_handler::setup_handler();
    let mut coroutine = Coroutine::new(|y, ()| {
        unsafe {
            println!("Before trap");
            ptr::write_volatile(1 as *mut u8, 0);
            println!("After trap");
        }
        y.suspend(1);
        2
    });
    trap_handler::HANDLER.with(|x| {
        x.set(Some((coroutine.trap_handler(), |h| unsafe {
            h.setup_trap_handler(|| panic!("foobar"))
        })))
    });
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(42));
}

#[test]
fn stack_overflow() {
    trap_handler::setup_handler();
    let mut coroutine = Coroutine::new(|y, ()| {
        #[allow(unconditional_recursion)]
        fn recurse(p: &mut [u8; 100]) {
            unsafe {
                // Ensure the stack allocation isn't optimized away.
                ptr::read_volatile(&p);
            }
            recurse(&mut [0; 100]);
        }

        recurse(&mut [0; 100]);
        y.suspend(1);
        2
    });
    trap_handler::HANDLER.with(|x| {
        x.set(Some((coroutine.trap_handler(), |h| unsafe {
            h.setup_trap_handler(|| 42)
        })))
    });
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(42));

    println!("Recovered from first stack overflow");

    // Now reuse the stack and do it again. Make sure the guard pages are
    // properly reset on Windows.
    let stack = coroutine.into_stack();
    let mut coroutine = Coroutine::with_stack(stack, |y, ()| {
        #[allow(unconditional_recursion)]
        fn recurse(p: &mut [u8; 100]) {
            unsafe {
                // Ensure the stack allocation isn't optimized away.
                ptr::read_volatile(&p);
            }
            recurse(&mut [0; 100]);
        }

        recurse(&mut [0; 100]);
        y.suspend(1);
        2
    });
    trap_handler::HANDLER.with(|x| {
        x.set(Some((coroutine.trap_handler(), |h| unsafe {
            h.setup_trap_handler(|| 42)
        })))
    });
    assert_eq!(coroutine.resume(()), CoroutineResult::Return(42));

    println!("Recovered from second stack overflow");
}

/// Helper module for setting up a trap handler.
mod trap_handler {
    extern crate std;

    use std::cell::Cell;
    use std::{io, thread_local};

    use crate::trap::{CoroutineTrapHandler, TrapHandlerRegs};

    thread_local! {
        pub static HANDLER: Cell<
            Option<(
                CoroutineTrapHandler<i32>,
                fn(&CoroutineTrapHandler<i32>) -> TrapHandlerRegs,
            )>,
        > = Cell::new(None);
    }

    #[cfg(unix)]
    pub fn setup_handler() {
        use std::{mem, ptr};
        unsafe extern "C" fn signal_handler(
            _signum: libc::c_int,
            _siginfo: &libc::siginfo_t,
            context: &mut libc::ucontext_t,
        ) {
            let (handler, f) = HANDLER.with(|x| x.get().unwrap());

            let sp;
            cfg_if::cfg_if! {
                if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    target_arch = "x86_64",
                ))] {
                    sp = context.uc_mcontext.gregs[libc::REG_RSP as usize] as usize;
                } else if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    target_arch = "x86",
                ))] {
                    sp = context.uc_mcontext.gregs[libc::REG_ESP as usize] as usize;
                } else if #[cfg(all(target_vendor = "apple", target_arch = "x86_64"))] {
                    sp = (*context.uc_mcontext).__ss.__rsp as usize;
                } else if #[cfg(all(
                        any(target_os = "linux", target_os = "android"),
                        target_arch = "aarch64",
                    ))] {
                    sp = context.uc_mcontext.sp as usize;
                } else if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    target_arch = "arm",
                ))] {
                    sp = context.uc_mcontext.arm_sp as usize;
                } else if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    any(target_arch = "riscv64", target_arch = "riscv32"),
                ))] {
                    sp = context.uc_mcontext.__gregs[libc::REG_SP] as usize;
                } else if #[cfg(all(target_vendor = "apple", target_arch = "aarch64"))] {
                    sp = (*context.uc_mcontext).__ss.__sp as usize;
                } else {
                    compile_error!("Unsupported platform");
                }
            };

            assert!(handler.stack_ptr_in_bounds(sp));
            let regs = f(&handler);

            cfg_if::cfg_if! {
                if #[cfg(all(
                        any(target_os = "linux", target_os = "android"),
                        target_arch = "x86_64",
                    ))] {
                    let TrapHandlerRegs { rip, rsp, rbp, rdi, rsi } = regs;
                    context.uc_mcontext.gregs[libc::REG_RIP as usize] = rip as i64;
                    context.uc_mcontext.gregs[libc::REG_RSP as usize] = rsp as i64;
                    context.uc_mcontext.gregs[libc::REG_RBP as usize] = rbp as i64;
                    context.uc_mcontext.gregs[libc::REG_RDI as usize] = rdi as i64;
                    context.uc_mcontext.gregs[libc::REG_RSI as usize] = rsi as i64;
                } else if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    target_arch = "x86",
                ))] {
                    let TrapHandlerRegs { eip, esp, ebp, ecx, edx } = regs;
                    context.uc_mcontext.gregs[libc::REG_EIP as usize] = eip as i32;
                    context.uc_mcontext.gregs[libc::REG_ESP as usize] = esp as i32;
                    context.uc_mcontext.gregs[libc::REG_EBP as usize] = ebp as i32;
                    context.uc_mcontext.gregs[libc::REG_ECX as usize] = ecx as i32;
                    context.uc_mcontext.gregs[libc::REG_EDX as usize] = edx as i32;
                } else if #[cfg(all(target_vendor = "apple", target_arch = "x86_64"))] {
                    let TrapHandlerRegs { rip, rsp, rbp, rdi, rsi } = regs;
                    (*context.uc_mcontext).__ss.__rip = rip;
                    (*context.uc_mcontext).__ss.__rsp = rsp;
                    (*context.uc_mcontext).__ss.__rbp = rbp;
                    (*context.uc_mcontext).__ss.__rdi = rdi;
                    (*context.uc_mcontext).__ss.__rsi = rsi;
                } else if #[cfg(all(
                        any(target_os = "linux", target_os = "android"),
                        target_arch = "aarch64",
                    ))] {
                    let TrapHandlerRegs { pc, sp, x0, x1, x29, lr } = regs;
                    context.uc_mcontext.pc = pc;
                    context.uc_mcontext.sp = sp;
                    context.uc_mcontext.regs[0] = x0;
                    context.uc_mcontext.regs[1] = x1;
                    context.uc_mcontext.regs[29] = x29;
                    context.uc_mcontext.regs[30] = lr;
                } else if #[cfg(all(
                        any(target_os = "linux", target_os = "android"),
                        target_arch = "arm",
                    ))] {
                    let TrapHandlerRegs {
                        pc,
                        r0,
                        r1,
                        r7,
                        r11,
                        r13,
                        r14,
                        cpsr_thumb,
                        cpsr_endian,
                    } = regs;
                    context.uc_mcontext.arm_pc = pc;
                    context.uc_mcontext.arm_r0 = r0;
                    context.uc_mcontext.arm_r1 = r1;
                    context.uc_mcontext.arm_r7 = r7;
                    context.uc_mcontext.arm_fp = r11;
                    context.uc_mcontext.arm_sp = r13;
                    context.uc_mcontext.arm_lr = r14;
                    if cpsr_thumb {
                        context.uc_mcontext.arm_cpsr |= 0x20;
                    } else {
                        context.uc_mcontext.arm_cpsr &= !0x20;
                    }
                    if cpsr_endian {
                        context.uc_mcontext.arm_cpsr |= 0x200;
                    } else {
                        context.uc_mcontext.arm_cpsr &= !0x200;
                    }
                } else if #[cfg(all(
                    any(target_os = "linux", target_os = "android"),
                    any(target_arch = "riscv64", target_arch = "riscv32"),
                ))] {
                    let TrapHandlerRegs { pc, ra, sp, a0, a1, s0 } = regs;
                    context.uc_mcontext.__gregs[libc::REG_PC] = pc as libc::c_ulong;
                    context.uc_mcontext.__gregs[libc::REG_RA] = ra as libc::c_ulong;
                    context.uc_mcontext.__gregs[libc::REG_SP] = sp as libc::c_ulong;
                    context.uc_mcontext.__gregs[libc::REG_A0] = a0 as libc::c_ulong;
                    context.uc_mcontext.__gregs[libc::REG_A0 + 1] = a1 as libc::c_ulong;
                    context.uc_mcontext.__gregs[libc::REG_S0] = s0 as libc::c_ulong;
                } else if #[cfg(all(target_vendor = "apple", target_arch = "aarch64"))] {
                    let TrapHandlerRegs { pc, sp, x0, x1, x29, lr } = regs;
                    (*context.uc_mcontext).__ss.__pc = pc;
                    (*context.uc_mcontext).__ss.__sp = sp;
                    (*context.uc_mcontext).__ss.__x[0] = x0;
                    (*context.uc_mcontext).__ss.__x[1] = x1;
                    (*context.uc_mcontext).__ss.__fp = x29;
                    (*context.uc_mcontext).__ss.__lr = lr;
                } else {
                    compile_error!("Unsupported platform");
                }
            };
        }

        unsafe {
            let mut handler: libc::sigaction = mem::zeroed();
            handler.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
            handler.sa_sigaction = signal_handler as usize;
            libc::sigfillset(&mut handler.sa_mask);
            if libc::sigaction(libc::SIGSEGV, &handler, ptr::null_mut()) != 0 {
                panic!(
                    "unable to install signal handler: {}",
                    io::Error::last_os_error(),
                );
            }
            if libc::sigaction(libc::SIGBUS, &handler, ptr::null_mut()) != 0 {
                panic!(
                    "unable to install signal handler: {}",
                    io::Error::last_os_error(),
                );
            }
        }
    }

    #[cfg(windows)]
    pub fn setup_handler() {
        use windows_sys::Win32::Foundation::{
            EXCEPTION_ACCESS_VIOLATION, EXCEPTION_STACK_OVERFLOW,
        };
        use windows_sys::Win32::System::Diagnostics::Debug::{
            AddVectoredExceptionHandler, EXCEPTION_POINTERS,
        };

        unsafe extern "system" fn exception_handler(
            exception_info: *mut EXCEPTION_POINTERS,
        ) -> i32 {
            match (*(*exception_info).ExceptionRecord).ExceptionCode {
                EXCEPTION_ACCESS_VIOLATION | EXCEPTION_STACK_OVERFLOW => {}
                _ => return 0, // EXCEPTION_CONTINUE_SEARCH
            }

            let (handler, f) = match HANDLER.with(|x| x.get()) {
                Some(handler) => handler,
                None => return 0, // EXCEPTION_CONTINUE_SEARCH
            };

            let sp;
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "x86_64")] {
                    sp = (*(*exception_info).ContextRecord).Rsp as usize;
                } else if #[cfg(target_arch = "x86")] {
                    sp = (*(*exception_info).ContextRecord).Esp as usize;
                } else {
                    compile_error!("Unsupported platform");
                }
            };

            if !handler.stack_ptr_in_bounds(sp) {
                return 0; // EXCEPTION_CONTINUE_SEARCH
            }
            let regs = f(&handler);

            cfg_if::cfg_if! {
                if #[cfg(target_arch = "x86_64")] {
                    let TrapHandlerRegs { rip, rsp, rbp, rdi, rsi } = regs;
                    (*(*exception_info).ContextRecord).Rip = rip;
                    (*(*exception_info).ContextRecord).Rsp = rsp;
                    (*(*exception_info).ContextRecord).Rbp = rbp;
                    (*(*exception_info).ContextRecord).Rdi = rdi;
                    (*(*exception_info).ContextRecord).Rsi = rsi;
                } else if #[cfg(target_arch = "x86")] {
                    let TrapHandlerRegs { eip, esp, ebp, ecx, edx } = regs;
                    (*(*exception_info).ContextRecord).Eip = eip;
                    (*(*exception_info).ContextRecord).Esp = esp;
                    (*(*exception_info).ContextRecord).Ebp = ebp;
                    (*(*exception_info).ContextRecord).Ecx = ecx;
                    (*(*exception_info).ContextRecord).Edx = edx;
                } else {
                    compile_error!("Unsupported platform");
                }
            };

            // EXCEPTION_CONTINUE_EXECUTION is -1. Not to be confused with
            // ExceptionContinueExecution which has a value of 0.
            -1
        }

        unsafe {
            if AddVectoredExceptionHandler(1, Some(exception_handler)).is_null() {
                panic!(
                    "failed to add exception handler: {}",
                    io::Error::last_os_error()
                );
            }
        }
    }
}
