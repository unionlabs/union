use core::{mem, slice};

use crate::data::*;
use crate::error::*;
use crate::flag::*;
use crate::number::*;
use crate::scheme::str_from_raw_parts;

pub trait SchemeBlockMut {
    fn handle(&mut self, packet: &Packet) -> Option<usize> {
        let res = match packet.a {
            SYS_OPEN => if let Some(path) = unsafe { str_from_raw_parts(packet.b as *const u8, packet.c) } {
                self.open(path, packet.d, packet.uid, packet.gid)
            } else {
                Err(Error::new(EINVAL))
            },
            SYS_CHMOD => if let Some(path) = unsafe { str_from_raw_parts(packet.b as *const u8, packet.c) } {
                self.chmod(path, packet.d as u16, packet.uid, packet.gid)
            } else {
                Err(Error::new(EINVAL))
            },
            SYS_RMDIR => if let Some(path) = unsafe { str_from_raw_parts(packet.b as *const u8, packet.c) } {
                self.rmdir(path, packet.uid, packet.gid)
            } else {
                Err(Error::new(EINVAL))
            },
            SYS_UNLINK => if let Some(path) = unsafe { str_from_raw_parts(packet.b as *const u8, packet.c) } {
                self.unlink(path, packet.uid, packet.gid)
            } else {
                Err(Error::new(EINVAL))
            },

            SYS_DUP => self.dup(packet.b, unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) }),
            SYS_READ => self.read(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_WRITE => self.write(packet.b, unsafe { slice::from_raw_parts(packet.c as *const u8, packet.d) }),
            SYS_LSEEK => self.seek(packet.b, packet.c as isize, packet.d).map(|o| o.map(|o| o as usize)),
            SYS_FCHMOD => self.fchmod(packet.b, packet.c as u16),
            SYS_FCHOWN => self.fchown(packet.b, packet.c as u32, packet.d as u32),
            SYS_FCNTL => self.fcntl(packet.b, packet.c, packet.d),
            SYS_FEVENT => self.fevent(packet.b, EventFlags::from_bits_truncate(packet.c)).map(|f| f.map(|f| f.bits())),
            SYS_FMAP_OLD => if packet.d >= mem::size_of::<OldMap>() {
                self.fmap_old(packet.b, unsafe { &*(packet.c as *const OldMap) })
            } else {
                Err(Error::new(EFAULT))
            },
            SYS_FMAP => if packet.d >= mem::size_of::<Map>() {
                self.fmap(packet.b, unsafe { &*(packet.c as *const Map) })
            } else {
                Err(Error::new(EFAULT))
            },
            SYS_FUNMAP_OLD => self.funmap_old(packet.b),
            SYS_FUNMAP => self.funmap(packet.b, packet.c),
            SYS_FPATH => self.fpath(packet.b, unsafe { slice::from_raw_parts_mut(packet.c as *mut u8, packet.d) }),
            SYS_FRENAME => if let Some(path) = unsafe { str_from_raw_parts(packet.c as *const u8, packet.d) } {
                self.frename(packet.b, path, packet.uid, packet.gid)
            } else {
                Err(Error::new(EINVAL))
            },
            SYS_FSTAT => if packet.d >= mem::size_of::<Stat>() {
                self.fstat(packet.b, unsafe { &mut *(packet.c as *mut Stat) })
            } else {
                Err(Error::new(EFAULT))
            },
            SYS_FSTATVFS => if packet.d >= mem::size_of::<StatVfs>() {
                self.fstatvfs(packet.b, unsafe { &mut *(packet.c as *mut StatVfs) })
            } else {
                Err(Error::new(EFAULT))
            },
            SYS_FSYNC => self.fsync(packet.b),
            SYS_FTRUNCATE => self.ftruncate(packet.b, packet.c),
            SYS_FUTIMENS => if packet.d >= mem::size_of::<TimeSpec>() {
                self.futimens(packet.b, unsafe { slice::from_raw_parts(packet.c as *const TimeSpec, packet.d / mem::size_of::<TimeSpec>()) })
            } else {
                Err(Error::new(EFAULT))
            },
            SYS_CLOSE => self.close(packet.b),
            _ => Err(Error::new(ENOSYS))
        };

        res.transpose().map(Error::mux)
    }

    /* Scheme operations */

    #[allow(unused_variables)]
    fn open(&mut self, path: &str, flags: usize, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn chmod(&mut self, path: &str, mode: u16, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn rmdir(&mut self, path: &str, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(ENOENT))
    }

    #[allow(unused_variables)]
    fn unlink(&mut self, path: &str, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(ENOENT))
    }

    /* Resource operations */
    #[allow(unused_variables)]
    fn dup(&mut self, old_id: usize, buf: &[u8]) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn write(&mut self, id: usize, buf: &[u8]) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn seek(&mut self, id: usize, pos: isize, whence: usize) -> Result<Option<isize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fchmod(&mut self, id: usize, mode: u16) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fchown(&mut self, id: usize, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fcntl(&mut self, id: usize, cmd: usize, arg: usize) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fevent(&mut self, id: usize, flags: EventFlags) -> Result<Option<EventFlags>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fmap_old(&mut self, id: usize, map: &OldMap) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }
    #[allow(unused_variables)]
    fn fmap(&mut self, id: usize, map: &Map) -> Result<Option<usize>> {
        if map.flags.contains(MapFlags::MAP_FIXED) {
            return Err(Error::new(EINVAL));
        }
        self.fmap_old(id, &OldMap {
            offset: map.offset,
            size: map.size,
            flags: map.flags,
        })
    }

    #[allow(unused_variables)]
    fn funmap_old(&mut self, address: usize) -> Result<Option<usize>> {
        Ok(Some(0))
    }

    #[allow(unused_variables)]
    fn funmap(&mut self, address: usize, length: usize) -> Result<Option<usize>> {
        Ok(Some(0))
    }

    #[allow(unused_variables)]
    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn frename(&mut self, id: usize, path: &str, uid: u32, gid: u32) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fstat(&mut self, id: usize, stat: &mut Stat) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fstatvfs(&mut self, id: usize, stat: &mut StatVfs) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn fsync(&mut self, id: usize) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn ftruncate(&mut self, id: usize, len: usize) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn futimens(&mut self, id: usize, times: &[TimeSpec]) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }

    #[allow(unused_variables)]
    fn close(&mut self, id: usize) -> Result<Option<usize>> {
        Err(Error::new(EBADF))
    }
}
