use super::super::c;
use bitflags::bitflags;

bitflags! {
    /// `FD_*` constants for use with [`fcntl_getfd`] and [`fcntl_setfd`].
    ///
    /// [`fcntl_getfd`]: crate::fs::fcntl_getfd
    /// [`fcntl_setfd`]: crate::fs::fcntl_setfd
    pub struct FdFlags: c::c_int {
        /// `FD_CLOEXEC`
        const CLOEXEC = c::FD_CLOEXEC;
    }
}

bitflags! {
    /// `*_OK` constants for use with [`accessat`].
    ///
    /// [`accessat`]: fn.accessat.html
    pub struct Access: c::c_int {
        /// `R_OK`
        const READ_OK = c::R_OK;

        /// `W_OK`
        const WRITE_OK = c::W_OK;

        /// `X_OK`
        const EXEC_OK = c::X_OK;

        /// `F_OK`
        const EXISTS = c::F_OK;
    }
}

#[cfg(not(target_os = "redox"))]
bitflags! {
    /// `AT_*` constants for use with [`openat`], [`statat`], and other `*at`
    /// functions.
    ///
    /// [`openat`]: crate::fs::openat
    /// [`statat`]: crate::fs::statat
    pub struct AtFlags: c::c_int {
        /// `AT_REMOVEDIR`
        const REMOVEDIR = c::AT_REMOVEDIR;

        /// `AT_SYMLINK_FOLLOW`
        const SYMLINK_FOLLOW = c::AT_SYMLINK_FOLLOW;

        /// `AT_SYMLINK_NOFOLLOW`
        const SYMLINK_NOFOLLOW = c::AT_SYMLINK_NOFOLLOW;

        /// `AT_EMPTY_PATH`
        #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "linux",
        ))]
        const EMPTY_PATH = c::AT_EMPTY_PATH;

        /// `AT_EACCESS`
        #[cfg(not(any(target_os = "emscripten", target_os = "android")))]
        const EACCESS = c::AT_EACCESS;

        /// `AT_STATX_SYNC_AS_STAT`
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const STATX_SYNC_AS_STAT = c::AT_STATX_SYNC_AS_STAT;

        /// `AT_STATX_FORCE_SYNC`
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const STATX_FORCE_SYNC = c::AT_STATX_FORCE_SYNC;

        /// `AT_STATX_DONT_SYNC`
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const STATX_DONT_SYNC = c::AT_STATX_DONT_SYNC;
    }
}

bitflags! {
    /// `S_I*` constants for use with [`openat`], [`chmodat`], and [`fchmod`].
    ///
    /// [`openat`]: crate::fs::openat
    /// [`chmodat`]: crate::fs::chmodat
    /// [`fchmod`]: crate::fs::fchmod
    pub struct Mode: RawMode {
        /// `S_IRWXU`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const RWXU = c::S_IRWXU as RawMode;

        /// `S_IRUSR`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const RUSR = c::S_IRUSR as RawMode;

        /// `S_IWUSR`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const WUSR = c::S_IWUSR as RawMode;

        /// `S_IXUSR`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const XUSR = c::S_IXUSR as RawMode;

        /// `S_IRWXG`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const RWXG = c::S_IRWXG as RawMode;

        /// `S_IRGRP`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const RGRP = c::S_IRGRP as RawMode;

        /// `S_IWGRP`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const WGRP = c::S_IWGRP as RawMode;

        /// `S_IXGRP`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const XGRP = c::S_IXGRP as RawMode;

        /// `S_IRWXO`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const RWXO = c::S_IRWXO as RawMode;

        /// `S_IROTH`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const ROTH = c::S_IROTH as RawMode;

        /// `S_IWOTH`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const WOTH = c::S_IWOTH as RawMode;

        /// `S_IXOTH`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const XOTH = c::S_IXOTH as RawMode;

        /// `S_ISUID`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const SUID = c::S_ISUID as RawMode;

        /// `S_ISGID`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const SGID = c::S_ISGID as RawMode;

        /// `S_ISVTX`
        #[cfg(not(target_os = "wasi"))] // WASI doesn't have Unix-style mode flags.
        const SVTX = c::S_ISVTX as RawMode;
    }
}

