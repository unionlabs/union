use super::error::{Error, Result, ENOSYS};

pub unsafe fn syscall0(_a: usize) -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub unsafe fn syscall1(_a: usize, _b: usize) -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub unsafe fn syscall2(_a: usize, _b: usize, _c: usize) -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub unsafe fn syscall3(_a: usize, _b: usize, _c: usize, _d: usize) -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub unsafe fn syscall4(_a: usize, _b: usize, _c: usize, _d: usize, _e: usize) -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub unsafe fn syscall5(_a: usize, _b: usize, _c: usize, _d: usize, _e: usize, _f: usize)
                       -> Result<usize> {
    Err(Error::new(ENOSYS))
}
