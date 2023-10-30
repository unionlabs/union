#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg(feature = "close")]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

use io_lifetimes::raw::{AsRawFilelike, AsRawSocketlike};
use io_lifetimes::views::{FilelikeView, SocketlikeView};
use io_lifetimes::{
    AsFilelike, AsSocketlike, BorrowedFilelike, FromFilelike, FromSocketlike, IntoFilelike,
    IntoSocketlike,
};
use std::io::{Read, Write};

struct Tester {}
impl Tester {
    fn use_file<Filelike: AsFilelike>(filelike: Filelike) {
        let mut buf = Vec::new();

        let filelike = filelike.as_filelike();

        let view = filelike.as_filelike_view::<std::fs::File>();
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();

        let view = unsafe {
            FilelikeView::<std::fs::File>::view_raw(
                filelike
                    .as_filelike_view::<std::fs::File>()
                    .as_raw_filelike(),
            )
        };
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();

        let _ = dbg!(filelike);
    }

    fn use_socket<Socketlike: AsSocketlike>(socketlike: Socketlike) {
        let mut buf = Vec::new();

        let socketlike = socketlike.as_socketlike();
        let view = socketlike.as_socketlike_view::<std::net::TcpStream>();
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();

        let view = unsafe {
            SocketlikeView::<std::net::TcpStream>::view_raw(
                socketlike
                    .as_socketlike_view::<std::net::TcpStream>()
                    .as_raw_socketlike(),
            )
        };
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();

        let _ = dbg!(socketlike);
    }

    fn from_file<Filelike: IntoFilelike>(filelike: Filelike) {
        let mut buf = Vec::new();

        let filelike = filelike.into_filelike();
        let view = filelike.as_filelike_view::<std::fs::File>();
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();
        drop(view);

        let _ = dbg!(&filelike);
        let _ = std::fs::File::from_filelike(filelike);
    }

    fn from_socket<Socketlike: IntoSocketlike>(socketlike: Socketlike) {
        let mut buf = Vec::new();

        let socketlike = socketlike.into_socketlike();
        let view = socketlike.as_socketlike_view::<std::net::TcpStream>();
        let _ = (&*view).read(&mut buf).is_ok();
        let _ = (&*view).write(&buf).is_ok();
        drop(view);

        let _ = dbg!(&socketlike);
        let _ = std::net::TcpStream::from_socketlike(socketlike);
    }

    fn from_into_file<Filelike: IntoFilelike>(filelike: Filelike) {
        let _ = std::fs::File::from_into_filelike(filelike);
    }

    fn from_into_socket<Socketlike: IntoSocketlike>(socketlike: Socketlike) {
        let _ = std::net::TcpStream::from_into_socketlike(socketlike);
    }
}

#[test]
fn test_api() {
    let file = std::fs::File::open("Cargo.toml").unwrap();
    Tester::use_file(&file);
    Tester::use_file(file.as_filelike());
    Tester::use_file(&*file.as_filelike_view::<std::fs::File>());
    Tester::use_file(file.as_filelike_view::<std::fs::File>().as_filelike());

    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    Tester::use_socket(&socket);
    Tester::use_socket(socket.as_socketlike());
    Tester::use_socket(&*socket.as_socketlike_view::<std::net::TcpListener>());
    Tester::use_socket(
        socket
            .as_socketlike_view::<std::net::TcpListener>()
            .as_socketlike(),
    );

    Tester::from_file(std::fs::File::open("Cargo.toml").unwrap().into_filelike());
    Tester::from_file(
        std::fs::File::open("Cargo.toml")
            .unwrap()
            .into_filelike()
            .into_filelike(),
    );
    Tester::from_socket(
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .into_socketlike(),
    );
    Tester::from_socket(
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .into_socketlike()
            .into_socketlike(),
    );

    Tester::from_into_file(std::fs::File::open("Cargo.toml").unwrap().into_filelike());
    Tester::from_into_socket(
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .into_socketlike(),
    );
}

#[test]
fn test_as() {
    let file = std::fs::File::open("Cargo.toml").unwrap();
    let borrow: BorrowedFilelike = file.as_filelike();
    let reborrow: BorrowedFilelike = borrow.as_filelike();
    let ref_reborrow: &BorrowedFilelike = &reborrow;
    let borrow_ref_reborrow: BorrowedFilelike = ref_reborrow.as_filelike();
    let _ref_borrow_ref_reborrow: &BorrowedFilelike = &borrow_ref_reborrow;
}
