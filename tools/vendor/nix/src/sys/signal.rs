// Portions of this file are Copyright 2014 The Rust Project Developers.
// See https://www.rust-lang.org/policies/licenses.

//! Operating system signals.

use crate::{Error, Result};
use crate::errno::Errno;
use crate::unistd::Pid;
use std::mem;
use std::fmt;
use std::str::FromStr;
#[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
use std::os::unix::io::RawFd;
use std::ptr;

#[cfg(not(any(target_os = "openbsd", target_os = "redox")))]
pub use self::sigevent::*;

libc_enum!{
    /// Types of operating system signals
    // Currently there is only one definition of c_int in libc, as well as only one
    // type for signal constants.
    // We would prefer to use the libc::c_int alias in the repr attribute. Unfortunately
    // this is not (yet) possible.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Signal {
        /// Hangup
        SIGHUP,
        /// Interrupt
        SIGINT,
        /// Quit
        SIGQUIT,
        /// Illegal instruction (not reset when caught)
        SIGILL,
        /// Trace trap (not reset when caught)
        SIGTRAP,
        /// Abort
        SIGABRT,
        /// Bus error
        SIGBUS,
        /// Floating point exception
        SIGFPE,
        /// Kill (cannot be caught or ignored)
        SIGKILL,
        /// User defined signal 1
        SIGUSR1,
        /// Segmentation violation
        SIGSEGV,
        /// User defined signal 2
        SIGUSR2,
        /// Write on a pipe with no one to read it
        SIGPIPE,
        /// Alarm clock
        SIGALRM,
        /// Software termination signal from kill
        SIGTERM,
        /// Stack fault (obsolete)
        #[cfg(all(any(target_os = "android", target_os = "emscripten",
                      target_os = "fuchsia", target_os = "linux"),
                  not(any(target_arch = "mips", target_arch = "mips64",
                          target_arch = "sparc64"))))]
        SIGSTKFLT,
        /// To parent on child stop or exit
        SIGCHLD,
        /// Continue a stopped process
        SIGCONT,
        /// Sendable stop signal not from tty
        SIGSTOP,
        /// Stop signal from tty
        SIGTSTP,
        /// To readers pgrp upon background tty read
        SIGTTIN,
        /// Like TTIN if (tp->t_local&LTOSTOP)
        SIGTTOU,
        /// Urgent condition on IO channel
        SIGURG,
        /// Exceeded CPU time limit
        SIGXCPU,
        /// Exceeded file size limit
        SIGXFSZ,
        /// Virtual time alarm
        SIGVTALRM,
        /// Profiling time alarm
        SIGPROF,
        /// Window size changes
        SIGWINCH,
        /// Input/output possible signal
        SIGIO,
        #[cfg(any(target_os = "android", target_os = "emscripten",
                  target_os = "fuchsia", target_os = "linux"))]
        /// Power failure imminent.
        SIGPWR,
        /// Bad system call
        SIGSYS,
        #[cfg(not(any(target_os = "android", target_os = "emscripten",
                      target_os = "fuchsia", target_os = "linux",
                      target_os = "redox")))]
        /// Emulator trap
        SIGEMT,
        #[cfg(not(any(target_os = "android", target_os = "emscripten",
                      target_os = "fuchsia", target_os = "linux",
                      target_os = "redox")))]
        /// Information request
        SIGINFO,
    }
    impl TryFrom<i32>
}

impl FromStr for Signal {
    type Err = Error;
    fn from_str(s: &str) -> Result<Signal> {
        Ok(match s {
            "SIGHUP" => Signal::SIGHUP,
            "SIGINT" => Signal::SIGINT,
            "SIGQUIT" => Signal::SIGQUIT,
            "SIGILL" => Signal::SIGILL,
            "SIGTRAP" => Signal::SIGTRAP,
            "SIGABRT" => Signal::SIGABRT,
            "SIGBUS" => Signal::SIGBUS,
            "SIGFPE" => Signal::SIGFPE,
            "SIGKILL" => Signal::SIGKILL,
            "SIGUSR1" => Signal::SIGUSR1,
            "SIGSEGV" => Signal::SIGSEGV,
            "SIGUSR2" => Signal::SIGUSR2,
            "SIGPIPE" => Signal::SIGPIPE,
            "SIGALRM" => Signal::SIGALRM,
            "SIGTERM" => Signal::SIGTERM,
            #[cfg(all(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux"),
                      not(any(target_arch = "mips", target_arch = "mips64",
                              target_arch = "sparc64"))))]
            "SIGSTKFLT" => Signal::SIGSTKFLT,
            "SIGCHLD" => Signal::SIGCHLD,
            "SIGCONT" => Signal::SIGCONT,
            "SIGSTOP" => Signal::SIGSTOP,
            "SIGTSTP" => Signal::SIGTSTP,
            "SIGTTIN" => Signal::SIGTTIN,
            "SIGTTOU" => Signal::SIGTTOU,
            "SIGURG" => Signal::SIGURG,
            "SIGXCPU" => Signal::SIGXCPU,
            "SIGXFSZ" => Signal::SIGXFSZ,
            "SIGVTALRM" => Signal::SIGVTALRM,
            "SIGPROF" => Signal::SIGPROF,
            "SIGWINCH" => Signal::SIGWINCH,
            "SIGIO" => Signal::SIGIO,
            #[cfg(any(target_os = "android", target_os = "emscripten",
                      target_os = "fuchsia", target_os = "linux"))]
            "SIGPWR" => Signal::SIGPWR,
            "SIGSYS" => Signal::SIGSYS,
            #[cfg(not(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux",
                          target_os = "redox")))]
            "SIGEMT" => Signal::SIGEMT,
            #[cfg(not(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux",
                          target_os = "redox")))]
            "SIGINFO" => Signal::SIGINFO,
            _ => return Err(Errno::EINVAL),
        })
    }
}