impl Mode {
    /// Construct a `Mode` from the mode bits of the `st_mode` field of
    /// a `Stat`.
    #[inline]
    pub const fn from_raw_mode(st_mode: RawMode) -> Self {
        Self::from_bits_truncate(st_mode)
    }

    /// Construct an `st_mode` value from `Stat`.
    #[inline]
    pub const fn as_raw_mode(self) -> RawMode {
        self.bits()
    }
}

bitflags! {
    /// `O_*` constants for use with [`openat`].
    ///
    /// [`openat`]: crate::fs::openat
    pub struct OFlags: c::c_int {
        /// `O_ACCMODE`
        const ACCMODE = c::O_ACCMODE;

        /// Similar to `ACCMODE`, but just includes the read/write flags, and
        /// no other flags.
        ///
        /// Some implementations include `O_PATH` in `O_ACCMODE`, when
        /// sometimes we really just want the read/write bits. Caution is
        /// indicated, as the presence of `O_PATH` may mean that the read/write
        /// bits don't have their usual meaning.
        const RWMODE = c::O_RDONLY | c::O_WRONLY | c::O_RDWR;

        /// `O_APPEND`
        const APPEND = c::O_APPEND;

        /// `O_CREAT`
        #[doc(alias = "CREAT")]
        const CREATE = c::O_CREAT;

        /// `O_DIRECTORY`
        const DIRECTORY = c::O_DIRECTORY;

        /// `O_DSYNC`
        #[cfg(not(any(target_os = "dragonfly", target_os = "freebsd", target_os = "redox")))]
        const DSYNC = c::O_DSYNC;

        /// `O_EXCL`
        const EXCL = c::O_EXCL;

        /// `O_FSYNC`
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            all(target_os = "linux", not(target_env = "musl")),
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        const FSYNC = c::O_FSYNC;

        /// `O_NOFOLLOW`
        const NOFOLLOW = c::O_NOFOLLOW;

        /// `O_NONBLOCK`
        const NONBLOCK = c::O_NONBLOCK;

        /// `O_RDONLY`
        const RDONLY = c::O_RDONLY;

        /// `O_WRONLY`
        const WRONLY = c::O_WRONLY;

        /// `O_RDWR`
        const RDWR = c::O_RDWR;

        /// `O_NOCTTY`
        #[cfg(not(target_os = "redox"))]
        const NOCTTY = c::O_NOCTTY;

        /// `O_RSYNC`
        #[cfg(any(
            target_os = "android",
            target_os = "emscripten",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "wasi",
        ))]
        const RSYNC = c::O_RSYNC;

        /// `O_SYNC`
        #[cfg(not(target_os = "redox"))]
        const SYNC = c::O_SYNC;

        /// `O_TRUNC`
        const TRUNC = c::O_TRUNC;

        /// `O_PATH`
        #[cfg(any(
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "linux",
            target_os = "redox",
        ))]
        const PATH = c::O_PATH;

        /// `O_CLOEXEC`
        const CLOEXEC = c::O_CLOEXEC;

        /// `O_TMPFILE`
        #[cfg(any(
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "linux",
        ))]
        const TMPFILE = c::O_TMPFILE;

        /// `O_NOATIME`
        #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "linux",
        ))]
        const NOATIME = c::O_NOATIME;
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
bitflags! {
    /// `CLONE_*` constants for use with [`fclonefileat`].
    ///
    /// [`fclonefileat`]: crate::fs::fclonefileat
    pub struct CloneFlags: c::c_int {
        /// `CLONE_NOFOLLOW`
        const NOFOLLOW = 1;

        /// `CLONE_NOOWNERCOPY`
        const NOOWNERCOPY = 2;
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
mod copyfile {
    pub(super) const ACL: u32 = 1 << 0;
    pub(super) const STAT: u32 = 1 << 1;
    pub(super) const XATTR: u32 = 1 << 2;
    pub(super) const DATA: u32 = 1 << 3;
    pub(super) const SECURITY: u32 = STAT | ACL;
    pub(super) const METADATA: u32 = SECURITY | XATTR;
    pub(super) const ALL: u32 = METADATA | DATA;
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
bitflags! {
    /// `COPYFILE_*` constants.
    pub struct CopyfileFlags: c::c_uint {
        /// `COPYFILE_ACL`
        const ACL = copyfile::ACL;

        /// `COPYFILE_STAT`
        const STAT = copyfile::STAT;

        /// `COPYFILE_XATTR`
        const XATTR = copyfile::XATTR;

        /// `COPYFILE_DATA`
        const DATA = copyfile::DATA;

        /// `COPYFILE_SECURITY`
        const SECURITY = copyfile::SECURITY;

        /// `COPYFILE_METADATA`
        const METADATA = copyfile::METADATA;

        /// `COPYFILE_ALL`
        const ALL = copyfile::ALL;
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
bitflags! {
    /// `RESOLVE_*` constants for use with [`openat2`].
    ///
    /// [`openat2`]: crate::fs::openat2
    #[derive(Default)]
    pub struct ResolveFlags: u64 {
        /// `RESOLVE_NO_XDEV`
        const NO_XDEV = 0x01;

        /// `RESOLVE_NO_MAGICLINKS`
        const NO_MAGICLINKS = 0x02;

        /// `RESOLVE_NO_SYMLINKS`
        const NO_SYMLINKS = 0x04;

        /// `RESOLVE_BENEATH`
        const BENEATH = 0x08;

        /// `RESOLVE_IN_ROOT`
        const IN_ROOT = 0x10;

        /// `RESOLVE_CACHED` (since Linux 5.12)
        const CACHED = 0x20;
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
bitflags! {
    /// `RENAME_*` constants for use with [`renameat_with`].
    ///
    /// [`renameat_with`]: crate::fs::renameat_with
    pub struct RenameFlags: c::c_uint {
        /// `RENAME_EXCHANGE`
        const EXCHANGE = c::RENAME_EXCHANGE as _;

        /// `RENAME_NOREPLACE`
        const NOREPLACE = c::RENAME_NOREPLACE as _;

        /// `RENAME_WHITEOUT`
        const WHITEOUT = c::RENAME_WHITEOUT as _;
    }
}

/// `S_IF*` constants for use with [`mknodat`] and [`Stat`]'s `st_mode` field.
///
/// [`mknodat`]: crate::fs::mknodat
/// [`Stat`]: crate::fs::Stat
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileType {
    /// `S_IFREG`
    RegularFile = c::S_IFREG as isize,

    /// `S_IFDIR`
    Directory = c::S_IFDIR as isize,

    /// `S_IFLNK`
    Symlink = c::S_IFLNK as isize,

    /// `S_IFIFO`
    #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFIFO`.
    #[doc(alias = "IFO")]
    Fifo = c::S_IFIFO as isize,

    /// `S_IFSOCK`
    #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFSOCK`.
    Socket = c::S_IFSOCK as isize,

    /// `S_IFCHR`
    CharacterDevice = c::S_IFCHR as isize,

    /// `S_IFBLK`
    BlockDevice = c::S_IFBLK as isize,

    /// An unknown filesystem object.
    Unknown,
}

impl FileType {
    /// Construct a `FileType` from the `S_IFMT` bits of the `st_mode` field of
    /// a `Stat`.
    pub const fn from_raw_mode(st_mode: RawMode) -> Self {
        match (st_mode as c::mode_t) & c::S_IFMT {
            c::S_IFREG => Self::RegularFile,
            c::S_IFDIR => Self::Directory,
            c::S_IFLNK => Self::Symlink,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFIFO`.
            c::S_IFIFO => Self::Fifo,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFSOCK`.
            c::S_IFSOCK => Self::Socket,
            c::S_IFCHR => Self::CharacterDevice,
            c::S_IFBLK => Self::BlockDevice,
            _ => Self::Unknown,
        }
    }

    /// Construct an `st_mode` value from `Stat`.
    pub const fn as_raw_mode(self) -> RawMode {
        match self {
            Self::RegularFile => c::S_IFREG as RawMode,
            Self::Directory => c::S_IFDIR as RawMode,
            Self::Symlink => c::S_IFLNK as RawMode,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFIFO`.
            Self::Fifo => c::S_IFIFO as RawMode,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `S_IFSOCK`.
            Self::Socket => c::S_IFSOCK as RawMode,
            Self::CharacterDevice => c::S_IFCHR as RawMode,
            Self::BlockDevice => c::S_IFBLK as RawMode,
            Self::Unknown => c::S_IFMT as RawMode,
        }
    }

    /// Construct a `FileType` from the `d_type` field of a `c::dirent`.
    #[cfg(not(any(target_os = "illumos", target_os = "redox")))]
    pub(crate) const fn from_dirent_d_type(d_type: u8) -> Self {
        match d_type {
            c::DT_REG => Self::RegularFile,
            c::DT_DIR => Self::Directory,
            c::DT_LNK => Self::Symlink,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `DT_SOCK`.
            c::DT_SOCK => Self::Socket,
            #[cfg(not(target_os = "wasi"))] // TODO: Use WASI's `DT_FIFO`.
            c::DT_FIFO => Self::Fifo,
            c::DT_CHR => Self::CharacterDevice,
            c::DT_BLK => Self::BlockDevice,
            // c::DT_UNKNOWN |
            _ => Self::Unknown,
        }
    }
}

/// `POSIX_FADV_*` constants for use with [`fadvise`].
///
/// [`fadvise`]: crate::fs::fadvise
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Advice {
    /// `POSIX_FADV_NORMAL`
    Normal = c::POSIX_FADV_NORMAL as c::c_uint,

    /// `POSIX_FADV_SEQUENTIAL`
    Sequential = c::POSIX_FADV_SEQUENTIAL as c::c_uint,

    /// `POSIX_FADV_RANDOM`
    Random = c::POSIX_FADV_RANDOM as c::c_uint,

    /// `POSIX_FADV_NOREUSE`
    NoReuse = c::POSIX_FADV_NOREUSE as c::c_uint,

    /// `POSIX_FADV_WILLNEED`
    WillNeed = c::POSIX_FADV_WILLNEED as c::c_uint,

    /// `POSIX_FADV_DONTNEED`
    DontNeed = c::POSIX_FADV_DONTNEED as c::c_uint,
}

#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
bitflags! {
    /// `MFD_*` constants for use with [`memfd_create`].
    ///
    /// [`memfd_create`]: crate::fs::memfd_create
    pub struct MemfdFlags: c::c_uint {
        /// `MFD_CLOEXEC`
        const CLOEXEC = c::MFD_CLOEXEC;

        /// `MFD_ALLOW_SEALING`
        const ALLOW_SEALING = c::MFD_ALLOW_SEALING;

        /// `MFD_HUGETLB` (since Linux 4.14)
        const HUGETLB = c::MFD_HUGETLB;

        /// `MFD_HUGE_64KB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_64KB = c::MFD_HUGE_64KB;
        /// `MFD_HUGE_512JB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_512KB = c::MFD_HUGE_512KB;
        /// `MFD_HUGE_1MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_1MB = c::MFD_HUGE_1MB;
        /// `MFD_HUGE_2MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_2MB = c::MFD_HUGE_2MB;
        /// `MFD_HUGE_8MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_8MB = c::MFD_HUGE_8MB;
        /// `MFD_HUGE_16MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_16MB = c::MFD_HUGE_16MB;
        /// `MFD_HUGE_32MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_32MB = c::MFD_HUGE_32MB;
        /// `MFD_HUGE_256MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_256MB = c::MFD_HUGE_256MB;
        /// `MFD_HUGE_512MB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_512MB = c::MFD_HUGE_512MB;
        /// `MFD_HUGE_1GB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_1GB = c::MFD_HUGE_1GB;
        /// `MFD_HUGE_2GB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_2GB = c::MFD_HUGE_2GB;
        /// `MFD_HUGE_16GB`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        const HUGE_16GB = c::MFD_HUGE_16GB;
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "linux",
))]
bitflags! {
    /// `F_SEAL_*` constants for use with [`fcntl_add_seals`] and
    /// [`fcntl_get_seals`].
    ///
    /// [`fcntl_add_seals`]: crate::fs::fcntl_add_seals
    /// [`fcntl_get_seals`]: crate::fs::fcntl_get_seals
    pub struct SealFlags: i32 {
       /// `F_SEAL_SEAL`.
       const SEAL = c::F_SEAL_SEAL;
       /// `F_SEAL_SHRINK`.
       const SHRINK = c::F_SEAL_SHRINK;
       /// `F_SEAL_GROW`.
       const GROW = c::F_SEAL_GROW;
       /// `F_SEAL_WRITE`.
       const WRITE = c::F_SEAL_WRITE;
       /// `F_SEAL_FUTURE_WRITE` (since Linux 5.1)
       #[cfg(any(target_os = "android", target_os = "linux"))]
       const FUTURE_WRITE = c::F_SEAL_FUTURE_WRITE;
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
bitflags! {
    /// `STATX_*` constants for use with [`statx`].
    ///
    /// [`statx`]: crate::fs::statx
    pub struct StatxFlags: u32 {
        /// `STATX_TYPE`
        const TYPE = c::STATX_TYPE;

        /// `STATX_MODE`
        const MODE = c::STATX_MODE;

        /// `STATX_NLINK`
        const NLINK = c::STATX_NLINK;

        /// `STATX_UID`
        const UID = c::STATX_UID;

        /// `STATX_GID`
        const GID = c::STATX_GID;

        /// `STATX_ATIME`
        const ATIME = c::STATX_ATIME;

        /// `STATX_MTIME`
        const MTIME = c::STATX_MTIME;

        /// `STATX_CTIME`
        const CTIME = c::STATX_CTIME;

        /// `STATX_INO`
        const INO = c::STATX_INO;

        /// `STATX_SIZE`
        const SIZE = c::STATX_SIZE;

        /// `STATX_BLOCKS`
        const BLOCKS = c::STATX_BLOCKS;

        /// `STATX_BASIC_STATS`
        const BASIC_STATS = c::STATX_BASIC_STATS;

        /// `STATX_BTIME`
        const BTIME = c::STATX_BTIME;

        /// `STATX_MNT_ID` (since Linux 5.8)
        const MNT_ID = c::STATX_MNT_ID;

        /// `STATX_ALL`
        const ALL = c::STATX_ALL;
    }
}

#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
bitflags! {
    /// `STATX_*` constants for use with [`statx`].
    ///
    /// [`statx`]: crate::fs::statx
    pub struct StatxFlags: u32 {
        /// `STATX_TYPE`
        const TYPE = 0x0001;

        /// `STATX_MODE`
        const MODE = 0x0002;

        /// `STATX_NLINK`
        const NLINK = 0x0004;

        /// `STATX_UID`
        const UID = 0x0008;

        /// `STATX_GID`
        const GID = 0x0010;

        /// `STATX_ATIME`
        const ATIME = 0x0020;

        /// `STATX_MTIME`
        const MTIME = 0x0040;

        /// `STATX_CTIME`
        const CTIME = 0x0080;

        /// `STATX_INO`
        const INO = 0x0100;

        /// `STATX_SIZE`
        const SIZE = 0x0200;

        /// `STATX_BLOCKS`
        const BLOCKS = 0x0400;

        /// `STATX_BASIC_STATS`
        const BASIC_STATS = 0x07ff;

        /// `STATX_BTIME`
        const BTIME = 0x800;

        /// `STATX_MNT_ID` (since Linux 5.8)
        const MNT_ID = 0x1000;

        /// `STATX_ALL`
        const ALL = 0xfff;
    }
}

#[cfg(not(any(
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
bitflags! {
    /// `FALLOC_FL_*` constants for use with [`fallocate`].
    ///
    /// [`fallocate`]: crate::fs::fallocate
    pub struct FallocateFlags: i32 {
        /// `FALLOC_FL_KEEP_SIZE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "wasi",
        )))]
        const KEEP_SIZE = c::FALLOC_FL_KEEP_SIZE;
        /// `FALLOC_FL_PUNCH_HOLE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "wasi",
        )))]
        const PUNCH_HOLE = c::FALLOC_FL_PUNCH_HOLE;
        /// `FALLOC_FL_NO_HIDE_STALE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "linux",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "wasi",
        )))]
        const NO_HIDE_STALE = c::FALLOC_FL_NO_HIDE_STALE;
        /// `FALLOC_FL_COLLAPSE_RANGE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "emscripten",
            target_os = "wasi",
        )))]
        const COLLAPSE_RANGE = c::FALLOC_FL_COLLAPSE_RANGE;
        /// `FALLOC_FL_ZERO_RANGE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "emscripten",
            target_os = "wasi",
        )))]
        const ZERO_RANGE = c::FALLOC_FL_ZERO_RANGE;
        /// `FALLOC_FL_INSERT_RANGE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "emscripten",
            target_os = "wasi",
        )))]
        const INSERT_RANGE = c::FALLOC_FL_INSERT_RANGE;
        /// `FALLOC_FL_UNSHARE_RANGE`
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "emscripten",
            target_os = "wasi",
        )))]
        const UNSHARE_RANGE = c::FALLOC_FL_UNSHARE_RANGE;
    }
}

