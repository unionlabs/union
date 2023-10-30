#![cfg(any(target_os = "android", target_os = "linux"))]

use rustix::fd::AsFd;
use rustix::io::epoll::{self, Epoll};
use rustix::io::{ioctl_fionbio, read, write, OwnedFd};
use rustix::net::{
    accept, bind_v4, connect_v4, getsockname, listen, socket, AddressFamily, Ipv4Addr, Protocol,
    SocketAddrAny, SocketAddrV4, SocketType,
};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const BUFFER_SIZE: usize = 20;

fn server(ready: Arc<(Mutex<u16>, Condvar)>) {
    let listen_sock = socket(AddressFamily::INET, SocketType::STREAM, Protocol::default()).unwrap();
    bind_v4(&listen_sock, &SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
    listen(&listen_sock, 1).unwrap();

    let who = match getsockname(&listen_sock).unwrap() {
        SocketAddrAny::V4(addr) => addr,
        _ => panic!(),
    };

    {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        *port = who.port();
        cvar.notify_all();
    }

    let epoll = Epoll::new(epoll::CreateFlags::CLOEXEC, epoll::Owning::<OwnedFd>::new()).unwrap();

    // Test into conversions.
    let fd: OwnedFd = epoll.into();
    let epoll: Epoll<epoll::Owning<OwnedFd>> = fd.into();
    let fd: RawFd = epoll.into_raw_fd();
    let epoll = unsafe { Epoll::<epoll::Owning<OwnedFd>>::from_raw_fd(fd) };

    let raw_listen_sock = listen_sock.as_fd().as_raw_fd();
    epoll.add(listen_sock, epoll::EventFlags::IN).unwrap();

    let mut event_list = epoll::EventVec::with_capacity(4);
    loop {
        epoll.wait(&mut event_list, -1).unwrap();
        for (_event_flags, target) in &event_list {
            if target.as_raw_fd() == raw_listen_sock {
                let conn_sock = accept(&*target).unwrap();
                ioctl_fionbio(&conn_sock, true).unwrap();
                epoll
                    .add(conn_sock, epoll::EventFlags::OUT | epoll::EventFlags::ET)
                    .unwrap();
            } else {
                write(&*target, b"hello\n").unwrap();
                let _ = epoll.del(target).unwrap();
            }
        }
    }
}

fn client(ready: Arc<(Mutex<u16>, Condvar)>) {
    let port = {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        while *port == 0 {
            port = cvar.wait(port).unwrap();
        }
        *port
    };

    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut buffer = vec![0; BUFFER_SIZE];

    for _ in 0..16 {
        let data_socket =
            socket(AddressFamily::INET, SocketType::STREAM, Protocol::default()).unwrap();
        connect_v4(&data_socket, &addr).unwrap();

        let nread = read(&data_socket, &mut buffer).unwrap();
        assert_eq!(String::from_utf8_lossy(&buffer[..nread]), "hello\n");
    }
}

#[test]
fn test_epoll() {
    let ready = Arc::new((Mutex::new(0_u16), Condvar::new()));
    let ready_clone = Arc::clone(&ready);

    let _server = thread::Builder::new()
        .name("server".to_string())
        .spawn(move || {
            server(ready);
        })
        .unwrap();
    let client = thread::Builder::new()
        .name("client".to_string())
        .spawn(move || {
            client(ready_clone);
        })
        .unwrap();
    client.join().unwrap();
}
