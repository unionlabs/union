#[rustversion::not(nightly)]
#[ignore]
#[test]
fn test_backtrace() {}

#[rustversion::nightly]
#[test]
fn test_backtrace() {
    use anyhow::anyhow;

    let error = anyhow!("oh no!");
    let _ = error.backtrace();
}
