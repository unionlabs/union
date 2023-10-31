#[test]
fn encode_decode() {
    #[cfg(unix)]
    use rustix::net::SocketAddrUnix;
    use rustix::net::{
        Ipv4Addr, Ipv6Addr, SocketAddrAny, SocketAddrStorage, SocketAddrV4, SocketAddrV6,
    };

    unsafe {
        let orig = SocketAddrV4::new(Ipv4Addr::new(2, 3, 5, 6), 33);
        let mut encoded = std::mem::MaybeUninit::<SocketAddrStorage>::uninit();
        let len = SocketAddrAny::V4(orig).write(encoded.as_mut_ptr());
        let decoded = SocketAddrAny::read(encoded.as_ptr(), len).unwrap();
        assert_eq!(decoded, SocketAddrAny::V4(orig));

        let orig = SocketAddrV6::new(Ipv6Addr::new(2, 3, 5, 6, 8, 9, 11, 12), 33, 34, 36);
        let mut encoded = std::mem::MaybeUninit::<SocketAddrStorage>::uninit();
        let len = SocketAddrAny::V6(orig).write(encoded.as_mut_ptr());
        let decoded = SocketAddrAny::read(encoded.as_ptr(), len).unwrap();
        assert_eq!(decoded, SocketAddrAny::V6(orig));

        #[cfg(not(windows))]
        {
            let orig = SocketAddrUnix::new("/path/to/socket").unwrap();
            let mut encoded = std::mem::MaybeUninit::<SocketAddrStorage>::uninit();
            let len = SocketAddrAny::Unix(orig.clone()).write(encoded.as_mut_ptr());
            let decoded = SocketAddrAny::read(encoded.as_ptr(), len).unwrap();
            assert_eq!(decoded, SocketAddrAny::Unix(orig));
        }
    }
}

#[cfg(not(windows))]
#[test]
fn test_unix_addr() {
    use rustix::cstr;
    use rustix::net::SocketAddrUnix;

    assert_eq!(
        SocketAddrUnix::new("/").unwrap().path().unwrap(),
        cstr!("/")
    );
    assert_eq!(
        SocketAddrUnix::new("//").unwrap().path().unwrap(),
        cstr!("//")
    );
    assert_eq!(
        SocketAddrUnix::new("/foo/bar").unwrap().path().unwrap(),
        cstr!("/foo/bar")
    );
    assert_eq!(
        SocketAddrUnix::new("foo").unwrap().path().unwrap(),
        cstr!("foo")
    );
    SocketAddrUnix::new("/foo\0/bar").unwrap_err();
    assert!(SocketAddrUnix::new("").unwrap().path().is_none());

    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        assert!(SocketAddrUnix::new("foo")
            .unwrap()
            .abstract_name()
            .is_none());

        assert_eq!(
            SocketAddrUnix::new_abstract_name(b"test")
                .unwrap()
                .abstract_name()
                .unwrap(),
            b"test"
        );
        assert_eq!(
            SocketAddrUnix::new_abstract_name(b"")
                .unwrap()
                .abstract_name()
                .unwrap(),
            b""
        );
        assert_eq!(
            SocketAddrUnix::new_abstract_name(b"this\0that")
                .unwrap()
                .abstract_name()
                .unwrap(),
            b"this\0that"
        );
        SocketAddrUnix::new_abstract_name(&[b'a'; 500]).unwrap_err();
        assert!(SocketAddrUnix::new_abstract_name(b"test")
            .unwrap()
            .path()
            .is_none());
    }
}
