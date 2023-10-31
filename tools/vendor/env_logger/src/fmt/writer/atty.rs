/*
This internal module contains the terminal detection implementation.

If the `atty` crate is available then we use it to detect whether we're
attached to a particular TTY. If the `atty` crate is not available we
assume we're not attached to anything. This effectively prevents styles
from being printed.
*/

#[cfg(feature = "atty")]
mod imp {
    pub(in crate::fmt) fn is_stdout() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    pub(in crate::fmt) fn is_stderr() -> bool {
        atty::is(atty::Stream::Stderr)
    }
}

#[cfg(not(feature = "atty"))]
mod imp {
    pub(in crate::fmt) fn is_stdout() -> bool {
        false
    }

    pub(in crate::fmt) fn is_stderr() -> bool {
        false
    }
}

pub(in crate::fmt) use self::imp::*;
