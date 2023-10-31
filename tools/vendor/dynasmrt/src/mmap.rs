//! This module implements some wrappers around Mmap/MmapMut to also support a cheap "empty" variant.
// Unfortunately Memmap itself doesn't support a cheap zero-length variant

use std::ops::{Deref, DerefMut};
use std::io;

use memmap2::{Mmap, MmapMut};

use crate::AssemblyOffset;

/// A structure holding a buffer of executable memory. It also derefs to a `&[u8]`.
/// This structure does not allocate when its size is 0.
#[derive(Debug)]
pub struct ExecutableBuffer {
    // length of the buffer that has actually been written to
    length: usize,
    // backing buffer
    buffer: Option<Mmap>
}

/// ExecutableBuffer equivalent that holds a buffer of mutable memory instead of executable memory. It also derefs to a `&mut [u8]`.
/// This structure does not allocate when its size is 0.
#[derive(Debug)]
pub struct MutableBuffer {
    // length of the buffer that has actually been written to
    length: usize,
    // backing buffer
    buffer: Option<MmapMut>
}

impl ExecutableBuffer {
    /// Obtain a pointer into the executable memory from an offset into it.
    /// When an offset returned from `DynasmLabelApi::offset` is used, the resulting pointer
    /// will point to the start of the first instruction after the offset call,
    /// which can then be jumped or called to divert control flow into the executable
    /// buffer. Note that if this buffer is accessed through an Executor, these pointers
    /// will only be valid as long as its lock is held. When no locks are held,
    /// The assembler is free to relocate the executable buffer when it requires
    /// more memory than available.
    pub fn ptr(&self, offset: AssemblyOffset) -> *const u8 {
        &self[offset.0] as *const u8
    }

    /// Create a new executable buffer, backed by a buffer of size `size`.
    /// It will start with an initialized length of 0.
    pub fn new(size: usize) -> io::Result<ExecutableBuffer> {
        let buffer = if size == 0 {
            None
        } else {
            Some(MmapMut::map_anon(size)?.make_exec()?)
        };

        Ok(ExecutableBuffer {
            length: 0,
            buffer
        })
    }

    /// Query the backing size of this executable buffer
    pub fn size(&self) -> usize {
        self.buffer.as_ref().map(|b| b.len()).unwrap_or(0) as usize
    }

    /// Change this executable buffer into a mutable buffer.
    pub fn make_mut(self) -> io::Result<MutableBuffer> {
        let buffer = if let Some(map) = self.buffer {
            Some(map.make_mut()?)
        } else {
            None
        };

        Ok(MutableBuffer {
            length: self.length,
            buffer
        })
    }
}

impl MutableBuffer {
    /// Create a new mutable buffer, backed by a buffer of size `size`.
    /// It will start with an initialized length of 0.
    pub fn new(size: usize) -> io::Result<MutableBuffer> {
        let buffer = if size == 0 {
            None
        } else {
            Some(MmapMut::map_anon(size)?)
        };

        Ok(MutableBuffer {
            length: 0,
            buffer
        })
    }

    /// Query the backing size of this mutable buffer
    pub fn size(&self) -> usize {
        self.buffer.as_ref().map(|b| b.len()).unwrap_or(0) as usize
    }

    /// Set the length of the usable part of this mutable buffer. The length
    /// should not be set larger than the allocated size, otherwise methods can panic.
    pub fn set_len(&mut self, length: usize) {
        self.length = length
    }

    /// Change this mutable buffer into an executable buffer.
    pub fn make_exec(self) -> io::Result<ExecutableBuffer> {
        let buffer = if let Some(map) = self.buffer {
            Some(map.make_exec()?)
        } else {
            None
        };

        Ok(ExecutableBuffer {
            length: self.length,
            buffer
        })
    }
}

impl Default for ExecutableBuffer {
    fn default() -> ExecutableBuffer {
        ExecutableBuffer {
            length: 0,
            buffer: None
        }
    }
}

impl Default for MutableBuffer {
    fn default() -> MutableBuffer {
        MutableBuffer {
            length: 0,
            buffer: None
        }
    }
}

impl Deref for ExecutableBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        if let Some(map) = &self.buffer {
            &map[..self.length]
        } else {
            &[]
        }
    }
}

impl Deref for MutableBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        if let Some(map) = &self.buffer {
            &map[..self.length]
        } else {
            &[]
        }
    }
}

impl DerefMut for MutableBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        if let Some(map) = &mut self.buffer {
            &mut map[..self.length]
        } else {
            &mut []
        }
    }
}