impl Signal {
    /// Returns name of signal.
    ///
    /// This function is equivalent to `<Signal as AsRef<str>>::as_ref()`,
    /// with difference that returned string is `'static`
    /// and not bound to `self`'s lifetime.
    pub const fn as_str(self) -> &'static str {
        match self {
            Signal::SIGHUP => "SIGHUP",
            Signal::SIGINT => "SIGINT",
            Signal::SIGQUIT => "SIGQUIT",
            Signal::SIGILL => "SIGILL",
            Signal::SIGTRAP => "SIGTRAP",
            Signal::SIGABRT => "SIGABRT",
            Signal::SIGBUS => "SIGBUS",
            Signal::SIGFPE => "SIGFPE",
            Signal::SIGKILL => "SIGKILL",
            Signal::SIGUSR1 => "SIGUSR1",
            Signal::SIGSEGV => "SIGSEGV",
            Signal::SIGUSR2 => "SIGUSR2",
            Signal::SIGPIPE => "SIGPIPE",
            Signal::SIGALRM => "SIGALRM",
            Signal::SIGTERM => "SIGTERM",
            #[cfg(all(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux"),
                      not(any(target_arch = "mips", target_arch = "mips64", target_arch = "sparc64"))))]
            Signal::SIGSTKFLT => "SIGSTKFLT",
            Signal::SIGCHLD => "SIGCHLD",
            Signal::SIGCONT => "SIGCONT",
            Signal::SIGSTOP => "SIGSTOP",
            Signal::SIGTSTP => "SIGTSTP",
            Signal::SIGTTIN => "SIGTTIN",
            Signal::SIGTTOU => "SIGTTOU",
            Signal::SIGURG => "SIGURG",
            Signal::SIGXCPU => "SIGXCPU",
            Signal::SIGXFSZ => "SIGXFSZ",
            Signal::SIGVTALRM => "SIGVTALRM",
            Signal::SIGPROF => "SIGPROF",
            Signal::SIGWINCH => "SIGWINCH",
            Signal::SIGIO => "SIGIO",
            #[cfg(any(target_os = "android", target_os = "emscripten",
                      target_os = "fuchsia", target_os = "linux"))]
            Signal::SIGPWR => "SIGPWR",
            Signal::SIGSYS => "SIGSYS",
            #[cfg(not(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux",
                          target_os = "redox")))]
            Signal::SIGEMT => "SIGEMT",
            #[cfg(not(any(target_os = "android", target_os = "emscripten",
                          target_os = "fuchsia", target_os = "linux",
                          target_os = "redox")))]
            Signal::SIGINFO => "SIGINFO",
        }
    }
}

impl AsRef<str> for Signal {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

pub use self::Signal::*;

#[cfg(target_os = "redox")]
const SIGNALS: [Signal; 29] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGSYS];
#[cfg(all(any(target_os = "linux", target_os = "android",
              target_os = "emscripten", target_os = "fuchsia"),
          not(any(target_arch = "mips", target_arch = "mips64",
                  target_arch = "sparc64"))))]
const SIGNALS: [Signal; 31] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGSTKFLT,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPWR,
    SIGSYS];
#[cfg(all(any(target_os = "linux", target_os = "android",
              target_os = "emscripten", target_os = "fuchsia"),
          any(target_arch = "mips", target_arch = "mips64",
              target_arch = "sparc64")))]
const SIGNALS: [Signal; 30] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPWR,
    SIGSYS];
#[cfg(not(any(target_os = "linux", target_os = "android",
              target_os = "fuchsia", target_os = "emscripten",
              target_os = "redox")))]
const SIGNALS: [Signal; 31] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGSYS,
    SIGEMT,
    SIGINFO];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Iterate through all signals defined by this operating system
pub struct SignalIterator {
    next: usize,
}

impl Iterator for SignalIterator {
    type Item = Signal;

    fn next(&mut self) -> Option<Signal> {
        if self.next < SIGNALS.len() {
            let next_signal = SIGNALS[self.next];
            self.next += 1;
            Some(next_signal)
        } else {
            None
        }
    }
}

impl Signal {
    /// Iterate through all signals defined by this OS
    pub const fn iterator() -> SignalIterator {
        SignalIterator{next: 0}
    }
}

/// Alias for [`SIGABRT`]
pub const SIGIOT : Signal = SIGABRT;
/// Alias for [`SIGIO`]
pub const SIGPOLL : Signal = SIGIO;
/// Alias for [`SIGSYS`]
pub const SIGUNUSED : Signal = SIGSYS;

