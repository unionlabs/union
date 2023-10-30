use super::super::c;
use linux_raw_sys::general::membarrier_cmd;

/// `sysinfo`
pub type Sysinfo = linux_raw_sys::general::sysinfo;

/// A command for use with [`membarrier`] and [`membarrier_cpu`].
///
/// For `MEMBARRIER_CMD_QUERY`, see [`membarrier_query`].
///
/// [`membarrier`]: crate::process::membarrier
/// [`membarrier_cpu`]: crate::process::membarrier_cpu
/// [`membarrier_query`]: crate::process::membarrier_query
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum MembarrierCommand {
    /// `MEMBARRIER_CMD_GLOBAL`
    #[doc(alias = "Shared")]
    #[doc(alias = "MEMBARRIER_CMD_SHARED")]
    Global = membarrier_cmd::MEMBARRIER_CMD_GLOBAL as _,
    /// `MEMBARRIER_CMD_GLOBAL_EXPEDITED`
    GlobalExpedited = membarrier_cmd::MEMBARRIER_CMD_GLOBAL_EXPEDITED as _,
    /// `MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED`
    RegisterGlobalExpedited = membarrier_cmd::MEMBARRIER_CMD_REGISTER_GLOBAL_EXPEDITED as _,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED`
    PrivateExpedited = membarrier_cmd::MEMBARRIER_CMD_PRIVATE_EXPEDITED as _,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED`
    RegisterPrivateExpedited = membarrier_cmd::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED as _,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE`
    PrivateExpeditedSyncCore = membarrier_cmd::MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE as _,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE`
    RegisterPrivateExpeditedSyncCore =
        membarrier_cmd::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE as _,
    /// `MEMBARRIER_CMD_PRIVATE_EXPEDITED_RSEQ` (since Linux 5.10)
    PrivateExpeditedRseq = membarrier_cmd::MEMBARRIER_CMD_PRIVATE_EXPEDITED_RSEQ as _,
    /// `MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_RSEQ` (since Linux 5.10)
    RegisterPrivateExpeditedRseq =
        membarrier_cmd::MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_RSEQ as _,
}

/// A resource value for use with [`getrlimit`], [`setrlimit`], and
/// [`prlimit`].
///
/// [`getrlimit`]: crate::process::getrlimit
/// [`setrlimit`]: crate::process::setrlimit
/// [`prlimit`]: crate::process::prlimit
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Resource {
    /// `RLIMIT_CPU`
    Cpu = linux_raw_sys::general::RLIMIT_CPU,
    /// `RLIMIT_FSIZE`
    Fsize = linux_raw_sys::general::RLIMIT_FSIZE,
    /// `RLIMIT_DATA`
    Data = linux_raw_sys::general::RLIMIT_DATA,
    /// `RLIMIT_STACK`
    Stack = linux_raw_sys::general::RLIMIT_STACK,
    /// `RLIMIT_CORE`
    Core = linux_raw_sys::general::RLIMIT_CORE,
    /// `RLIMIT_RSS`
    Rss = linux_raw_sys::general::RLIMIT_RSS,
    /// `RLIMIT_NPROC`
    Nproc = linux_raw_sys::general::RLIMIT_NPROC,
    /// `RLIMIT_NOFILE`
    Nofile = linux_raw_sys::general::RLIMIT_NOFILE,
    /// `RLIMIT_MEMLOCK`
    Memlock = linux_raw_sys::general::RLIMIT_MEMLOCK,
    /// `RLIMIT_AS`
    As = linux_raw_sys::general::RLIMIT_AS,
    /// `RLIMIT_LOCKS`
    Locks = linux_raw_sys::general::RLIMIT_LOCKS,
    /// `RLIMIT_SIGPENDING`
    Sigpending = linux_raw_sys::general::RLIMIT_SIGPENDING,
    /// `RLIMIT_MSGQUEUE`
    Msgqueue = linux_raw_sys::general::RLIMIT_MSGQUEUE,
    /// `RLIMIT_NICE`
    Nice = linux_raw_sys::general::RLIMIT_NICE,
    /// `RLIMIT_RTPRIO`
    Rtprio = linux_raw_sys::general::RLIMIT_RTPRIO,
    /// `RLIMIT_RTTIME`
    Rttime = linux_raw_sys::general::RLIMIT_RTTIME,
}

