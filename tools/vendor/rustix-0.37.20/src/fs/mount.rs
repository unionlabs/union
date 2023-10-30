//! Linux `mount`.

use crate::backend::fs::types::{
    InternalMountFlags, MountFlags, MountFlagsArg, MountPropagationFlags, UnmountFlags,
};
use crate::{backend, io, path};

/// `mount(source, target, filesystemtype, mountflags, data)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
pub fn mount<Source: path::Arg, Target: path::Arg, Fs: path::Arg, Data: path::Arg>(
    source: Source,
    target: Target,
    file_system_type: Fs,
    flags: MountFlags,
    data: Data,
) -> io::Result<()> {
    source.into_with_c_str(|source| {
        target.into_with_c_str(|target| {
            file_system_type.into_with_c_str(|file_system_type| {
                data.into_with_c_str(|data| {
                    backend::fs::syscalls::mount(
                        Some(source),
                        target,
                        Some(file_system_type),
                        MountFlagsArg(flags.bits()),
                        Some(data),
                    )
                })
            })
        })
    })
}

/// `mount(NULL, target, NULL, MS_REMOUNT | mountflags, data)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
#[doc(alias = "mount")]
pub fn remount<Target: path::Arg, Data: path::Arg>(
    target: Target,
    flags: MountFlags,
    data: Data,
) -> io::Result<()> {
    target.into_with_c_str(|target| {
        data.into_with_c_str(|data| {
            backend::fs::syscalls::mount(
                None,
                target,
                None,
                MountFlagsArg(InternalMountFlags::REMOUNT.bits() | flags.bits()),
                Some(data),
            )
        })
    })
}

/// `mount(source, target, NULL, MS_BIND, NULL)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
#[doc(alias = "mount")]
pub fn bind_mount<Source: path::Arg, Target: path::Arg>(
    source: Source,
    target: Target,
) -> io::Result<()> {
    source.into_with_c_str(|source| {
        target.into_with_c_str(|target| {
            backend::fs::syscalls::mount(
                Some(source),
                target,
                None,
                MountFlagsArg(MountFlags::BIND.bits()),
                None,
            )
        })
    })
}

/// `mount(source, target, NULL, MS_BIND | MS_REC, NULL)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
#[doc(alias = "mount")]
pub fn recursive_bind_mount<Source: path::Arg, Target: path::Arg>(
    source: Source,
    target: Target,
) -> io::Result<()> {
    source.into_with_c_str(|source| {
        target.into_with_c_str(|target| {
            backend::fs::syscalls::mount(
                Some(source),
                target,
                None,
                MountFlagsArg(MountFlags::BIND.bits() | MountPropagationFlags::REC.bits()),
                None,
            )
        })
    })
}

/// `mount(NULL, target, NULL, mountflags, NULL)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
#[doc(alias = "mount")]
pub fn change_mount<Target: path::Arg>(
    target: Target,
    flags: MountPropagationFlags,
) -> io::Result<()> {
    target.into_with_c_str(|target| {
        backend::fs::syscalls::mount(None, target, None, MountFlagsArg(flags.bits()), None)
    })
}

/// `mount(source, target, NULL, MS_MOVE, NULL)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/mount.2.html
#[inline]
#[doc(alias = "mount")]
pub fn move_mount<Source: path::Arg, Target: path::Arg>(
    source: Source,
    target: Target,
) -> io::Result<()> {
    source.into_with_c_str(|source| {
        target.into_with_c_str(|target| {
            backend::fs::syscalls::mount(
                Some(source),
                target,
                None,
                MountFlagsArg(InternalMountFlags::MOVE.bits()),
                None,
            )
        })
    })
}

/// `umount2(target, flags)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/umount.2.html
#[doc(alias = "umount", alias = "umount2")]
pub fn unmount<Target: path::Arg>(target: Target, flags: UnmountFlags) -> io::Result<()> {
    target.into_with_c_str(|target| backend::fs::syscalls::unmount(target, flags))
}