#[cfg(not(target_os = "redox"))]
type SaFlags_t = libc::c_int;
#[cfg(target_os = "redox")]
type SaFlags_t = libc::c_ulong;

libc_bitflags!{
    /// Controls the behavior of a [`SigAction`]
    pub struct SaFlags: SaFlags_t {
        /// When catching a [`Signal::SIGCHLD`] signal, the signal will be
        /// generated only when a child process exits, not when a child process
        /// stops.
        SA_NOCLDSTOP;
        /// When catching a [`Signal::SIGCHLD`] signal, the system will not
        /// create zombie processes when children of the calling process exit.
        SA_NOCLDWAIT;
        /// Further occurrences of the delivered signal are not masked during
        /// the execution of the handler.
        SA_NODEFER;
        /// The system will deliver the signal to the process on a signal stack,
        /// specified by each thread with sigaltstack(2).
        SA_ONSTACK;
        /// The handler is reset back to the default at the moment the signal is
        /// delivered.
        SA_RESETHAND;
        /// Requests that certain system calls restart if interrupted by this
        /// signal.  See the man page for complete details.
        SA_RESTART;
        /// This flag is controlled internally by Nix.
        SA_SIGINFO;
    }
}

libc_enum! {
    /// Specifies how certain functions should manipulate a signal mask
    #[repr(i32)]
    #[non_exhaustive]
    pub enum SigmaskHow {
        /// The new mask is the union of the current mask and the specified set.
        SIG_BLOCK,
        /// The new mask is the intersection of the current mask and the
        /// complement of the specified set.
        SIG_UNBLOCK,
        /// The current mask is replaced by the specified set.
        SIG_SETMASK,
    }
}

/// Specifies a set of [`Signal`]s that may be blocked, waited for, etc.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SigSet {
    sigset: libc::sigset_t
}


impl SigSet {
    /// Initialize to include all signals.
    pub fn all() -> SigSet {
        let mut sigset = mem::MaybeUninit::uninit();
        let _ = unsafe { libc::sigfillset(sigset.as_mut_ptr()) };

        unsafe{ SigSet { sigset: sigset.assume_init() } }
    }

    /// Initialize to include nothing.
    pub fn empty() -> SigSet {
        let mut sigset = mem::MaybeUninit::uninit();
        let _ = unsafe { libc::sigemptyset(sigset.as_mut_ptr()) };

        unsafe{ SigSet { sigset: sigset.assume_init() } }
    }

    /// Add the specified signal to the set.
    pub fn add(&mut self, signal: Signal) {
        unsafe { libc::sigaddset(&mut self.sigset as *mut libc::sigset_t, signal as libc::c_int) };
    }

    /// Remove all signals from this set.
    pub fn clear(&mut self) {
        unsafe { libc::sigemptyset(&mut self.sigset as *mut libc::sigset_t) };
    }

    /// Remove the specified signal from this set.
    pub fn remove(&mut self, signal: Signal) {
        unsafe { libc::sigdelset(&mut self.sigset as *mut libc::sigset_t, signal as libc::c_int) };
    }

    /// Return whether this set includes the specified signal.
    pub fn contains(&self, signal: Signal) -> bool {
        let res = unsafe { libc::sigismember(&self.sigset as *const libc::sigset_t, signal as libc::c_int) };

        match res {
            1 => true,
            0 => false,
            _ => unreachable!("unexpected value from sigismember"),
        }
    }

    /// Merge all of `other`'s signals into this set.
    // TODO: use libc::sigorset on supported operating systems.
    pub fn extend(&mut self, other: &SigSet) {
        for signal in Signal::iterator() {
            if other.contains(signal) {
                self.add(signal);
            }
        }
    }

    /// Gets the currently blocked (masked) set of signals for the calling thread.
    pub fn thread_get_mask() -> Result<SigSet> {
        let mut oldmask = mem::MaybeUninit::uninit();
        do_pthread_sigmask(SigmaskHow::SIG_SETMASK, None, Some(oldmask.as_mut_ptr()))?;
        Ok(unsafe{ SigSet{sigset: oldmask.assume_init()}})
    }

    /// Sets the set of signals as the signal mask for the calling thread.
    pub fn thread_set_mask(&self) -> Result<()> {
        pthread_sigmask(SigmaskHow::SIG_SETMASK, Some(self), None)
    }

    /// Adds the set of signals to the signal mask for the calling thread.
    pub fn thread_block(&self) -> Result<()> {
        pthread_sigmask(SigmaskHow::SIG_BLOCK, Some(self), None)
    }

    /// Removes the set of signals from the signal mask for the calling thread.
    pub fn thread_unblock(&self) -> Result<()> {
        pthread_sigmask(SigmaskHow::SIG_UNBLOCK, Some(self), None)
    }

    /// Sets the set of signals as the signal mask, and returns the old mask.
    pub fn thread_swap_mask(&self, how: SigmaskHow) -> Result<SigSet> {
        let mut oldmask = mem::MaybeUninit::uninit();
        do_pthread_sigmask(how, Some(self), Some(oldmask.as_mut_ptr()))?;
        Ok(unsafe{ SigSet{sigset: oldmask.assume_init()}})
    }

