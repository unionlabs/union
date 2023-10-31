use nix::sys::socket::{AddressFamily, InetAddr, SockAddr, UnixAddr, getsockname, sockaddr, sockaddr_in6, sockaddr_storage_to_addr};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::{self, MaybeUninit};
use std::net::{self, Ipv6Addr, SocketAddr, SocketAddrV6};
use std::os::unix::io::RawFd;
use std::path::Path;
use std::slice;
use std::str::FromStr;
use libc::{c_char, sockaddr_storage};
#[cfg(any(target_os = "linux", target_os= "android"))]
use crate::*;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = InetAddr::from_std(&actual);

    match addr {
        InetAddr::V4(addr) => {
            let ip: u32 = 0x7f00_0001;
            let port: u16 = 3000;
            let saddr = addr.sin_addr.s_addr;

            assert_eq!(saddr, ip.to_be());
            assert_eq!(addr.sin_port, port.to_be());
        }
        _ => panic!("nope"),
    }

    assert_eq!(addr.to_string(), "127.0.0.1:3000");

    let inet = addr.to_std();
    assert_eq!(actual, inet);
}

#[test]
pub fn test_inetv4_addr_roundtrip_sockaddr_storage_to_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = InetAddr::from_std(&actual);
    let sockaddr = SockAddr::new_inet(addr);

    let (storage, ffi_size) = {
        let mut storage = MaybeUninit::<sockaddr_storage>::zeroed();
        let storage_ptr = storage.as_mut_ptr().cast::<sockaddr>();
        let (ffi_ptr, ffi_size) = sockaddr.as_ffi_pair();
        assert_eq!(mem::size_of::<sockaddr>(), ffi_size as usize);
        unsafe {
            storage_ptr.copy_from_nonoverlapping(ffi_ptr as *const sockaddr, 1);
            (storage.assume_init(), ffi_size)
        }
    };

    let from_storage = sockaddr_storage_to_addr(&storage, ffi_size as usize).unwrap();
    assert_eq!(from_storage, sockaddr);
    let from_storage = sockaddr_storage_to_addr(&storage, mem::size_of::<sockaddr_storage>()).unwrap();
    assert_eq!(from_storage, sockaddr);
}

#[test]
pub fn test_inetv6_addr_to_sock_addr() {
    let port: u16 = 3000;
    let flowinfo: u32 = 1;
    let scope_id: u32 = 2;
    let ip: Ipv6Addr = "fe80::1".parse().unwrap();

    let actual = SocketAddr::V6(SocketAddrV6::new(ip, port, flowinfo, scope_id));
    let addr = InetAddr::from_std(&actual);

    match addr {
        InetAddr::V6(addr) => {
            assert_eq!(addr.sin6_port, port.to_be());
            assert_eq!(addr.sin6_flowinfo, flowinfo);
            assert_eq!(addr.sin6_scope_id, scope_id);
        }
        _ => panic!("nope"),
    }

    assert_eq!(actual, addr.to_std());
}
#[test]
pub fn test_inetv6_addr_roundtrip_sockaddr_storage_to_addr() {
    let port: u16 = 3000;
    let flowinfo: u32 = 1;
    let scope_id: u32 = 2;
    let ip: Ipv6Addr = "fe80::1".parse().unwrap();

    let actual = SocketAddr::V6(SocketAddrV6::new(ip, port, flowinfo, scope_id));
    let addr = InetAddr::from_std(&actual);
    let sockaddr = SockAddr::new_inet(addr);

    let (storage, ffi_size) = {
        let mut storage = MaybeUninit::<sockaddr_storage>::zeroed();
        let storage_ptr = storage.as_mut_ptr().cast::<sockaddr_in6>();
        let (ffi_ptr, ffi_size) = sockaddr.as_ffi_pair();
        assert_eq!(mem::size_of::<sockaddr_in6>(), ffi_size as usize);
        unsafe {
            storage_ptr.copy_from_nonoverlapping((ffi_ptr as *const sockaddr).cast::<sockaddr_in6>(), 1);
            (storage.assume_init(), ffi_size)
        }
    };

    let from_storage = sockaddr_storage_to_addr(&storage, ffi_size as usize).unwrap();
    assert_eq!(from_storage, sockaddr);
    let from_storage = sockaddr_storage_to_addr(&storage, mem::size_of::<sockaddr_storage>()).unwrap();
    assert_eq!(from_storage, sockaddr);
}

