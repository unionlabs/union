use bitflags::bitflags as inner_bitflags;
use core::{mem, ops::Deref, slice};

macro_rules! bitflags {
    (
        $(#[$outer:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )+
        }
    ) => {
        // First, use the inner bitflags
        inner_bitflags! {
            #[derive(Default)]
            $(#[$outer])*
            pub struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )+
            }
        }

        // Secondly, re-export all inner constants
        // (`pub use self::Struct::*` doesn't work)
        $(
            $(#[$inner $($args)*])*
            pub const $Flag: $BitFlags = $BitFlags::$Flag;
        )+
    }
}

bitflags! {
    pub struct CloneFlags: usize {
        const CLONE_VM = 0x100;
        const CLONE_FS = 0x200;
        const CLONE_FILES = 0x400;
        const CLONE_SIGHAND = 0x800;
        const CLONE_VFORK = 0x4000;
        const CLONE_THREAD = 0x10000;
        const CLONE_STACK = 0x1000_0000;
    }
}

pub const CLOCK_REALTIME: usize = 1;
pub const CLOCK_MONOTONIC: usize = 4;

bitflags! {
    pub struct EventFlags: usize {
        const EVENT_NONE = 0;
        const EVENT_READ = 1;
        const EVENT_WRITE = 2;
    }
}

pub const F_DUPFD: usize = 0;
pub const F_GETFD: usize = 1;
pub const F_SETFD: usize = 2;
pub const F_GETFL: usize = 3;
pub const F_SETFL: usize = 4;

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
pub const FUTEX_REQUEUE: usize = 2;
pub const FUTEX_WAIT64: usize = 3;

bitflags! {
    pub struct MapFlags: usize {
        const PROT_NONE = 0x0000_0000;
        const PROT_EXEC = 0x0001_0000;
        const PROT_WRITE = 0x0002_0000;
        const PROT_READ = 0x0004_0000;

        const MAP_SHARED = 0x0001;
        const MAP_PRIVATE = 0x0002;

        /// Only accepted for mmap2(2).
        const MAP_FIXED = 0x0004;
        const MAP_FIXED_NOREPLACE = 0x000C;
    }
}

pub const MODE_TYPE: u16 = 0xF000;
pub const MODE_DIR: u16 = 0x4000;
pub const MODE_FILE: u16 = 0x8000;
pub const MODE_SYMLINK: u16 = 0xA000;
pub const MODE_FIFO: u16 = 0x1000;
pub const MODE_CHR: u16 = 0x2000;

pub const MODE_PERM: u16 = 0x0FFF;
pub const MODE_SETUID: u16 = 0o4000;
pub const MODE_SETGID: u16 = 0o2000;

pub const O_RDONLY: usize =     0x0001_0000;
pub const O_WRONLY: usize =     0x0002_0000;
pub const O_RDWR: usize =       0x0003_0000;
pub const O_NONBLOCK: usize =   0x0004_0000;
pub const O_APPEND: usize =     0x0008_0000;
pub const O_SHLOCK: usize =     0x0010_0000;
pub const O_EXLOCK: usize =     0x0020_0000;
pub const O_ASYNC: usize =      0x0040_0000;
pub const O_FSYNC: usize =      0x0080_0000;
pub const O_CLOEXEC: usize =    0x0100_0000;
pub const O_CREAT: usize =      0x0200_0000;
pub const O_TRUNC: usize =      0x0400_0000;
pub const O_EXCL: usize =       0x0800_0000;
pub const O_DIRECTORY: usize =  0x1000_0000;
pub const O_STAT: usize =       0x2000_0000;
pub const O_SYMLINK: usize =    0x4000_0000;
pub const O_NOFOLLOW: usize =   0x8000_0000;
pub const O_ACCMODE: usize =    O_RDONLY | O_WRONLY | O_RDWR;

