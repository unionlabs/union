use libc::{gid_t, pid_t, uid_t};

/// Credentials of a process.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct UCred {
    /// PID (process ID) of the process.
    pid: Option<pid_t>,
    /// UID (user ID) of the process.
    uid: uid_t,
    /// GID (group ID) of the process.
    gid: gid_t,
}

impl UCred {
    /// Gets UID (user ID) of the process.
    pub fn uid(&self) -> uid_t {
        self.uid
    }

    /// Gets GID (group ID) of the process.
    pub fn gid(&self) -> gid_t {
        self.gid
    }

    /// Gets PID (process ID) of the process.
    ///
    /// This is only implemented under Linux, Android, iOS, macOS, Solaris and
    /// Illumos. On other platforms this will always return `None`.
    pub fn pid(&self) -> Option<pid_t> {
        self.pid
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "openbsd"))]
pub(crate) use self::impl_linux::get_peer_cred;

#[cfg(any(target_os = "netbsd"))]
pub(crate) use self::impl_netbsd::get_peer_cred;

#[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
pub(crate) use self::impl_bsd::get_peer_cred;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) use self::impl_macos::get_peer_cred;

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
pub(crate) use self::impl_solaris::get_peer_cred;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "openbsd"))]
pub(crate) mod impl_linux {
    use crate::net::unix::UnixStream;

    use libc::{c_void, getsockopt, socklen_t, SOL_SOCKET, SO_PEERCRED};
    use std::{io, mem};

    #[cfg(target_os = "openbsd")]
    use libc::sockpeercred as ucred;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    use libc::ucred;

    pub(crate) fn get_peer_cred(sock: &UnixStream) -> io::Result<super::UCred> {
        use std::os::unix::io::AsRawFd;

        unsafe {
            let raw_fd = sock.as_raw_fd();

            let mut ucred = ucred {
                pid: 0,
                uid: 0,
                gid: 0,
            };

            let ucred_size = mem::size_of::<ucred>();

            // These paranoid checks should be optimized-out
            assert!(mem::size_of::<u32>() <= mem::size_of::<usize>());
            assert!(ucred_size <= u32::MAX as usize);

            let mut ucred_size = ucred_size as socklen_t;

            let ret = getsockopt(
                raw_fd,
                SOL_SOCKET,
                SO_PEERCRED,
                &mut ucred as *mut ucred as *mut c_void,
                &mut ucred_size,
            );
            if ret == 0 && ucred_size as usize == mem::size_of::<ucred>() {
                Ok(super::UCred {
                    uid: ucred.uid,
                    gid: ucred.gid,
                    pid: Some(ucred.pid),
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

#[cfg(any(target_os = "netbsd"))]
pub(crate) mod impl_netbsd {
    use crate::net::unix::UnixStream;

    use libc::{c_void, getsockopt, socklen_t, unpcbid, LOCAL_PEEREID, SOL_SOCKET};
    use std::io;
    use std::mem::size_of;
    use std::os::unix::io::AsRawFd;

    pub(crate) fn get_peer_cred(sock: &UnixStream) -> io::Result<super::UCred> {
        unsafe {
            let raw_fd = sock.as_raw_fd();

            let mut unpcbid = unpcbid {
                unp_pid: 0,
                unp_euid: 0,
                unp_egid: 0,
            };

            let unpcbid_size = size_of::<unpcbid>();
            let mut unpcbid_size = unpcbid_size as socklen_t;

            let ret = getsockopt(
                raw_fd,
                SOL_SOCKET,
                LOCAL_PEEREID,
                &mut unpcbid as *mut unpcbid as *mut c_void,
                &mut unpcbid_size,
            );
            if ret == 0 && unpcbid_size as usize == size_of::<unpcbid>() {
                Ok(super::UCred {
                    uid: unpcbid.unp_euid,
                    gid: unpcbid.unp_egid,
                    pid: Some(unpcbid.unp_pid),
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
pub(crate) mod impl_bsd {
    use crate::net::unix::UnixStream;

    use libc::getpeereid;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::unix::io::AsRawFd;

    pub(crate) fn get_peer_cred(sock: &UnixStream) -> io::Result<super::UCred> {
        unsafe {
            let raw_fd = sock.as_raw_fd();

            let mut uid = MaybeUninit::uninit();
            let mut gid = MaybeUninit::uninit();

            let ret = getpeereid(raw_fd, uid.as_mut_ptr(), gid.as_mut_ptr());

            if ret == 0 {
                Ok(super::UCred {
                    uid: uid.assume_init(),
                    gid: gid.assume_init(),
                    pid: None,
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) mod impl_macos {
    use crate::net::unix::UnixStream;

    use libc::{c_void, getpeereid, getsockopt, pid_t, LOCAL_PEEREPID, SOL_LOCAL};
    use std::io;
    use std::mem::size_of;
    use std::mem::MaybeUninit;
    use std::os::unix::io::AsRawFd;

    pub(crate) fn get_peer_cred(sock: &UnixStream) -> io::Result<super::UCred> {
        unsafe {
            let raw_fd = sock.as_raw_fd();

            let mut uid = MaybeUninit::uninit();
            let mut gid = MaybeUninit::uninit();
            let mut pid: MaybeUninit<pid_t> = MaybeUninit::uninit();
            let mut pid_size: MaybeUninit<u32> = MaybeUninit::new(size_of::<pid_t>() as u32);

            if getsockopt(
                raw_fd,
                SOL_LOCAL,
                LOCAL_PEEREPID,
                pid.as_mut_ptr() as *mut c_void,
                pid_size.as_mut_ptr(),
            ) != 0
            {
                return Err(io::Error::last_os_error());
            }

            assert!(pid_size.assume_init() == (size_of::<pid_t>() as u32));

            let ret = getpeereid(raw_fd, uid.as_mut_ptr(), gid.as_mut_ptr());

            if ret == 0 {
                Ok(super::UCred {
                    uid: uid.assume_init(),
                    gid: gid.assume_init(),
                    pid: Some(pid.assume_init()),
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
pub(crate) mod impl_solaris {
    use crate::net::unix::UnixStream;
    use std::io;
    use std::os::unix::io::AsRawFd;
    use std::ptr;

    pub(crate) fn get_peer_cred(sock: &UnixStream) -> io::Result<super::UCred> {
        unsafe {
            let raw_fd = sock.as_raw_fd();

            let mut cred = ptr::null_mut();
            let ret = libc::getpeerucred(raw_fd, &mut cred);

            if ret == 0 {
                let uid = libc::ucred_geteuid(cred);
                let gid = libc::ucred_getegid(cred);
                let pid = libc::ucred_getpid(cred);

                libc::ucred_free(cred);

                Ok(super::UCred {
                    uid,
                    gid,
                    pid: Some(pid),
                })
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}
