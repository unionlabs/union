use super::arch::*;
use super::data::{Map, SigAction, Stat, StatVfs, TimeSpec};
use super::error::Result;
use super::flag::*;
use super::number::*;

use core::{mem, ptr};

// Signal restorer
extern "C" fn restorer() -> ! {
    sigreturn().unwrap();
    unreachable!();
}

/// Change the process's working directory
///
/// This function will attempt to set the process's working directory to `path`, which can be
/// either a relative, scheme relative, or absolute path.
///
/// On success, `Ok(0)` will be returned. On error, one of the following errors will be returned.
///
/// # Errors
///
/// * `EACCES` - permission is denied for one of the components of `path`, or `path`
/// * `EFAULT` - `path` does not point to the process's addressible memory
/// * `EIO` - an I/O error occurred
/// * `ENOENT` - `path` does not exit
/// * `ENOTDIR` - `path` is not a directory
pub fn chdir<T: AsRef<str>>(path: T) -> Result<usize> {
    unsafe { syscall2(SYS_CHDIR, path.as_ref().as_ptr() as usize, path.as_ref().len()) }
}

#[deprecated(
    since = "0.1.55",
    note = "use fchmod instead"
)]
pub fn chmod<T: AsRef<str>>(path: T, mode: usize) -> Result<usize> {
    unsafe { syscall3(SYS_CHMOD, path.as_ref().as_ptr() as usize, path.as_ref().len(), mode) }
}

/// Produce a fork of the current process, or a new process thread
pub unsafe fn clone(flags: CloneFlags) -> Result<usize> {
    syscall1(SYS_CLONE, flags.bits())
}

/// Close a file
pub fn close(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_CLOSE, fd) }
}

/// Get the current system time
pub fn clock_gettime(clock: usize, tp: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_CLOCK_GETTIME, clock, tp as *mut TimeSpec as usize) }
}

/// Copy and transform a file descriptor
pub fn dup(fd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall3(SYS_DUP, fd, buf.as_ptr() as usize, buf.len()) }
}

/// Copy and transform a file descriptor
pub fn dup2(fd: usize, newfd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall4(SYS_DUP2, fd, newfd, buf.as_ptr() as usize, buf.len()) }
}

/// Exit the current process
pub fn exit(status: usize) -> Result<usize> {
    unsafe { syscall1(SYS_EXIT, status) }
}

/// Change file permissions
pub fn fchmod(fd: usize, mode: u16) -> Result<usize> {
    unsafe { syscall2(SYS_FCHMOD, fd, mode as usize) }

}

/// Change file ownership
pub fn fchown(fd: usize, uid: u32, gid: u32) -> Result<usize> {
    unsafe { syscall3(SYS_FCHOWN, fd, uid as usize, gid as usize) }

}

/// Change file descriptor flags
pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> Result<usize> {
    unsafe { syscall3(SYS_FCNTL, fd, cmd, arg) }
}

/// Replace the current process with a new executable
pub fn fexec(fd: usize, args: &[[usize; 2]], vars: &[[usize; 2]]) -> Result<usize> {
    unsafe { syscall5(SYS_FEXEC, fd, args.as_ptr() as usize, args.len(), vars.as_ptr() as usize, vars.len()) }
}

/// Map a file into memory, but with the ability to set the address to map into, either as a hint
/// or as a requirement of the map.
///
/// # Errors
/// `EACCES` - the file descriptor was not open for reading
/// `EBADF` - if the file descriptor was invalid
/// `ENODEV` - mmapping was not supported
/// `EINVAL` - invalid combination of flags
/// `EEXIST` - if [`MapFlags::MAP_FIXED`] was set, and the address specified was already in use.
///
pub unsafe fn fmap(fd: usize, map: &Map) -> Result<usize> {
    syscall3(SYS_FMAP, fd, map as *const Map as usize, mem::size_of::<Map>())
}

/// Unmap whole (or partial) continous memory-mapped files
pub unsafe fn funmap(addr: usize, len: usize) -> Result<usize> {
    syscall2(SYS_FUNMAP, addr, len)
}

/// Retrieve the canonical path of a file
pub fn fpath(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_FPATH, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

/// Rename a file
pub fn frename<T: AsRef<str>>(fd: usize, path: T) -> Result<usize> {
    unsafe { syscall3(SYS_FRENAME, fd, path.as_ref().as_ptr() as usize, path.as_ref().len()) }
}

/// Get metadata about a file
pub fn fstat(fd: usize, stat: &mut Stat) -> Result<usize> {
    unsafe { syscall3(SYS_FSTAT, fd, stat as *mut Stat as usize, mem::size_of::<Stat>()) }
}

/// Get metadata about a filesystem
pub fn fstatvfs(fd: usize, stat: &mut StatVfs) -> Result<usize> {
    unsafe { syscall3(SYS_FSTATVFS, fd, stat as *mut StatVfs as usize, mem::size_of::<StatVfs>()) }
}

/// Sync a file descriptor to its underlying medium
pub fn fsync(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_FSYNC, fd) }
}

