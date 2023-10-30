use core::{slice, str};

pub use self::scheme::Scheme;
pub use self::scheme_mut::SchemeMut;
pub use self::scheme_block::SchemeBlock;
pub use self::scheme_block_mut::SchemeBlockMut;
pub use self::seek::*;

unsafe fn str_from_raw_parts(ptr: *const u8, len: usize) -> Option<&'static str> {
    let slice = slice::from_raw_parts(ptr, len);
    str::from_utf8(slice).ok()
}

mod scheme;
mod scheme_mut;
mod scheme_block;
mod scheme_block_mut;
mod seek;
