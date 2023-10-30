use rustix::net::{
    AddressFamily, Ipv6Addr, Protocol, RecvFlags, SendFlags, SocketAddrAny, SocketAddrV4,
    SocketAddrV6, SocketType,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Test `connect_any`.
#[test]
fn net_v4_connect_any() -> std::io::Result<()> {
    let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Similar, but with V6.
#[test]
fn net_v6_connect_any() -> std::io::Result<()> {
    let localhost = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Test `connect` with a `SocketAddr`.
#[test]
fn net_v4_connect() -> std::io::Result<()> {
    let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let local_addr = match local_addr {
        SocketAddrAny::V4(v4) => SocketAddr::V4(v4),
        other => panic!("unexpected socket address {:?}", other),
    };
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Similar, but use V6.
#[test]
fn net_v6_connect() -> std::io::Result<()> {
    let localhost = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let local_addr = match local_addr {
        SocketAddrAny::V6(v6) => SocketAddr::V6(v6),
        other => panic!("unexpected socket address {:?}", other),
    };
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Test `bind_any`.
#[test]
fn net_v4_bind_any() -> std::io::Result<()> {
    let localhost = Ipv4Addr::LOCALHOST;
    let addr = SocketAddrAny::V4(SocketAddrV4::new(localhost, 0));
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind_any(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Similar, but use V6.
#[test]
fn net_v6_bind_any() -> std::io::Result<()> {
    let localhost = Ipv6Addr::LOCALHOST;
    let addr = SocketAddrAny::V6(SocketAddrV6::new(localhost, 0, 0, 0));
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind_any(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Test `sendto`.
#[test]
fn net_v4_sendto() -> std::io::Result<()> {
    let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let local_addr = match local_addr {
        SocketAddrAny::V4(v4) => SocketAddr::V4(v4),
        other => panic!("unexpected socket address {:?}", other),
    };
    let n = rustix::net::sendto(&sender, request, SendFlags::empty(), &local_addr).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let (n, from) =
        rustix::net::recvfrom(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);
    assert!(from.is_none());

    Ok(())
}

/// Similar, but with V6.
#[test]
fn net_v6_sendto() -> std::io::Result<()> {
    let localhost = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let local_addr = match local_addr {
        SocketAddrAny::V6(v6) => SocketAddr::V6(v6),
        other => panic!("unexpected socket address {:?}", other),
    };
    let n = rustix::net::sendto(&sender, request, SendFlags::empty(), &local_addr).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let (n, from) =
        rustix::net::recvfrom(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);
    assert!(from.is_none());

    Ok(())
}

/// Test `sendto_any`.
#[test]
fn net_v4_sendto_any() -> std::io::Result<()> {
    let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n =
        rustix::net::sendto_any(&sender, request, SendFlags::empty(), &local_addr).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let (n, from) =
        rustix::net::recvfrom(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);
    assert!(from.is_none());

    Ok(())
}

/// Test `sendto_any`.
#[test]
fn net_v6_sendto_any() -> std::io::Result<()> {
    let localhost = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n =
        rustix::net::sendto_any(&sender, request, SendFlags::empty(), &local_addr).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let accepted = rustix::net::accept(&listener).expect("accept");
    let mut response = [0_u8; 128];
    let (n, from) =
        rustix::net::recvfrom(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);
    assert!(from.is_none());

    Ok(())
}

/// Test `acceptfrom`.
#[test]
fn net_v4_acceptfrom() -> std::io::Result<()> {
    let localhost = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener =
        rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let (accepted, from) = rustix::net::acceptfrom(&listener).expect("accept");

    assert_ne!(from.clone().unwrap(), local_addr);

    let from = match from.unwrap() {
        SocketAddrAny::V4(v4) => v4,
        other => panic!("unexpected socket address {:?}", other),
    };
    let local_addr = match local_addr {
        SocketAddrAny::V4(v4) => v4,
        other => panic!("unexpected socket address {:?}", other),
    };

    assert_eq!(from.clone().ip(), local_addr.ip());
    assert_ne!(from.clone().port(), local_addr.port());

    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}

/// Similar, but with V6.
#[test]
fn net_v6_acceptfrom() -> std::io::Result<()> {
    let localhost = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let addr = SocketAddr::new(localhost, 0);
    let listener = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::bind(&listener, &addr).expect("bind");
    rustix::net::listen(&listener, 1).expect("listen");

    let local_addr = rustix::net::getsockname(&listener)?;
    let sender = rustix::net::socket(
        AddressFamily::INET6,
        SocketType::STREAM,
        Protocol::default(),
    )?;
    rustix::net::connect_any(&sender, &local_addr).expect("connect");
    let request = b"Hello, World!!!";
    let n = rustix::net::send(&sender, request, SendFlags::empty()).expect("send");
    drop(sender);

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    let (accepted, from) = rustix::net::acceptfrom(&listener).expect("accept");

    assert_ne!(from.clone().unwrap(), local_addr);

    let from = match from.unwrap() {
        SocketAddrAny::V6(v6) => v6,
        other => panic!("unexpected socket address {:?}", other),
    };
    let local_addr = match local_addr {
        SocketAddrAny::V6(v6) => v6,
        other => panic!("unexpected socket address {:?}", other),
    };

    assert_eq!(from.clone().ip(), local_addr.ip());
    assert_ne!(from.clone().port(), local_addr.port());

    let mut response = [0_u8; 128];
    let n = rustix::net::recv(&accepted, &mut response, RecvFlags::empty()).expect("recv");

    // Not strictly required, but it makes the test simpler.
    assert_eq!(n, request.len());

    assert_eq!(request, &response[..n]);

    Ok(())
}
