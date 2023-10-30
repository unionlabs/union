#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::error::Error;
use tokio::{
    runtime::{Builder, Runtime},
    sync::{broadcast, mpsc, oneshot, Mutex, RwLock},
};

mod support {
    pub mod panic;
}
use support::panic::test_panic;

#[test]
fn broadcast_channel_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let (_, _) = broadcast::channel::<u32>(0);
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn mutex_blocking_lock_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        rt.block_on(async {
            let mutex = Mutex::new(5_u32);
            mutex.blocking_lock();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn oneshot_blocking_recv_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        rt.block_on(async {
            let (_tx, rx) = oneshot::channel::<u8>();
            let _ = rx.blocking_recv();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn rwlock_with_max_readers_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let _ = RwLock::<u8>::with_max_readers(0, (u32::MAX >> 3) + 1);
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn rwlock_blocking_read_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        rt.block_on(async {
            let lock = RwLock::<u8>::new(0);
            let _ = lock.blocking_read();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn rwlock_blocking_write_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        rt.block_on(async {
            let lock = RwLock::<u8>::new(0);
            let _ = lock.blocking_write();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn mpsc_bounded_channel_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let (_, _) = mpsc::channel::<u8>(0);
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn mpsc_bounded_receiver_blocking_recv_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        let (_tx, mut rx) = mpsc::channel::<u8>(1);
        rt.block_on(async {
            let _ = rx.blocking_recv();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn mpsc_bounded_sender_blocking_send_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        let (tx, _rx) = mpsc::channel::<u8>(1);
        rt.block_on(async {
            let _ = tx.blocking_send(3);
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

#[test]
fn mpsc_unbounded_receiver_blocking_recv_panic_caller() -> Result<(), Box<dyn Error>> {
    let panic_location_file = test_panic(|| {
        let rt = basic();
        let (_tx, mut rx) = mpsc::unbounded_channel::<u8>();
        rt.block_on(async {
            let _ = rx.blocking_recv();
        });
    });

    // The panic location should be in this file
    assert_eq!(&panic_location_file.unwrap(), file!());

    Ok(())
}

fn basic() -> Runtime {
    Builder::new_current_thread().enable_all().build().unwrap()
}