    /// Suspends execution of the calling thread until one of the signals in the
    /// signal mask becomes pending, and returns the accepted signal.
    #[cfg(not(target_os = "redox"))] // RedoxFS does not yet support sigwait
    pub fn wait(&self) -> Result<Signal> {
        use std::convert::TryFrom;

        let mut signum = mem::MaybeUninit::uninit();
        let res = unsafe { libc::sigwait(&self.sigset as *const libc::sigset_t, signum.as_mut_ptr()) };

        Errno::result(res).map(|_| unsafe {
            Signal::try_from(signum.assume_init()).unwrap()
        })
    }
}

impl AsRef<libc::sigset_t> for SigSet {
    fn as_ref(&self) -> &libc::sigset_t {
        &self.sigset
    }
}

/// A signal handler.
#[allow(unknown_lints)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SigHandler {
    /// Default signal handling.
    SigDfl,
    /// Request that the signal be ignored.
    SigIgn,
    /// Use the given signal-catching function, which takes in the signal.
    Handler(extern fn(libc::c_int)),
    /// Use the given signal-catching function, which takes in the signal, information about how
    /// the signal was generated, and a pointer to the threads `ucontext_t`.
    #[cfg(not(target_os = "redox"))]
    SigAction(extern fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void))
}

/// Action to take on receipt of a signal. Corresponds to `sigaction`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SigAction {
    sigaction: libc::sigaction
}

impl SigAction {
    /// Creates a new action.
    ///
    /// The `SA_SIGINFO` bit in the `flags` argument is ignored (it will be set only if `handler`
    /// is the `SigAction` variant). `mask` specifies other signals to block during execution of
    /// the signal-catching function.
    pub fn new(handler: SigHandler, flags: SaFlags, mask: SigSet) -> SigAction {
        #[cfg(target_os = "redox")]
        unsafe fn install_sig(p: *mut libc::sigaction, handler: SigHandler) {
            (*p).sa_handler = match handler {
                SigHandler::SigDfl => libc::SIG_DFL,
                SigHandler::SigIgn => libc::SIG_IGN,
                SigHandler::Handler(f) => f as *const extern fn(libc::c_int) as usize,
            };
        }

        #[cfg(not(target_os = "redox"))]
        unsafe fn install_sig(p: *mut libc::sigaction, handler: SigHandler) {
            (*p).sa_sigaction = match handler {
                SigHandler::SigDfl => libc::SIG_DFL,
                SigHandler::SigIgn => libc::SIG_IGN,
                SigHandler::Handler(f) => f as *const extern fn(libc::c_int) as usize,
                SigHandler::SigAction(f) => f as *const extern fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void) as usize,
            };
        }

        let mut s = mem::MaybeUninit::<libc::sigaction>::uninit();
        unsafe {
            let p = s.as_mut_ptr();
            install_sig(p, handler);
            (*p).sa_flags = match handler {
                #[cfg(not(target_os = "redox"))]
                SigHandler::SigAction(_) => (flags | SaFlags::SA_SIGINFO).bits(),
                _ => (flags - SaFlags::SA_SIGINFO).bits(),
            };
            (*p).sa_mask = mask.sigset;

            SigAction { sigaction: s.assume_init() }
        }
    }

    /// Returns the flags set on the action.
    pub fn flags(&self) -> SaFlags {
        SaFlags::from_bits_truncate(self.sigaction.sa_flags)
    }

    /// Returns the set of signals that are blocked during execution of the action's
    /// signal-catching function.
    pub fn mask(&self) -> SigSet {
        SigSet { sigset: self.sigaction.sa_mask }
    }

    /// Returns the action's handler.
    #[cfg(not(target_os = "redox"))]
    pub fn handler(&self) -> SigHandler {
        match self.sigaction.sa_sigaction {
            libc::SIG_DFL => SigHandler::SigDfl,
            libc::SIG_IGN => SigHandler::SigIgn,
            p if self.flags().contains(SaFlags::SA_SIGINFO) =>
                SigHandler::SigAction(
                // Safe for one of two reasons:
                // * The SigHandler was created by SigHandler::new, in which
                //   case the pointer is correct, or
                // * The SigHandler was created by signal or sigaction, which
                //   are unsafe functions, so the caller should've somehow
                //   ensured that it is correctly initialized.
                unsafe{
                    *(&p as *const usize
                         as *const extern fn(_, _, _))
                }
                as extern fn(_, _, _)),
            p => SigHandler::Handler(
                // Safe for one of two reasons:
                // * The SigHandler was created by SigHandler::new, in which
                //   case the pointer is correct, or
                // * The SigHandler was created by signal or sigaction, which
                //   are unsafe functions, so the caller should've somehow
                //   ensured that it is correctly initialized.
                unsafe{
                    *(&p as *const usize
                         as *const extern fn(libc::c_int))
                }
                as extern fn(libc::c_int)),
        }
    }

    /// Returns the action's handler.
    #[cfg(target_os = "redox")]
    pub fn handler(&self) -> SigHandler {
        match self.sigaction.sa_handler {
            libc::SIG_DFL => SigHandler::SigDfl,
            libc::SIG_IGN => SigHandler::SigIgn,
            p => SigHandler::Handler(
                // Safe for one of two reasons:
                // * The SigHandler was created by SigHandler::new, in which
                //   case the pointer is correct, or
                // * The SigHandler was created by signal or sigaction, which
                //   are unsafe functions, so the caller should've somehow
                //   ensured that it is correctly initialized.
                unsafe{
                    *(&p as *const usize
                         as *const extern fn(libc::c_int))
                }
                as extern fn(libc::c_int)),
        }
    }
}

