use std::ffi::CString;
use std::str;

use nix::errno::Errno;
use nix::mqueue::{mq_open, mq_close, mq_send, mq_receive, mq_attr_member_t};
use nix::mqueue::{MqAttr, MQ_OFlag};
use nix::sys::stat::Mode;

#[test]
fn test_mq_send_and_receive() {
    const MSG_SIZE: mq_attr_member_t = 32;
    let attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name= &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();

    let oflag0 = MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY;
    let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
    let r0 = mq_open(mq_name, oflag0, mode, Some(&attr));
    if let Err(Errno::ENOSYS) = r0 {
        println!("message queues not supported or module not loaded?");
        return;
    };
    let mqd0 = r0.unwrap();
    let msg_to_send = "msg_1";
    mq_send(mqd0, msg_to_send.as_bytes(), 1).unwrap();

    let oflag1 = MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY;
    let mqd1 = mq_open(mq_name, oflag1, mode, Some(&attr)).unwrap();
    let mut buf = [0u8; 32];
    let mut prio = 0u32;
    let len = mq_receive(mqd1, &mut buf, &mut prio).unwrap();
    assert_eq!(prio, 1);

    mq_close(mqd1).unwrap();
    mq_close(mqd0).unwrap();
    assert_eq!(msg_to_send, str::from_utf8(&buf[0..len]).unwrap());
}


#[test]
#[cfg(not(any(target_os = "netbsd")))]
fn test_mq_getattr() {
    use nix::mqueue::mq_getattr;
    const MSG_SIZE: mq_attr_member_t = 32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new(b"/attr_test_get_attr".as_ref()).unwrap();
    let oflag = MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY;
    let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
    let r = mq_open(mq_name, oflag, mode, Some(&initial_attr));
    if let Err(Errno::ENOSYS) = r {
        println!("message queues not supported or module not loaded?");
        return;
    };
    let mqd = r.unwrap();

    let read_attr = mq_getattr(mqd).unwrap();
    assert_eq!(read_attr, initial_attr);
    mq_close(mqd).unwrap();
}

// FIXME: Fix failures for mips in QEMU
#[test]
#[cfg(not(any(target_os = "netbsd")))]
#[cfg_attr(all(
        qemu,
        any(target_arch = "mips", target_arch = "mips64")
    ), ignore
)]
fn test_mq_setattr() {
    use nix::mqueue::{mq_getattr, mq_setattr};
    const MSG_SIZE: mq_attr_member_t = 32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new(b"/attr_test_get_attr".as_ref()).unwrap();
    let oflag = MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY;
    let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
    let r = mq_open(mq_name, oflag, mode, Some(&initial_attr));
    if let Err(Errno::ENOSYS) = r {
        println!("message queues not supported or module not loaded?");
        return;
    };
    let mqd = r.unwrap();

    let new_attr =  MqAttr::new(0, 20, MSG_SIZE * 2, 100);
    let old_attr = mq_setattr(mqd, &new_attr).unwrap();
    assert_eq!(old_attr, initial_attr);

    let new_attr_get = mq_getattr(mqd).unwrap();
    // The following tests make sense. No changes here because according to the Linux man page only
    // O_NONBLOCK can be set (see tests below)
    assert_ne!(new_attr_get, new_attr);

    let new_attr_non_blocking =  MqAttr::new(MQ_OFlag::O_NONBLOCK.bits() as mq_attr_member_t, 10, MSG_SIZE, 0);
    mq_setattr(mqd, &new_attr_non_blocking).unwrap();
    let new_attr_get = mq_getattr(mqd).unwrap();

    // now the O_NONBLOCK flag has been set
    assert_ne!(new_attr_get, initial_attr);
    assert_eq!(new_attr_get, new_attr_non_blocking);
    mq_close(mqd).unwrap();
}

// FIXME: Fix failures for mips in QEMU
#[test]
#[cfg(not(any(target_os = "netbsd")))]
#[cfg_attr(all(
        qemu,
        any(target_arch = "mips", target_arch = "mips64")
    ), ignore
)]
fn test_mq_set_nonblocking() {
    use nix::mqueue::{mq_getattr, mq_set_nonblock, mq_remove_nonblock};
    const MSG_SIZE: mq_attr_member_t = 32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new(b"/attr_test_get_attr".as_ref()).unwrap();
    let oflag = MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY;
    let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
    let r = mq_open(mq_name, oflag, mode, Some(&initial_attr));
    if let Err(Errno::ENOSYS) = r {
        println!("message queues not supported or module not loaded?");
        return;
    };
    let mqd = r.unwrap();
    mq_set_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert_eq!(new_attr.unwrap().flags(), MQ_OFlag::O_NONBLOCK.bits() as mq_attr_member_t);
    mq_remove_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert_eq!(new_attr.unwrap().flags(), 0);
    mq_close(mqd).unwrap();
}

#[test]
#[cfg(not(any(target_os = "netbsd")))]
fn test_mq_unlink() {
    use nix::mqueue::mq_unlink;
    const MSG_SIZE: mq_attr_member_t = 32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name_opened = &CString::new(b"/mq_unlink_test".as_ref()).unwrap();
    let mq_name_not_opened = &CString::new(b"/mq_unlink_test".as_ref()).unwrap();
    let oflag = MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY;
    let mode = Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH;
    let r = mq_open(mq_name_opened, oflag, mode, Some(&initial_attr));
    if let Err(Errno::ENOSYS) = r {
        println!("message queues not supported or module not loaded?");
        return;
    };
    let mqd = r.unwrap();

    let res_unlink = mq_unlink(mq_name_opened);
    assert_eq!(res_unlink, Ok(()) );

    let res_unlink_not_opened = mq_unlink(mq_name_not_opened);
    assert_eq!(res_unlink_not_opened, Err(Errno::ENOENT) );

    mq_close(mqd).unwrap();
    let res_unlink_after_close = mq_unlink(mq_name_opened);
    assert_eq!(res_unlink_after_close, Err(Errno::ENOENT) );
}
