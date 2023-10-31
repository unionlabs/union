use rustix::process::sched_yield;

#[test]
fn test_sched_yield() {
    // Just make sure we can call it.
    sched_yield();
}