#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi",)))]
bitflags! {
    /// `ST_*` constants for use with [`StatVfs`].
    pub struct StatVfsMountFlags: u64 {
        /// `ST_MANDLOCK`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const MANDLOCK = libc::ST_MANDLOCK as u64;

        /// `ST_NOATIME`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const NOATIME = libc::ST_NOATIME as u64;

        /// `ST_NODEV`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const NODEV = libc::ST_NODEV as u64;

        /// `ST_NODIRATIME`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const NODIRATIME = libc::ST_NODIRATIME as u64;

        /// `ST_NOEXEC`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const NOEXEC = libc::ST_NOEXEC as u64;

        /// `ST_NOSUID`
        const NOSUID = libc::ST_NOSUID as u64;

        /// `ST_RDONLY`
        const RDONLY = libc::ST_RDONLY as u64;

        /// `ST_RELATIME`
        #[cfg(any(target_os = "android", all(target_os = "linux", target_env = "gnu")))]
        const RELATIME = libc::ST_RELATIME as u64;

        /// `ST_SYNCHRONOUS`
        #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))]
        const SYNCHRONOUS = libc::ST_SYNCHRONOUS as u64;
    }
}

/// `LOCK_*` constants for use with [`flock`]
///
/// [`flock`]: crate::fs::flock
#[cfg(not(target_os = "wasi"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FlockOperation {
    /// `LOCK_SH`
    LockShared = c::LOCK_SH,
    /// `LOCK_EX`
    LockExclusive = c::LOCK_EX,
    /// `LOCK_UN`
    Unlock = c::LOCK_UN,
    /// `LOCK_SH | LOCK_NB`
    NonBlockingLockShared = c::LOCK_SH | c::LOCK_NB,
    /// `LOCK_EX | LOCK_NB`
    NonBlockingLockExclusive = c::LOCK_EX | c::LOCK_NB,
    /// `LOCK_UN | LOCK_NB`
    NonBlockingUnlock = c::LOCK_UN | c::LOCK_NB,
}

