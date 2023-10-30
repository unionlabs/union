use crate::sync::batch_semaphore::Semaphore;
use tokio_test::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

#[test]
fn poll_acquire_one_available() {
    let s = Semaphore::new(100);
    assert_eq!(s.available_permits(), 100);

    // Polling for a permit succeeds immediately
    assert_ready_ok!(task::spawn(s.acquire(1)).poll());
    assert_eq!(s.available_permits(), 99);
}

#[test]
fn poll_acquire_many_available() {
    let s = Semaphore::new(100);
    assert_eq!(s.available_permits(), 100);

    // Polling for a permit succeeds immediately
    assert_ready_ok!(task::spawn(s.acquire(5)).poll());
    assert_eq!(s.available_permits(), 95);

    assert_ready_ok!(task::spawn(s.acquire(5)).poll());
    assert_eq!(s.available_permits(), 90);
}

#[test]
fn try_acquire_one_available() {
    let s = Semaphore::new(100);
    assert_eq!(s.available_permits(), 100);

    assert_ok!(s.try_acquire(1));
    assert_eq!(s.available_permits(), 99);

    assert_ok!(s.try_acquire(1));
    assert_eq!(s.available_permits(), 98);
}

#[test]
fn try_acquire_many_available() {
    let s = Semaphore::new(100);
    assert_eq!(s.available_permits(), 100);

    assert_ok!(s.try_acquire(5));
    assert_eq!(s.available_permits(), 95);

    assert_ok!(s.try_acquire(5));
    assert_eq!(s.available_permits(), 90);
}

#[test]
fn poll_acquire_one_unavailable() {
    let s = Semaphore::new(1);

    // Acquire the first permit
    assert_ready_ok!(task::spawn(s.acquire(1)).poll());
    assert_eq!(s.available_permits(), 0);

    let mut acquire_2 = task::spawn(s.acquire(1));
    // Try to acquire the second permit
    assert_pending!(acquire_2.poll());
    assert_eq!(s.available_permits(), 0);

    s.release(1);

    assert_eq!(s.available_permits(), 0);
    assert!(acquire_2.is_woken());
    assert_ready_ok!(acquire_2.poll());
    assert_eq!(s.available_permits(), 0);

    s.release(1);
    assert_eq!(s.available_permits(), 1);
}

#[test]
fn poll_acquire_many_unavailable() {
    let s = Semaphore::new(5);

    // Acquire the first permit
    assert_ready_ok!(task::spawn(s.acquire(1)).poll());
    assert_eq!(s.available_permits(), 4);

    // Try to acquire the second permit
    let mut acquire_2 = task::spawn(s.acquire(5));
    assert_pending!(acquire_2.poll());
    assert_eq!(s.available_permits(), 0);

    // Try to acquire the third permit
    let mut acquire_3 = task::spawn(s.acquire(3));
    assert_pending!(acquire_3.poll());
    assert_eq!(s.available_permits(), 0);

    s.release(1);

    assert_eq!(s.available_permits(), 0);
    assert!(acquire_2.is_woken());
    assert_ready_ok!(acquire_2.poll());

    assert!(!acquire_3.is_woken());
    assert_eq!(s.available_permits(), 0);

    s.release(1);
    assert!(!acquire_3.is_woken());
    assert_eq!(s.available_permits(), 0);

    s.release(2);
    assert!(acquire_3.is_woken());

    assert_ready_ok!(acquire_3.poll());
}

#[test]
fn try_acquire_one_unavailable() {
    let s = Semaphore::new(1);

    // Acquire the first permit
    assert_ok!(s.try_acquire(1));
    assert_eq!(s.available_permits(), 0);

    assert_err!(s.try_acquire(1));

    s.release(1);

    assert_eq!(s.available_permits(), 1);
    assert_ok!(s.try_acquire(1));

    s.release(1);
    assert_eq!(s.available_permits(), 1);
}

#[test]
fn try_acquire_many_unavailable() {
    let s = Semaphore::new(5);

    // Acquire the first permit
    assert_ok!(s.try_acquire(1));
    assert_eq!(s.available_permits(), 4);

    assert_err!(s.try_acquire(5));

    s.release(1);
    assert_eq!(s.available_permits(), 5);

    assert_ok!(s.try_acquire(5));

    s.release(1);
    assert_eq!(s.available_permits(), 1);

    s.release(1);
    assert_eq!(s.available_permits(), 2);
}

#[test]
fn poll_acquire_one_zero_permits() {
    let s = Semaphore::new(0);
    assert_eq!(s.available_permits(), 0);

    // Try to acquire the permit
    let mut acquire = task::spawn(s.acquire(1));
    assert_pending!(acquire.poll());

    s.release(1);

    assert!(acquire.is_woken());
    assert_ready_ok!(acquire.poll());
}

#[test]
#[should_panic]
#[cfg(not(target_arch = "wasm32"))] // wasm currently doesn't support unwinding
fn validates_max_permits() {
    use std::usize;
    Semaphore::new((usize::MAX >> 2) + 1);
}

#[test]
fn close_semaphore_prevents_acquire() {
    let s = Semaphore::new(5);
    s.close();

    assert_eq!(5, s.available_permits());

    assert_ready_err!(task::spawn(s.acquire(1)).poll());
    assert_eq!(5, s.available_permits());

    assert_ready_err!(task::spawn(s.acquire(1)).poll());
    assert_eq!(5, s.available_permits());
}

#[test]
fn close_semaphore_notifies_permit1() {
    let s = Semaphore::new(0);
    let mut acquire = task::spawn(s.acquire(1));

    assert_pending!(acquire.poll());

    s.close();

    assert!(acquire.is_woken());
    assert_ready_err!(acquire.poll());
}

#[test]
fn close_semaphore_notifies_permit2() {
    let s = Semaphore::new(2);

    // Acquire a couple of permits
    assert_ready_ok!(task::spawn(s.acquire(1)).poll());
    assert_ready_ok!(task::spawn(s.acquire(1)).poll());

    let mut acquire3 = task::spawn(s.acquire(1));
    let mut acquire4 = task::spawn(s.acquire(1));
    assert_pending!(acquire3.poll());
    assert_pending!(acquire4.poll());

    s.close();

    assert!(acquire3.is_woken());
    assert!(acquire4.is_woken());

    assert_ready_err!(acquire3.poll());
    assert_ready_err!(acquire4.poll());

    assert_eq!(0, s.available_permits());

    s.release(1);

    assert_eq!(1, s.available_permits());

    assert_ready_err!(task::spawn(s.acquire(1)).poll());

    s.release(1);

    assert_eq!(2, s.available_permits());
}

#[test]
fn cancel_acquire_releases_permits() {
    let s = Semaphore::new(10);
    s.try_acquire(4).expect("uncontended try_acquire succeeds");
    assert_eq!(6, s.available_permits());

    let mut acquire = task::spawn(s.acquire(8));
    assert_pending!(acquire.poll());

    assert_eq!(0, s.available_permits());
    drop(acquire);

    assert_eq!(6, s.available_permits());
    assert_ok!(s.try_acquire(6));
}