/// Changes the action taken by a process on receipt of a specific signal.
///
/// `signal` can be any signal except `SIGKILL` or `SIGSTOP`. On success, it returns the previous
/// action for the given signal. If `sigaction` fails, no new signal handler is installed.
///
/// # Safety
///
/// * Signal handlers may be called at any point during execution, which limits
///   what is safe to do in the body of the signal-catching function. Be certain
///   to only make syscalls that are explicitly marked safe for signal handlers
///   and only share global data using atomics.
///
/// * There is also no guarantee that the old signal handler was installed
///   correctly.  If it was installed by this crate, it will be.  But if it was
///   installed by, for example, C code, then there is no guarantee its function
///   pointer is valid.  In that case, this function effectively dereferences a
///   raw pointer of unknown provenance.
pub unsafe fn sigaction(signal: Signal, sigaction: &SigAction) -> Result<SigAction> {
    let mut oldact = mem::MaybeUninit::<libc::sigaction>::uninit();

    let res = libc::sigaction(signal as libc::c_int,
                              &sigaction.sigaction as *const libc::sigaction,
                              oldact.as_mut_ptr());

    Errno::result(res).map(|_| SigAction { sigaction: oldact.assume_init() })
}

/// Signal management (see [signal(3p)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/signal.html))
///
/// Installs `handler` for the given `signal`, returning the previous signal
/// handler. `signal` should only be used following another call to `signal` or
/// if the current handler is the default. The return value of `signal` is
/// undefined after setting the handler with [`sigaction`][SigActionFn].
///
/// # Safety
///
/// If the pointer to the previous signal handler is invalid, undefined
/// behavior could be invoked when casting it back to a [`SigAction`][SigActionStruct].
///
/// # Examples
///
/// Ignore `SIGINT`:
///
/// ```no_run
/// # use nix::sys::signal::{self, Signal, SigHandler};
/// unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();
/// ```
///
/// Use a signal handler to set a flag variable:
///
/// ```no_run
/// # #[macro_use] extern crate lazy_static;
/// # use std::convert::TryFrom;
/// # use std::sync::atomic::{AtomicBool, Ordering};
/// # use nix::sys::signal::{self, Signal, SigHandler};
/// lazy_static! {
///    static ref SIGNALED: AtomicBool = AtomicBool::new(false);
/// }
///
/// extern fn handle_sigint(signal: libc::c_int) {
///     let signal = Signal::try_from(signal).unwrap();
///     SIGNALED.store(signal == Signal::SIGINT, Ordering::Relaxed);
/// }
///
/// fn main() {
///     let handler = SigHandler::Handler(handle_sigint);
///     unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();
/// }
/// ```
///
/// # Errors
///
/// Returns [`Error(Errno::EOPNOTSUPP)`] if `handler` is
/// [`SigAction`][SigActionStruct]. Use [`sigaction`][SigActionFn] instead.
///
/// `signal` also returns any error from `libc::signal`, such as when an attempt
/// is made to catch a signal that cannot be caught or to ignore a signal that
/// cannot be ignored.
///
/// [`Error::UnsupportedOperation`]: ../../enum.Error.html#variant.UnsupportedOperation
/// [SigActionStruct]: struct.SigAction.html
/// [sigactionFn]: fn.sigaction.html
pub unsafe fn signal(signal: Signal, handler: SigHandler) -> Result<SigHandler> {
    let signal = signal as libc::c_int;
    let res = match handler {
        SigHandler::SigDfl => libc::signal(signal, libc::SIG_DFL),
        SigHandler::SigIgn => libc::signal(signal, libc::SIG_IGN),
        SigHandler::Handler(handler) => libc::signal(signal, handler as libc::sighandler_t),
        #[cfg(not(target_os = "redox"))]
        SigHandler::SigAction(_) => return Err(Errno::ENOTSUP),
    };
    Errno::result(res).map(|oldhandler| {
        match oldhandler {
            libc::SIG_DFL => SigHandler::SigDfl,
            libc::SIG_IGN => SigHandler::SigIgn,
            p => SigHandler::Handler(
                *(&p as *const usize
                     as *const extern fn(libc::c_int))
                as extern fn(libc::c_int)),
        }
    })
}

fn do_pthread_sigmask(how: SigmaskHow,
                       set: Option<&SigSet>,
                       oldset: Option<*mut libc::sigset_t>) -> Result<()> {
    if set.is_none() && oldset.is_none() {
        return Ok(())
    }

    let res = unsafe {
        // if set or oldset is None, pass in null pointers instead
        libc::pthread_sigmask(how as libc::c_int,
                             set.map_or_else(ptr::null::<libc::sigset_t>,
                                             |s| &s.sigset as *const libc::sigset_t),
                             oldset.unwrap_or(ptr::null_mut())
                             )
    };

    Errno::result(res).map(drop)
}

