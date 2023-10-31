// vim: tw=80

// Annoyingly, Cargo is unable to conditionally build an entire test binary.  So
// we must disable the test here rather than in Cargo.toml
#![cfg(target_os = "freebsd")]

use nix::errno::*;
use nix::libc::off_t;
use nix::sys::aio::*;
use nix::sys::signal::SigevNotify;
use nix::unistd::{SysconfVar, sysconf};
use std::os::unix::io::AsRawFd;
use std::{thread, time};
use sysctl::CtlValue;
use tempfile::tempfile;

const BYTES_PER_OP: usize = 512;

/// Attempt to collect final status for all of `liocb`'s operations, freeing
/// system resources
fn finish_liocb(liocb: &mut LioCb) {
    for j in 0..liocb.len() {
        loop {
            let e = liocb.error(j);
            match e {
                Ok(()) => break,
                Err(Errno::EINPROGRESS) =>
                    thread::sleep(time::Duration::from_millis(10)),
                Err(x) => panic!("aio_error({:?})", x)
            }
        }
        assert_eq!(liocb.aio_return(j).unwrap(), BYTES_PER_OP as isize);
    }
}

// Deliberately exceed system resource limits, causing lio_listio to return EIO.
// This test must run in its own process since it deliberately uses all AIO
// resources.  ATM it is only enabled on FreeBSD, because I don't know how to
// check system AIO limits on other operating systems.
#[test]
fn test_lio_listio_resubmit() {
    let mut resubmit_count = 0;

    // Lookup system resource limits
    let alm = sysconf(SysconfVar::AIO_LISTIO_MAX)
        .expect("sysconf").unwrap() as usize;
    let maqpp = if let CtlValue::Int(x) = sysctl::value(
            "vfs.aio.max_aio_queue_per_proc").unwrap(){
        x as usize
    } else {
        panic!("unknown sysctl");
    };

    // Find lio_listio sizes that satisfy the AIO_LISTIO_MAX constraint and also
    // result in a final lio_listio call that can only partially be queued
    let target_ops = maqpp + alm / 2;
    let num_listios = (target_ops + alm - 3) / (alm - 2);
    let ops_per_listio = (target_ops + num_listios - 1) / num_listios;
    assert!((num_listios - 1) * ops_per_listio < maqpp,
        "the last lio_listio won't make any progress; fix the algorithm");
    println!("Using {:?} LioCbs of {:?} operations apiece", num_listios,
             ops_per_listio);

    let f = tempfile().unwrap();
    let buffer_set = (0..num_listios).map(|_| {
        (0..ops_per_listio).map(|_| {
            vec![0u8; BYTES_PER_OP]
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut liocbs = (0..num_listios).map(|i| {
        let mut builder = LioCbBuilder::with_capacity(ops_per_listio);
        for j in 0..ops_per_listio {
            let offset = (BYTES_PER_OP * (i * ops_per_listio + j)) as off_t;
            builder = builder.emplace_slice(f.as_raw_fd(),
                                offset,
                                &buffer_set[i][j][..],
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_WRITE);
        }
        let mut liocb = builder.finish();
        let mut err = liocb.listio(LioMode::LIO_NOWAIT, SigevNotify::SigevNone);
        while err == Err(Errno::EIO) ||
              err == Err(Errno::EAGAIN) ||
              err == Err(Errno::EINTR) {
            // 
            thread::sleep(time::Duration::from_millis(10));
            resubmit_count += 1;
            err = liocb.listio_resubmit(LioMode::LIO_NOWAIT,
                                        SigevNotify::SigevNone);
        }
        liocb
    }).collect::<Vec<_>>();

    // Ensure that every AioCb completed
    for liocb in liocbs.iter_mut() {
        finish_liocb(liocb);
    }

    if resubmit_count > 0 {
        println!("Resubmitted {:?} times, test passed", resubmit_count);
    } else {
        println!("Never resubmitted.  Test ambiguous");
    }
}