/// `struct stat` for use with [`statat`] and [`fstat`].
///
/// [`statat`]: crate::fs::statat
/// [`fstat`]: crate::fs::fstat
#[cfg(not(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
)))]
pub type Stat = c::stat;

/// `struct stat` for use with [`statat`] and [`fstat`].
///
/// [`statat`]: crate::fs::statat
/// [`fstat`]: crate::fs::fstat
#[cfg(any(
    all(
        any(target_os = "android", target_os = "linux"),
        target_pointer_width = "64",
    ),
    target_os = "emscripten",
    target_os = "l4re",
))]
pub type Stat = c::stat64;

/// `struct stat` for use with [`statat`] and [`fstat`].
///
/// [`statat`]: crate::fs::statat
/// [`fstat`]: crate::fs::fstat
// On 32-bit, Linux's `struct stat64` has a 32-bit `st_mtime` and friends, so
// we use our own struct, populated from `statx` where possible, to avoid the
// y2038 bug.
#[cfg(all(
    any(target_os = "android", target_os = "linux"),
    target_pointer_width = "32",
))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(missing_docs)]
pub struct Stat {
    pub st_dev: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: u32,
    pub st_blocks: u64,
    pub st_atime: u64,
    pub st_atime_nsec: u32,
    pub st_mtime: u64,
    pub st_mtime_nsec: u32,
    pub st_ctime: u64,
    pub st_ctime_nsec: u32,
    pub st_ino: u64,
}

