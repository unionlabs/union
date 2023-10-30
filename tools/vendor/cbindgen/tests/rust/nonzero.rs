use std::num::*;

#[repr(C)]
pub struct NonZeroTest {
    pub a: NonZeroU8,
    pub b: NonZeroU16,
    pub c: NonZeroU32,
    pub d: NonZeroU64,
    pub e: NonZeroI8,
    pub f: NonZeroI16,
    pub g: NonZeroI32,
    pub h: NonZeroI64,
    pub i: Option<NonZeroI64>,
    pub j: *const Option<Option<NonZeroI64>>,
}

#[no_mangle]
pub extern "C" fn root(
    test: NonZeroTest,
    a: NonZeroU8,
    b: NonZeroU16,
    c: NonZeroU32,
    d: NonZeroU64,
    e: NonZeroI8,
    f: NonZeroI16,
    g: NonZeroI32,
    h: NonZeroI64,
    i: Option<NonZeroI64>,
    j: *const Option<Option<NonZeroI64>>,
) {}