#[test]
pub fn test_path_to_sock_addr() {
    let path = "/foo/bar";
    let actual = Path::new(path);
    let addr = UnixAddr::new(actual).unwrap();

    let expect: &[c_char] = unsafe {
        slice::from_raw_parts(path.as_ptr() as *const c_char, path.len())
    };
    assert_eq!(unsafe { &(*addr.as_ptr()).sun_path[..8] }, expect);

    assert_eq!(addr.path(), Some(actual));
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
pub fn test_addr_equality_path() {
    let path = "/foo/bar";
    let actual = Path::new(path);
    let addr1 = UnixAddr::new(actual).unwrap();
    let mut addr2 = addr1;

    unsafe { (*addr2.as_mut_ptr()).sun_path[10] = 127 };

    assert_eq!(addr1, addr2);
    assert_eq!(calculate_hash(&addr1), calculate_hash(&addr2));
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
pub fn test_abstract_sun_path_too_long() {
    let name = String::from("nix\0abstract\0tesnix\0abstract\0tesnix\0abstract\0tesnix\0abstract\0tesnix\0abstract\0testttttnix\0abstract\0test\0make\0sure\0this\0is\0long\0enough");
    let addr = UnixAddr::new_abstract(name.as_bytes());
    assert!(addr.is_err());
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[test]
pub fn test_addr_equality_abstract() {
    let name = String::from("nix\0abstract\0test");
    let addr1 = UnixAddr::new_abstract(name.as_bytes()).unwrap();
    let mut addr2 = addr1;

    assert_eq!(addr1, addr2);
    assert_eq!(calculate_hash(&addr1), calculate_hash(&addr2));

    unsafe { (*addr2.as_mut_ptr()).sun_path[17] = 127 };
    assert_ne!(addr1, addr2);
    assert_ne!(calculate_hash(&addr1), calculate_hash(&addr2));
}

// Test getting/setting abstract addresses (without unix socket creation)
#[cfg(target_os = "linux")]
#[test]
pub fn test_abstract_uds_addr() {
    let empty = String::new();
    let addr = UnixAddr::new_abstract(empty.as_bytes()).unwrap();
    let sun_path: [u8; 0] = [];
    assert_eq!(addr.as_abstract(), Some(&sun_path[..]));

    let name = String::from("nix\0abstract\0test");
    let addr = UnixAddr::new_abstract(name.as_bytes()).unwrap();
    let sun_path = [
        110u8, 105, 120, 0, 97, 98, 115, 116, 114, 97, 99, 116, 0, 116, 101, 115, 116
    ];
    assert_eq!(addr.as_abstract(), Some(&sun_path[..]));
    assert_eq!(addr.path(), None);

    // Internally, name is null-prefixed (abstract namespace)
    assert_eq!(unsafe { (*addr.as_ptr()).sun_path[0] }, 0);
}

#[test]
pub fn test_getsockname() {
    use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};
    use nix::sys::socket::{bind, SockAddr};

    let tempdir = tempfile::tempdir().unwrap();
    let sockname = tempdir.path().join("sock");
    let sock = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)
               .expect("socket failed");
    let sockaddr = SockAddr::new_unix(&sockname).unwrap();
    bind(sock, &sockaddr).expect("bind failed");
    assert_eq!(sockaddr, getsockname(sock).expect("getsockname failed"));
}

#[test]
pub fn test_socketpair() {
    use nix::unistd::{read, write};
    use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();
    write(fd1, b"hello").unwrap();
    let mut buf = [0;5];
    read(fd2, &mut buf).unwrap();

    assert_eq!(&buf[..], b"hello");
}

mod recvfrom {
    use nix::Result;
    use nix::sys::socket::*;
    use std::thread;
    use super::*;

    const MSG: &[u8] = b"Hello, World!";

    fn sendrecv<Fs, Fr>(rsock: RawFd, ssock: RawFd, f_send: Fs, mut f_recv: Fr) -> Option<SockAddr>
        where
            Fs: Fn(RawFd, &[u8], MsgFlags) -> Result<usize> + Send + 'static,
            Fr: FnMut(usize, Option<SockAddr>),
    {
        let mut buf: [u8; 13] = [0u8; 13];
        let mut l = 0;
        let mut from = None;

        let send_thread = thread::spawn(move || {
            let mut l = 0;
            while l < std::mem::size_of_val(MSG) {
                l += f_send(ssock, &MSG[l..], MsgFlags::empty()).unwrap();
            }
        });

        while l < std::mem::size_of_val(MSG) {
            let (len, from_) = recvfrom(rsock, &mut buf[l..]).unwrap();
            f_recv(len, from_);
            from = from_;
            l += len;
        }
        assert_eq!(&buf, MSG);
        send_thread.join().unwrap();
        from
    }

    #[test]
    pub fn stream() {
        let (fd2, fd1) = socketpair(AddressFamily::Unix, SockType::Stream,
                                    None, SockFlag::empty()).unwrap();
        // Ignore from for stream sockets
        let _ = sendrecv(fd1, fd2, |s, m, flags| {
            send(s, m, flags)
        }, |_, _| {});
    }

    #[test]
    pub fn udp() {
        let std_sa = SocketAddr::from_str("127.0.0.1:6789").unwrap();
        let inet_addr = InetAddr::from_std(&std_sa);
        let sock_addr = SockAddr::new_inet(inet_addr);
        let rsock = socket(AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None
        ).unwrap();
        bind(rsock, &sock_addr).unwrap();
        let ssock = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");
        let from = sendrecv(rsock, ssock, move |s, m, flags| {
            sendto(s, m, &sock_addr, flags)
        },|_, _| {});
        // UDP sockets should set the from address
        assert_eq!(AddressFamily::Inet, from.unwrap().family());
    }

    #[cfg(target_os = "linux")]
    mod udp_offload {
        use super::*;
        use nix::sys::uio::IoVec;
        use nix::sys::socket::sockopt::{UdpGroSegment, UdpGsoSegment};

        #[test]
        // Disable the test under emulation because it fails in Cirrus-CI.  Lack
        // of QEMU support is suspected.
        #[cfg_attr(qemu, ignore)]
        pub fn gso() {
            require_kernel_version!(udp_offload::gso, ">= 4.18");

            // In this test, we send the data and provide a GSO segment size.
            // Since we are sending the buffer of size 13, six UDP packets
            // with size 2 and two UDP packet with size 1 will be sent.
            let segment_size: u16 = 2;

            let std_sa = SocketAddr::from_str("127.0.0.1:6791").unwrap();
            let inet_addr = InetAddr::from_std(&std_sa);
            let sock_addr = SockAddr::new_inet(inet_addr);
            let rsock = socket(AddressFamily::Inet,
                               SockType::Datagram,
                               SockFlag::empty(),
                               None
            ).unwrap();

            setsockopt(rsock, UdpGsoSegment, &(segment_size as _))
                .expect("setsockopt UDP_SEGMENT failed");

            bind(rsock, &sock_addr).unwrap();
            let ssock = socket(
                AddressFamily::Inet,
                SockType::Datagram,
                SockFlag::empty(),
                None,
            ).expect("send socket failed");

            let mut num_packets_received: i32 = 0;

            sendrecv(rsock, ssock, move |s, m, flags| {
                let iov = [IoVec::from_slice(m)];
                let cmsg = ControlMessage::UdpGsoSegments(&segment_size);
                sendmsg(s, &iov, &[cmsg], flags, Some(&sock_addr))
            }, {
                let num_packets_received_ref = &mut num_packets_received;

                move |len, _| {
                    // check that we receive UDP packets with payload size
                    // less or equal to segment size
                    assert!(len <= segment_size as usize);
                    *num_packets_received_ref += 1;
                }
            });

            // Buffer size is 13, we will receive six packets of size 2,
            // and one packet of size 1.
            assert_eq!(7, num_packets_received);
        }

        #[test]
        // Disable the test on emulated platforms because it fails in Cirrus-CI.
        // Lack of QEMU support is suspected.
        #[cfg_attr(qemu, ignore)]
        pub fn gro() {
            require_kernel_version!(udp_offload::gro, ">= 5.3");

            // It's hard to guarantee receiving GRO packets. Just checking
            // that `setsockopt` doesn't fail with error

            let rsock = socket(AddressFamily::Inet,
                               SockType::Datagram,
                               SockFlag::empty(),
                               None
            ).unwrap();

            setsockopt(rsock, UdpGroSegment, &true)
                .expect("setsockopt UDP_GRO failed");
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
    ))]
    #[test]
    pub fn udp_sendmmsg() {
        use nix::sys::uio::IoVec;

        let std_sa = SocketAddr::from_str("127.0.0.1:6793").unwrap();
        let std_sa2 = SocketAddr::from_str("127.0.0.1:6794").unwrap();
        let inet_addr = InetAddr::from_std(&std_sa);
        let inet_addr2 = InetAddr::from_std(&std_sa2);
        let sock_addr = SockAddr::new_inet(inet_addr);
        let sock_addr2 = SockAddr::new_inet(inet_addr2);

        let rsock = socket(AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None
        ).unwrap();
        bind(rsock, &sock_addr).unwrap();
        let ssock = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");

        let from = sendrecv(rsock, ssock, move |s, m, flags| {
            let iov = [IoVec::from_slice(m)];
            let mut msgs = vec![
                SendMmsgData {
                    iov: &iov,
                    cmsgs: &[],
                    addr: Some(sock_addr),
                    _lt: Default::default(),
                }
            ];

            let batch_size = 15;

            for _ in 0..batch_size {
                msgs.push(
                    SendMmsgData {
                        iov: &iov,
                        cmsgs: &[],
                        addr: Some(sock_addr2),
                        _lt: Default::default(),
                    }
                );
            }
            sendmmsg(s, msgs.iter(), flags)
                .map(move |sent_bytes| {
                    assert!(!sent_bytes.is_empty());
                    for sent in &sent_bytes {
                        assert_eq!(*sent, m.len());
                    }
                    sent_bytes.len()
                })
        }, |_, _ | {});
        // UDP sockets should set the from address
        assert_eq!(AddressFamily::Inet, from.unwrap().family());
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
    ))]
    #[test]
    pub fn udp_recvmmsg() {
        use nix::sys::uio::IoVec;
        use nix::sys::socket::{MsgFlags, recvmmsg};

        const NUM_MESSAGES_SENT: usize = 2;
        const DATA: [u8; 2] = [1,2];

        let std_sa = SocketAddr::from_str("127.0.0.1:6798").unwrap();
        let inet_addr = InetAddr::from_std(&std_sa);
        let sock_addr = SockAddr::new_inet(inet_addr);

        let rsock = socket(AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None
        ).unwrap();
        bind(rsock, &sock_addr).unwrap();
        let ssock = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");

        let send_thread = thread::spawn(move || {
            for _ in 0..NUM_MESSAGES_SENT {
                sendto(ssock, &DATA[..], &sock_addr, MsgFlags::empty()).unwrap();
            }
        });

        let mut msgs = std::collections::LinkedList::new();

        // Buffers to receive exactly `NUM_MESSAGES_SENT` messages
        let mut receive_buffers = [[0u8; 32]; NUM_MESSAGES_SENT];
        let iovs: Vec<_> = receive_buffers.iter_mut().map(|buf| {
            [IoVec::from_mut_slice(&mut buf[..])]
        }).collect();

        for iov in &iovs {
            msgs.push_back(RecvMmsgData {
                iov,
                cmsg_buffer: None,
            })
        };

        let res = recvmmsg(rsock, &mut msgs, MsgFlags::empty(), None).expect("recvmmsg");
        assert_eq!(res.len(), DATA.len());

        for RecvMsg { address, bytes, .. } in res.into_iter() {
            assert_eq!(AddressFamily::Inet, address.unwrap().family());
            assert_eq!(DATA.len(), bytes);
        }

        for buf in &receive_buffers {
            assert_eq!(&buf[..DATA.len()], DATA);
        }

        send_thread.join().unwrap();
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
    ))]
    #[test]
    pub fn udp_recvmmsg_dontwait_short_read() {
        use nix::sys::uio::IoVec;
        use nix::sys::socket::{MsgFlags, recvmmsg};

        const NUM_MESSAGES_SENT: usize = 2;
        const DATA: [u8; 4] = [1,2,3,4];

        let std_sa = SocketAddr::from_str("127.0.0.1:6799").unwrap();
        let inet_addr = InetAddr::from_std(&std_sa);
        let sock_addr = SockAddr::new_inet(inet_addr);

        let rsock = socket(AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None
        ).unwrap();
        bind(rsock, &sock_addr).unwrap();
        let ssock = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");

        let send_thread = thread::spawn(move || {
            for _ in 0..NUM_MESSAGES_SENT {
                sendto(ssock, &DATA[..], &sock_addr, MsgFlags::empty()).unwrap();
            }
        });
        // Ensure we've sent all the messages before continuing so `recvmmsg`
        // will return right away
        send_thread.join().unwrap();

        let mut msgs = std::collections::LinkedList::new();

        // Buffers to receive >`NUM_MESSAGES_SENT` messages to ensure `recvmmsg`
        // will return when there are fewer than requested messages in the
        // kernel buffers when using `MSG_DONTWAIT`.
        let mut receive_buffers = [[0u8; 32]; NUM_MESSAGES_SENT + 2];
        let iovs: Vec<_> = receive_buffers.iter_mut().map(|buf| {
            [IoVec::from_mut_slice(&mut buf[..])]
        }).collect();

        for iov in &iovs {
            msgs.push_back(RecvMmsgData {
                iov,
                cmsg_buffer: None,
            })
        };

        let res = recvmmsg(rsock, &mut msgs, MsgFlags::MSG_DONTWAIT, None).expect("recvmmsg");
        assert_eq!(res.len(), NUM_MESSAGES_SENT);

        for RecvMsg { address, bytes, .. } in res.into_iter() {
            assert_eq!(AddressFamily::Inet, address.unwrap().family());
            assert_eq!(DATA.len(), bytes);
        }

        for buf in &receive_buffers[..NUM_MESSAGES_SENT] {
            assert_eq!(&buf[..DATA.len()], DATA);
        }
    }
}