/// Truncate or extend a file to a specified length
pub fn ftruncate(fd: usize, len: usize) -> Result<usize> {
    unsafe { syscall2(SYS_FTRUNCATE, fd, len) }
}

// Change modify and/or access times
pub fn futimens(fd: usize, times: &[TimeSpec]) -> Result<usize> {
    unsafe { syscall3(SYS_FUTIMENS, fd, times.as_ptr() as usize, times.len() * mem::size_of::<TimeSpec>()) }
}

/// Fast userspace mutex
pub unsafe fn futex(addr: *mut i32, op: usize, val: i32, val2: usize, addr2: *mut i32)
                    -> Result<usize> {
    syscall5(SYS_FUTEX, addr as usize, op, (val as isize) as usize, val2, addr2 as usize)
}

/// Get the current working directory
pub fn getcwd(buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall2(SYS_GETCWD, buf.as_mut_ptr() as usize, buf.len()) }
}

/// Get the effective group ID
pub fn getegid() -> Result<usize> {
    unsafe { syscall0(SYS_GETEGID) }
}

/// Get the effective namespace
pub fn getens() -> Result<usize> {
    unsafe { syscall0(SYS_GETENS) }
}

/// Get the effective user ID
pub fn geteuid() -> Result<usize> {
    unsafe { syscall0(SYS_GETEUID) }
}

/// Get the current group ID
pub fn getgid() -> Result<usize> {
    unsafe { syscall0(SYS_GETGID) }
}

/// Get the current namespace
pub fn getns() -> Result<usize> {
    unsafe { syscall0(SYS_GETNS) }
}

/// Get the current process ID
pub fn getpid() -> Result<usize> {
    unsafe { syscall0(SYS_GETPID) }
}

/// Get the process group ID
pub fn getpgid(pid: usize) -> Result<usize> {
    unsafe { syscall1(SYS_GETPGID, pid) }
}

/// Get the parent process ID
pub fn getppid() -> Result<usize> {
    unsafe { syscall0(SYS_GETPPID) }
}

/// Get the current user ID
pub fn getuid() -> Result<usize> {
    unsafe { syscall0(SYS_GETUID) }
}

/// Set the I/O privilege level
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
/// * `EINVAL` - `level > 3`
pub unsafe fn iopl(level: usize) -> Result<usize> {
    syscall1(SYS_IOPL, level)
}

/// Send a signal `sig` to the process identified by `pid`
pub fn kill(pid: usize, sig: usize) -> Result<usize> {
    unsafe { syscall2(SYS_KILL, pid, sig) }
}

/// Create a link to a file
pub unsafe fn link(old: *const u8, new: *const u8) -> Result<usize> {
    syscall2(SYS_LINK, old as usize, new as usize)
}

/// Seek to `offset` bytes in a file descriptor
pub fn lseek(fd: usize, offset: isize, whence: usize) -> Result<usize> {
    unsafe { syscall3(SYS_LSEEK, fd, offset as usize, whence) }
}

/// Make a new scheme namespace
pub fn mkns(schemes: &[[usize; 2]]) -> Result<usize> {
    unsafe { syscall2(SYS_MKNS, schemes.as_ptr() as usize, schemes.len()) }
}

/// Change mapping flags
pub unsafe fn mprotect(addr: usize, size: usize, flags: MapFlags) -> Result<usize> {
    syscall3(SYS_MPROTECT, addr, size, flags.bits())
}

/// Sleep for the time specified in `req`
pub fn nanosleep(req: &TimeSpec, rem: &mut TimeSpec) -> Result<usize> {
    unsafe { syscall2(SYS_NANOSLEEP, req as *const TimeSpec as usize,
                                     rem as *mut TimeSpec as usize) }
}

/// Open a file
pub fn open<T: AsRef<str>>(path: T, flags: usize) -> Result<usize> {
    unsafe { syscall3(SYS_OPEN, path.as_ref().as_ptr() as usize, path.as_ref().len(), flags) }
}

/// Allocate frames, linearly in physical memory.
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
/// * `ENOMEM` - the system has run out of available memory
pub unsafe fn physalloc(size: usize) -> Result<usize> {
    syscall1(SYS_PHYSALLOC, size)
}

/// Allocate frames, linearly in physical memory, with an extra set of flags. If the flags contain
/// [`PARTIAL_ALLOC`], this will result in `physalloc3` with `min = 1`.
///
/// Refer to the simpler [`physalloc`] and the more complex [`physalloc3`], that this convenience
/// function is based on.
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
/// * `ENOMEM` - the system has run out of available memory
pub unsafe fn physalloc2(size: usize, flags: usize) -> Result<usize> {
    let mut ret = 1usize;
    physalloc3(size, flags, &mut ret)
}

/// Allocate frames, linearly in physical memory, with an extra set of flags. If the flags contain
/// [`PARTIAL_ALLOC`], the `min` parameter specifies the number of frames that have to be allocated
/// for this operation to succeed. The return value is the offset of the first frame, and `min` is
/// overwritten with the number of frames actually allocated.
///
/// Refer to the simpler [`physalloc`] and the simpler library function [`physalloc2`].
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
/// * `ENOMEM` - the system has run out of available memory
/// * `EINVAL` - `min = 0`
pub unsafe fn physalloc3(size: usize, flags: usize, min: &mut usize) -> Result<usize> {
    syscall3(SYS_PHYSALLOC3, size, flags, min as *mut usize as usize)
}

