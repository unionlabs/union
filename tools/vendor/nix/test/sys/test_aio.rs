use libc::{c_int, c_void};
use nix::Result;
use nix::errno::*;
use nix::sys::aio::*;
use nix::sys::signal::{SaFlags, SigAction, sigaction, SigevNotify, SigHandler, Signal, SigSet};
use nix::sys::time::{TimeSpec, TimeValLike};
use std::io::{Write, Read, Seek, SeekFrom};
use std::ops::Deref;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time};
use tempfile::tempfile;

// Helper that polls an AioCb for completion or error
fn poll_aio(aiocb: &mut Pin<Box<AioCb>>) -> Result<()> {
    loop {
        let err = aiocb.error();
        if err != Err(Errno::EINPROGRESS) { return err; };
        thread::sleep(time::Duration::from_millis(10));
    }
}

// Helper that polls a component of an LioCb for completion or error
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn poll_lio(liocb: &mut LioCb, i: usize) -> Result<()> {
    loop {
        let err = liocb.error(i);
        if err != Err(Errno::EINPROGRESS) { return err; };
        thread::sleep(time::Duration::from_millis(10));
    }
}

#[test]
fn test_accessors() {
    let mut rbuf = vec![0; 4];
    let aiocb = AioCb::from_mut_slice( 1001,
                           2,   //offset
                           &mut rbuf,
                           42,   //priority
                           SigevNotify::SigevSignal {
                               signal: Signal::SIGUSR2,
                               si_value: 99
                           },
                           LioOpcode::LIO_NOP);
    assert_eq!(1001, aiocb.fd());
    assert_eq!(Some(LioOpcode::LIO_NOP), aiocb.lio_opcode());
    assert_eq!(4, aiocb.nbytes());
    assert_eq!(2, aiocb.offset());
    assert_eq!(42, aiocb.priority());
    let sev = aiocb.sigevent().sigevent();
    assert_eq!(Signal::SIGUSR2 as i32, sev.sigev_signo);
    assert_eq!(99, sev.sigev_value.sival_ptr as i64);
}

// Tests AioCb.cancel.  We aren't trying to test the OS's implementation, only
// our bindings.  So it's sufficient to check that AioCb.cancel returned any
// AioCancelStat value.
#[test]
#[cfg_attr(target_env = "musl", ignore)]
fn test_cancel() {
    let wbuf: &[u8] = b"CDEF";

    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                            0,   //offset
                            wbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
    let err = aiocb.error();
    assert!(err == Ok(()) || err == Err(Errno::EINPROGRESS));

    let cancelstat = aiocb.cancel();
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aiocb.aio_return();
}

// Tests using aio_cancel_all for all outstanding IOs.
#[test]
#[cfg_attr(target_env = "musl", ignore)]
fn test_aio_cancel_all() {
    let wbuf: &[u8] = b"CDEF";

    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_slice(f.as_raw_fd(),
                            0,   //offset
                            wbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
    let err = aiocb.error();
    assert!(err == Ok(()) || err == Err(Errno::EINPROGRESS));

    let cancelstat = aio_cancel_all(f.as_raw_fd());
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aiocb.aio_return();
}

#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_fsync() {
    const INITIAL: &[u8] = b"abcdef123456";
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    let mut aiocb = AioCb::from_fd( f.as_raw_fd(),
                            0,   //priority
                            SigevNotify::SigevNone);
    let err = aiocb.fsync(AioFsyncMode::O_SYNC);
    assert!(err.is_ok());
    poll_aio(&mut aiocb).unwrap();
    aiocb.aio_return().unwrap();
}

/// `AioCb::fsync` should not modify the `AioCb` object if `libc::aio_fsync` returns
/// an error
// Skip on Linux, because Linux's AIO implementation can't detect errors
// synchronously
#[test]
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
fn test_fsync_error() {
    use std::mem;

    const INITIAL: &[u8] = b"abcdef123456";
    // Create an invalid AioFsyncMode
    let mode = unsafe { mem::transmute(666) };
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    let mut aiocb = AioCb::from_fd( f.as_raw_fd(),
                            0,   //priority
                            SigevNotify::SigevNone);
    let err = aiocb.fsync(mode);
    assert!(err.is_err());
}

