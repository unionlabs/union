#![allow(missing_docs)]

use crate::void;
use core::marker::{PhantomData, PhantomPinned};
use core::mem;

// . size = 0
// . align = 1
// . ffi-safe
// . !Send
// . !Sync
// . !Unpin
#[repr(C, packed)]
pub struct Opaque {
    _private: [*const void; 0],
    _pinned: PhantomData<PhantomPinned>,
}

const_assert_eq!(0, mem::size_of::<Opaque>());
const_assert_eq!(1, mem::align_of::<Opaque>());