/// A signal number for use with [`kill_process`], [`kill_process_group`],
/// and [`kill_current_process_group`].
///
/// [`kill_process`]: crate::process::kill_process
/// [`kill_process_group`]: crate::process::kill_process_group
/// [`kill_current_process_group`]: crate::process::kill_current_process_group
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Signal {
    /// `SIGHUP`
    Hup = linux_raw_sys::general::SIGHUP,
    /// `SIGINT`
    Int = linux_raw_sys::general::SIGINT,
    /// `SIGQUIT`
    Quit = linux_raw_sys::general::SIGQUIT,
    /// `SIGILL`
    Ill = linux_raw_sys::general::SIGILL,
    /// `SIGTRAP`
    Trap = linux_raw_sys::general::SIGTRAP,
    /// `SIGABRT`, aka `SIGIOT`
    #[doc(alias = "Iot")]
    #[doc(alias = "Abrt")]
    Abort = linux_raw_sys::general::SIGABRT,
    /// `SIGEMT`
    #[cfg(any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "sparc",
        target_arch = "sparc64"
    ))]
    Emt = linux_raw_sys::general::SIGEMT,
    /// `SIGBUS`
    Bus = linux_raw_sys::general::SIGBUS,
    /// `SIGFPE`
    Fpe = linux_raw_sys::general::SIGFPE,
    /// `SIGKILL`
    Kill = linux_raw_sys::general::SIGKILL,
    /// `SIGUSR1`
    Usr1 = linux_raw_sys::general::SIGUSR1,
    /// `SIGSEGV`
    Segv = linux_raw_sys::general::SIGSEGV,
    /// `SIGUSR2`
    Usr2 = linux_raw_sys::general::SIGUSR2,
    /// `SIGPIPE`
    Pipe = linux_raw_sys::general::SIGPIPE,
    /// `SIGALRM`
    #[doc(alias = "Alrm")]
    Alarm = linux_raw_sys::general::SIGALRM,
    /// `SIGTERM`
    Term = linux_raw_sys::general::SIGTERM,
    /// `SIGSTKFLT`
    #[cfg(not(any(target_arch = "mips", target_arch = "mips64")))]
    Stkflt = linux_raw_sys::general::SIGSTKFLT,
    /// `SIGCHLD`
    #[doc(alias = "Chld")]
    Child = linux_raw_sys::general::SIGCHLD,
    /// `SIGCONT`
    Cont = linux_raw_sys::general::SIGCONT,
    /// `SIGSTOP`
    Stop = linux_raw_sys::general::SIGSTOP,
    /// `SIGTSTP`
    Tstp = linux_raw_sys::general::SIGTSTP,
    /// `SIGTTIN`
    Ttin = linux_raw_sys::general::SIGTTIN,
    /// `SIGTTOU`
    Ttou = linux_raw_sys::general::SIGTTOU,
    /// `SIGURG`
    Urg = linux_raw_sys::general::SIGURG,
    /// `SIGXCPU`
    Xcpu = linux_raw_sys::general::SIGXCPU,
    /// `SIGXFSZ`
    Xfsz = linux_raw_sys::general::SIGXFSZ,
    /// `SIGVTALRM`
    #[doc(alias = "Vtalrm")]
    Vtalarm = linux_raw_sys::general::SIGVTALRM,
    /// `SIGPROF`
    Prof = linux_raw_sys::general::SIGPROF,
    /// `SIGWINCH`
    Winch = linux_raw_sys::general::SIGWINCH,
    /// `SIGIO`, aka `SIGPOLL`
    #[doc(alias = "Poll")]
    Io = linux_raw_sys::general::SIGIO,
    /// `SIGPWR`
    #[doc(alias = "Pwr")]
    Power = linux_raw_sys::general::SIGPWR,
    /// `SIGSYS`, aka `SIGUNUSED`
    #[doc(alias = "Unused")]
    Sys = linux_raw_sys::general::SIGSYS,
    /// `SIGRTMIN`
    Rtmin = linux_raw_sys::general::SIGRTMIN,
}

