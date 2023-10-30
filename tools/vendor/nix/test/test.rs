#[macro_use]
extern crate cfg_if;
#[cfg_attr(not(target_os = "redox"), macro_use)]
extern crate nix;
#[macro_use]
extern crate lazy_static;

mod common;
mod sys;
#[cfg(not(target_os = "redox"))]
mod test_dir;
mod test_fcntl;
#[cfg(any(target_os = "android",
          target_os = "linux"))]
mod test_kmod;
#[cfg(target_os = "freebsd")]
mod test_nmount;
#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "fushsia",
          target_os = "linux",
          target_os = "netbsd"))]
mod test_mq;
#[cfg(not(target_os = "redox"))]
mod test_net;
mod test_nix_path;
mod test_resource;
mod test_poll;
#[cfg(not(any(target_os = "redox", target_os = "fuchsia")))]
mod test_pty;
#[cfg(any(target_os = "android",
          target_os = "linux"))]
mod test_sched;
#[cfg(any(target_os = "android",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos"))]
mod test_sendfile;
mod test_stat;
mod test_time;
mod test_unistd;

use std::os::unix::io::RawFd;
use std::path::PathBuf;
use parking_lot::{Mutex, RwLock, RwLockWriteGuard};
use nix::unistd::{chdir, getcwd, read};


/// Helper function analogous to `std::io::Read::read_exact`, but for `RawFD`s
fn read_exact(f: RawFd, buf: &mut  [u8]) {
    let mut len = 0;
    while len < buf.len() {
        // get_mut would be better than split_at_mut, but it requires nightly
        let (_, remaining) = buf.split_at_mut(len);
        len += read(f, remaining).unwrap();
    }
}

lazy_static! {
    /// Any test that changes the process's current working directory must grab
    /// the RwLock exclusively.  Any process that cares about the current
    /// working directory must grab it shared.
    pub static ref CWD_LOCK: RwLock<()> = RwLock::new(());
    /// Any test that creates child processes must grab this mutex, regardless
    /// of what it does with those children.
    pub static ref FORK_MTX: Mutex<()> = Mutex::new(());
    /// Any test that changes the process's supplementary groups must grab this
    /// mutex
    pub static ref GROUPS_MTX: Mutex<()> = Mutex::new(());
    /// Any tests that loads or unloads kernel modules must grab this mutex
    pub static ref KMOD_MTX: Mutex<()> = Mutex::new(());
    /// Any test that calls ptsname(3) must grab this mutex.
    pub static ref PTSNAME_MTX: Mutex<()> = Mutex::new(());
    /// Any test that alters signal handling must grab this mutex.
    pub static ref SIGNAL_MTX: Mutex<()> = Mutex::new(());
}

/// RAII object that restores a test's original directory on drop
struct DirRestore<'a> {
    d: PathBuf,
    _g: RwLockWriteGuard<'a, ()>
}

impl<'a> DirRestore<'a> {
    fn new() -> Self {
        let guard = crate::CWD_LOCK.write();
        DirRestore{
            _g: guard,
            d: getcwd().unwrap(),
        }
    }
}

impl<'a> Drop for DirRestore<'a> {
    fn drop(&mut self) {
        let r = chdir(&self.d);
        if std::thread::panicking() {
            r.unwrap();
        }
    }
}
