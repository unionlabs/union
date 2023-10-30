use crate::backend::c;

/// A signal number for use with [`kill_process`], [`kill_process_group`],
/// and [`kill_current_process_group`].
///
/// [`kill_process`]: crate::process::kill_process
/// [`kill_process_group`]: crate::process::kill_process_group
/// [`kill_current_process_group`]: crate::process::kill_current_process_group
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Signal {
    /// `SIGHUP`
    Hup = c::SIGHUP,
    /// `SIGINT`
    Int = c::SIGINT,
    /// `SIGQUIT`
    Quit = c::SIGQUIT,
    /// `SIGILL`
    Ill = c::SIGILL,
    /// `SIGTRAP`
    Trap = c::SIGTRAP,
    /// `SIGABRT`, aka `SIGIOT`
    #[doc(alias = "Iot")]
    #[doc(alias = "Abrt")]
    Abort = c::SIGABRT,
    /// `SIGBUS`
    Bus = c::SIGBUS,
    /// `SIGFPE`
    Fpe = c::SIGFPE,
    /// `SIGKILL`
    Kill = c::SIGKILL,
    /// `SIGUSR1`
    Usr1 = c::SIGUSR1,
    /// `SIGSEGV`
    Segv = c::SIGSEGV,
    /// `SIGUSR2`
    Usr2 = c::SIGUSR2,
    /// `SIGPIPE`
    Pipe = c::SIGPIPE,
    /// `SIGALRM`
    #[doc(alias = "Alrm")]
    Alarm = c::SIGALRM,
    /// `SIGTERM`
    Term = c::SIGTERM,
    /// `SIGSTKFLT`
    #[cfg(not(any(
        bsd,
        solarish,
        target_os = "aix",
        target_os = "haiku",
        all(
            linux_kernel,
            any(
                target_arch = "mips",
                target_arch = "mips64",
                target_arch = "sparc",
                target_arch = "sparc64"
            ),
        )
    )))]
    Stkflt = c::SIGSTKFLT,
    /// `SIGCHLD`
    #[doc(alias = "Chld")]
    Child = c::SIGCHLD,
    /// `SIGCONT`
    Cont = c::SIGCONT,
    /// `SIGSTOP`
    Stop = c::SIGSTOP,
    /// `SIGTSTP`
    Tstp = c::SIGTSTP,
    /// `SIGTTIN`
    Ttin = c::SIGTTIN,
    /// `SIGTTOU`
    Ttou = c::SIGTTOU,
    /// `SIGURG`
    Urg = c::SIGURG,
    /// `SIGXCPU`
    Xcpu = c::SIGXCPU,
    /// `SIGXFSZ`
    Xfsz = c::SIGXFSZ,
    /// `SIGVTALRM`
    #[doc(alias = "Vtalrm")]
    Vtalarm = c::SIGVTALRM,
    /// `SIGPROF`
    Prof = c::SIGPROF,
    /// `SIGWINCH`
    Winch = c::SIGWINCH,
    /// `SIGIO`, aka `SIGPOLL`
    #[doc(alias = "Poll")]
    #[cfg(not(target_os = "haiku"))]
    Io = c::SIGIO,
    /// `SIGPWR`
    #[cfg(not(any(bsd, target_os = "haiku")))]
    #[doc(alias = "Pwr")]
    Power = c::SIGPWR,
    /// `SIGSYS`, aka `SIGUNUSED`
    #[doc(alias = "Unused")]
    Sys = c::SIGSYS,
    /// `SIGEMT`
    #[cfg(any(
        bsd,
        solarish,
        target_os = "aix",
        target_os = "hermit",
        all(
            linux_kernel,
            any(
                target_arch = "mips",
                target_arch = "mips64",
                target_arch = "sparc",
                target_arch = "sparc64"
            )
        )
    ))]
    Emt = c::SIGEMT,
    /// `SIGINFO`
    #[cfg(bsd)]
    Info = c::SIGINFO,
    /// `SIGTHR`
    #[cfg(target_os = "freebsd")]
    #[doc(alias = "Lwp")]
    Thr = c::SIGTHR,
    /// `SIGLIBRT`
    #[cfg(target_os = "freebsd")]
    Librt = c::SIGLIBRT,
}

impl Signal {
    /// Convert a raw signal number into a `Signal`, if possible.
    pub fn from_raw(sig: c::c_int) -> Option<Self> {
        match sig {
            c::SIGHUP => Some(Self::Hup),
            c::SIGINT => Some(Self::Int),
            c::SIGQUIT => Some(Self::Quit),
            c::SIGILL => Some(Self::Ill),
            c::SIGTRAP => Some(Self::Trap),
            c::SIGABRT => Some(Self::Abort),
            c::SIGBUS => Some(Self::Bus),
            c::SIGFPE => Some(Self::Fpe),
            c::SIGKILL => Some(Self::Kill),
            c::SIGUSR1 => Some(Self::Usr1),
            c::SIGSEGV => Some(Self::Segv),
            c::SIGUSR2 => Some(Self::Usr2),
            c::SIGPIPE => Some(Self::Pipe),
            c::SIGALRM => Some(Self::Alarm),
            c::SIGTERM => Some(Self::Term),
            #[cfg(not(any(
                bsd,
                solarish,
                target_os = "aix",
                target_os = "haiku",
                all(
                    linux_kernel,
                    any(
                        target_arch = "mips",
                        target_arch = "mips64",
                        target_arch = "sparc",
                        target_arch = "sparc64"
                    ),
                )
            )))]
            c::SIGSTKFLT => Some(Self::Stkflt),
            c::SIGCHLD => Some(Self::Child),
            c::SIGCONT => Some(Self::Cont),
            c::SIGSTOP => Some(Self::Stop),
            c::SIGTSTP => Some(Self::Tstp),
            c::SIGTTIN => Some(Self::Ttin),
            c::SIGTTOU => Some(Self::Ttou),
            c::SIGURG => Some(Self::Urg),
            c::SIGXCPU => Some(Self::Xcpu),
            c::SIGXFSZ => Some(Self::Xfsz),
            c::SIGVTALRM => Some(Self::Vtalarm),
            c::SIGPROF => Some(Self::Prof),
            c::SIGWINCH => Some(Self::Winch),
            #[cfg(not(target_os = "haiku"))]
            c::SIGIO => Some(Self::Io),
            #[cfg(not(any(bsd, target_os = "haiku")))]
            c::SIGPWR => Some(Self::Power),
            c::SIGSYS => Some(Self::Sys),
            #[cfg(any(
                bsd,
                solarish,
                target_os = "aix",
                target_os = "hermit",
                all(
                    linux_kernel,
                    any(
                        target_arch = "mips",
                        target_arch = "mips64",
                        target_arch = "sparc",
                        target_arch = "sparc64"
                    )
                )
            ))]
            c::SIGEMT => Some(Self::Emt),
            #[cfg(bsd)]
            c::SIGINFO => Some(Self::Info),
            #[cfg(target_os = "freebsd")]
            c::SIGTHR => Some(Self::Thr),
            #[cfg(target_os = "freebsd")]
            c::SIGLIBRT => Some(Self::Librt),
            _ => None,
        }
    }
}

#[test]
fn test_sizes() {
    use core::mem::size_of;

    assert_eq!(size_of::<Signal>(), size_of::<c::c_int>());
}
