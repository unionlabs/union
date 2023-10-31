#[inline]
pub unsafe extern "C" fn creat64(path: *const ::c_char, mode: ::mode_t) -> ::c_int {
    ::creat(path, mode)
}

#[inline]
pub unsafe extern "C" fn fallocate64(
    fd: ::c_int,
    mode: ::c_int,
    offset: ::off64_t,
    len: ::off64_t,
) -> ::c_int {
    ::fallocate(fd, mode, offset, len)
}

#[inline]
pub unsafe extern "C" fn fgetpos64(stream: *mut ::FILE, pos: *mut ::fpos64_t) -> ::c_int {
    ::fgetpos(stream, pos as *mut _)
}

#[inline]
pub unsafe extern "C" fn fopen64(pathname: *const ::c_char, mode: *const ::c_char) -> *mut ::FILE {
    ::fopen(pathname, mode)
}

#[inline]
pub unsafe extern "C" fn freopen64(
    pathname: *const ::c_char,
    mode: *const ::c_char,
    stream: *mut ::FILE,
) -> *mut ::FILE {
    ::freopen(pathname, mode, stream)
}

#[inline]
pub unsafe extern "C" fn fseeko64(
    stream: *mut ::FILE,
    offset: ::off64_t,
    whence: ::c_int,
) -> ::c_int {
    ::fseeko(stream, offset, whence)
}

#[inline]
pub unsafe extern "C" fn fsetpos64(stream: *mut ::FILE, pos: *const ::fpos64_t) -> ::c_int {
    ::fsetpos(stream, pos as *mut _)
}

#[inline]
pub unsafe extern "C" fn fstat64(fildes: ::c_int, buf: *mut ::stat64) -> ::c_int {
    ::fstat(fildes, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn fstatat64(
    fd: ::c_int,
    path: *const ::c_char,
    buf: *mut ::stat64,
    flag: ::c_int,
) -> ::c_int {
    ::fstatat(fd, path, buf as *mut _, flag)
}

#[inline]
pub unsafe extern "C" fn fstatfs64(fd: ::c_int, buf: *mut ::statfs64) -> ::c_int {
    ::fstatfs(fd, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn fstatvfs64(fd: ::c_int, buf: *mut ::statvfs64) -> ::c_int {
    ::fstatvfs(fd, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn ftello64(stream: *mut ::FILE) -> ::off64_t {
    ::ftello(stream)
}

#[inline]
pub unsafe extern "C" fn ftruncate64(fd: ::c_int, length: ::off64_t) -> ::c_int {
    ::ftruncate(fd, length)
}

#[inline]
pub unsafe extern "C" fn getrlimit64(resource: ::c_int, rlim: *mut ::rlimit64) -> ::c_int {
    ::getrlimit(resource, rlim as *mut _)
}

#[inline]
pub unsafe extern "C" fn lseek64(fd: ::c_int, offset: ::off64_t, whence: ::c_int) -> ::off64_t {
    ::lseek(fd, offset, whence)
}

#[inline]
pub unsafe extern "C" fn lstat64(path: *const ::c_char, buf: *mut ::stat64) -> ::c_int {
    ::lstat(path, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn mmap64(
    addr: *mut ::c_void,
    length: ::size_t,
    prot: ::c_int,
    flags: ::c_int,
    fd: ::c_int,
    offset: ::off64_t,
) -> *mut ::c_void {
    ::mmap(addr, length, prot, flags, fd, offset)
}

// These functions are variadic in the C ABI since the `mode` argument is "optional".  Variadic
// `extern "C"` functions are unstable in Rust so we cannot write a shim function for these
// entrypoints.  See https://github.com/rust-lang/rust/issues/44930.
//
// These aliases are mostly fine though, neither function takes a LFS64-namespaced type as an
// argument, nor do their names clash with any declared types.
pub use open as open64;
pub use openat as openat64;

#[inline]
pub unsafe extern "C" fn posix_fadvise64(
    fd: ::c_int,
    offset: ::off64_t,
    len: ::off64_t,
    advice: ::c_int,
) -> ::c_int {
    ::posix_fadvise(fd, offset, len, advice)
}

#[inline]
pub unsafe extern "C" fn posix_fallocate64(
    fd: ::c_int,
    offset: ::off64_t,
    len: ::off64_t,
) -> ::c_int {
    ::posix_fallocate(fd, offset, len)
}

#[inline]
pub unsafe extern "C" fn pread64(
    fd: ::c_int,
    buf: *mut ::c_void,
    count: ::size_t,
    offset: ::off64_t,
) -> ::ssize_t {
    ::pread(fd, buf, count, offset)
}

#[inline]
pub unsafe extern "C" fn preadv64(
    fd: ::c_int,
    iov: *const ::iovec,
    iovcnt: ::c_int,
    offset: ::off64_t,
) -> ::ssize_t {
    ::preadv(fd, iov, iovcnt, offset)
}

#[inline]
pub unsafe extern "C" fn prlimit64(
    pid: ::pid_t,
    resource: ::c_int,
    new_limit: *const ::rlimit64,
    old_limit: *mut ::rlimit64,
) -> ::c_int {
    ::prlimit(pid, resource, new_limit as *mut _, old_limit as *mut _)
}

#[inline]
pub unsafe extern "C" fn pwrite64(
    fd: ::c_int,
    buf: *const ::c_void,
    count: ::size_t,
    offset: ::off64_t,
) -> ::ssize_t {
    ::pwrite(fd, buf, count, offset)
}

#[inline]
pub unsafe extern "C" fn pwritev64(
    fd: ::c_int,
    iov: *const ::iovec,
    iovcnt: ::c_int,
    offset: ::off64_t,
) -> ::ssize_t {
    ::pwritev(fd, iov, iovcnt, offset)
}

#[inline]
pub unsafe extern "C" fn readdir64(dirp: *mut ::DIR) -> *mut ::dirent64 {
    ::readdir(dirp) as *mut _
}

#[inline]
pub unsafe extern "C" fn readdir64_r(
    dirp: *mut ::DIR,
    entry: *mut ::dirent64,
    result: *mut *mut ::dirent64,
) -> ::c_int {
    ::readdir_r(dirp, entry as *mut _, result as *mut _)
}

#[inline]
pub unsafe extern "C" fn sendfile64(
    out_fd: ::c_int,
    in_fd: ::c_int,
    offset: *mut ::off64_t,
    count: ::size_t,
) -> ::ssize_t {
    ::sendfile(out_fd, in_fd, offset, count)
}

#[inline]
pub unsafe extern "C" fn setrlimit64(resource: ::c_int, rlim: *const ::rlimit64) -> ::c_int {
    ::setrlimit(resource, rlim as *mut _)
}

#[inline]
pub unsafe extern "C" fn stat64(pathname: *const ::c_char, statbuf: *mut ::stat64) -> ::c_int {
    ::stat(pathname, statbuf as *mut _)
}

#[inline]
pub unsafe extern "C" fn statfs64(pathname: *const ::c_char, buf: *mut ::statfs64) -> ::c_int {
    ::statfs(pathname, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn statvfs64(path: *const ::c_char, buf: *mut ::statvfs64) -> ::c_int {
    ::statvfs(path, buf as *mut _)
}

#[inline]
pub unsafe extern "C" fn tmpfile64() -> *mut ::FILE {
    ::tmpfile()
}

#[inline]
pub unsafe extern "C" fn truncate64(path: *const ::c_char, length: ::off64_t) -> ::c_int {
    ::truncate(path, length)
}
