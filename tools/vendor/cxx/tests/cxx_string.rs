use cxx::{let_cxx_string, CxxString};

#[test]
fn test_async_cxx_string() {
    async fn f() {
        let_cxx_string!(s = "...");

        async fn g(_: &CxxString) {}
        g(&s).await;
    }

    // https://github.com/dtolnay/cxx/issues/693
    fn assert_send(_: impl Send) {}
    assert_send(f());
}

#[test]
fn test_debug() {
    let_cxx_string!(s = "x\"y\'z");

    assert_eq!(format!("{:?}", s), r#""x\"y'z""#);
}