/// `struct statfs` for use with [`statfs`] and [`fstatfs`].
///
/// [`statfs`]: crate::fs::statfs
/// [`fstatfs`]: crate::fs::fstatfs
#[cfg(not(any(
    target_os = "android",
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "linux",
    target_os = "l4re",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[allow(clippy::module_name_repetitions)]
pub type StatFs = c::statfs;

/// `struct statfs` for use with [`statfs`] and [`fstatfs`].
///
/// [`statfs`]: crate::fs::statfs
/// [`fstatfs`]: crate::fs::fstatfs
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub type StatFs = c::statfs64;

/// `struct statvfs` for use with [`statvfs`] and [`fstatvfs`].
///
/// [`statvfs`]: crate::fs::statvfs
/// [`fstatvfs`]: crate::fs::fstatvfs
#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi",)))]
#[allow(missing_docs)]
pub struct StatVfs {
    pub f_bsize: u64,
    pub f_frsize: u64,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_favail: u64,
    pub f_fsid: u64,
    pub f_flag: StatVfsMountFlags,
    pub f_namemax: u64,
}

/// `struct statx` for use with [`statx`].
///
/// [`statx`]: crate::fs::statx
#[cfg(all(target_os = "linux", target_env = "gnu"))]
// Use the glibc `struct statx`.
pub type Statx = c::statx;

