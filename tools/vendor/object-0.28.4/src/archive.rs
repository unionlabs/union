//! Archive definitions.
//!
//! These definitions are independent of read/write support, although we do implement
//! some traits useful for those.

use crate::pod::Pod;

/// File identification bytes stored at the beginning of the file.
pub const MAGIC: [u8; 8] = *b"!<arch>\n";

/// File identification bytes stored at the beginning of a thin archive.
///
/// A thin archive only contains a symbol table and file names.
pub const THIN_MAGIC: [u8; 8] = *b"!<thin>\n";

/// The terminator for each archive member header.
pub const TERMINATOR: [u8; 2] = *b"`\n";

/// The header at the start of an archive member.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    /// The file name.
    pub name: [u8; 16],
    /// File modification timestamp in decimal.
    pub date: [u8; 12],
    /// User ID in decimal.
    pub uid: [u8; 6],
    /// Group ID in decimal.
    pub gid: [u8; 6],
    /// File mode in octal.
    pub mode: [u8; 8],
    /// File size in decimal.
    pub size: [u8; 10],
    /// Must be equal to `TERMINATOR`.
    pub terminator: [u8; 2],
}

unsafe_impl_pod!(Header);
