extern crate std;

use std::cell::Cell;
use std::rc::Rc;
use std::string::ToString;
use std::{println, ptr};

use crate::coroutine::on_stack;
use crate::stack::DefaultStack;

#[test]
fn smoke() {
    let hit = Rc::new(Cell::new(false));
    let hit2 = hit.clone();
    let result = on_stack(DefaultStack::default(), move || {
        hit2.set(true);
        "hello".to_string()
    });
    assert!(hit.get());
    assert_eq!(result, "hello".to_string());
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
        on_stack(DefaultStack::default(), || {
            assert_contains_host();
        });
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
    let result = panic::catch_unwind(AssertUnwindSafe(move || {
        on_stack(DefaultStack::default(), move || {
            drop(&b);
            panic!("foobar");
        })
    }));
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

// The Windows stack starts out small with only one page comitted. Check that it
// gets properly grown by the kernel as needed.
#[test]
fn stack_growth() {
    on_stack(DefaultStack::default(), || {
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
        on_stack(stack, || {
            panic!("foobar");
        });
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
        on_stack(stack, || {
            panic!("foobar");
        });
    })
    .join();
    std::panic::resume_unwind(result.unwrap_err());
}
