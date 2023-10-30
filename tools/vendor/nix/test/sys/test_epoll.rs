use nix::sys::epoll::{EpollCreateFlags, EpollFlags, EpollOp, EpollEvent};
use nix::sys::epoll::{epoll_create1, epoll_ctl};
use nix::errno::Errno;

#[test]
pub fn test_epoll_errno() {
    let efd = epoll_create1(EpollCreateFlags::empty()).unwrap();
    let result = epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::ENOENT);

    let result = epoll_ctl(efd, EpollOp::EpollCtlAdd, 1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Errno::EINVAL);
}

#[test]
pub fn test_epoll_ctl() {
    let efd = epoll_create1(EpollCreateFlags::empty()).unwrap();
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLERR, 1);
    epoll_ctl(efd, EpollOp::EpollCtlAdd, 1, &mut event).unwrap();
    epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None).unwrap();
}
