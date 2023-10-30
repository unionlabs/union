use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::UnlockFile;

use std::ops;
use std::os::windows::prelude::*;

use super::utils::syscall;
use super::RwLock;

#[derive(Debug)]
pub struct RwLockReadGuard<'lock, T: AsRawHandle> {
    pub(crate) lock: &'lock RwLock<T>,
}

impl<T: AsRawHandle> ops::Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lock.inner
    }
}

impl<T: AsRawHandle> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let handle = self.lock.inner.as_raw_handle() as HANDLE;
        syscall(unsafe { UnlockFile(handle, 0, 0, 1, 0) })
            .expect("Could not unlock the file descriptor");
    }
}