#[test]
// On Cirrus on Linux, this test fails due to a glibc bug.
// https://github.com/nix-rust/nix/issues/1099
#[cfg_attr(target_os = "linux", ignore)]
// On Cirrus, aio_suspend is failing with EINVAL
// https://github.com/nix-rust/nix/issues/1361
#[cfg_attr(target_os = "macos", ignore)]
fn test_aio_suspend() {
    const INITIAL: &[u8] = b"abcdef123456";
    const WBUF: &[u8] = b"CDEFG";
    let timeout = TimeSpec::seconds(10);
    let mut rbuf = vec![0; 4];
    let rlen = rbuf.len();
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();

    let mut wcb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_WRITE);

    let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                            8,   //offset
                            &mut rbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_READ);
    wcb.write().unwrap();
    rcb.read().unwrap();
    loop {
        {
            let cbbuf = [wcb.as_ref(), rcb.as_ref()];
            let r = aio_suspend(&cbbuf[..], Some(timeout));
            match r {
                Err(Errno::EINTR) => continue,
                Err(e) => panic!("aio_suspend returned {:?}", e),
                Ok(_) => ()
            };
        }
        if rcb.error() != Err(Errno::EINPROGRESS) &&
           wcb.error() != Err(Errno::EINPROGRESS) {
            break
        }
    }

    assert_eq!(wcb.aio_return().unwrap() as usize, WBUF.len());
    assert_eq!(rcb.aio_return().unwrap() as usize, rlen);
}

// Test a simple aio operation with no completion notification.  We must poll
// for completion
#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_read() {
    const INITIAL: &[u8] = b"abcdef123456";
    let mut rbuf = vec![0; 4];
    const EXPECT: &[u8] = b"cdef";
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    {
        let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                               2,   //offset
                               &mut rbuf,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_NOP);
        aiocb.read().unwrap();

        let err = poll_aio(&mut aiocb);
        assert_eq!(err, Ok(()));
        assert_eq!(aiocb.aio_return().unwrap() as usize, EXPECT.len());
    }

    assert_eq!(EXPECT, rbuf.deref().deref());
}

/// `AioCb::read` should not modify the `AioCb` object if `libc::aio_read`
/// returns an error
// Skip on Linux, because Linux's AIO implementation can't detect errors
// synchronously
#[test]
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
fn test_read_error() {
    const INITIAL: &[u8] = b"abcdef123456";
    let mut rbuf = vec![0; 4];
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                           -1,   //an invalid offset
                           &mut rbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    assert!(aiocb.read().is_err());
}

// Tests from_mut_slice
#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_read_into_mut_slice() {
    const INITIAL: &[u8] = b"abcdef123456";
    let mut rbuf = vec![0; 4];
    const EXPECT: &[u8] = b"cdef";
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    {
        let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                               2,   //offset
                               &mut rbuf,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_NOP);
        aiocb.read().unwrap();

        let err = poll_aio(&mut aiocb);
        assert_eq!(err, Ok(()));
        assert_eq!(aiocb.aio_return().unwrap() as usize, EXPECT.len());
    }

    assert_eq!(rbuf, EXPECT);
}

// Tests from_ptr
#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_read_into_pointer() {
    const INITIAL: &[u8] = b"abcdef123456";
    let mut rbuf = vec![0; 4];
    const EXPECT: &[u8] = b"cdef";
    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    {
        // Safety: ok because rbuf lives until after poll_aio
        let mut aiocb = unsafe {
            AioCb::from_mut_ptr( f.as_raw_fd(),
                                 2,   //offset
                                 rbuf.as_mut_ptr() as *mut c_void,
                                 rbuf.len(),
                                 0,   //priority
                                 SigevNotify::SigevNone,
                                 LioOpcode::LIO_NOP)
        };
        aiocb.read().unwrap();

        let err = poll_aio(&mut aiocb);
        assert_eq!(err, Ok(()));
        assert_eq!(aiocb.aio_return().unwrap() as usize, EXPECT.len());
    }

    assert_eq!(rbuf, EXPECT);
}

// Test reading into an immutable buffer.  It should fail
// FIXME: This test fails to panic on Linux/musl
#[test]
#[should_panic(expected = "Can't read into an immutable buffer")]
#[cfg_attr(target_env = "musl", ignore)]
fn test_read_immutable_buffer() {
    let rbuf: &[u8] = b"CDEF";
    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           rbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    aiocb.read().unwrap();
}


