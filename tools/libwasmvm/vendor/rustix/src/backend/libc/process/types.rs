use super::super::c;

/// `sysinfo`
#[cfg(linux_kernel)]
pub type Sysinfo = c::sysinfo;

/// A command for use with [`membarrier`] and [`membarrier_cpu`].
///
/// For `MEMBARRIER_CMD_QUERY`, see [`membarrier_query`].
///
/// [`membarrier`]: crate::process::membarrier
/// [`membarrier_cpu`]: crate::process::membarrier_cpu
/// [`membarrier_query`]: crate::process::membarrier_query
#[cfg(linux_kernel)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum MembarrierCommand {
    /// `MEMBARRIER_CMD_GLOBAL`
    #[doc(alias = "Shared")]
    #[doc(alias = "MEMBARRIER_CMD_SHARED")]
    Global = c::MEMBARRIER_CMD_GLOBAL as u32,
    /// `MEMBARRIER_CMD_GLOBAL_EXPEDITED`
    GlobalExpedited = c::MEMBARRIER_CMD_GLOBAL_EXPEDITED as u32,
    /// `MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED`
    RegisterGlobalExpedited = c::MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED as u32,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED`
    PrivateExpedited = c::MEMBARRIER_CMD_PRIVATE_EXPEDITED as u32,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED`
    RegisterPrivateExpedited = c::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED as u32,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE`
    PrivateExpeditedSyncCore = c::MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE as u32,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE`
    RegisterPrivateExpeditedSyncCore =
        c::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE as u32,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED_RSEQ` (since Linux 5.10)
    PrivateExpeditedRseq = c::MEMBARRIER_CMD_PRIVATE_EXPEDITED_RSEQ as u32,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_RSEQ` (since Linux 5.10)
    RegisterPrivateExpeditedRseq = c::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_RSEQ as u32,
}

/// A resource value for use with [`getrlimit`], [`setrlimit`], and
/// [`prlimit`].
///
/// [`getrlimit`]: crate::process::getrlimit
/// [`setrlimit`]: crate::process::setrlimit
/// [`prlimit`]: crate::process::prlimit
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Resource {
    /// `RLIMIT_CPU`
    Cpu = c::RLIMIT_CPU as c::c_int,
    /// `RLIMIT_FSIZE`
    Fsize = c::RLIMIT_FSIZE as c::c_int,
    /// `RLIMIT_DATA`
    Data = c::RLIMIT_DATA as c::c_int,
    /// `RLIMIT_STACK`
    Stack = c::RLIMIT_STACK as c::c_int,
    /// `RLIMIT_CORE`
    #[cfg(not(target_os = "haiku"))]
    Core = c::RLIMIT_CORE as c::c_int,
    /// `RLIMIT_RSS`
    #[cfg(not(any(apple, solarish, target_os = "haiku")))]
    Rss = c::RLIMIT_RSS as c::c_int,
    /// `RLIMIT_NPROC`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    Nproc = c::RLIMIT_NPROC as c::c_int,
    /// `RLIMIT_NOFILE`
    Nofile = c::RLIMIT_NOFILE as c::c_int,
    /// `RLIMIT_MEMLOCK`
    #[cfg(not(any(solarish, target_os = "aix", target_os = "haiku")))]
    Memlock = c::RLIMIT_MEMLOCK as c::c_int,
    /// `RLIMIT_AS`
    #[cfg(not(target_os = "openbsd"))]
    As = c::RLIMIT_AS as c::c_int,
    /// `RLIMIT_LOCKS`
    #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
    Locks = c::RLIMIT_LOCKS as c::c_int,
    /// `RLIMIT_SIGPENDING`
    #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
    Sigpending = c::RLIMIT_SIGPENDING as c::c_int,
    /// `RLIMIT_MSGQUEUE`
    #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
    Msgqueue = c::RLIMIT_MSGQUEUE as c::c_int,
    /// `RLIMIT_NICE`
    #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
    Nice = c::RLIMIT_NICE as c::c_int,
    /// `RLIMIT_RTPRIO`
    #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
    Rtprio = c::RLIMIT_RTPRIO as c::c_int,
    /// `RLIMIT_RTTIME`
    #[cfg(not(any(
        bsd,
        solarish,
        target_os = "aix",
        target_os = "android",
        target_os = "emscripten",
        target_os = "haiku",
    )))]
    Rttime = c::RLIMIT_RTTIME as c::c_int,
}

#[cfg(apple)]
#[allow(non_upper_case_globals)]
impl Resource {
    /// `RLIMIT_RSS`
    pub const Rss: Self = Self::As;
}

/// A signal number for use with [`kill_process`], [`kill_process_group`],
/// and [`kill_current_process_group`].
///
/// [`kill_process`]: crate::process::kill_process
/// [`kill_process_group`]: crate::process::kill_process_group
/// [`kill_current_process_group`]: crate::process::kill_current_process_group
#[cfg(not(target_os = "wasi"))]
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
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "hermit"))]
    Emt = c::SIGEMT,
    /// `SIGEMT`
    #[cfg(all(
        linux_kernel,
        any(
            target_arch = "mips",
            target_arch = "mips64",
            target_arch = "sparc",
            target_arch = "sparc64"
        )
    ))]
    Emt = linux_raw_sys::general::SIGEMT as i32,
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

#[cfg(not(target_os = "wasi"))]
impl Signal {
    /// Convert a raw signal number into a `Signal`, if possible.
    pub fn from_raw(sig: i32) -> Option<Self> {
        match sig as _ {
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
            #[cfg(bsd)]
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

pub const EXIT_SUCCESS: c::c_int = c::EXIT_SUCCESS;
pub const EXIT_FAILURE: c::c_int = c::EXIT_FAILURE;
#[cfg(not(target_os = "wasi"))]
pub const EXIT_SIGNALED_SIGABRT: c::c_int = 128 + c::SIGABRT;

/// A process identifier as a raw integer.
#[cfg(not(target_os = "wasi"))]
pub type RawPid = c::pid_t;
/// A non-zero process identifier as a raw non-zero integer.
#[cfg(not(target_os = "wasi"))]
pub type RawNonZeroPid = core::num::NonZeroI32;
/// A group identifier as a raw integer.
#[cfg(not(target_os = "wasi"))]
pub type RawGid = c::gid_t;
/// A user identifier as a raw integer.
#[cfg(not(target_os = "wasi"))]
pub type RawUid = c::uid_t;
/// A CPU identifier as a raw integer.
#[cfg(linux_kernel)]
pub type RawCpuid = u32;
#[cfg(target_os = "freebsd")]
pub type RawId = c::id_t;

#[cfg(not(target_os = "wasi"))]
pub(crate) type RawUname = c::utsname;

#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
pub(crate) type RawCpuSet = c::cpu_set_t;

#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
#[inline]
pub(crate) fn raw_cpu_set_new() -> RawCpuSet {
    let mut set = unsafe { core::mem::zeroed() };
    super::cpu_set::CPU_ZERO(&mut set);
    set
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
pub(crate) const CPU_SETSIZE: usize = c::CPU_SETSIZE as usize;
#[cfg(target_os = "dragonfly")]
pub(crate) const CPU_SETSIZE: usize = 256;