/// Manages the signal mask (set of blocked signals) for the calling thread.
///
/// If the `set` parameter is `Some(..)`, then the signal mask will be updated with the signal set.
/// The `how` flag decides the type of update. If `set` is `None`, `how` will be ignored,
/// and no modification will take place.
///
/// If the 'oldset' parameter is `Some(..)` then the current signal mask will be written into it.
///
/// If both `set` and `oldset` is `Some(..)`, the current signal mask will be written into oldset,
/// and then it will be updated with `set`.
///
/// If both `set` and `oldset` is None, this function is a no-op.
///
/// For more information, visit the [`pthread_sigmask`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/pthread_sigmask.html),
/// or [`sigprocmask`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sigprocmask.html) man pages.
pub fn pthread_sigmask(how: SigmaskHow,
                       set: Option<&SigSet>,
                       oldset: Option<&mut SigSet>) -> Result<()>
{
    do_pthread_sigmask(how, set, oldset.map(|os| &mut os.sigset as *mut _ ))
}

/// Examine and change blocked signals.
///
/// For more informations see the [`sigprocmask` man
/// pages](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sigprocmask.html).
pub fn sigprocmask(how: SigmaskHow, set: Option<&SigSet>, oldset: Option<&mut SigSet>) -> Result<()> {
    if set.is_none() && oldset.is_none() {
        return Ok(())
    }

    let res = unsafe {
        // if set or oldset is None, pass in null pointers instead
        libc::sigprocmask(how as libc::c_int,
                          set.map_or_else(ptr::null::<libc::sigset_t>,
                                          |s| &s.sigset as *const libc::sigset_t),
                          oldset.map_or_else(ptr::null_mut::<libc::sigset_t>,
                                             |os| &mut os.sigset as *mut libc::sigset_t))
    };

    Errno::result(res).map(drop)
}

/// Send a signal to a process
///
/// # Arguments
///
/// * `pid` -    Specifies which processes should receive the signal.
///   - If positive, specifies an individual process
///   - If zero, the signal will be sent to all processes whose group
///     ID is equal to the process group ID of the sender.  This is a
///     variant of [`killpg`].
///   - If `-1` and the process has super-user privileges, the signal
///     is sent to all processes exclusing system processes.
///   - If less than `-1`, the signal is sent to all processes whose
///     process group ID is equal to the absolute value of `pid`.
/// * `signal` - Signal to send. If `None`, error checking is performed
///              but no signal is actually sent.
///
/// See Also
/// [`kill(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/kill.html)
pub fn kill<T: Into<Option<Signal>>>(pid: Pid, signal: T) -> Result<()> {
    let res = unsafe { libc::kill(pid.into(),
                                  match signal.into() {
                                      Some(s) => s as libc::c_int,
                                      None => 0,
                                  }) };

    Errno::result(res).map(drop)
}

/// Send a signal to a process group
///
/// # Arguments
///
/// * `pgrp` -   Process group to signal.  If less then or equal 1, the behavior
///              is platform-specific.
/// * `signal` - Signal to send. If `None`, `killpg` will only preform error
///              checking and won't send any signal.
///
/// See Also [killpg(3)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/killpg.html).
#[cfg(not(target_os = "fuchsia"))]
pub fn killpg<T: Into<Option<Signal>>>(pgrp: Pid, signal: T) -> Result<()> {
    let res = unsafe { libc::killpg(pgrp.into(),
                                  match signal.into() {
                                      Some(s) => s as libc::c_int,
                                      None => 0,
                                  }) };

    Errno::result(res).map(drop)
}

/// Send a signal to the current thread
///
/// See Also [raise(3)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/raise.html)
pub fn raise(signal: Signal) -> Result<()> {
    let res = unsafe { libc::raise(signal as libc::c_int) };

    Errno::result(res).map(drop)
}


/// Identifies a thread for [`SigevNotify::SigevThreadId`]
#[cfg(target_os = "freebsd")]
pub type type_of_thread_id = libc::lwpid_t;
/// Identifies a thread for [`SigevNotify::SigevThreadId`]
#[cfg(target_os = "linux")]
pub type type_of_thread_id = libc::pid_t;

/// Specifies the notification method used by a [`SigEvent`]
// sigval is actually a union of a int and a void*.  But it's never really used
// as a pointer, because neither libc nor the kernel ever dereference it.  nix
// therefore presents it as an intptr_t, which is how kevent uses it.
#[cfg(not(any(target_os = "openbsd", target_os = "redox")))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SigevNotify {
    /// No notification will be delivered
    SigevNone,
    /// Notify by delivering a signal to the process.
    SigevSignal {
        /// Signal to deliver
        signal: Signal,
        /// Will be present in the `si_value` field of the [`libc::siginfo_t`]
        /// structure of the queued signal.
        si_value: libc::intptr_t
    },
    // Note: SIGEV_THREAD is not implemented because libc::sigevent does not
    // expose a way to set the union members needed by SIGEV_THREAD.
    /// Notify by delivering an event to a kqueue.
    #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
    SigevKevent {
        /// File descriptor of the kqueue to notify.
        kq: RawFd,
        /// Will be contained in the kevent's `udata` field.
        udata: libc::intptr_t
    },
    /// Notify by delivering a signal to a thread.
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    SigevThreadId {
        /// Signal to send
        signal: Signal,
        /// LWP ID of the thread to notify
        thread_id: type_of_thread_id,
        /// Will be present in the `si_value` field of the [`libc::siginfo_t`]
        /// structure of the queued signal.
        si_value: libc::intptr_t
    },
}