// Test a simple aio operation with no completion notification.  We must poll
// for completion.  Unlike test_aio_read, this test uses AioCb::from_slice
#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_write() {
    const INITIAL: &[u8] = b"abcdef123456";
    let wbuf = "CDEF".to_string().into_bytes();
    let mut rbuf = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &wbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    aiocb.write().unwrap();

    let err = poll_aio(&mut aiocb);
    assert_eq!(err, Ok(()));
    assert_eq!(aiocb.aio_return().unwrap() as usize, wbuf.len());

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf, EXPECT);
}

// Tests `AioCb::from_ptr`
#[test]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_write_from_pointer() {
    const INITIAL: &[u8] = b"abcdef123456";
    let wbuf = "CDEF".to_string().into_bytes();
    let mut rbuf = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    // Safety: ok because aiocb outlives poll_aio
    let mut aiocb = unsafe {
        AioCb::from_ptr( f.as_raw_fd(),
                         2,   //offset
                         wbuf.as_ptr() as *const c_void,
                         wbuf.len(),
                         0,   //priority
                         SigevNotify::SigevNone,
                         LioOpcode::LIO_NOP)
    };
    aiocb.write().unwrap();

    let err = poll_aio(&mut aiocb);
    assert_eq!(err, Ok(()));
    assert_eq!(aiocb.aio_return().unwrap() as usize, wbuf.len());

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf, EXPECT);
}

/// `AioCb::write` should not modify the `AioCb` object if `libc::aio_write`
/// returns an error
// Skip on Linux, because Linux's AIO implementation can't detect errors
// synchronously
#[test]
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
fn test_write_error() {
    let wbuf = "CDEF".to_string().into_bytes();
    let mut aiocb = AioCb::from_slice( 666, // An invalid file descriptor
                           0,   //offset
                           &wbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    assert!(aiocb.write().is_err());
}

lazy_static! {
    pub static ref SIGNALED: AtomicBool = AtomicBool::new(false);
}

extern fn sigfunc(_: c_int) {
    SIGNALED.store(true, Ordering::Relaxed);
}

// Test an aio operation with completion delivered by a signal
// FIXME: This test is ignored on mips because of failures in qemu in CI
#[test]
#[cfg_attr(any(all(target_env = "musl", target_arch = "x86_64"), target_arch = "mips", target_arch = "mips64"), ignore)]
fn test_write_sigev_signal() {
    let _m = crate::SIGNAL_MTX.lock();
    let sa = SigAction::new(SigHandler::Handler(sigfunc),
                            SaFlags::SA_RESETHAND,
                            SigSet::empty());
    SIGNALED.store(false, Ordering::Relaxed);
    unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();

    const INITIAL: &[u8] = b"abcdef123456";
    const WBUF: &[u8] = b"CDEF";
    let mut rbuf = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write_all(INITIAL).unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           WBUF,
                           0,   //priority
                           SigevNotify::SigevSignal {
                               signal: Signal::SIGUSR2,
                               si_value: 0  //TODO: validate in sigfunc
                           },
                           LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
    while !SIGNALED.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_millis(10));
    }

    assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf, EXPECT);
}

// Test LioCb::listio with LIO_WAIT, so all AIO ops should be complete by the
// time listio returns.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_liocb_listio_wait() {
    const INITIAL: &[u8] = b"abcdef123456";
    const WBUF: &[u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let rlen = rbuf.len();
    let mut rbuf2 = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();

    f.write_all(INITIAL).unwrap();

    {
        let mut liocb = LioCbBuilder::with_capacity(2)
            .emplace_slice(
                f.as_raw_fd(),
                2,   //offset
                WBUF,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_WRITE
            ).emplace_mut_slice(
                f.as_raw_fd(),
                8,   //offset
                &mut rbuf,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_READ
            ).finish();
        let err = liocb.listio(LioMode::LIO_WAIT, SigevNotify::SigevNone);
        err.expect("lio_listio");

        assert_eq!(liocb.aio_return(0).unwrap() as usize, WBUF.len());
        assert_eq!(liocb.aio_return(1).unwrap() as usize, rlen);
    }
    assert_eq!(rbuf.deref().deref(), b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf2, EXPECT);
}

