use rustix::path::DecInt;

#[test]
fn test_dec_int() {
    assert_eq!(DecInt::new(0).as_ref().to_str().unwrap(), "0");
    assert_eq!(DecInt::new(-1).as_ref().to_str().unwrap(), "-1");
    assert_eq!(DecInt::new(789).as_ref().to_str().unwrap(), "789");
    assert_eq!(
        DecInt::new(i64::MIN).as_ref().to_str().unwrap(),
        i64::MIN.to_string()
    );
    assert_eq!(
        DecInt::new(i64::MAX).as_ref().to_str().unwrap(),
        i64::MAX.to_string()
    );
    assert_eq!(
        DecInt::new(u64::MAX).as_ref().to_str().unwrap(),
        u64::MAX.to_string()
    );
}
