use super::mystd::fs::File;
use super::mystd::os::unix::prelude::*;
use core::ops::Deref;
use core::ptr;
use core::slice;

pub struct Mmap {
    ptr: *mut libc::c_void,
    len: usize,
}

impl Mmap {
    pub unsafe fn map(file: &File, len: usize) -> Option<Mmap> {
        let ptr = libc::mmap(
            ptr::null_mut(),
            len,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        );
        if ptr == libc::MAP_FAILED {
            return None;
        }
        Some(Mmap { ptr, len })
    }
}

impl Deref for Mmap {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr as *const u8, self.len) }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe {
            let r = libc::munmap(self.ptr, self.len);
            debug_assert_eq!(r, 0);
        }
    }
}
