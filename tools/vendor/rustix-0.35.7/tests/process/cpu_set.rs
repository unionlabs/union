#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
fn test_cpu_set() {
    let set = rustix::process::sched_getaffinity(None).unwrap();

    let mut count = 0;
    for i in 0..rustix::process::CpuSet::MAX_CPU {
        if set.is_set(i) {
            count += 1;
        }
    }

    assert_eq!(count, set.count());
}