// Test LioCb::listio with LIO_NOWAIT and no SigEvent, so we must use some other
// mechanism to check for the individual AioCb's completion.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[cfg_attr(all(target_env = "musl", target_arch = "x86_64"), ignore)]
fn test_liocb_listio_nowait() {
    const INITIAL: &[u8] = b"abcdef123456";
    const WBUF: &[u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let rlen = rbuf.len();
    let mut rbuf2 = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();

    f.write_all(INITIAL).unwrap();

    {
        let mut liocb = LioCbBuilder::with_capacity(2)
            .emplace_slice(
                f.as_raw_fd(),
                2,   //offset
                WBUF,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_WRITE
            ).emplace_mut_slice(
                f.as_raw_fd(),
                8,   //offset
                &mut rbuf,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_READ
            ).finish();
        let err = liocb.listio(LioMode::LIO_NOWAIT, SigevNotify::SigevNone);
        err.expect("lio_listio");

        poll_lio(&mut liocb, 0).unwrap();
        poll_lio(&mut liocb, 1).unwrap();
        assert_eq!(liocb.aio_return(0).unwrap() as usize, WBUF.len());
        assert_eq!(liocb.aio_return(1).unwrap() as usize, rlen);
    }
    assert_eq!(rbuf.deref().deref(), b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf2, EXPECT);
}

// Test LioCb::listio with LIO_NOWAIT and a SigEvent to indicate when all
// AioCb's are complete.
// FIXME: This test is ignored on mips/mips64 because of failures in qemu in CI.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[cfg_attr(any(target_arch = "mips", target_arch = "mips64", target_env = "musl"), ignore)]
fn test_liocb_listio_signal() {
    let _m = crate::SIGNAL_MTX.lock();
    const INITIAL: &[u8] = b"abcdef123456";
    const WBUF: &[u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let rlen = rbuf.len();
    let mut rbuf2 = Vec::new();
    const EXPECT: &[u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();
    let sa = SigAction::new(SigHandler::Handler(sigfunc),
                            SaFlags::SA_RESETHAND,
                            SigSet::empty());
    let sigev_notify = SigevNotify::SigevSignal { signal: Signal::SIGUSR2,
                                                  si_value: 0 };

    f.write_all(INITIAL).unwrap();

    {
        let mut liocb = LioCbBuilder::with_capacity(2)
            .emplace_slice(
                f.as_raw_fd(),
                2,   //offset
                WBUF,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_WRITE
            ).emplace_mut_slice(
                f.as_raw_fd(),
                8,   //offset
                &mut rbuf,
                0,   //priority
                SigevNotify::SigevNone,
                LioOpcode::LIO_READ
            ).finish();
        SIGNALED.store(false, Ordering::Relaxed);
        unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();
        let err = liocb.listio(LioMode::LIO_NOWAIT, sigev_notify);
        err.expect("lio_listio");
        while !SIGNALED.load(Ordering::Relaxed) {
            thread::sleep(time::Duration::from_millis(10));
        }

        assert_eq!(liocb.aio_return(0).unwrap() as usize, WBUF.len());
        assert_eq!(liocb.aio_return(1).unwrap() as usize, rlen);
    }
    assert_eq!(rbuf.deref().deref(), b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert_eq!(len, EXPECT.len());
    assert_eq!(rbuf2, EXPECT);
}

// Try to use LioCb::listio to read into an immutable buffer.  It should fail
// FIXME: This test fails to panic on Linux/musl
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[should_panic(expected = "Can't read into an immutable buffer")]
#[cfg_attr(target_env = "musl", ignore)]
fn test_liocb_listio_read_immutable() {
    let rbuf: &[u8] = b"abcd";
    let f = tempfile().unwrap();


    let mut liocb = LioCbBuilder::with_capacity(1)
        .emplace_slice(
            f.as_raw_fd(),
            2,   //offset
            rbuf,
            0,   //priority
            SigevNotify::SigevNone,
            LioOpcode::LIO_READ
        ).finish();
    let _ = liocb.listio(LioMode::LIO_NOWAIT, SigevNotify::SigevNone);
}