/// `struct statx_timestamp` for use with [`Statx`].
#[cfg(all(target_os = "linux", target_env = "gnu"))]
// Use the glibc `struct statx_timestamp`.
pub type StatxTimestamp = c::statx;

/// `struct statx` for use with [`statx`].
///
/// [`statx`]: crate::fs::statx
// Non-glibc ABIs don't currently declare a `struct statx`, so we declare it
// ourselves.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
#[repr(C)]
#[allow(missing_docs)]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_attributes: u64,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    __statx_pad1: [u16; 1],
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
    __statx_pad2: u64,
    __statx_pad3: [u64; 12],
}

/// `struct statx_timestamp` for use with [`Statx`].
// Non-glibc ABIs don't currently declare a `struct statx_timestamp`, so we
// declare it ourselves.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
#[repr(C)]
#[allow(missing_docs)]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    pub __statx_timestamp_pad1: [i32; 1],
}

/// `mode_t`
#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
pub type RawMode = c::mode_t;

/// `mode_t`
#[cfg(all(target_os = "android", target_pointer_width = "32"))]
pub type RawMode = c::c_uint;

/// `dev_t`
#[cfg(not(all(target_os = "android", target_pointer_width = "32")))]
pub type Dev = c::dev_t;

/// `dev_t`
#[cfg(all(target_os = "android", target_pointer_width = "32"))]
pub type Dev = c::c_ulonglong;