// Test error handling of our recvmsg wrapper
#[test]
pub fn test_recvmsg_ebadf() {
    use nix::errno::Errno;
    use nix::sys::socket::{MsgFlags, recvmsg};
    use nix::sys::uio::IoVec;

    let mut buf = [0u8; 5];
    let iov = [IoVec::from_mut_slice(&mut buf[..])];
    let fd = -1;    // Bad file descriptor
    let r = recvmsg(fd, &iov, None, MsgFlags::empty());
    assert_eq!(r.err().unwrap(), Errno::EBADF);
}

// Disable the test on emulated platforms due to a bug in QEMU versions <
// 2.12.0.  https://bugs.launchpad.net/qemu/+bug/1701808
#[cfg_attr(qemu, ignore)]
#[test]
pub fn test_scm_rights() {
    use nix::sys::uio::IoVec;
    use nix::unistd::{pipe, read, write, close};
    use nix::sys::socket::{socketpair, sendmsg, recvmsg,
                           AddressFamily, SockType, SockFlag,
                           ControlMessage, ControlMessageOwned, MsgFlags};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();
    let (r, w) = pipe().unwrap();
    let mut received_r: Option<RawFd> = None;

    {
        let iov = [IoVec::from_slice(b"hello")];
        let fds = [r];
        let cmsg = ControlMessage::ScmRights(&fds);
        assert_eq!(sendmsg(fd1, &iov, &[cmsg], MsgFlags::empty(), None).unwrap(), 5);
        close(r).unwrap();
        close(fd1).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let mut cmsgspace = cmsg_space!([RawFd; 1]);
        let msg = recvmsg(fd2, &iov, Some(&mut cmsgspace), MsgFlags::empty()).unwrap();

        for cmsg in msg.cmsgs() {
            if let ControlMessageOwned::ScmRights(fd) = cmsg {
                assert_eq!(received_r, None);
                assert_eq!(fd.len(), 1);
                received_r = Some(fd[0]);
            } else {
                panic!("unexpected cmsg");
            }
        }
        assert_eq!(msg.bytes, 5);
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        close(fd2).unwrap();
    }

    let received_r = received_r.expect("Did not receive passed fd");
    // Ensure that the received file descriptor works
    write(w, b"world").unwrap();
    let mut buf = [0u8; 5];
    read(received_r, &mut buf).unwrap();
    assert_eq!(&buf[..], b"world");
    close(received_r).unwrap();
    close(w).unwrap();
}

// Disable the test on emulated platforms due to not enabled support of AF_ALG in QEMU from rust cross
#[cfg(any(target_os = "linux", target_os= "android"))]
#[cfg_attr(qemu, ignore)]
#[test]
pub fn test_af_alg_cipher() {
    use nix::sys::uio::IoVec;
    use nix::unistd::read;
    use nix::sys::socket::{socket, sendmsg, bind, accept, setsockopt,
                           AddressFamily, SockType, SockFlag, SockAddr,
                           ControlMessage, MsgFlags};
    use nix::sys::socket::sockopt::AlgSetKey;

    skip_if_cirrus!("Fails for an unknown reason Cirrus CI.  Bug #1352");
    // Travis's seccomp profile blocks AF_ALG
    // https://docs.docker.com/engine/security/seccomp/
    skip_if_seccomp!(test_af_alg_cipher);

    let alg_type = "skcipher";
    let alg_name = "ctr-aes-aesni";
    // 256-bits secret key
    let key = vec![0u8; 32];
    // 16-bytes IV
    let iv_len = 16;
    let iv = vec![1u8; iv_len];
    // 256-bytes plain payload
    let payload_len = 256;
    let payload = vec![2u8; payload_len];

    let sock = socket(AddressFamily::Alg, SockType::SeqPacket, SockFlag::empty(), None)
        .expect("socket failed");

    let sockaddr = SockAddr::new_alg(alg_type, alg_name);
    bind(sock, &sockaddr).expect("bind failed");

    if let SockAddr::Alg(alg) = sockaddr {
        assert_eq!(alg.alg_name().to_string_lossy(), alg_name);
        assert_eq!(alg.alg_type().to_string_lossy(), alg_type);
    } else {
        panic!("unexpected SockAddr");
    }

    setsockopt(sock, AlgSetKey::default(), &key).expect("setsockopt");
    let session_socket = accept(sock).expect("accept failed");

    let msgs = [ControlMessage::AlgSetOp(&libc::ALG_OP_ENCRYPT), ControlMessage::AlgSetIv(iv.as_slice())];
    let iov = IoVec::from_slice(&payload);
    sendmsg(session_socket, &[iov], &msgs, MsgFlags::empty(), None).expect("sendmsg encrypt");

    // allocate buffer for encrypted data
    let mut encrypted = vec![0u8; payload_len];
    let num_bytes = read(session_socket, &mut encrypted).expect("read encrypt");
    assert_eq!(num_bytes, payload_len);

    let iov = IoVec::from_slice(&encrypted);

    let iv = vec![1u8; iv_len];

    let msgs = [ControlMessage::AlgSetOp(&libc::ALG_OP_DECRYPT), ControlMessage::AlgSetIv(iv.as_slice())];
    sendmsg(session_socket, &[iov], &msgs, MsgFlags::empty(), None).expect("sendmsg decrypt");

    // allocate buffer for decrypted data
    let mut decrypted = vec![0u8; payload_len];
    let num_bytes = read(session_socket, &mut decrypted).expect("read decrypt");

    assert_eq!(num_bytes, payload_len);
    assert_eq!(decrypted, payload);
}

