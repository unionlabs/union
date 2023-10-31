use crate::sync::Notify;
use std::future::Future;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

#[test]
fn notify_clones_waker_before_lock() {
    const VTABLE: &RawWakerVTable = &RawWakerVTable::new(clone_w, wake, wake_by_ref, drop_w);

    unsafe fn clone_w(data: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::<Notify>::from_raw(data as *const Notify));
        // Or some other arbitrary code that shouldn't be executed while the
        // Notify wait list is locked.
        arc.notify_one();
        let _arc_clone: ManuallyDrop<_> = arc.clone();
        RawWaker::new(data, VTABLE)
    }

    unsafe fn drop_w(data: *const ()) {
        let _ = Arc::<Notify>::from_raw(data as *const Notify);
    }

    unsafe fn wake(_data: *const ()) {
        unreachable!()
    }

    unsafe fn wake_by_ref(_data: *const ()) {
        unreachable!()
    }

    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    let waker =
        unsafe { Waker::from_raw(RawWaker::new(Arc::into_raw(notify2) as *const _, VTABLE)) };
    let mut cx = Context::from_waker(&waker);

    let future = notify.notified();
    pin!(future);

    // The result doesn't matter, we're just testing that we don't deadlock.
    let _ = future.poll(&mut cx);
}

#[test]
fn notify_simple() {
    let notify = Notify::new();

    let mut fut1 = tokio_test::task::spawn(notify.notified());
    assert!(fut1.poll().is_pending());

    let mut fut2 = tokio_test::task::spawn(notify.notified());
    assert!(fut2.poll().is_pending());

    notify.notify_waiters();

    assert!(fut1.poll().is_ready());
    assert!(fut2.poll().is_ready());
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn watch_test() {
    let rt = crate::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        let (tx, mut rx) = crate::sync::watch::channel(());

        crate::spawn(async move {
            let _ = tx.send(());
        });

        let _ = rx.changed().await;
    });
}
