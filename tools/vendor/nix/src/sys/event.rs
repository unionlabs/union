/* TOOD: Implement for other kqueue based systems
 */

use crate::{Errno, Result};
#[cfg(not(target_os = "netbsd"))]
use libc::{timespec, time_t, c_int, c_long, intptr_t, uintptr_t};
#[cfg(target_os = "netbsd")]
use libc::{timespec, time_t, c_long, intptr_t, uintptr_t, size_t};
use std::convert::TryInto;
use std::os::unix::io::RawFd;
use std::ptr;

// Redefine kevent in terms of programmer-friendly enums and bitfields.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KEvent {
    kevent: libc::kevent,
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos",
          target_os = "openbsd"))]
type type_of_udata = *mut libc::c_void;
#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos"))]
type type_of_data = intptr_t;
#[cfg(any(target_os = "netbsd"))]
type type_of_udata = intptr_t;
#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
type type_of_data = i64;

#[cfg(target_os = "netbsd")]
type type_of_event_filter = u32;
#[cfg(not(target_os = "netbsd"))]
type type_of_event_filter = i16;
libc_enum! {
    #[cfg_attr(target_os = "netbsd", repr(u32))]
    #[cfg_attr(not(target_os = "netbsd"), repr(i16))]
    #[non_exhaustive]
    pub enum EventFilter {
        EVFILT_AIO,
        /// Returns whenever there is no remaining data in the write buffer
        #[cfg(target_os = "freebsd")]
        EVFILT_EMPTY,
        #[cfg(target_os = "dragonfly")]
        EVFILT_EXCEPT,
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos"))]
        EVFILT_FS,
        #[cfg(target_os = "freebsd")]
        EVFILT_LIO,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        EVFILT_MACHPORT,
        EVFILT_PROC,
        /// Returns events associated with the process referenced by a given
        /// process descriptor, created by `pdfork()`. The events to monitor are:
        ///
        /// - NOTE_EXIT: the process has exited. The exit status will be stored in data.
        #[cfg(target_os = "freebsd")]
        EVFILT_PROCDESC,
        EVFILT_READ,
        /// Returns whenever an asynchronous `sendfile()` call completes.
        #[cfg(target_os = "freebsd")]
        EVFILT_SENDFILE,
        EVFILT_SIGNAL,
        EVFILT_TIMER,
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos"))]
        EVFILT_USER,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        EVFILT_VM,
        EVFILT_VNODE,
        EVFILT_WRITE,
    }
    impl TryFrom<type_of_event_filter>
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos",
          target_os = "openbsd"))]
pub type type_of_event_flag = u16;
#[cfg(any(target_os = "netbsd"))]
pub type type_of_event_flag = u32;
libc_bitflags!{
    pub struct EventFlag: type_of_event_flag {
        EV_ADD;
        EV_CLEAR;
        EV_DELETE;
        EV_DISABLE;
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd",
                  target_os = "ios", target_os = "macos",
                  target_os = "netbsd", target_os = "openbsd"))]
        EV_DISPATCH;
        #[cfg(target_os = "freebsd")]
        EV_DROP;
        EV_ENABLE;
        EV_EOF;
        EV_ERROR;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_FLAG0;
        EV_FLAG1;
        #[cfg(target_os = "dragonfly")]
        EV_NODATA;
        EV_ONESHOT;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_OOBAND;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_POLL;
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd",
                  target_os = "ios", target_os = "macos",
                  target_os = "netbsd", target_os = "openbsd"))]
        EV_RECEIPT;
        EV_SYSFLAGS;
    }
}

libc_bitflags!(
    pub struct FilterFlag: u32 {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_ABSOLUTE;
        NOTE_ATTRIB;
        NOTE_CHILD;
        NOTE_DELETE;
        #[cfg(target_os = "openbsd")]
        NOTE_EOF;
        NOTE_EXEC;
        NOTE_EXIT;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_EXITSTATUS;
        NOTE_EXTEND;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFAND;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFCOPY;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFCTRLMASK;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFLAGSMASK;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFNOP;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFOR;
        NOTE_FORK;
        NOTE_LINK;
        NOTE_LOWAT;
        #[cfg(target_os = "freebsd")]
        NOTE_MSECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_NONE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_NSECONDS;
        #[cfg(target_os = "dragonfly")]
        NOTE_OOB;
        NOTE_PCTRLMASK;
        NOTE_PDATAMASK;
        NOTE_RENAME;
        NOTE_REVOKE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_SECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_SIGNAL;
        NOTE_TRACK;
        NOTE_TRACKERR;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_TRIGGER;
        #[cfg(target_os = "openbsd")]
        NOTE_TRUNCATE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_USECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_ERROR;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE_SUDDEN_TERMINATE;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE_TERMINATE;
        NOTE_WRITE;
    }
);