/// `__fsword_t`
#[cfg(all(
    target_os = "linux",
    not(target_env = "musl"),
    not(target_arch = "s390x"),
))]
pub type FsWord = c::__fsword_t;

/// `__fsword_t`
#[cfg(all(
    any(target_os = "android", all(target_os = "linux", target_env = "musl")),
    target_pointer_width = "32",
))]
pub type FsWord = u32;

/// `__fsword_t`
#[cfg(all(
    any(target_os = "android", all(target_os = "linux", target_env = "musl")),
    not(target_arch = "s390x"),
    target_pointer_width = "64",
))]
pub type FsWord = u64;

/// `__fsword_t`
// s390x uses `u32` for `statfs` entries, even though `__fsword_t` is `u64`.
#[cfg(all(target_os = "linux", target_arch = "s390x"))]
pub type FsWord = u32;

#[cfg(not(target_os = "redox"))]
pub use c::{UTIME_NOW, UTIME_OMIT};

/// `PROC_SUPER_MAGIC`—The magic number for the procfs filesystem.
#[cfg(all(
    any(target_os = "android", target_os = "linux"),
    not(target_env = "musl"),
))]
pub const PROC_SUPER_MAGIC: FsWord = c::PROC_SUPER_MAGIC as FsWord;

/// `NFS_SUPER_MAGIC`—The magic number for the NFS filesystem.
#[cfg(all(
    any(target_os = "android", target_os = "linux"),
    not(target_env = "musl"),
))]
pub const NFS_SUPER_MAGIC: FsWord = c::NFS_SUPER_MAGIC as FsWord;

/// `PROC_SUPER_MAGIC`—The magic number for the procfs filesystem.
#[cfg(all(any(target_os = "android", target_os = "linux"), target_env = "musl"))]
pub const PROC_SUPER_MAGIC: FsWord = 0x0000_9fa0;

/// `NFS_SUPER_MAGIC`—The magic number for the NFS filesystem.
#[cfg(all(any(target_os = "android", target_os = "linux"), target_env = "musl"))]
pub const NFS_SUPER_MAGIC: FsWord = 0x0000_6969;

/// `copyfile_state_t`—State for use with [`fcopyfile`].
///
/// [`fcopyfile`]: crate::fs::fcopyfile
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct copyfile_state_t(pub(crate) *mut c::c_void);
