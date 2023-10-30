use crate::sync::AtomicWaker;
use tokio_test::task;

use std::task::Waker;

trait AssertSend: Send {}
trait AssertSync: Send {}

impl AssertSend for AtomicWaker {}
impl AssertSync for AtomicWaker {}

impl AssertSend for Waker {}
impl AssertSync for Waker {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

#[test]
fn basic_usage() {
    let mut waker = task::spawn(AtomicWaker::new());

    waker.enter(|cx, waker| waker.register_by_ref(cx.waker()));
    waker.wake();

    assert!(waker.is_woken());
}

#[test]
fn wake_without_register() {
    let mut waker = task::spawn(AtomicWaker::new());
    waker.wake();

    // Registering should not result in a notification
    waker.enter(|cx, waker| waker.register_by_ref(cx.waker()));

    assert!(!waker.is_woken());
}

#[test]
#[cfg(not(target_arch = "wasm32"))] // wasm currently doesn't support unwinding
fn atomic_waker_panic_safe() {
    use std::panic;
    use std::ptr;
    use std::task::{RawWaker, RawWakerVTable, Waker};

    static PANICKING_VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| panic!("clone"),
        |_| unimplemented!("wake"),
        |_| unimplemented!("wake_by_ref"),
        |_| (),
    );

    static NONPANICKING_VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(ptr::null(), &NONPANICKING_VTABLE),
        |_| unimplemented!("wake"),
        |_| unimplemented!("wake_by_ref"),
        |_| (),
    );

    let panicking = unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &PANICKING_VTABLE)) };
    let nonpanicking = unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &NONPANICKING_VTABLE)) };

    let atomic_waker = AtomicWaker::new();

    let panicking = panic::AssertUnwindSafe(&panicking);

    let result = panic::catch_unwind(|| {
        let panic::AssertUnwindSafe(panicking) = panicking;
        atomic_waker.register_by_ref(panicking);
    });

    assert!(result.is_err());
    assert!(atomic_waker.take_waker().is_none());

    atomic_waker.register_by_ref(&nonpanicking);
    assert!(atomic_waker.take_waker().is_some());
}