// Disable the test on emulated platforms due to not enabled support of AF_ALG
// in QEMU from rust cross
#[cfg(any(target_os = "linux", target_os= "android"))]
#[cfg_attr(qemu, ignore)]
#[test]
pub fn test_af_alg_aead() {
    use libc::{ALG_OP_DECRYPT, ALG_OP_ENCRYPT};
    use nix::fcntl::{fcntl, FcntlArg, OFlag};
    use nix::sys::uio::IoVec;
    use nix::unistd::{read, close};
    use nix::sys::socket::{socket, sendmsg, bind, accept, setsockopt,
                           AddressFamily, SockType, SockFlag, SockAddr,
                           ControlMessage, MsgFlags};
    use nix::sys::socket::sockopt::{AlgSetKey, AlgSetAeadAuthSize};

    skip_if_cirrus!("Fails for an unknown reason Cirrus CI.  Bug #1352");
    // Travis's seccomp profile blocks AF_ALG
    // https://docs.docker.com/engine/security/seccomp/
    skip_if_seccomp!(test_af_alg_aead);

    let auth_size = 4usize;
    let assoc_size = 16u32;

    let alg_type = "aead";
    let alg_name = "gcm(aes)";
    // 256-bits secret key
    let key = vec![0u8; 32];
    // 12-bytes IV
    let iv_len = 12;
    let iv = vec![1u8; iv_len];
    // 256-bytes plain payload
    let payload_len = 256;
    let mut payload = vec![2u8; payload_len + (assoc_size as usize) + auth_size];

    for i in 0..assoc_size {
        payload[i as usize] = 10;
    }

    let len = payload.len();

    for i in 0..auth_size {
        payload[len - 1 - i] = 0;
    }

    let sock = socket(AddressFamily::Alg, SockType::SeqPacket, SockFlag::empty(), None)
        .expect("socket failed");

    let sockaddr = SockAddr::new_alg(alg_type, alg_name);
    bind(sock, &sockaddr).expect("bind failed");

    setsockopt(sock, AlgSetAeadAuthSize, &auth_size).expect("setsockopt AlgSetAeadAuthSize");
    setsockopt(sock, AlgSetKey::default(), &key).expect("setsockopt AlgSetKey");
    let session_socket = accept(sock).expect("accept failed");

    let msgs = [
        ControlMessage::AlgSetOp(&ALG_OP_ENCRYPT),
        ControlMessage::AlgSetIv(iv.as_slice()),
        ControlMessage::AlgSetAeadAssoclen(&assoc_size)];
    let iov = IoVec::from_slice(&payload);
    sendmsg(session_socket, &[iov], &msgs, MsgFlags::empty(), None).expect("sendmsg encrypt");

    // allocate buffer for encrypted data
    let mut encrypted = vec![0u8; (assoc_size as usize) + payload_len + auth_size];
    let num_bytes = read(session_socket, &mut encrypted).expect("read encrypt");
    assert_eq!(num_bytes, payload_len + auth_size + (assoc_size as usize));
    close(session_socket).expect("close");

    for i in 0..assoc_size {
        encrypted[i as usize] = 10;
    }

    let iov = IoVec::from_slice(&encrypted);

    let iv = vec![1u8; iv_len];

    let session_socket = accept(sock).expect("accept failed");

    let msgs = [
        ControlMessage::AlgSetOp(&ALG_OP_DECRYPT),
        ControlMessage::AlgSetIv(iv.as_slice()),
        ControlMessage::AlgSetAeadAssoclen(&assoc_size),
    ];
    sendmsg(session_socket, &[iov], &msgs, MsgFlags::empty(), None).expect("sendmsg decrypt");

    // allocate buffer for decrypted data
    let mut decrypted = vec![0u8; payload_len + (assoc_size as usize) + auth_size];
    // Starting with kernel 4.9, the interface changed slightly such that the
    // authentication tag memory is only needed in the output buffer for encryption
    // and in the input buffer for decryption.
    // Do not block on read, as we may have fewer bytes than buffer size
    fcntl(session_socket,FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).expect("fcntl non_blocking");
    let num_bytes = read(session_socket, &mut decrypted).expect("read decrypt");

    assert!(num_bytes >= payload_len + (assoc_size as usize));
    assert_eq!(decrypted[(assoc_size as usize)..(payload_len + (assoc_size as usize))], payload[(assoc_size as usize)..payload_len + (assoc_size as usize)]);
}

// Verify `ControlMessage::Ipv4PacketInfo` for `sendmsg`.
// This creates a (udp) socket bound to localhost, then sends a message to
// itself but uses Ipv4PacketInfo to force the source address to be localhost.
//
// This would be a more interesting test if we could assume that the test host
// has more than one IP address (since we could select a different address to
// test from).
#[cfg(any(target_os = "linux",
        target_os = "macos",
        target_os = "netbsd"))]
#[test]
pub fn test_sendmsg_ipv4packetinfo() {
    use cfg_if::cfg_if;
    use nix::sys::uio::IoVec;
    use nix::sys::socket::{socket, sendmsg, bind,
                           AddressFamily, SockType, SockFlag, SockAddr,
                           ControlMessage, MsgFlags};

    let sock = socket(AddressFamily::Inet,
                      SockType::Datagram,
                      SockFlag::empty(),
                      None)
        .expect("socket failed");

    let std_sa = SocketAddr::from_str("127.0.0.1:4000").unwrap();
    let inet_addr = InetAddr::from_std(&std_sa);
    let sock_addr = SockAddr::new_inet(inet_addr);

    bind(sock, &sock_addr).expect("bind failed");

    let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let iov = [IoVec::from_slice(&slice)];

    if let InetAddr::V4(sin) = inet_addr {
        cfg_if! {
            if #[cfg(target_os = "netbsd")] {
                let _dontcare = sin;
                let pi = libc::in_pktinfo {
                    ipi_ifindex: 0, /* Unspecified interface */
                    ipi_addr: libc::in_addr { s_addr: 0 },
                };
            } else {
                let pi = libc::in_pktinfo {
                    ipi_ifindex: 0, /* Unspecified interface */
                    ipi_addr: libc::in_addr { s_addr: 0 },
                    ipi_spec_dst: sin.sin_addr,
                };
            }
        }

        let cmsg = [ControlMessage::Ipv4PacketInfo(&pi)];

        sendmsg(sock, &iov, &cmsg, MsgFlags::empty(), Some(&sock_addr))
            .expect("sendmsg");
    } else {
        panic!("No IPv4 addresses available for testing?");
    }
}

// Verify `ControlMessage::Ipv6PacketInfo` for `sendmsg`.
// This creates a (udp) socket bound to ip6-localhost, then sends a message to
// itself but uses Ipv6PacketInfo to force the source address to be
// ip6-localhost.
//
// This would be a more interesting test if we could assume that the test host
// has more than one IP address (since we could select a different address to
// test from).
#[cfg(any(target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "freebsd"))]
#[test]
pub fn test_sendmsg_ipv6packetinfo() {
    use nix::errno::Errno;
    use nix::sys::uio::IoVec;
    use nix::sys::socket::{socket, sendmsg, bind,
                           AddressFamily, SockType, SockFlag, SockAddr,
                           ControlMessage, MsgFlags};

    let sock = socket(AddressFamily::Inet6,
                      SockType::Datagram,
                      SockFlag::empty(),
                      None)
        .expect("socket failed");

    let std_sa = SocketAddr::from_str("[::1]:6000").unwrap();
    let inet_addr = InetAddr::from_std(&std_sa);
    let sock_addr = SockAddr::new_inet(inet_addr);

    if let Err(Errno::EADDRNOTAVAIL) = bind(sock, &sock_addr) {
        println!("IPv6 not available, skipping test.");
        return;
    }

    let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let iov = [IoVec::from_slice(&slice)];

    if let InetAddr::V6(sin) = inet_addr {
        let pi = libc::in6_pktinfo {
            ipi6_ifindex: 0, /* Unspecified interface */
            ipi6_addr: sin.sin6_addr,
        };

        let cmsg = [ControlMessage::Ipv6PacketInfo(&pi)];

        sendmsg(sock, &iov, &cmsg, MsgFlags::empty(), Some(&sock_addr))
            .expect("sendmsg");
    } else {
        println!("No IPv6 addresses available for testing: skipping testing Ipv6PacketInfo");
    }
}

/// Tests that passing multiple fds using a single `ControlMessage` works.
// Disable the test on emulated platforms due to a bug in QEMU versions <
// 2.12.0.  https://bugs.launchpad.net/qemu/+bug/1701808
#[cfg_attr(qemu, ignore)]
#[test]
fn test_scm_rights_single_cmsg_multiple_fds() {
    use std::os::unix::net::UnixDatagram;
    use std::os::unix::io::{RawFd, AsRawFd};
    use std::thread;
    use nix::sys::socket::{ControlMessage, ControlMessageOwned, MsgFlags,
        sendmsg, recvmsg};
    use nix::sys::uio::IoVec;

    let (send, receive) = UnixDatagram::pair().unwrap();
    let thread = thread::spawn(move || {
        let mut buf = [0u8; 8];
        let iovec = [IoVec::from_mut_slice(&mut buf)];
        let mut space = cmsg_space!([RawFd; 2]);
        let msg = recvmsg(
            receive.as_raw_fd(),
            &iovec,
            Some(&mut space),
            MsgFlags::empty()
        ).unwrap();
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));

        let mut cmsgs = msg.cmsgs();
        match cmsgs.next() {
            Some(ControlMessageOwned::ScmRights(fds)) => {
                assert_eq!(fds.len(), 2,
                           "unexpected fd count (expected 2 fds, got {})",
                           fds.len());
            },
            _ => panic!(),
        }
        assert!(cmsgs.next().is_none(), "unexpected control msg");

        assert_eq!(msg.bytes, 8);
        assert_eq!(iovec[0].as_slice(), [1u8, 2, 3, 4, 5, 6, 7, 8]);
    });

    let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let iov = [IoVec::from_slice(&slice)];
    let fds = [libc::STDIN_FILENO, libc::STDOUT_FILENO];    // pass stdin and stdout
    let cmsg = [ControlMessage::ScmRights(&fds)];
    sendmsg(send.as_raw_fd(), &iov, &cmsg, MsgFlags::empty(), None).unwrap();
    thread.join().unwrap();
}

