use crate::future::Future;
use crate::runtime::task::harness::Harness;
use crate::runtime::task::{Header, Schedule};

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops;
use std::ptr::NonNull;
use std::task::{RawWaker, RawWakerVTable, Waker};

pub(super) struct WakerRef<'a, S: 'static> {
    waker: ManuallyDrop<Waker>,
    _p: PhantomData<(&'a Header, S)>,
}

/// Returns a `WakerRef` which avoids having to pre-emptively increase the
/// refcount if there is no need to do so.
pub(super) fn waker_ref<T, S>(header: &NonNull<Header>) -> WakerRef<'_, S>
where
    T: Future,
    S: Schedule,
{
    // `Waker::will_wake` uses the VTABLE pointer as part of the check. This
    // means that `will_wake` will always return false when using the current
    // task's waker. (discussion at rust-lang/rust#66281).
    //
    // To fix this, we use a single vtable. Since we pass in a reference at this
    // point and not an *owned* waker, we must ensure that `drop` is never
    // called on this waker instance. This is done by wrapping it with
    // `ManuallyDrop` and then never calling drop.
    let waker = unsafe { ManuallyDrop::new(Waker::from_raw(raw_waker::<T, S>(*header))) };

    WakerRef {
        waker,
        _p: PhantomData,
    }
}

impl<S> ops::Deref for WakerRef<'_, S> {
    type Target = Waker;

    fn deref(&self) -> &Waker {
        &self.waker
    }
}

cfg_trace! {
    macro_rules! trace {
        ($harness:expr, $op:expr) => {
            if let Some(id) = $harness.id() {
                tracing::trace!(
                    target: "tokio::task::waker",
                    op = $op,
                    task.id = id.into_u64(),
                );
            }
        }
    }
}

cfg_not_trace! {
    macro_rules! trace {
        ($harness:expr, $op:expr) => {
            // noop
            let _ = &$harness;
        }
    }
}

unsafe fn clone_waker<T, S>(ptr: *const ()) -> RawWaker
where
    T: Future,
    S: Schedule,
{
    let header = ptr as *const Header;
    let ptr = NonNull::new_unchecked(ptr as *mut Header);
    let harness = Harness::<T, S>::from_raw(ptr);
    trace!(harness, "waker.clone");
    (*header).state.ref_inc();
    raw_waker::<T, S>(ptr)
}

unsafe fn drop_waker<T, S>(ptr: *const ())
where
    T: Future,
    S: Schedule,
{
    let ptr = NonNull::new_unchecked(ptr as *mut Header);
    let harness = Harness::<T, S>::from_raw(ptr);
    trace!(harness, "waker.drop");
    harness.drop_reference();
}

unsafe fn wake_by_val<T, S>(ptr: *const ())
where
    T: Future,
    S: Schedule,
{
    let ptr = NonNull::new_unchecked(ptr as *mut Header);
    let harness = Harness::<T, S>::from_raw(ptr);
    trace!(harness, "waker.wake");
    harness.wake_by_val();
}

// Wake without consuming the waker
unsafe fn wake_by_ref<T, S>(ptr: *const ())
where
    T: Future,
    S: Schedule,
{
    let ptr = NonNull::new_unchecked(ptr as *mut Header);
    let harness = Harness::<T, S>::from_raw(ptr);
    trace!(harness, "waker.wake_by_ref");
    harness.wake_by_ref();
}

fn raw_waker<T, S>(header: NonNull<Header>) -> RawWaker
where
    T: Future,
    S: Schedule,
{
    let ptr = header.as_ptr() as *const ();
    let vtable = &RawWakerVTable::new(
        clone_waker::<T, S>,
        wake_by_val::<T, S>,
        wake_by_ref::<T, S>,
        drop_waker::<T, S>,
    );
    RawWaker::new(ptr, vtable)
}