#[cfg(not(any(target_os = "openbsd", target_os = "redox")))]
mod sigevent {
    use std::mem;
    use std::ptr;
    use super::SigevNotify;
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    use super::type_of_thread_id;

    /// Used to request asynchronous notification of the completion of certain
    /// events, such as POSIX AIO and timers.
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct SigEvent {
        sigevent: libc::sigevent
    }

    impl SigEvent {
        /// **Note:** this constructor does not allow the user to set the
        /// `sigev_notify_kevent_flags` field.  That's considered ok because on FreeBSD
        /// at least those flags don't do anything useful.  That field is part of a
        /// union that shares space with the more genuinely useful fields.
        ///
        /// **Note:** This constructor also doesn't allow the caller to set the
        /// `sigev_notify_function` or `sigev_notify_attributes` fields, which are
        /// required for `SIGEV_THREAD`.  That's considered ok because on no operating
        /// system is `SIGEV_THREAD` the most efficient way to deliver AIO
        /// notification.  FreeBSD and DragonFly BSD programs should prefer `SIGEV_KEVENT`.
        /// Linux, Solaris, and portable programs should prefer `SIGEV_THREAD_ID` or
        /// `SIGEV_SIGNAL`.  That field is part of a union that shares space with the
        /// more genuinely useful `sigev_notify_thread_id`
        // Allow invalid_value warning on Fuchsia only.
        // See https://github.com/nix-rust/nix/issues/1441
        #[cfg_attr(target_os = "fuchsia", allow(invalid_value))]
        pub fn new(sigev_notify: SigevNotify) -> SigEvent {
            let mut sev = unsafe { mem::MaybeUninit::<libc::sigevent>::zeroed().assume_init() };
            sev.sigev_notify = match sigev_notify {
                SigevNotify::SigevNone => libc::SIGEV_NONE,
                SigevNotify::SigevSignal{..} => libc::SIGEV_SIGNAL,
                #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
                SigevNotify::SigevKevent{..} => libc::SIGEV_KEVENT,
                #[cfg(target_os = "freebsd")]
                SigevNotify::SigevThreadId{..} => libc::SIGEV_THREAD_ID,
                #[cfg(all(target_os = "linux", target_env = "gnu", not(target_arch = "mips")))]
                SigevNotify::SigevThreadId{..} => libc::SIGEV_THREAD_ID,
                #[cfg(any(all(target_os = "linux", target_env = "musl"), target_arch = "mips"))]
                SigevNotify::SigevThreadId{..} => 4  // No SIGEV_THREAD_ID defined
            };
            sev.sigev_signo = match sigev_notify {
                SigevNotify::SigevSignal{ signal, .. } => signal as libc::c_int,
                #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
                SigevNotify::SigevKevent{ kq, ..} => kq,
                #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                SigevNotify::SigevThreadId{ signal, .. } => signal as libc::c_int,
                _ => 0
            };
            sev.sigev_value.sival_ptr = match sigev_notify {
                SigevNotify::SigevNone => ptr::null_mut::<libc::c_void>(),
                SigevNotify::SigevSignal{ si_value, .. } => si_value as *mut libc::c_void,
                #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
                SigevNotify::SigevKevent{ udata, .. } => udata as *mut libc::c_void,
                #[cfg(any(target_os = "freebsd", target_os = "linux"))]
                SigevNotify::SigevThreadId{ si_value, .. } => si_value as *mut libc::c_void,
            };
            SigEvent::set_tid(&mut sev, &sigev_notify);
            SigEvent{sigevent: sev}
        }

        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        fn set_tid(sev: &mut libc::sigevent, sigev_notify: &SigevNotify) {
            sev.sigev_notify_thread_id = match *sigev_notify {
                SigevNotify::SigevThreadId { thread_id, .. } => thread_id,
                _ => 0 as type_of_thread_id
            };
        }

        #[cfg(not(any(target_os = "freebsd", target_os = "linux")))]
        fn set_tid(_sev: &mut libc::sigevent, _sigev_notify: &SigevNotify) {
        }

        /// Return a copy of the inner structure
        pub fn sigevent(&self) -> libc::sigevent {
            self.sigevent
        }
    }