impl Signal {
    /// Convert a raw signal number into a `Signal`, if possible.
    pub fn from_raw(sig: i32) -> Option<Self> {
        match sig as _ {
            linux_raw_sys::general::SIGHUP => Some(Self::Hup),
            linux_raw_sys::general::SIGINT => Some(Self::Int),
            linux_raw_sys::general::SIGQUIT => Some(Self::Quit),
            linux_raw_sys::general::SIGILL => Some(Self::Ill),
            linux_raw_sys::general::SIGTRAP => Some(Self::Trap),
            linux_raw_sys::general::SIGABRT => Some(Self::Abort),
            linux_raw_sys::general::SIGBUS => Some(Self::Bus),
            linux_raw_sys::general::SIGFPE => Some(Self::Fpe),
            linux_raw_sys::general::SIGKILL => Some(Self::Kill),
            linux_raw_sys::general::SIGUSR1 => Some(Self::Usr1),
            linux_raw_sys::general::SIGSEGV => Some(Self::Segv),
            linux_raw_sys::general::SIGUSR2 => Some(Self::Usr2),
            linux_raw_sys::general::SIGPIPE => Some(Self::Pipe),
            linux_raw_sys::general::SIGALRM => Some(Self::Alarm),
            linux_raw_sys::general::SIGTERM => Some(Self::Term),
            #[cfg(not(any(target_arch = "mips", target_arch = "mips64")))]
            linux_raw_sys::general::SIGSTKFLT => Some(Self::Stkflt),
            linux_raw_sys::general::SIGCHLD => Some(Self::Child),
            linux_raw_sys::general::SIGCONT => Some(Self::Cont),
            linux_raw_sys::general::SIGSTOP => Some(Self::Stop),
            linux_raw_sys::general::SIGTSTP => Some(Self::Tstp),
            linux_raw_sys::general::SIGTTIN => Some(Self::Ttin),
            linux_raw_sys::general::SIGTTOU => Some(Self::Ttou),
            linux_raw_sys::general::SIGURG => Some(Self::Urg),
            linux_raw_sys::general::SIGXCPU => Some(Self::Xcpu),
            linux_raw_sys::general::SIGXFSZ => Some(Self::Xfsz),
            linux_raw_sys::general::SIGVTALRM => Some(Self::Vtalarm),
            linux_raw_sys::general::SIGPROF => Some(Self::Prof),
            linux_raw_sys::general::SIGWINCH => Some(Self::Winch),
            linux_raw_sys::general::SIGIO => Some(Self::Io),
            linux_raw_sys::general::SIGPWR => Some(Self::Power),
            linux_raw_sys::general::SIGSYS => Some(Self::Sys),
            linux_raw_sys::general::SIGRTMIN => Some(Self::Rtmin),
            _ => None,
        }
    }
}

/// `EXIT_SUCCESS`
pub const EXIT_SUCCESS: c::c_int = 0;
/// `EXIT_FAILURE`
pub const EXIT_FAILURE: c::c_int = 1;
/// The status value of a child terminated with a [`Signal::Abort`] signal.
pub const EXIT_SIGNALED_SIGABRT: c::c_int = 128 + linux_raw_sys::general::SIGABRT as i32;

/// A process identifier as a raw integer.
pub type RawPid = u32;
/// A non-zero process identifier as a raw non-zero integer.
pub type RawNonZeroPid = core::num::NonZeroU32;
/// A group identifier as a raw integer.
pub type RawGid = u32;
/// A user identifier as a raw integer.
pub type RawUid = u32;
/// A CPU identifier as a raw integer.
pub type RawCpuid = u32;

pub(crate) type RawUname = linux_raw_sys::general::new_utsname;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RawCpuSet {
    #[cfg(all(target_pointer_width = "32", not(target_arch = "x86_64")))]
    pub(crate) bits: [u32; 32],
    #[cfg(not(all(target_pointer_width = "32", not(target_arch = "x86_64"))))]
    pub(crate) bits: [u64; 16],
}

#[inline]
pub(crate) fn raw_cpu_set_new() -> RawCpuSet {
    #[cfg(all(target_pointer_width = "32", not(target_arch = "x86_64")))]
    {
        RawCpuSet { bits: [0; 32] }
    }
    #[cfg(not(all(target_pointer_width = "32", not(target_arch = "x86_64"))))]
    {
        RawCpuSet { bits: [0; 16] }
    }
}

pub(crate) const CPU_SETSIZE: usize = 8 * core::mem::size_of::<RawCpuSet>();