/// Free physically allocated pages
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
pub unsafe fn physfree(physical_address: usize, size: usize) -> Result<usize> {
    syscall2(SYS_PHYSFREE, physical_address, size)
}

/// Map physical memory to virtual memory
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
pub unsafe fn physmap(physical_address: usize, size: usize, flags: PhysmapFlags) -> Result<usize> {
    syscall3(SYS_PHYSMAP, physical_address, size, flags.bits())
}

/// Unmap previously mapped physical memory
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
/// * `EFAULT` - `virtual_address` has not been mapped
pub unsafe fn physunmap(virtual_address: usize) -> Result<usize> {
    syscall1(SYS_PHYSUNMAP, virtual_address)
}

/// Create a pair of file descriptors referencing the read and write ends of a pipe
pub fn pipe2(fds: &mut [usize; 2], flags: usize) -> Result<usize> {
    unsafe { syscall2(SYS_PIPE2, fds.as_ptr() as usize, flags) }
}

/// Read from a file descriptor into a buffer
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

/// Remove a directory
pub fn rmdir<T: AsRef<str>>(path: T) -> Result<usize> {
    unsafe { syscall2(SYS_RMDIR, path.as_ref().as_ptr() as usize, path.as_ref().len()) }
}

/// Set the process group ID
pub fn setpgid(pid: usize, pgid: usize) -> Result<usize> {
    unsafe { syscall2(SYS_SETPGID, pid, pgid) }
}

/// Set the current process group IDs
pub fn setregid(rgid: usize, egid: usize) -> Result<usize> {
    unsafe { syscall2(SYS_SETREGID, rgid, egid) }
}

/// Make a new scheme namespace
pub fn setrens(rns: usize, ens: usize) -> Result<usize> {
    unsafe { syscall2(SYS_SETRENS, rns, ens) }
}

/// Set the current process user IDs
pub fn setreuid(ruid: usize, euid: usize) -> Result<usize> {
    unsafe { syscall2(SYS_SETREUID, ruid, euid) }
}

/// Set up a signal handler
pub fn sigaction(sig: usize, act: Option<&SigAction>, oldact: Option<&mut SigAction>) -> Result<usize> {
    unsafe { syscall4(SYS_SIGACTION, sig,
                      act.map(|x| x as *const _).unwrap_or_else(ptr::null) as usize,
                      oldact.map(|x| x as *mut _).unwrap_or_else(ptr::null_mut) as usize,
                      restorer as usize) }
}

/// Get and/or set signal masks
pub fn sigprocmask(how: usize, set: Option<&[u64; 2]>, oldset: Option<&mut [u64; 2]>) -> Result<usize> {
    unsafe { syscall3(SYS_SIGPROCMASK, how,
                      set.map(|x| x as *const _).unwrap_or_else(ptr::null) as usize,
                      oldset.map(|x| x as *mut _).unwrap_or_else(ptr::null_mut) as usize) }
}

// Return from signal handler
pub fn sigreturn() -> Result<usize> {
    unsafe { syscall0(SYS_SIGRETURN) }
}

/// Set the file mode creation mask
pub fn umask(mask: usize) -> Result<usize> {
    unsafe { syscall1(SYS_UMASK, mask) }
}

/// Remove a file
pub fn unlink<T: AsRef<str>>(path: T) -> Result<usize> {
    unsafe { syscall2(SYS_UNLINK, path.as_ref().as_ptr() as usize, path.as_ref().len()) }
}

/// Convert a virtual address to a physical one
///
/// # Errors
///
/// * `EPERM` - `uid != 0`
pub unsafe fn virttophys(virtual_address: usize) -> Result<usize> {
    syscall1(SYS_VIRTTOPHYS, virtual_address)
}

/// Check if a child process has exited or received a signal
pub fn waitpid(pid: usize, status: &mut usize, options: WaitFlags) -> Result<usize> {
    unsafe { syscall3(SYS_WAITPID, pid, status as *mut usize as usize, options.bits()) }
}

/// Write a buffer to a file descriptor
///
/// The kernel will attempt to write the bytes in `buf` to the file descriptor `fd`, returning
/// either an `Err`, explained below, or `Ok(count)` where `count` is the number of bytes which
/// were written.
///
/// # Errors
///
/// * `EAGAIN` - the file descriptor was opened with `O_NONBLOCK` and writing would block
/// * `EBADF` - the file descriptor is not valid or is not open for writing
/// * `EFAULT` - `buf` does not point to the process's addressible memory
/// * `EIO` - an I/O error occurred
/// * `ENOSPC` - the device containing the file descriptor has no room for data
/// * `EPIPE` - the file descriptor refers to a pipe or socket whose reading end is closed
pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}

/// Yield the process's time slice to the kernel
///
/// This function will return Ok(0) on success
pub fn sched_yield() -> Result<usize> {
    unsafe { syscall0(SYS_YIELD) }
}
