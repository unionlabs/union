use rayon_core::ThreadPoolBuilder;

use std::env;
use std::process::Command;

#[cfg(target_os = "linux")]
use std::os::unix::process::ExitStatusExt;

fn force_stack_overflow(depth: u32) {
    let _buffer = [0u8; 1024 * 1024];
    if depth > 0 {
        force_stack_overflow(depth - 1);
    }
}

#[cfg(unix)]
fn disable_core() {
    unsafe {
        libc::setrlimit(
            libc::RLIMIT_CORE,
            &libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            },
        );
    }
}

#[cfg(unix)]
fn overflow_code() -> Option<i32> {
    None
}

#[cfg(windows)]
fn overflow_code() -> Option<i32> {
    use std::os::windows::process::ExitStatusExt;
    use std::process::ExitStatus;

    ExitStatus::from_raw(0xc00000fd /*STATUS_STACK_OVERFLOW*/).code()
}

fn main() {
    if env::args().len() == 1 {
        // first check that the recursivecall actually causes a stack overflow, and does not get optimized away
        {
            let status = Command::new(env::current_exe().unwrap())
                .arg("8")
                .status()
                .unwrap();

            #[cfg(any(unix, windows))]
            assert_eq!(status.code(), overflow_code());

            #[cfg(target_os = "linux")]
            assert!(
                status.signal() == Some(11 /*SIGABRT*/) || status.signal() == Some(6 /*SIGSEGV*/)
            );
        }

        // now run with a larger stack and verify correct operation
        {
            let status = Command::new(env::current_exe().unwrap())
                .arg("48")
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(0));
            #[cfg(target_os = "linux")]
            assert_eq!(status.signal(), None);
        }
    } else {
        let stack_size_in_mb: usize = env::args().nth(1).unwrap().parse().unwrap();
        let pool = ThreadPoolBuilder::new()
            .stack_size(stack_size_in_mb * 1024 * 1024)
            .build()
            .unwrap();
        pool.install(|| {
            #[cfg(unix)]
            disable_core();
            force_stack_overflow(32);
        });
    }
}