// Verify `sendmsg` builds a valid `msghdr` when passing an empty
// `cmsgs` argument.  This should result in a msghdr with a nullptr
// msg_control field and a msg_controllen of 0 when calling into the
// raw `sendmsg`.
#[test]
pub fn test_sendmsg_empty_cmsgs() {
    use nix::sys::uio::IoVec;
    use nix::unistd::close;
    use nix::sys::socket::{socketpair, sendmsg, recvmsg,
                           AddressFamily, SockType, SockFlag, MsgFlags};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();

    {
        let iov = [IoVec::from_slice(b"hello")];
        assert_eq!(sendmsg(fd1, &iov, &[], MsgFlags::empty(), None).unwrap(), 5);
        close(fd1).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let mut cmsgspace = cmsg_space!([RawFd; 1]);
        let msg = recvmsg(fd2, &iov, Some(&mut cmsgspace), MsgFlags::empty()).unwrap();

        for _ in msg.cmsgs() {
            panic!("unexpected cmsg");
        }
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        assert_eq!(msg.bytes, 5);
        close(fd2).unwrap();
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
))]
#[test]
fn test_scm_credentials() {
    use nix::sys::uio::IoVec;
    use nix::unistd::{close, getpid, getuid, getgid};
    use nix::sys::socket::{socketpair, sendmsg, recvmsg,
                           AddressFamily, SockType, SockFlag,
                           ControlMessage, ControlMessageOwned, MsgFlags,
                           UnixCredentials};
    #[cfg(any(target_os = "android", target_os = "linux"))]
    use nix::sys::socket::{setsockopt, sockopt::PassCred};

    let (send, recv) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
        .unwrap();
    #[cfg(any(target_os = "android", target_os = "linux"))]
    setsockopt(recv, PassCred, &true).unwrap();

    {
        let iov = [IoVec::from_slice(b"hello")];
        #[cfg(any(target_os = "android", target_os = "linux"))]
        let cred = UnixCredentials::new();
        #[cfg(any(target_os = "android", target_os = "linux"))]
        let cmsg = ControlMessage::ScmCredentials(&cred);
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        let cmsg = ControlMessage::ScmCreds;
        assert_eq!(sendmsg(send, &iov, &[cmsg], MsgFlags::empty(), None).unwrap(), 5);
        close(send).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let mut cmsgspace = cmsg_space!(UnixCredentials);
        let msg = recvmsg(recv, &iov, Some(&mut cmsgspace), MsgFlags::empty()).unwrap();
        let mut received_cred = None;

        for cmsg in msg.cmsgs() {
            let cred = match cmsg {
                #[cfg(any(target_os = "android", target_os = "linux"))]
                ControlMessageOwned::ScmCredentials(cred) => cred,
                #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
                ControlMessageOwned::ScmCreds(cred) => cred,
                other => panic!("unexpected cmsg {:?}", other),
            };
            assert!(received_cred.is_none());
            assert_eq!(cred.pid(), getpid().as_raw());
            assert_eq!(cred.uid(), getuid().as_raw());
            assert_eq!(cred.gid(), getgid().as_raw());
            received_cred = Some(cred);
        }
        received_cred.expect("no creds received");
        assert_eq!(msg.bytes, 5);
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        close(recv).unwrap();
    }
}

/// Ensure that we can send `SCM_CREDENTIALS` and `SCM_RIGHTS` with a single
/// `sendmsg` call.
#[cfg(any(target_os = "android", target_os = "linux"))]
// qemu's handling of multiple cmsgs is bugged, ignore tests under emulation
// see https://bugs.launchpad.net/qemu/+bug/1781280
#[cfg_attr(qemu, ignore)]
#[test]
fn test_scm_credentials_and_rights() {
    let space = cmsg_space!(libc::ucred, RawFd);
    test_impl_scm_credentials_and_rights(space);
}

/// Ensure that passing a an oversized control message buffer to recvmsg
/// still works.
#[cfg(any(target_os = "android", target_os = "linux"))]
// qemu's handling of multiple cmsgs is bugged, ignore tests under emulation
// see https://bugs.launchpad.net/qemu/+bug/1781280
#[cfg_attr(qemu, ignore)]
#[test]
fn test_too_large_cmsgspace() {
    let space = vec![0u8; 1024];
    test_impl_scm_credentials_and_rights(space);
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_impl_scm_credentials_and_rights(mut space: Vec<u8>) {
    use libc::ucred;
    use nix::sys::uio::IoVec;
    use nix::unistd::{pipe, write, close, getpid, getuid, getgid};
    use nix::sys::socket::{socketpair, sendmsg, recvmsg, setsockopt,
                           SockType, SockFlag,
                           ControlMessage, ControlMessageOwned, MsgFlags};
    use nix::sys::socket::sockopt::PassCred;

    let (send, recv) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
        .unwrap();
    setsockopt(recv, PassCred, &true).unwrap();

    let (r, w) = pipe().unwrap();
    let mut received_r: Option<RawFd> = None;

    {
        let iov = [IoVec::from_slice(b"hello")];
        let cred = ucred {
            pid: getpid().as_raw(),
            uid: getuid().as_raw(),
            gid: getgid().as_raw(),
        }.into();
        let fds = [r];
        let cmsgs = [
            ControlMessage::ScmCredentials(&cred),
            ControlMessage::ScmRights(&fds),
        ];
        assert_eq!(sendmsg(send, &iov, &cmsgs, MsgFlags::empty(), None).unwrap(), 5);
        close(r).unwrap();
        close(send).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let msg = recvmsg(recv, &iov, Some(&mut space), MsgFlags::empty()).unwrap();
        let mut received_cred = None;

        assert_eq!(msg.cmsgs().count(), 2, "expected 2 cmsgs");

        for cmsg in msg.cmsgs() {
            match cmsg {
                ControlMessageOwned::ScmRights(fds) => {
                    assert_eq!(received_r, None, "already received fd");
                    assert_eq!(fds.len(), 1);
                    received_r = Some(fds[0]);
                }
                ControlMessageOwned::ScmCredentials(cred) => {
                    assert!(received_cred.is_none());
                    assert_eq!(cred.pid(), getpid().as_raw());
                    assert_eq!(cred.uid(), getuid().as_raw());
                    assert_eq!(cred.gid(), getgid().as_raw());
                    received_cred = Some(cred);
                }
                _ => panic!("unexpected cmsg"),
            }
        }
        received_cred.expect("no creds received");
        assert_eq!(msg.bytes, 5);
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        close(recv).unwrap();
    }

    let received_r = received_r.expect("Did not receive passed fd");
    // Ensure that the received file descriptor works
    write(w, b"world").unwrap();
    let mut buf = [0u8; 5];
    read(received_r, &mut buf).unwrap();
    assert_eq!(&buf[..], b"world");
    close(received_r).unwrap();
    close(w).unwrap();
}

// Test creating and using named unix domain sockets
#[test]
pub fn test_unixdomain() {
    use nix::sys::socket::{SockType, SockFlag};
    use nix::sys::socket::{bind, socket, connect, listen, accept, SockAddr};
    use nix::unistd::{read, write, close};
    use std::thread;

    let tempdir = tempfile::tempdir().unwrap();
    let sockname = tempdir.path().join("sock");
    let s1 = socket(AddressFamily::Unix, SockType::Stream,
                    SockFlag::empty(), None).expect("socket failed");
    let sockaddr = SockAddr::new_unix(&sockname).unwrap();
    bind(s1, &sockaddr).expect("bind failed");
    listen(s1, 10).expect("listen failed");

    let thr = thread::spawn(move || {
        let s2 = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)
                 .expect("socket failed");
        connect(s2, &sockaddr).expect("connect failed");
        write(s2, b"hello").expect("write failed");
        close(s2).unwrap();
    });

    let s3 = accept(s1).expect("accept failed");

    let mut buf = [0;5];
    read(s3, &mut buf).unwrap();
    close(s3).unwrap();
    close(s1).unwrap();
    thr.join().unwrap();

    assert_eq!(&buf[..], b"hello");
}

