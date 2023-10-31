#[test]
fn test_sockopts() {
    use rustix::net::{AddressFamily, Protocol, SocketType};
    use std::time::Duration;

    let s =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default()).unwrap();

    // On a new socket we shouldn't have a timeout yet.
    assert!(
        rustix::net::sockopt::get_socket_timeout(&s, rustix::net::sockopt::Timeout::Recv)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        rustix::net::sockopt::get_socket_type(&s).unwrap(),
        SocketType::STREAM
    );
    #[cfg(not(windows))]
    assert_eq!(
        rustix::net::sockopt::get_socket_broadcast(&s).unwrap(),
        false
    );
    // On a new socket we shouldn't have a linger yet.
    assert!(rustix::net::sockopt::get_socket_linger(&s)
        .unwrap()
        .is_none());
    #[cfg(any(target_os = "android", target_os = "linux"))]
    assert_eq!(
        rustix::net::sockopt::get_socket_passcred(&s).unwrap(),
        false
    );
    assert_ne!(rustix::net::sockopt::get_ip_ttl(&s).unwrap(), 0);
    assert_ne!(rustix::net::sockopt::get_ip_ttl(&s).unwrap(), 77);
    #[cfg(not(any(
        windows,
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    )))]
    assert_eq!(
        rustix::net::sockopt::get_ip_multicast_loop(&s).unwrap(),
        true
    );
    #[cfg(not(any(
        windows,
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    )))]
    assert_eq!(rustix::net::sockopt::get_ip_multicast_ttl(&s).unwrap(), 1);
    assert_eq!(rustix::net::sockopt::get_tcp_nodelay(&s).unwrap(), false);

    // Set a timeout.
    rustix::net::sockopt::set_socket_timeout(
        &s,
        rustix::net::sockopt::Timeout::Recv,
        Some(Duration::new(1, 1)),
    )
    .unwrap();

    // Check that we have a timeout of at least the time we set.
    if cfg!(not(target_os = "freebsd")) {
        assert!(
            rustix::net::sockopt::get_socket_timeout(&s, rustix::net::sockopt::Timeout::Recv)
                .unwrap()
                .unwrap()
                >= Duration::new(1, 1)
        );
    } else {
        // On FreeBSD <= 12, it appears the system rounds the timeout down.
        assert!(
            rustix::net::sockopt::get_socket_timeout(&s, rustix::net::sockopt::Timeout::Recv)
                .unwrap()
                .unwrap()
                >= Duration::new(1, 0)
        );
    }

    #[cfg(not(windows))]
    {
        // Set the broadcast flag;
        rustix::net::sockopt::set_socket_broadcast(&s, true).unwrap();

        // Check that the broadcast flag is set. This has no effect on stream
        // sockets, and not all platforms even remember the value.
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        assert_eq!(
            rustix::net::sockopt::get_socket_broadcast(&s).unwrap(),
            true
        );
    }

    // Set a linger.
    rustix::net::sockopt::set_socket_linger(&s, Some(Duration::new(1, 1))).unwrap();

    // Check that we have a linger of at least the time we set.
    assert!(
        dbg!(rustix::net::sockopt::get_socket_linger(&s)
            .unwrap()
            .unwrap())
            >= Duration::new(1, 1)
    );

    #[cfg(any(target_os = "android", target_os = "linux"))]
    {
        // Set the passcred flag;
        rustix::net::sockopt::set_socket_passcred(&s, true).unwrap();

        // Check that the passcred flag is set.
        assert_eq!(rustix::net::sockopt::get_socket_passcred(&s).unwrap(), true);
    }

    // Set the ip ttl.
    rustix::net::sockopt::set_ip_ttl(&s, 77).unwrap();

    // Check the ip ttl.
    assert_eq!(rustix::net::sockopt::get_ip_ttl(&s).unwrap(), 77);

    #[cfg(not(any(
        windows,
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    )))]
    {
        // Set the multicast loop flag;
        rustix::net::sockopt::set_ip_multicast_loop(&s, false).unwrap();

        // Check that the multicast loop flag is set.
        assert_eq!(
            rustix::net::sockopt::get_ip_multicast_loop(&s).unwrap(),
            false
        );
    }

    // Set the nodelay flag;
    rustix::net::sockopt::set_tcp_nodelay(&s, true).unwrap();

    // Check that the nodelay flag is set.
    assert_eq!(rustix::net::sockopt::get_tcp_nodelay(&s).unwrap(), true);
}