    impl<'a> From<&'a libc::sigevent> for SigEvent {
        fn from(sigevent: &libc::sigevent) -> Self {
            SigEvent{ sigevent: *sigevent }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "redox"))]
    use std::thread;
    use super::*;

    #[test]
    fn test_contains() {
        let mut mask = SigSet::empty();
        mask.add(SIGUSR1);

        assert!(mask.contains(SIGUSR1));
        assert!(!mask.contains(SIGUSR2));

        let all = SigSet::all();
        assert!(all.contains(SIGUSR1));
        assert!(all.contains(SIGUSR2));
    }

    #[test]
    fn test_clear() {
        let mut set = SigSet::all();
        set.clear();
        for signal in Signal::iterator() {
            assert!(!set.contains(signal));
        }
    }

    #[test]
    fn test_from_str_round_trips() {
        for signal in Signal::iterator() {
            assert_eq!(signal.as_ref().parse::<Signal>().unwrap(), signal);
            assert_eq!(signal.to_string().parse::<Signal>().unwrap(), signal);
        }
    }

    #[test]
    fn test_from_str_invalid_value() {
        let errval = Err(Errno::EINVAL);
        assert_eq!("NOSIGNAL".parse::<Signal>(), errval);
        assert_eq!("kill".parse::<Signal>(), errval);
        assert_eq!("9".parse::<Signal>(), errval);
    }

    #[test]
    fn test_extend() {
        let mut one_signal = SigSet::empty();
        one_signal.add(SIGUSR1);

        let mut two_signals = SigSet::empty();
        two_signals.add(SIGUSR2);
        two_signals.extend(&one_signal);

        assert!(two_signals.contains(SIGUSR1));
        assert!(two_signals.contains(SIGUSR2));
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_thread_signal_set_mask() {
        thread::spawn(|| {
            let prev_mask = SigSet::thread_get_mask()
                .expect("Failed to get existing signal mask!");

            let mut test_mask = prev_mask;
            test_mask.add(SIGUSR1);

            assert!(test_mask.thread_set_mask().is_ok());
            let new_mask = SigSet::thread_get_mask()
                .expect("Failed to get new mask!");

            assert!(new_mask.contains(SIGUSR1));
            assert!(!new_mask.contains(SIGUSR2));

            prev_mask.thread_set_mask().expect("Failed to revert signal mask!");
        }).join().unwrap();
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_thread_signal_block() {
        thread::spawn(|| {
            let mut mask = SigSet::empty();
            mask.add(SIGUSR1);

            assert!(mask.thread_block().is_ok());

            assert!(SigSet::thread_get_mask().unwrap().contains(SIGUSR1));
        }).join().unwrap();
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_thread_signal_unblock() {
        thread::spawn(|| {
            let mut mask = SigSet::empty();
            mask.add(SIGUSR1);

            assert!(mask.thread_unblock().is_ok());

            assert!(!SigSet::thread_get_mask().unwrap().contains(SIGUSR1));
        }).join().unwrap();
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_thread_signal_swap() {
        thread::spawn(|| {
            let mut mask = SigSet::empty();
            mask.add(SIGUSR1);
            mask.thread_block().unwrap();

            assert!(SigSet::thread_get_mask().unwrap().contains(SIGUSR1));

            let mut mask2 = SigSet::empty();
            mask2.add(SIGUSR2);

            let oldmask = mask2.thread_swap_mask(SigmaskHow::SIG_SETMASK)
                .unwrap();

            assert!(oldmask.contains(SIGUSR1));
            assert!(!oldmask.contains(SIGUSR2));

            assert!(SigSet::thread_get_mask().unwrap().contains(SIGUSR2));
        }).join().unwrap();
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_sigaction() {
        thread::spawn(|| {
            extern fn test_sigaction_handler(_: libc::c_int) {}
            extern fn test_sigaction_action(_: libc::c_int,
                _: *mut libc::siginfo_t, _: *mut libc::c_void) {}

            let handler_sig = SigHandler::Handler(test_sigaction_handler);

            let flags = SaFlags::SA_ONSTACK | SaFlags::SA_RESTART |
                        SaFlags::SA_SIGINFO;

            let mut mask = SigSet::empty();
            mask.add(SIGUSR1);

            let action_sig = SigAction::new(handler_sig, flags, mask);

            assert_eq!(action_sig.flags(),
                       SaFlags::SA_ONSTACK | SaFlags::SA_RESTART);
            assert_eq!(action_sig.handler(), handler_sig);

            mask = action_sig.mask();
            assert!(mask.contains(SIGUSR1));
            assert!(!mask.contains(SIGUSR2));

            let handler_act = SigHandler::SigAction(test_sigaction_action);
            let action_act = SigAction::new(handler_act, flags, mask);
            assert_eq!(action_act.handler(), handler_act);

            let action_dfl = SigAction::new(SigHandler::SigDfl, flags, mask);
            assert_eq!(action_dfl.handler(), SigHandler::SigDfl);

            let action_ign = SigAction::new(SigHandler::SigIgn, flags, mask);
            assert_eq!(action_ign.handler(), SigHandler::SigIgn);
        }).join().unwrap();
    }

    #[test]
    #[cfg(not(target_os = "redox"))]
    fn test_sigwait() {
        thread::spawn(|| {
            let mut mask = SigSet::empty();
            mask.add(SIGUSR1);
            mask.add(SIGUSR2);
            mask.thread_block().unwrap();

            raise(SIGUSR1).unwrap();
            assert_eq!(mask.wait().unwrap(), SIGUSR1);
        }).join().unwrap();
    }
}