// Test creating and using named system control sockets
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[test]
pub fn test_syscontrol() {
    use nix::errno::Errno;
    use nix::sys::socket::{socket, SockAddr, SockType, SockFlag, SockProtocol};

    let fd = socket(AddressFamily::System, SockType::Datagram,
                    SockFlag::empty(), SockProtocol::KextControl)
             .expect("socket failed");
    let _sockaddr = SockAddr::new_sys_control(fd, "com.apple.net.utun_control", 0).expect("resolving sys_control name failed");
    assert_eq!(SockAddr::new_sys_control(fd, "foo.bar.lol", 0).err(), Some(Errno::ENOENT));

    // requires root privileges
    // connect(fd, &sockaddr).expect("connect failed");
}

#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn loopback_address(family: AddressFamily) -> Option<nix::ifaddrs::InterfaceAddress> {
    use std::io;
    use std::io::Write;
    use nix::ifaddrs::getifaddrs;
    use nix::net::if_::*;

    let addrs = match getifaddrs() {
        Ok(iter) => iter,
        Err(e) => {
            let stdioerr = io::stderr();
            let mut handle = stdioerr.lock();
            writeln!(handle, "getifaddrs: {:?}", e).unwrap();
            return None;
        },
    };
    // return first address matching family
    for ifaddr in addrs {
        if ifaddr.flags.contains(InterfaceFlags::IFF_LOOPBACK) {
            match ifaddr.address {
                Some(SockAddr::Inet(InetAddr::V4(..))) => {
                    match family {
                        AddressFamily::Inet => return Some(ifaddr),
                        _ => continue
                    }
                },
                Some(SockAddr::Inet(InetAddr::V6(..))) => {
                    match family {
                        AddressFamily::Inet6 => return Some(ifaddr),
                        _ => continue
                    }
                },
                _ => continue,
            }
        }
    }
    None
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
))]
// qemu doesn't seem to be emulating this correctly in these architectures
#[cfg_attr(all(
    qemu,
    any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "powerpc64",
    )
), ignore)]
#[test]
pub fn test_recv_ipv4pktinfo() {
    use nix::sys::socket::sockopt::Ipv4PacketInfo;
    use nix::sys::socket::{bind, SockFlag, SockType};
    use nix::sys::socket::{getsockname, setsockopt, socket};
    use nix::sys::socket::{recvmsg, sendmsg, ControlMessageOwned, MsgFlags};
    use nix::sys::uio::IoVec;
    use nix::net::if_::*;

    let lo_ifaddr = loopback_address(AddressFamily::Inet);
    let (lo_name, lo) = match lo_ifaddr {
        Some(ifaddr) => (ifaddr.interface_name,
                         ifaddr.address.expect("Expect IPv4 address on interface")),
        None => return,
    };
    let receive = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("receive socket failed");
    bind(receive, &lo).expect("bind failed");
    let sa = getsockname(receive).expect("getsockname failed");
    setsockopt(receive, Ipv4PacketInfo, &true).expect("setsockopt failed");

    {
        let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let iov = [IoVec::from_slice(&slice)];

        let send = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");
        sendmsg(send, &iov, &[], MsgFlags::empty(), Some(&sa)).expect("sendmsg failed");
    }

    {
        let mut buf = [0u8; 8];
        let iovec = [IoVec::from_mut_slice(&mut buf)];
        let mut space = cmsg_space!(libc::in_pktinfo);
        let msg = recvmsg(
            receive,
            &iovec,
            Some(&mut space),
            MsgFlags::empty(),
        ).expect("recvmsg failed");
        assert!(
            !msg.flags
                .intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC)
        );

        let mut cmsgs = msg.cmsgs();
        if let Some(ControlMessageOwned::Ipv4PacketInfo(pktinfo)) = cmsgs.next() {
            let i = if_nametoindex(lo_name.as_bytes()).expect("if_nametoindex");
            assert_eq!(
                pktinfo.ipi_ifindex as libc::c_uint,
                i,
                "unexpected ifindex (expected {}, got {})",
                i,
                pktinfo.ipi_ifindex
            );
        }
        assert!(cmsgs.next().is_none(), "unexpected additional control msg");
        assert_eq!(msg.bytes, 8);
        assert_eq!(
            iovec[0].as_slice(),
            [1u8, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
// qemu doesn't seem to be emulating this correctly in these architectures
#[cfg_attr(all(
    qemu,
    any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "powerpc64",
    )
), ignore)]
#[test]
pub fn test_recvif() {
    use nix::net::if_::*;
    use nix::sys::socket::sockopt::{Ipv4RecvIf, Ipv4RecvDstAddr};
    use nix::sys::socket::{bind, SockFlag, SockType};
    use nix::sys::socket::{getsockname, setsockopt, socket, SockAddr};
    use nix::sys::socket::{recvmsg, sendmsg, ControlMessageOwned, MsgFlags};
    use nix::sys::uio::IoVec;

    let lo_ifaddr = loopback_address(AddressFamily::Inet);
    let (lo_name, lo) = match lo_ifaddr {
        Some(ifaddr) => (ifaddr.interface_name,
                         ifaddr.address.expect("Expect IPv4 address on interface")),
        None => return,
    };
    let receive = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    ).expect("receive socket failed");
    bind(receive, &lo).expect("bind failed");
    let sa = getsockname(receive).expect("getsockname failed");
    setsockopt(receive, Ipv4RecvIf, &true).expect("setsockopt IP_RECVIF failed");
    setsockopt(receive, Ipv4RecvDstAddr, &true).expect("setsockopt IP_RECVDSTADDR failed");

    {
        let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let iov = [IoVec::from_slice(&slice)];

        let send = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");
        sendmsg(send, &iov, &[], MsgFlags::empty(), Some(&sa)).expect("sendmsg failed");
    }

    {
        let mut buf = [0u8; 8];
        let iovec = [IoVec::from_mut_slice(&mut buf)];
        let mut space = cmsg_space!(libc::sockaddr_dl, libc::in_addr);
        let msg = recvmsg(
            receive,
            &iovec,
            Some(&mut space),
            MsgFlags::empty(),
        ).expect("recvmsg failed");
        assert!(
            !msg.flags
                .intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC)
        );
        assert_eq!(msg.cmsgs().count(), 2, "expected 2 cmsgs");

        let mut rx_recvif = false;
        let mut rx_recvdstaddr = false;
        for cmsg in msg.cmsgs() {
            match cmsg {
                ControlMessageOwned::Ipv4RecvIf(dl) => {
                    rx_recvif = true;
                    let i = if_nametoindex(lo_name.as_bytes()).expect("if_nametoindex");
                    assert_eq!(
                        dl.sdl_index as libc::c_uint,
                        i,
                        "unexpected ifindex (expected {}, got {})",
                        i,
                        dl.sdl_index
                    );
                },
                ControlMessageOwned::Ipv4RecvDstAddr(addr) => {
                    rx_recvdstaddr = true;
                    if let SockAddr::Inet(InetAddr::V4(a)) = lo {
                        assert_eq!(a.sin_addr.s_addr,
                                   addr.s_addr,
                                   "unexpected destination address (expected {}, got {})",
                                   a.sin_addr.s_addr,
                                   addr.s_addr);
                    } else {
                        panic!("unexpected Sockaddr");
                    }
                },
                _ => panic!("unexpected additional control msg"),
            }
        }
        assert!(rx_recvif);
        assert!(rx_recvdstaddr);
        assert_eq!(msg.bytes, 8);
        assert_eq!(
            iovec[0].as_slice(),
            [1u8, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
// qemu doesn't seem to be emulating this correctly in these architectures
#[cfg_attr(all(
    qemu,
    any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "powerpc64",
    )
), ignore)]
#[test]
pub fn test_recv_ipv6pktinfo() {
    use nix::net::if_::*;
    use nix::sys::socket::sockopt::Ipv6RecvPacketInfo;
    use nix::sys::socket::{bind, SockFlag, SockType};
    use nix::sys::socket::{getsockname, setsockopt, socket};
    use nix::sys::socket::{recvmsg, sendmsg, ControlMessageOwned, MsgFlags};
    use nix::sys::uio::IoVec;

    let lo_ifaddr = loopback_address(AddressFamily::Inet6);
    let (lo_name, lo) = match lo_ifaddr {
        Some(ifaddr) => (ifaddr.interface_name,
                         ifaddr.address.expect("Expect IPv4 address on interface")),
        None => return,
    };
    let receive = socket(
        AddressFamily::Inet6,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    ).expect("receive socket failed");
    bind(receive, &lo).expect("bind failed");
    let sa = getsockname(receive).expect("getsockname failed");
    setsockopt(receive, Ipv6RecvPacketInfo, &true).expect("setsockopt failed");

    {
        let slice = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let iov = [IoVec::from_slice(&slice)];

        let send = socket(
            AddressFamily::Inet6,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        ).expect("send socket failed");
        sendmsg(send, &iov, &[], MsgFlags::empty(), Some(&sa)).expect("sendmsg failed");
    }

    {
        let mut buf = [0u8; 8];
        let iovec = [IoVec::from_mut_slice(&mut buf)];
        let mut space = cmsg_space!(libc::in6_pktinfo);
        let msg = recvmsg(
            receive,
            &iovec,
            Some(&mut space),
            MsgFlags::empty(),
        ).expect("recvmsg failed");
        assert!(
            !msg.flags
                .intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC)
        );

        let mut cmsgs = msg.cmsgs();
        if let Some(ControlMessageOwned::Ipv6PacketInfo(pktinfo)) = cmsgs.next()
        {
            let i = if_nametoindex(lo_name.as_bytes()).expect("if_nametoindex");
            assert_eq!(
                pktinfo.ipi6_ifindex as libc::c_uint,
                i,
                "unexpected ifindex (expected {}, got {})",
                i,
                pktinfo.ipi6_ifindex
            );
        }
        assert!(cmsgs.next().is_none(), "unexpected additional control msg");
        assert_eq!(msg.bytes, 8);
        assert_eq!(
            iovec[0].as_slice(),
            [1u8, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[cfg_attr(graviton, ignore = "Not supported by the CI environment")]
#[test]
pub fn test_vsock() {
    use nix::errno::Errno;
    use nix::sys::socket::{AddressFamily, socket, bind, connect, listen,
                           SockAddr, SockType, SockFlag};
    use nix::unistd::{close};
    use std::thread;

    let port: u32 = 3000;

    let s1 = socket(AddressFamily::Vsock,  SockType::Stream,
                    SockFlag::empty(), None)
             .expect("socket failed");

    // VMADDR_CID_HYPERVISOR is reserved, so we expect an EADDRNOTAVAIL error.
    let sockaddr = SockAddr::new_vsock(libc::VMADDR_CID_HYPERVISOR, port);
    assert_eq!(bind(s1, &sockaddr).err(),
               Some(Errno::EADDRNOTAVAIL));

    let sockaddr = SockAddr::new_vsock(libc::VMADDR_CID_ANY, port);
    assert_eq!(bind(s1, &sockaddr), Ok(()));
    listen(s1, 10).expect("listen failed");

    let thr = thread::spawn(move || {
        let cid: u32 = libc::VMADDR_CID_HOST;

        let s2 = socket(AddressFamily::Vsock, SockType::Stream,
                        SockFlag::empty(), None)
                 .expect("socket failed");

        let sockaddr = SockAddr::new_vsock(cid, port);

        // The current implementation does not support loopback devices, so,
        // for now, we expect a failure on the connect.
        assert_ne!(connect(s2, &sockaddr), Ok(()));

        close(s2).unwrap();
    });

    close(s1).unwrap();
    thr.join().unwrap();
}

// Disable the test on emulated platforms because it fails in Cirrus-CI.  Lack
// of QEMU support is suspected.
#[cfg_attr(qemu, ignore)]
#[cfg(all(target_os = "linux"))]
#[test]
fn test_recvmsg_timestampns() {
    use nix::sys::socket::*;
    use nix::sys::uio::IoVec;
    use nix::sys::time::*;
    use std::time::*;

    // Set up
    let message = "Ohayō!".as_bytes();
    let in_socket = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None).unwrap();
    setsockopt(in_socket, sockopt::ReceiveTimestampns, &true).unwrap();
    let localhost = InetAddr::new(IpAddr::new_v4(127, 0, 0, 1), 0);
    bind(in_socket, &SockAddr::new_inet(localhost)).unwrap();
    let address = getsockname(in_socket).unwrap();
    // Get initial time
    let time0 = SystemTime::now();
    // Send the message
    let iov = [IoVec::from_slice(message)];
    let flags = MsgFlags::empty();
    let l = sendmsg(in_socket, &iov, &[], flags, Some(&address)).unwrap();
    assert_eq!(message.len(), l);
    // Receive the message
    let mut buffer = vec![0u8; message.len()];
    let mut cmsgspace = nix::cmsg_space!(TimeSpec);
    let iov = [IoVec::from_mut_slice(&mut buffer)];
    let r = recvmsg(in_socket, &iov, Some(&mut cmsgspace), flags).unwrap();
    let rtime = match r.cmsgs().next() {
        Some(ControlMessageOwned::ScmTimestampns(rtime)) => rtime,
        Some(_) => panic!("Unexpected control message"),
        None => panic!("No control message")
    };
    // Check the final time
    let time1 = SystemTime::now();
    // the packet's received timestamp should lie in-between the two system
    // times, unless the system clock was adjusted in the meantime.
    let rduration = Duration::new(rtime.tv_sec() as u64,
    rtime.tv_nsec() as u32);
    assert!(time0.duration_since(UNIX_EPOCH).unwrap() <= rduration);
    assert!(rduration <= time1.duration_since(UNIX_EPOCH).unwrap());
    // Close socket
    nix::unistd::close(in_socket).unwrap();
}

// Disable the test on emulated platforms because it fails in Cirrus-CI.  Lack
// of QEMU support is suspected.
#[cfg_attr(qemu, ignore)]
#[cfg(all(target_os = "linux"))]
#[test]
fn test_recvmmsg_timestampns() {
    use nix::sys::socket::*;
    use nix::sys::uio::IoVec;
    use nix::sys::time::*;
    use std::time::*;

    // Set up
    let message = "Ohayō!".as_bytes();
    let in_socket = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None).unwrap();
    setsockopt(in_socket, sockopt::ReceiveTimestampns, &true).unwrap();
    let localhost = InetAddr::new(IpAddr::new_v4(127, 0, 0, 1), 0);
    bind(in_socket, &SockAddr::new_inet(localhost)).unwrap();
    let address = getsockname(in_socket).unwrap();
    // Get initial time
    let time0 = SystemTime::now();
    // Send the message
    let iov = [IoVec::from_slice(message)];
    let flags = MsgFlags::empty();
    let l = sendmsg(in_socket, &iov, &[], flags, Some(&address)).unwrap();
    assert_eq!(message.len(), l);
    // Receive the message
    let mut buffer = vec![0u8; message.len()];
    let mut cmsgspace = nix::cmsg_space!(TimeSpec);
    let iov = [IoVec::from_mut_slice(&mut buffer)];
    let mut data = vec![
        RecvMmsgData {
            iov,
            cmsg_buffer: Some(&mut cmsgspace),
        },
    ];
    let r = recvmmsg(in_socket, &mut data, flags, None).unwrap();
    let rtime = match r[0].cmsgs().next() {
        Some(ControlMessageOwned::ScmTimestampns(rtime)) => rtime,
        Some(_) => panic!("Unexpected control message"),
        None => panic!("No control message")
    };
    // Check the final time
    let time1 = SystemTime::now();
    // the packet's received timestamp should lie in-between the two system
    // times, unless the system clock was adjusted in the meantime.
    let rduration = Duration::new(rtime.tv_sec() as u64,
    rtime.tv_nsec() as u32);
    assert!(time0.duration_since(UNIX_EPOCH).unwrap() <= rduration);
    assert!(rduration <= time1.duration_since(UNIX_EPOCH).unwrap());
    // Close socket
    nix::unistd::close(in_socket).unwrap();
}

// Disable the test on emulated platforms because it fails in Cirrus-CI.  Lack
// of QEMU support is suspected.
#[cfg_attr(qemu, ignore)]
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[test]
fn test_recvmsg_rxq_ovfl() {
    use nix::Error;
    use nix::sys::socket::*;
    use nix::sys::uio::IoVec;
    use nix::sys::socket::sockopt::{RxqOvfl, RcvBuf};

    let message = [0u8; 2048];
    let bufsize = message.len() * 2;

    let in_socket = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None).unwrap();
    let out_socket = socket(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None).unwrap();

    let localhost = InetAddr::new(IpAddr::new_v4(127, 0, 0, 1), 0);
    bind(in_socket, &SockAddr::new_inet(localhost)).unwrap();

    let address = getsockname(in_socket).unwrap();
    connect(out_socket, &address).unwrap();

    // Set SO_RXQ_OVFL flag.
    setsockopt(in_socket, RxqOvfl, &1).unwrap();

    // Set the receiver buffer size to hold only 2 messages.
    setsockopt(in_socket, RcvBuf, &bufsize).unwrap();

    let mut drop_counter = 0;

    for _ in 0..2 {
        let iov = [IoVec::from_slice(&message)];
        let flags = MsgFlags::empty();

        // Send the 3 messages (the receiver buffer can only hold 2 messages)
        // to create an overflow.
        for _ in 0..3 {
            let l = sendmsg(out_socket, &iov, &[], flags, Some(&address)).unwrap();
            assert_eq!(message.len(), l);
        }

        // Receive the message and check the drop counter if any.
        loop {
            let mut buffer = vec![0u8; message.len()];
            let mut cmsgspace = nix::cmsg_space!(u32);

            let iov = [IoVec::from_mut_slice(&mut buffer)];

            match recvmsg(
                in_socket,
                &iov,
                Some(&mut cmsgspace),
                MsgFlags::MSG_DONTWAIT) {
                Ok(r) => {
                    drop_counter = match r.cmsgs().next() {
                        Some(ControlMessageOwned::RxqOvfl(drop_counter)) => drop_counter,
                        Some(_) => panic!("Unexpected control message"),
                        None => 0,
                    };
                },
                Err(Error::EAGAIN) => { break; },
                _ => { panic!("unknown recvmsg() error"); },
            }
        }
    }

    // One packet lost.
    assert_eq!(drop_counter, 1);

    // Close sockets
    nix::unistd::close(in_socket).unwrap();
    nix::unistd::close(out_socket).unwrap();
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
))]
mod linux_errqueue {
    use nix::sys::socket::*;
    use super::{FromStr, SocketAddr};

