pub fn send_signal(signal: libc::c_int) {
    use libc::{getpid, kill};

    unsafe {
        assert_eq!(kill(getpid(), signal), 0);
    }
}
