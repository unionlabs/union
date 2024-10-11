use crate::gas_meter::gas_meter_t;
use crate::iterator::GoIter;
use crate::memory::{U8SliceView, UnmanagedVector};
use crate::vtables::Vtable;

// this represents something passed in from the caller side of FFI
#[repr(C)]
pub struct db_t {
    _private: [u8; 0],
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Default)]
pub struct DbVtable {
    pub read_db: Option<
        extern "C" fn(
            db: *mut db_t,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            key: U8SliceView,
            value_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
    pub write_db: Option<
        extern "C" fn(
            db: *mut db_t,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            key: U8SliceView,
            value: U8SliceView,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
    pub remove_db: Option<
        extern "C" fn(
            db: *mut db_t,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            key: U8SliceView,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
    // order -> Ascending = 1, Descending = 2
    // Note: we cannot set gas_meter on the returned GoIter due to cgo memory safety.
    // Since we have the pointer in rust already, we must set that manually
    pub scan_db: Option<
        extern "C" fn(
            db: *mut db_t,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            start: U8SliceView,
            end: U8SliceView,
            order: i32,
            iterator_out: *mut GoIter,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
}

impl Vtable for DbVtable {}

#[repr(C)]
pub struct Db {
    pub gas_meter: *mut gas_meter_t,
    pub state: *mut db_t,
    pub vtable: DbVtable,
}