    // Send a UDP datagram to a bogus destination address and observe an ICMP error (v4).
    //
    // Disable the test on QEMU because QEMU emulation of IP_RECVERR is broken (as documented on PR
    // #1514).
    #[cfg_attr(qemu, ignore)]
    #[test]
    fn test_recverr_v4() {
        #[repr(u8)]
        enum IcmpTypes {
            DestUnreach = 3, // ICMP_DEST_UNREACH
        }
        #[repr(u8)]
        enum IcmpUnreachCodes {
            PortUnreach = 3, // ICMP_PORT_UNREACH
        }

        test_recverr_impl::<sockaddr_in, _, _>(
            "127.0.0.1:6800",
            AddressFamily::Inet,
            sockopt::Ipv4RecvErr,
            libc::SO_EE_ORIGIN_ICMP,
            IcmpTypes::DestUnreach as u8,
            IcmpUnreachCodes::PortUnreach as u8,
            // Closure handles protocol-specific testing and returns generic sock_extended_err for
            // protocol-independent test impl.
            |cmsg| {
                if let ControlMessageOwned::Ipv4RecvErr(ext_err, err_addr) = cmsg {
                    if let Some(origin) = err_addr {
                        // Validate that our network error originated from 127.0.0.1:0.
                        assert_eq!(origin.sin_family, AddressFamily::Inet as _);
                        assert_eq!(Ipv4Addr(origin.sin_addr), Ipv4Addr::new(127, 0, 0, 1));
                        assert_eq!(origin.sin_port, 0);
                    } else {
                        panic!("Expected some error origin");
                    }
                    *ext_err
                } else {
                    panic!("Unexpected control message {:?}", cmsg);
                }
            },
        )
    }

