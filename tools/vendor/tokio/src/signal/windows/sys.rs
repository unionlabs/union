use std::convert::TryFrom;
use std::io;
use std::sync::Once;

use crate::signal::registry::{globals, EventId, EventInfo, Init, Storage};
use crate::signal::RxFuture;

use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::um::consoleapi::SetConsoleCtrlHandler;
use winapi::um::wincon::{CTRL_BREAK_EVENT, CTRL_C_EVENT};

pub(super) fn ctrl_c() -> io::Result<RxFuture> {
    new(CTRL_C_EVENT)
}

pub(super) fn ctrl_break() -> io::Result<RxFuture> {
    new(CTRL_BREAK_EVENT)
}

fn new(signum: DWORD) -> io::Result<RxFuture> {
    global_init()?;
    let rx = globals().register_listener(signum as EventId);
    Ok(RxFuture::new(rx))
}

#[derive(Debug)]
pub(crate) struct OsStorage {
    ctrl_c: EventInfo,
    ctrl_break: EventInfo,
}

impl Init for OsStorage {
    fn init() -> Self {
        Self {
            ctrl_c: EventInfo::default(),
            ctrl_break: EventInfo::default(),
        }
    }
}

impl Storage for OsStorage {
    fn event_info(&self, id: EventId) -> Option<&EventInfo> {
        match DWORD::try_from(id) {
            Ok(CTRL_C_EVENT) => Some(&self.ctrl_c),
            Ok(CTRL_BREAK_EVENT) => Some(&self.ctrl_break),
            _ => None,
        }
    }

    fn for_each<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(&'a EventInfo),
    {
        f(&self.ctrl_c);
        f(&self.ctrl_break);
    }
}

#[derive(Debug)]
pub(crate) struct OsExtraData {}

impl Init for OsExtraData {
    fn init() -> Self {
        Self {}
    }
}

fn global_init() -> io::Result<()> {
    static INIT: Once = Once::new();

    let mut init = None;

    INIT.call_once(|| unsafe {
        let rc = SetConsoleCtrlHandler(Some(handler), TRUE);
        let ret = if rc == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        };

        init = Some(ret);
    });

    init.unwrap_or_else(|| Ok(()))
}

unsafe extern "system" fn handler(ty: DWORD) -> BOOL {
    let globals = globals();
    globals.record_event(ty as EventId);

    // According to https://docs.microsoft.com/en-us/windows/console/handlerroutine
    // the handler routine is always invoked in a new thread, thus we don't
    // have the same restrictions as in Unix signal handlers, meaning we can
    // go ahead and perform the broadcast here.
    if globals.broadcast() {
        TRUE
    } else {
        // No one is listening for this notification any more
        // let the OS fire the next (possibly the default) handler.
        FALSE
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::runtime::Runtime;

    use tokio_test::{assert_ok, assert_pending, assert_ready_ok, task};

    #[test]
    fn ctrl_c() {
        let rt = rt();
        let _enter = rt.enter();

        let mut ctrl_c = task::spawn(crate::signal::ctrl_c());

        assert_pending!(ctrl_c.poll());

        // Windows doesn't have a good programmatic way of sending events
        // like sending signals on Unix, so we'll stub out the actual OS
        // integration and test that our handling works.
        unsafe {
            super::handler(CTRL_C_EVENT);
        }

        assert_ready_ok!(ctrl_c.poll());
    }

    #[test]
    fn ctrl_break() {
        let rt = rt();

        rt.block_on(async {
            let mut ctrl_break = assert_ok!(crate::signal::windows::ctrl_break());

            // Windows doesn't have a good programmatic way of sending events
            // like sending signals on Unix, so we'll stub out the actual OS
            // integration and test that our handling works.
            unsafe {
                super::handler(CTRL_BREAK_EVENT);
            }

            ctrl_break.recv().await.unwrap();
        });
    }

    fn rt() -> Runtime {
        crate::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
    }
}