bitflags! {
    pub struct PhysmapFlags: usize {
        const PHYSMAP_WRITE = 0x0000_0001;
        const PHYSMAP_WRITE_COMBINE = 0x0000_0002;
        const PHYSMAP_NO_CACHE = 0x0000_0004;
    }
}
bitflags! {
    /// Extra flags for [`physalloc2`] or [`physalloc3`].
    ///
    /// [`physalloc2`]: ../call/fn.physalloc2.html
    /// [`physalloc3`]: ../call/fn.physalloc3.html
    pub struct PhysallocFlags: usize {
        /// Only allocate memory within the 32-bit physical memory space. This is necessary for
        /// some devices may not support 64-bit memory.
        const SPACE_32 =        0x0000_0001;

        /// The frame that will be allocated, is going to reside anywhere in 64-bit space. This
        /// flag is redundant for the most part, except when overriding some other default.
        const SPACE_64 =        0x0000_0002;

        /// Do a "partial allocation", which means that not all of the frames specified in the
        /// frame count `size` actually have to be allocated. This means that if the allocator was
        /// unable to find a physical memory range large enough, it can instead return whatever
        /// range it decides is optimal. Thus, instead of letting one driver get an expensive
        /// 128MiB physical memory range when the physical memory has become fragmented, and
        /// failing, it can instead be given a more optimal range. If the device supports
        /// scatter-gather lists, then the driver only has to allocate more ranges, and the device
        /// will do vectored I/O.
        ///
        /// PARTIAL_ALLOC supports different allocation strategies, refer to
        /// [`Optimal`], [`GreatestRange`].
        ///
        /// [`Optimal`]: ./enum.PartialAllocStrategy.html
        /// [`GreatestRange`]: ./enum.PartialAllocStrategy.html
        const PARTIAL_ALLOC =   0x0000_0004;
    }
}

/// The bitmask of the partial allocation strategy. Currently four different strategies are
/// supported. If [`PARTIAL_ALLOC`] is not set, this bitmask is no longer reserved.
pub const PARTIAL_ALLOC_STRATEGY_MASK: usize = 0x0003_0000;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum PartialAllocStrategy {
    /// The allocator decides itself the size of the memory range, based on e.g. free memory ranges
    /// and other processes which require large physical memory chunks.
    Optimal = 0x0001_0000,

    /// The allocator returns the absolute greatest range it can find.
    GreatestRange = 0x0002_0000,

    /// The allocator returns the first range that fits the minimum count, without searching extra.
    Greedy = 0x0003_0000,
}
impl Default for PartialAllocStrategy {
    fn default() -> Self {
        Self::Optimal
    }
}

impl PartialAllocStrategy {
    pub fn from_raw(raw: usize) -> Option<Self> {
        match raw {
            0x0001_0000 => Some(Self::Optimal),
            0x0002_0000 => Some(Self::GreatestRange),
            0x0003_0000 => Some(Self::Greedy),
            _ => None,
        }
    }
}

// The top 48 bits of PTRACE_* are reserved, for now

bitflags! {
    pub struct PtraceFlags: u64 {
        /// Stop before a syscall is handled. Send PTRACE_FLAG_IGNORE to not
        /// handle the syscall.
        const PTRACE_STOP_PRE_SYSCALL = 0x0000_0000_0000_0001;
        /// Stop after a syscall is handled.
        const PTRACE_STOP_POST_SYSCALL = 0x0000_0000_0000_0002;
        /// Stop after exactly one instruction. TODO: This may not handle
        /// fexec/signal boundaries. Should it?
        const PTRACE_STOP_SINGLESTEP = 0x0000_0000_0000_0004;
        /// Stop before a signal is handled. Send PTRACE_FLAG_IGNORE to not
        /// handle signal.
        const PTRACE_STOP_SIGNAL = 0x0000_0000_0000_0008;
        /// Stop on a software breakpoint, such as the int3 instruction for
        /// x86_64.
        const PTRACE_STOP_BREAKPOINT = 0x0000_0000_0000_0010;
        /// Stop just before exiting for good.
        const PTRACE_STOP_EXIT = 0x0000_0000_0000_0020;

        const PTRACE_STOP_MASK = 0x0000_0000_0000_00FF;


        /// Sent when a child is cloned, giving you the opportunity to trace it.
        /// If you don't catch this, the child is started as normal.
        const PTRACE_EVENT_CLONE = 0x0000_0000_0000_0100;

        const PTRACE_EVENT_MASK = 0x0000_0000_0000_0F00;


        /// Special meaning, depending on the event. Usually, when fired before
        /// an action, it will skip performing that action.
        const PTRACE_FLAG_IGNORE = 0x0000_0000_0000_1000;

        const PTRACE_FLAG_MASK = 0x0000_0000_0000_F000;
    }
}
impl Deref for PtraceFlags {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        // Same as to_ne_bytes but in-place
        unsafe {
            slice::from_raw_parts(
                &self.bits as *const _ as *const u8,
                mem::size_of::<u64>()
            )
        }
    }
}