    // Essentially the same test as v4.
    //
    // Disable the test on QEMU because QEMU emulation of IPV6_RECVERR is broken (as documented on
    // PR #1514).
    #[cfg_attr(qemu, ignore)]
    #[test]
    fn test_recverr_v6() {
        #[repr(u8)]
        enum IcmpV6Types {
            DestUnreach = 1, // ICMPV6_DEST_UNREACH
        }
        #[repr(u8)]
        enum IcmpV6UnreachCodes {
            PortUnreach = 4, // ICMPV6_PORT_UNREACH
        }

        test_recverr_impl::<sockaddr_in6, _, _>(
            "[::1]:6801",
            AddressFamily::Inet6,
            sockopt::Ipv6RecvErr,
            libc::SO_EE_ORIGIN_ICMP6,
            IcmpV6Types::DestUnreach as u8,
            IcmpV6UnreachCodes::PortUnreach as u8,
            // Closure handles protocol-specific testing and returns generic sock_extended_err for
            // protocol-independent test impl.
            |cmsg| {
                if let ControlMessageOwned::Ipv6RecvErr(ext_err, err_addr) = cmsg {
                    if let Some(origin) = err_addr {
                        // Validate that our network error originated from localhost:0.
                        assert_eq!(origin.sin6_family, AddressFamily::Inet6 as _);
                        assert_eq!(
                            Ipv6Addr(origin.sin6_addr),
                            Ipv6Addr::from_std(&"::1".parse().unwrap()),
                        );
                        assert_eq!(origin.sin6_port, 0);
                    } else {
                        panic!("Expected some error origin");
                    }
                    *ext_err
                } else {
                    panic!("Unexpected control message {:?}", cmsg);
                }
            },
        )
    }

    fn test_recverr_impl<SA, OPT, TESTF>(sa: &str,
                                         af: AddressFamily,
                                         opt: OPT,
                                         ee_origin: u8,
                                         ee_type: u8,
                                         ee_code: u8,
                                         testf: TESTF)
        where
            OPT: SetSockOpt<Val = bool>,
            TESTF: FnOnce(&ControlMessageOwned) -> libc::sock_extended_err,
    {
        use nix::errno::Errno;
        use nix::sys::uio::IoVec;

        const MESSAGE_CONTENTS: &str = "ABCDEF";

        let sock_addr = {
            let std_sa = SocketAddr::from_str(sa).unwrap();
            let inet_addr = InetAddr::from_std(&std_sa);
            SockAddr::new_inet(inet_addr)
        };
        let sock = socket(af, SockType::Datagram, SockFlag::SOCK_CLOEXEC, None).unwrap();
        setsockopt(sock, opt, &true).unwrap();
        if let Err(e) = sendto(sock, MESSAGE_CONTENTS.as_bytes(), &sock_addr, MsgFlags::empty()) {
            assert_eq!(e, Errno::EADDRNOTAVAIL);
            println!("{:?} not available, skipping test.", af);
            return;
        }

        let mut buf = [0u8; 8];
        let iovec = [IoVec::from_mut_slice(&mut buf)];
        let mut cspace = cmsg_space!(libc::sock_extended_err, SA);

        let msg = recvmsg(sock, &iovec, Some(&mut cspace), MsgFlags::MSG_ERRQUEUE).unwrap();
        // The sent message / destination associated with the error is returned:
        assert_eq!(msg.bytes, MESSAGE_CONTENTS.as_bytes().len());
        assert_eq!(&buf[..msg.bytes], MESSAGE_CONTENTS.as_bytes());
        // recvmsg(2): "The original destination address of the datagram that caused the error is
        // supplied via msg_name;" however, this is not literally true.  E.g., an earlier version
        // of this test used 0.0.0.0 (::0) as the destination address, which was mutated into
        // 127.0.0.1 (::1).
        assert_eq!(msg.address, Some(sock_addr));

        // Check for expected control message.
        let ext_err = match msg.cmsgs().next() {
            Some(cmsg) => testf(&cmsg),
            None => panic!("No control message"),
        };

        assert_eq!(ext_err.ee_errno, libc::ECONNREFUSED as u32);
        assert_eq!(ext_err.ee_origin, ee_origin);
        // ip(7): ee_type and ee_code are set from the type and code fields of the ICMP (ICMPv6)
        // header.
        assert_eq!(ext_err.ee_type, ee_type);
        assert_eq!(ext_err.ee_code, ee_code);
        // ip(7): ee_info contains the discovered MTU for EMSGSIZE errors.
        assert_eq!(ext_err.ee_info, 0);
    }
}
