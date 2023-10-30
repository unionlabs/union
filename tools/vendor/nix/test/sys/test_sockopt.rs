use rand::{thread_rng, Rng};
use nix::sys::socket::{socket, sockopt, getsockopt, setsockopt, AddressFamily, SockType, SockFlag, SockProtocol};
#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::*;

// NB: FreeBSD supports LOCAL_PEERCRED for SOCK_SEQPACKET, but OSX does not.
#[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
))]
#[test]
pub fn test_local_peercred_seqpacket() {
    use nix::{
        unistd::{Gid, Uid},
        sys::socket::socketpair
    };

    let (fd1, _fd2) = socketpair(AddressFamily::Unix, SockType::SeqPacket, None,
                                SockFlag::empty()).unwrap();
    let xucred = getsockopt(fd1, sockopt::LocalPeerCred).unwrap();
    assert_eq!(xucred.version(), 0);
    assert_eq!(Uid::from_raw(xucred.uid()), Uid::current());
    assert_eq!(Gid::from_raw(xucred.groups()[0]), Gid::current());
}

#[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "ios"
))]
#[test]
pub fn test_local_peercred_stream() {
    use nix::{
        unistd::{Gid, Uid},
        sys::socket::socketpair
    };

    let (fd1, _fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None,
                                SockFlag::empty()).unwrap();
    let xucred = getsockopt(fd1, sockopt::LocalPeerCred).unwrap();
    assert_eq!(xucred.version(), 0);
    assert_eq!(Uid::from_raw(xucred.uid()), Uid::current());
    assert_eq!(Gid::from_raw(xucred.groups()[0]), Gid::current());
}

#[cfg(target_os = "linux")]
#[test]
fn is_so_mark_functional() {
    use nix::sys::socket::sockopt;

    require_capability!("is_so_mark_functional", CAP_NET_ADMIN);

    let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();
    setsockopt(s, sockopt::Mark, &1337).unwrap();
    let mark = getsockopt(s, sockopt::Mark).unwrap();
    assert_eq!(mark, 1337);
}

#[test]
fn test_so_buf() {
    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), SockProtocol::Udp)
             .unwrap();
    let bufsize: usize = thread_rng().gen_range(4096..131_072);
    setsockopt(fd, sockopt::SndBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::SndBuf).unwrap();
    assert!(actual >= bufsize);
    setsockopt(fd, sockopt::RcvBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::RcvBuf).unwrap();
    assert!(actual >= bufsize);
}

#[test]
fn test_so_tcp_maxseg() {
    use std::net::SocketAddr;
    use std::str::FromStr;
    use nix::sys::socket::{accept, bind, connect, listen, InetAddr, SockAddr};
    use nix::unistd::{close, write};

    let std_sa = SocketAddr::from_str("127.0.0.1:4001").unwrap();
    let inet_addr = InetAddr::from_std(&std_sa);
    let sock_addr = SockAddr::new_inet(inet_addr);

    let rsock = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), SockProtocol::Tcp)
                .unwrap();
    bind(rsock, &sock_addr).unwrap();
    listen(rsock, 10).unwrap();
    let initial = getsockopt(rsock, sockopt::TcpMaxSeg).unwrap();
    // Initial MSS is expected to be 536 (https://tools.ietf.org/html/rfc879#section-1) but some
    // platforms keep it even lower. This might fail if you've tuned your initial MSS to be larger
    // than 700
    cfg_if! {
        if #[cfg(any(target_os = "android", target_os = "linux"))] {
            let segsize: u32 = 873;
            assert!(initial < segsize);
            setsockopt(rsock, sockopt::TcpMaxSeg, &segsize).unwrap();
        } else {
            assert!(initial < 700);
        }
    }

    // Connect and check the MSS that was advertised
    let ssock = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), SockProtocol::Tcp)
                .unwrap();
    connect(ssock, &sock_addr).unwrap();
    let rsess = accept(rsock).unwrap();
    write(rsess, b"hello").unwrap();
    let actual = getsockopt(ssock, sockopt::TcpMaxSeg).unwrap();
    // Actual max segment size takes header lengths into account, max IPv4 options (60 bytes) + max
    // TCP options (40 bytes) are subtracted from the requested maximum as a lower boundary.
    cfg_if! {
        if #[cfg(any(target_os = "android", target_os = "linux"))] {
            assert!((segsize - 100) <= actual);
            assert!(actual <= segsize);
        } else {
            assert!(initial < actual);
            assert!(536 < actual);
        }
    }
    close(rsock).unwrap();
    close(ssock).unwrap();
}

// The CI doesn't supported getsockopt and setsockopt on emulated processors.
// It's beleived that a QEMU issue, the tests run ok on a fully emulated system.
// Current CI just run the binary with QEMU but the Kernel remains the same as the host.
// So the syscall doesn't work properly unless the kernel is also emulated.
#[test]
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_os = "freebsd", target_os = "linux")
))]
fn test_tcp_congestion() {
    use std::ffi::OsString;

    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();

    let val = getsockopt(fd, sockopt::TcpCongestion).unwrap();
    setsockopt(fd, sockopt::TcpCongestion, &val).unwrap();

    setsockopt(fd, sockopt::TcpCongestion, &OsString::from("tcp_congestion_does_not_exist")).unwrap_err();

    assert_eq!(
        getsockopt(fd, sockopt::TcpCongestion).unwrap(),
        val
    );
}

#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_bindtodevice() {
    skip_if_not_root!("test_bindtodevice");

    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();

    let val = getsockopt(fd, sockopt::BindToDevice).unwrap();
    setsockopt(fd, sockopt::BindToDevice, &val).unwrap();

    assert_eq!(
        getsockopt(fd, sockopt::BindToDevice).unwrap(),
        val
    );
}

#[test]
fn test_so_tcp_keepalive() {
    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), SockProtocol::Tcp).unwrap();
    setsockopt(fd, sockopt::KeepAlive, &true).unwrap();
    assert!(getsockopt(fd, sockopt::KeepAlive).unwrap());

    #[cfg(any(target_os = "android",
              target_os = "dragonfly",
              target_os = "freebsd",
              target_os = "linux",
              target_os = "nacl"))] {
        let x = getsockopt(fd, sockopt::TcpKeepIdle).unwrap();
        setsockopt(fd, sockopt::TcpKeepIdle, &(x + 1)).unwrap();
        assert_eq!(getsockopt(fd, sockopt::TcpKeepIdle).unwrap(), x + 1);

        let x = getsockopt(fd, sockopt::TcpKeepCount).unwrap();
        setsockopt(fd, sockopt::TcpKeepCount, &(x + 1)).unwrap();
        assert_eq!(getsockopt(fd, sockopt::TcpKeepCount).unwrap(), x + 1);

        let x = getsockopt(fd, sockopt::TcpKeepInterval).unwrap();
        setsockopt(fd, sockopt::TcpKeepInterval, &(x + 1)).unwrap();
        assert_eq!(getsockopt(fd, sockopt::TcpKeepInterval).unwrap(), x + 1);
    }
}

#[test]
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
fn test_ttl_opts() {
    let fd4 = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None).unwrap();
    setsockopt(fd4, sockopt::Ipv4Ttl, &1)
        .expect("setting ipv4ttl on an inet socket should succeed");
    let fd6 = socket(AddressFamily::Inet6, SockType::Datagram, SockFlag::empty(), None).unwrap();
    setsockopt(fd6, sockopt::Ipv6Ttl, &1)
        .expect("setting ipv6ttl on an inet6 socket should succeed");
}