pub const SEEK_SET: usize = 0;
pub const SEEK_CUR: usize = 1;
pub const SEEK_END: usize = 2;

pub const SIGHUP: usize =   1;
pub const SIGINT: usize =   2;
pub const SIGQUIT: usize =  3;
pub const SIGILL: usize =   4;
pub const SIGTRAP: usize =  5;
pub const SIGABRT: usize =  6;
pub const SIGBUS: usize =   7;
pub const SIGFPE: usize =   8;
pub const SIGKILL: usize =  9;
pub const SIGUSR1: usize =  10;
pub const SIGSEGV: usize =  11;
pub const SIGUSR2: usize =  12;
pub const SIGPIPE: usize =  13;
pub const SIGALRM: usize =  14;
pub const SIGTERM: usize =  15;
pub const SIGSTKFLT: usize= 16;
pub const SIGCHLD: usize =  17;
pub const SIGCONT: usize =  18;
pub const SIGSTOP: usize =  19;
pub const SIGTSTP: usize =  20;
pub const SIGTTIN: usize =  21;
pub const SIGTTOU: usize =  22;
pub const SIGURG: usize =   23;
pub const SIGXCPU: usize =  24;
pub const SIGXFSZ: usize =  25;
pub const SIGVTALRM: usize= 26;
pub const SIGPROF: usize =  27;
pub const SIGWINCH: usize = 28;
pub const SIGIO: usize =    29;
pub const SIGPWR: usize =   30;
pub const SIGSYS: usize =   31;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;

pub const SIG_BLOCK: usize = 0;
pub const SIG_UNBLOCK: usize = 1;
pub const SIG_SETMASK: usize = 2;

bitflags! {
    pub struct SigActionFlags: usize {
        const SA_NOCLDSTOP = 0x00000001;
        const SA_NOCLDWAIT = 0x00000002;
        const SA_SIGINFO =   0x00000004;
        const SA_RESTORER =  0x04000000;
        const SA_ONSTACK =   0x08000000;
        const SA_RESTART =   0x10000000;
        const SA_NODEFER =   0x40000000;
        const SA_RESETHAND = 0x80000000;
    }
}

// Auxiliery vector types
pub const AT_NULL: usize = 0;
pub const AT_PHDR: usize = 3;
pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_ENTRY: usize = 9;

bitflags! {
    pub struct WaitFlags: usize {
        const WNOHANG =    0x01;
        const WUNTRACED =  0x02;
        const WCONTINUED = 0x08;
    }
}

/// True if status indicates the child is stopped.
pub fn wifstopped(status: usize) -> bool {
    (status & 0xff) == 0x7f
}

/// If wifstopped(status), the signal that stopped the child.
pub fn wstopsig(status: usize) -> usize {
    (status >> 8) & 0xff
}

/// True if status indicates the child continued after a stop.
pub fn wifcontinued(status: usize) -> bool {
    status == 0xffff
}

/// True if STATUS indicates termination by a signal.
pub fn wifsignaled(status: usize) -> bool {
    ((status & 0x7f) + 1) as i8 >= 2
}

/// If wifsignaled(status), the terminating signal.
pub fn wtermsig(status: usize) -> usize {
    status & 0x7f
}

/// True if status indicates normal termination.
pub fn wifexited(status: usize) -> bool {
    wtermsig(status) == 0
}

/// If wifexited(status), the exit status.
pub fn wexitstatus(status: usize) -> usize {
    (status >> 8) & 0xff
}

/// True if status indicates a core dump was created.
pub fn wcoredump(status: usize) -> bool {
    (status & 0x80) != 0
}
