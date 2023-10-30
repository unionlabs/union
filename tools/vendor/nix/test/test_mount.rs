mod common;

// Impelmentation note: to allow unprivileged users to run it, this test makes
// use of user and mount namespaces. On systems that allow unprivileged user
// namespaces (Linux >= 3.8 compiled with CONFIG_USER_NS), the test should run
// without root.

#[cfg(target_os = "linux")]
mod test_mount {
    use std::fs::{self, File};
    use std::io::{self, Read, Write};
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{self, Command};

    use libc::{EACCES, EROFS};

    use nix::errno::Errno;
    use nix::mount::{mount, umount, MsFlags};
    use nix::sched::{unshare, CloneFlags};
    use nix::sys::stat::{self, Mode};
    use nix::unistd::getuid;

    static SCRIPT_CONTENTS: &[u8] = b"#!/bin/sh
exit 23";

    const EXPECTED_STATUS: i32 = 23;

    const NONE: Option<&'static [u8]> = None;
    #[allow(clippy::bind_instead_of_map)]   // False positive
    pub fn test_mount_tmpfs_without_flags_allows_rwx() {
        let tempdir = tempfile::tempdir().unwrap();

        mount(NONE,
              tempdir.path(),
              Some(b"tmpfs".as_ref()),
              MsFlags::empty(),
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        let test_path = tempdir.path().join("test");

        // Verify write.
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode((Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO).bits())
            .open(&test_path)
            .or_else(|e|
                if Errno::from_i32(e.raw_os_error().unwrap()) == Errno::EOVERFLOW {
                    // Skip tests on certain Linux kernels which have a bug
                    // regarding tmpfs in namespaces.
                    // Ubuntu 14.04 and 16.04 are known to be affected; 16.10 is
                    // not.  There is no legitimate reason for open(2) to return
                    // EOVERFLOW here.
                    // https://bugs.launchpad.net/ubuntu/+source/linux/+bug/1659087
                    let stderr = io::stderr();
                    let mut handle = stderr.lock();
                    writeln!(handle, "Buggy Linux kernel detected.  Skipping test.")
                    .unwrap();
                    process::exit(0);
               } else {
                   panic!("open failed: {}", e);
               }
            )
            .and_then(|mut f| f.write(SCRIPT_CONTENTS))
            .unwrap_or_else(|e| panic!("write failed: {}", e));

        // Verify read.
        let mut buf = Vec::new();
        File::open(&test_path)
            .and_then(|mut f| f.read_to_end(&mut buf))
            .unwrap_or_else(|e| panic!("read failed: {}", e));
        assert_eq!(buf, SCRIPT_CONTENTS);

        // Verify execute.
        assert_eq!(EXPECTED_STATUS,
                   Command::new(&test_path)
                       .status()
                       .unwrap_or_else(|e| panic!("exec failed: {}", e))
                       .code()
                       .unwrap_or_else(|| panic!("child killed by signal")));

        umount(tempdir.path()).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_rdonly_disallows_write() {
        let tempdir = tempfile::tempdir().unwrap();

        mount(NONE,
              tempdir.path(),
              Some(b"tmpfs".as_ref()),
              MsFlags::MS_RDONLY,
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        // EROFS: Read-only file system
        assert_eq!(EROFS as i32,
                   File::create(tempdir.path().join("test")).unwrap_err().raw_os_error().unwrap());

        umount(tempdir.path()).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_noexec_disallows_exec() {
        let tempdir = tempfile::tempdir().unwrap();

        mount(NONE,
              tempdir.path(),
              Some(b"tmpfs".as_ref()),
              MsFlags::MS_NOEXEC,
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        let test_path = tempdir.path().join("test");

        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode((Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO).bits())
            .open(&test_path)
            .and_then(|mut f| f.write(SCRIPT_CONTENTS))
            .unwrap_or_else(|e| panic!("write failed: {}", e));

        // Verify that we cannot execute despite a+x permissions being set.
        let mode = stat::Mode::from_bits_truncate(fs::metadata(&test_path)
                                                      .map(|md| md.permissions().mode())
                                                      .unwrap_or_else(|e| {
                                                          panic!("metadata failed: {}", e)
                                                      }));

        assert!(mode.contains(Mode::S_IXUSR | Mode::S_IXGRP | Mode::S_IXOTH),
                "{:?} did not have execute permissions",
                &test_path);

        // EACCES: Permission denied
        assert_eq!(EACCES as i32,
                   Command::new(&test_path).status().unwrap_err().raw_os_error().unwrap());

        umount(tempdir.path()).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_bind() {
        let tempdir = tempfile::tempdir().unwrap();
        let file_name = "test";

        {
            let mount_point = tempfile::tempdir().unwrap();

            mount(Some(tempdir.path()),
                  mount_point.path(),
                  NONE,
                  MsFlags::MS_BIND,
                  NONE)
                .unwrap_or_else(|e| panic!("mount failed: {}", e));

            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode((Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO).bits())
                .open(mount_point.path().join(file_name))
                .and_then(|mut f| f.write(SCRIPT_CONTENTS))
                .unwrap_or_else(|e| panic!("write failed: {}", e));

            umount(mount_point.path()).unwrap_or_else(|e| panic!("umount failed: {}", e));
        }

        // Verify the file written in the mount shows up in source directory, even
        // after unmounting.

        let mut buf = Vec::new();
        File::open(tempdir.path().join(file_name))
            .and_then(|mut f| f.read_to_end(&mut buf))
            .unwrap_or_else(|e| panic!("read failed: {}", e));
        assert_eq!(buf, SCRIPT_CONTENTS);
    }

    pub fn setup_namespaces() {
        // Hold on to the uid in the parent namespace.
        let uid = getuid();

        unshare(CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUSER).unwrap_or_else(|e| {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            writeln!(handle,
                     "unshare failed: {}. Are unprivileged user namespaces available?",
                     e).unwrap();
            writeln!(handle, "mount is not being tested").unwrap();
            // Exit with success because not all systems support unprivileged user namespaces, and
            // that's not what we're testing for.
            process::exit(0);
        });

        // Map user as uid 1000.
        fs::OpenOptions::new()
            .write(true)
            .open("/proc/self/uid_map")
            .and_then(|mut f| f.write(format!("1000 {} 1\n", uid).as_bytes()))
            .unwrap_or_else(|e| panic!("could not write uid map: {}", e));
    }
}


// Test runner

/// Mimic normal test output (hackishly).
#[cfg(target_os = "linux")]
macro_rules! run_tests {
    ( $($test_fn:ident),* ) => {{
        println!();

        $(
            print!("test test_mount::{} ... ", stringify!($test_fn));
            $test_fn();
            println!("ok");
        )*

        println!();
    }}
}

#[cfg(target_os = "linux")]
fn main() {
    use test_mount::{setup_namespaces, test_mount_tmpfs_without_flags_allows_rwx,
                     test_mount_rdonly_disallows_write, test_mount_noexec_disallows_exec,
                     test_mount_bind};
    skip_if_cirrus!("Fails for an unknown reason Cirrus CI.  Bug #1351");
    setup_namespaces();

    run_tests!(test_mount_tmpfs_without_flags_allows_rwx,
               test_mount_rdonly_disallows_write,
               test_mount_noexec_disallows_exec,
               test_mount_bind);
}

#[cfg(not(target_os = "linux"))]
fn main() {}
