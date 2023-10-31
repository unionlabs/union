//! Thread local runtime context
use crate::runtime::{Handle, TryCurrentError};

use std::cell::RefCell;

thread_local! {
    static CONTEXT: RefCell<Option<Handle>> = const { RefCell::new(None) }
}

pub(crate) fn try_current() -> Result<Handle, crate::runtime::TryCurrentError> {
    match CONTEXT.try_with(|ctx| ctx.borrow().clone()) {
        Ok(Some(handle)) => Ok(handle),
        Ok(None) => Err(TryCurrentError::new_no_context()),
        Err(_access_error) => Err(TryCurrentError::new_thread_local_destroyed()),
    }
}

#[track_caller]
pub(crate) fn current() -> Handle {
    match try_current() {
        Ok(handle) => handle,
        Err(e) => panic!("{}", e),
    }
}

cfg_io_driver! {
    pub(crate) fn io_handle() -> crate::runtime::driver::IoHandle {
        match CONTEXT.try_with(|ctx| {
            let ctx = ctx.borrow();
            ctx.as_ref().expect(crate::util::error::CONTEXT_MISSING_ERROR).as_inner().io_handle.clone()
        }) {
            Ok(io_handle) => io_handle,
            Err(_) => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
        }
    }
}

cfg_signal_internal! {
    #[cfg(unix)]
    pub(crate) fn signal_handle() -> crate::runtime::driver::SignalHandle {
        match CONTEXT.try_with(|ctx| {
            let ctx = ctx.borrow();
            ctx.as_ref().expect(crate::util::error::CONTEXT_MISSING_ERROR).as_inner().signal_handle.clone()
        }) {
            Ok(signal_handle) => signal_handle,
            Err(_) => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
        }
    }
}

cfg_time! {
    pub(crate) fn time_handle() -> crate::runtime::driver::TimeHandle {
        match CONTEXT.try_with(|ctx| {
            let ctx = ctx.borrow();
            ctx.as_ref().expect(crate::util::error::CONTEXT_MISSING_ERROR).as_inner().time_handle.clone()
        }) {
            Ok(time_handle) => time_handle,
            Err(_) => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
        }
    }

    cfg_test_util! {
        pub(crate) fn clock() -> Option<crate::runtime::driver::Clock> {
            match CONTEXT.try_with(|ctx| (*ctx.borrow()).as_ref().map(|ctx| ctx.as_inner().clock.clone())) {
                Ok(clock) => clock,
                Err(_) => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
            }
        }
    }
}

cfg_rt! {
    pub(crate) fn spawn_handle() -> Option<crate::runtime::Spawner> {
        match CONTEXT.try_with(|ctx| (*ctx.borrow()).as_ref().map(|ctx| ctx.spawner.clone())) {
            Ok(spawner) => spawner,
            Err(_) => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
        }
    }
}

/// Sets this [`Handle`] as the current active [`Handle`].
///
/// [`Handle`]: Handle
pub(crate) fn enter(new: Handle) -> EnterGuard {
    match try_enter(new) {
        Some(guard) => guard,
        None => panic!("{}", crate::util::error::THREAD_LOCAL_DESTROYED_ERROR),
    }
}

/// Sets this [`Handle`] as the current active [`Handle`].
///
/// [`Handle`]: Handle
pub(crate) fn try_enter(new: Handle) -> Option<EnterGuard> {
    CONTEXT
        .try_with(|ctx| {
            let old = ctx.borrow_mut().replace(new);
            EnterGuard(old)
        })
        .ok()
}

#[derive(Debug)]
pub(crate) struct EnterGuard(Option<Handle>);

impl Drop for EnterGuard {
    fn drop(&mut self) {
        CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = self.0.take();
        });
    }
}
