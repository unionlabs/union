use io_lifetimes::raw::AsRawFilelike;

#[test]
fn test_proc_self() {
    // Verify that this API works at all
    let fd = rustix::io::proc_self_fd().unwrap();
    assert_ne!(fd.as_raw_filelike(), 0);
}