pub fn kqueue() -> Result<RawFd> {
    let res = unsafe { libc::kqueue() };

    Errno::result(res)
}


// KEvent can't derive Send because on some operating systems, udata is defined
// as a void*.  However, KEvent's public API always treats udata as an intptr_t,
// which is safe to Send.
unsafe impl Send for KEvent {
}

impl KEvent {
    pub fn new(ident: uintptr_t, filter: EventFilter, flags: EventFlag,
               fflags:FilterFlag, data: intptr_t, udata: intptr_t) -> KEvent {
        KEvent { kevent: libc::kevent {
            ident,
            filter: filter as type_of_event_filter,
            flags: flags.bits(),
            fflags: fflags.bits(),
            data: data as type_of_data,
            udata: udata as type_of_udata
        } }
    }

    pub fn ident(&self) -> uintptr_t {
        self.kevent.ident
    }

    pub fn filter(&self) -> Result<EventFilter> {
        self.kevent.filter.try_into()
    }

    pub fn flags(&self) -> EventFlag {
        EventFlag::from_bits(self.kevent.flags).unwrap()
    }

    pub fn fflags(&self) -> FilterFlag {
        FilterFlag::from_bits(self.kevent.fflags).unwrap()
    }

    pub fn data(&self) -> intptr_t {
        self.kevent.data as intptr_t
    }

    pub fn udata(&self) -> intptr_t {
        self.kevent.udata as intptr_t
    }
}

pub fn kevent(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_ms: usize) -> Result<usize> {

    // Convert ms to timespec
    let timeout = timespec {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_nsec: ((timeout_ms % 1000) * 1_000_000) as c_long
    };

    kevent_ts(kq, changelist, eventlist, Some(timeout))
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "openbsd"))]
type type_of_nchanges = c_int;
#[cfg(target_os = "netbsd")]
type type_of_nchanges = size_t;

pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        libc::kevent(
            kq,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as type_of_nchanges,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as type_of_nchanges,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: usize,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: intptr_t) {

    ev.kevent.ident  = ident as uintptr_t;
    ev.kevent.filter = filter as type_of_event_filter;
    ev.kevent.flags  = flags.bits();
    ev.kevent.fflags = fflags.bits();
    ev.kevent.data   = 0;
    ev.kevent.udata  = udata as type_of_udata;
}

#[test]
fn test_struct_kevent() {
    use std::mem;

    let udata : intptr_t = 12345;

    let actual = KEvent::new(0xdead_beef,
                             EventFilter::EVFILT_READ,
                             EventFlag::EV_ONESHOT | EventFlag::EV_ADD,
                             FilterFlag::NOTE_CHILD | FilterFlag::NOTE_EXIT,
                             0x1337,
                             udata);
    assert_eq!(0xdead_beef, actual.ident());
    let filter = actual.kevent.filter;
    assert_eq!(libc::EVFILT_READ, filter);
    assert_eq!(libc::EV_ONESHOT | libc::EV_ADD, actual.flags().bits());
    assert_eq!(libc::NOTE_CHILD | libc::NOTE_EXIT, actual.fflags().bits());
    assert_eq!(0x1337, actual.data() as type_of_data);
    assert_eq!(udata as type_of_udata, actual.udata() as type_of_udata);
    assert_eq!(mem::size_of::<libc::kevent>(), mem::size_of::<KEvent>());
}

#[test]
fn test_kevent_filter() {
    let udata : intptr_t = 12345;

    let actual = KEvent::new(0xdead_beef,
                             EventFilter::EVFILT_READ,
                             EventFlag::EV_ONESHOT | EventFlag::EV_ADD,
                             FilterFlag::NOTE_CHILD | FilterFlag::NOTE_EXIT,
                             0x1337,
                             udata);
    assert_eq!(EventFilter::EVFILT_READ, actual.filter().unwrap());
}
