use crate::FileTime;
use libc::{time_t, timespec};
use std::fs;
use std::os::unix::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod utimes;
        mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os = "android")] {
        mod android;
        pub use self::android::*;
    } else if #[cfg(target_os = "macos")] {
        mod utimes;
        mod macos;
        pub use self::macos::*;
    } else if #[cfg(any(target_os = "solaris",
                        target_os = "illumos",
                        target_os = "emscripten",
                        target_os = "freebsd",
                        target_os = "netbsd",
                        target_os = "openbsd",
                        target_os = "haiku"))] {
        mod utimensat;
        pub use self::utimensat::*;
    } else {
        mod utimes;
        pub use self::utimes::*;
    }
}

#[allow(dead_code)]
fn to_timespec(ft: &Option<FileTime>) -> timespec {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "macos",
                     target_os = "illumos",
                     target_os = "freebsd"))] {
            // https://github.com/apple/darwin-xnu/blob/a449c6a3b8014d9406c2ddbdc81795da24aa7443/bsd/sys/stat.h#L541
            // https://github.com/illumos/illumos-gate/blob/master/usr/src/boot/sys/sys/stat.h#L312
            // https://svnweb.freebsd.org/base/head/sys/sys/stat.h?view=markup#l359
            const UTIME_OMIT: i64 = -2;
        } else if #[cfg(target_os = "openbsd")] {
            // https://github.com/openbsd/src/blob/master/sys/sys/stat.h#L189
            const UTIME_OMIT: i64 = -1;
        } else if #[cfg(target_os = "haiku")] {
            // https://git.haiku-os.org/haiku/tree/headers/posix/sys/stat.h?#n106
            const UTIME_OMIT: i64 = 1000000001;
        } else {
            // http://cvsweb.netbsd.org/bsdweb.cgi/src/sys/sys/stat.h?annotate=1.68.30.1
            // https://github.com/emscripten-core/emscripten/blob/master/system/include/libc/sys/stat.h#L71
            const UTIME_OMIT: i64 = 1_073_741_822;
        }
    }

    if let &Some(ft) = ft {
        timespec {
            tv_sec: ft.seconds() as time_t,
            tv_nsec: ft.nanoseconds() as _,
        }
    } else {
        timespec {
            tv_sec: 0,
            tv_nsec: UTIME_OMIT as _,
        }
    }
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.mtime(),
        nanos: meta.mtime_nsec() as u32,
    }
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.atime(),
        nanos: meta.atime_nsec() as u32,
    }
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    macro_rules! birthtim {
        ($(($e:expr, $i:ident)),*) => {
            #[cfg(any($(target_os = $e),*))]
            fn imp(meta: &fs::Metadata) -> Option<FileTime> {
                $(
                    #[cfg(target_os = $e)]
                    use std::os::$i::fs::MetadataExt;
                )*
                Some(FileTime {
                    seconds: meta.st_birthtime(),
                    nanos: meta.st_birthtime_nsec() as u32,
                })
            }

            #[cfg(all($(not(target_os = $e)),*))]
            fn imp(_meta: &fs::Metadata) -> Option<FileTime> {
                None
            }
        }
    }

    birthtim! {
        ("bitrig", bitrig),
        ("freebsd", freebsd),
        ("ios", ios),
        ("macos", macos),
        ("openbsd", openbsd)
    }

    imp(meta)
}
