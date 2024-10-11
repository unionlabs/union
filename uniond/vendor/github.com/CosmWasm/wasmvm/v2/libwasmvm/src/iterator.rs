use cosmwasm_std::Record;
use cosmwasm_vm::{BackendError, BackendResult, GasInfo};

use crate::error::GoError;
use crate::gas_meter::gas_meter_t;
use crate::memory::UnmanagedVector;
use crate::vtables::Vtable;

/// A reference to some tables on the Go side which allow accessing
/// the actual iterator instance.
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct IteratorReference {
    /// An ID assigned to this contract call
    pub call_id: u64,
    /// An ID assigned to this iterator
    pub iterator_id: u64,
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Default)]
pub struct IteratorVtable {
    pub next: Option<
        extern "C" fn(
            iterator: IteratorReference,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            key_out: *mut UnmanagedVector,
            value_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
    pub next_key: Option<
        extern "C" fn(
            iterator: IteratorReference,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            key_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
    pub next_value: Option<
        extern "C" fn(
            iterator: IteratorReference,
            gas_meter: *mut gas_meter_t,
            gas_used: *mut u64,
            value_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
}

impl Vtable for IteratorVtable {}

#[repr(C)]
pub struct GoIter {
    pub gas_meter: *mut gas_meter_t,
    /// A reference which identifies the iterator and allows finding and accessing the
    /// actual iterator instance in Go. Once fully initalized, this is immutable.
    pub reference: IteratorReference,
    pub vtable: IteratorVtable,
}

impl GoIter {
    /// Creates an incomplete GoIter with unset fields.
    /// This is not ready to be used until those fields are set.
    ///
    /// This is needed to create a correct instance in Rust
    /// which is then filled in Go (see `fn scan`).
    pub fn stub() -> Self {
        GoIter {
            reference: IteratorReference::default(),
            gas_meter: std::ptr::null_mut(),
            vtable: IteratorVtable::default(),
        }
    }

    pub fn next(&mut self) -> BackendResult<Option<Record>> {
        let next = self
            .vtable
            .next
            .expect("iterator vtable function 'next' not set");

        let mut output_key = UnmanagedVector::default();
        let mut output_value = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_result: GoError = (next)(
            self.reference,
            self.gas_meter,
            &mut used_gas as *mut u64,
            &mut output_key as *mut UnmanagedVector,
            &mut output_value as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the `UnmanagedVector`s here, no matter if we need the data.
        let output_key = output_key.consume();
        let output_value = output_value.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || "Failed to fetch next item from iterator".to_string();
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = match output_key {
            Some(key) => {
                if let Some(value) = output_value {
                    Ok(Some((key, value)))
                } else {
                    Err(BackendError::unknown(
                        "Failed to read value while reading the next key in the db",
                    ))
                }
            }
            None => Ok(None),
        };
        (result, gas_info)
    }

    pub fn next_key(&mut self) -> BackendResult<Option<Vec<u8>>> {
        let next_key = self
            .vtable
            .next_key
            .expect("iterator vtable function 'next_key' not set");
        self.next_key_or_val(next_key)
    }

    pub fn next_value(&mut self) -> BackendResult<Option<Vec<u8>>> {
        let next_value = self
            .vtable
            .next_value
            .expect("iterator vtable function 'next_value' not set");
        self.next_key_or_val(next_value)
    }

    #[inline(always)]
    fn next_key_or_val(
        &mut self,
        next: extern "C" fn(
            iterator: IteratorReference,
            gas_meter: *mut gas_meter_t,
            gas_limit: *mut u64,
            key_or_value_out: *mut UnmanagedVector, // key if called from next_key; value if called from next_value
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    ) -> BackendResult<Option<Vec<u8>>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let go_result: GoError = (next)(
            self.reference,
            self.gas_meter,
            &mut used_gas as *mut u64,
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the `UnmanagedVector`s here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || "Failed to fetch next item from iterator".to_string();
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        (Ok(output), gas_info)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn goiter_stub_works() {
        // can be created and dropped
        {
            let _iter = GoIter::stub();
        }

        // creates an all null-instance
        let iter = GoIter::stub();
        assert!(iter.gas_meter.is_null());
        assert_eq!(iter.reference.call_id, 0);
        assert_eq!(iter.reference.iterator_id, 0);
        assert!(iter.vtable.next.is_none());
        assert!(iter.vtable.next_key.is_none());
        assert!(iter.vtable.next_value.is_none());
    }
}
