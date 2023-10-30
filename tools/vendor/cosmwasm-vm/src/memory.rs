use wasmer::{Array, ValueType, WasmPtr};

use crate::conversion::to_u32;
use crate::errors::{
    CommunicationError, CommunicationResult, RegionValidationError, RegionValidationResult,
    VmResult,
};

/****** read/write to wasm memory buffer ****/

/// Describes some data allocated in Wasm's linear memory.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This is the same as `cosmwasm_std::memory::Region`
/// but defined here to allow Wasmer specific implementation.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct Region {
    /// The beginning of the region expressed as bytes from the beginning of the linear memory
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32,
}

unsafe impl ValueType for Region {}

/// Expects a (fixed size) Region struct at ptr, which is read. This links to the
/// memory region, which is copied in the second step.
/// Errors if the length of the region exceeds `max_length`.
pub fn read_region(memory: &wasmer::Memory, ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
    let region = get_region(memory, ptr)?;

    if region.length > to_u32(max_length)? {
        return Err(
            CommunicationError::region_length_too_big(region.length as usize, max_length).into(),
        );
    }

    match WasmPtr::<u8, Array>::new(region.offset).deref(memory, 0, region.length) {
        Some(cells) => {
            // In case you want to do some premature optimization, this shows how to cast a `&'mut [Cell<u8>]` to `&mut [u8]`:
            // https://github.com/wasmerio/wasmer/blob/0.13.1/lib/wasi/src/syscalls/mod.rs#L79-L81
            let len = region.length as usize;
            let mut result = vec![0u8; len];
            for i in 0..len {
                result[i] = cells[i].get();
            }
            Ok(result)
        }
        None => Err(CommunicationError::deref_err(region.offset, format!(
            "Tried to access memory of region {:?} in wasm memory of size {} bytes. This typically happens when the given Region pointer does not point to a proper Region struct.",
            region,
            memory.size().bytes().0
        )).into()),
    }
}

/// maybe_read_region is like read_region, but gracefully handles null pointer (0) by returning None
/// meant to be used where the argument is optional (like scan)
#[cfg(feature = "iterator")]
pub fn maybe_read_region(
    memory: &wasmer::Memory,
    ptr: u32,
    max_length: usize,
) -> VmResult<Option<Vec<u8>>> {
    if ptr == 0 {
        Ok(None)
    } else {
        read_region(memory, ptr, max_length).map(Some)
    }
}

/// A prepared and sufficiently large memory Region is expected at ptr that points to pre-allocated memory.
///
/// Returns number of bytes written on success.
pub fn write_region(memory: &wasmer::Memory, ptr: u32, data: &[u8]) -> VmResult<()> {
    let mut region = get_region(memory, ptr)?;

    let region_capacity = region.capacity as usize;
    if data.len() > region_capacity {
        return Err(CommunicationError::region_too_small(region_capacity, data.len()).into());
    }
    match WasmPtr::<u8, Array>::new(region.offset).deref(memory, 0, region.capacity) {
        Some(cells) => {
            // In case you want to do some premature optimization, this shows how to cast a `&'mut [Cell<u8>]` to `&mut [u8]`:
            // https://github.com/wasmerio/wasmer/blob/0.13.1/lib/wasi/src/syscalls/mod.rs#L79-L81
            for i in 0..data.len() {
                cells[i].set(data[i])
            }
            region.length = data.len() as u32;
            set_region(memory, ptr, region)?;
            Ok(())
        },
        None => Err(CommunicationError::deref_err(region.offset, format!(
            "Tried to access memory of region {:?} in wasm memory of size {} bytes. This typically happens when the given Region pointer does not point to a proper Region struct.",
            region,
            memory.size().bytes().0
        )).into()),
    }
}

/// Reads in a Region at ptr in wasm memory and returns a copy of it
fn get_region(memory: &wasmer::Memory, ptr: u32) -> CommunicationResult<Region> {
    let wptr = WasmPtr::<Region>::new(ptr);
    match wptr.deref(memory) {
        Some(cell) => {
            let region = cell.get();
            validate_region(&region)?;
            Ok(region)
        }
        None => Err(CommunicationError::deref_err(
            ptr,
            "Could not dereference this pointer to a Region",
        )),
    }
}

/// Performs plausibility checks in the given Region. Regions are always created by the
/// contract and this can be used to detect problems in the standard library of the contract.
fn validate_region(region: &Region) -> RegionValidationResult<()> {
    if region.offset == 0 {
        return Err(RegionValidationError::zero_offset());
    }
    if region.length > region.capacity {
        return Err(RegionValidationError::length_exceeds_capacity(
            region.length,
            region.capacity,
        ));
    }
    if region.capacity > (u32::MAX - region.offset) {
        return Err(RegionValidationError::out_of_range(
            region.offset,
            region.capacity,
        ));
    }
    Ok(())
}

/// Overrides a Region at ptr in wasm memory with data
fn set_region(memory: &wasmer::Memory, ptr: u32, data: Region) -> CommunicationResult<()> {
    let wptr = WasmPtr::<Region>::new(ptr);

    match wptr.deref(memory) {
        Some(cell) => {
            cell.set(data);
            Ok(())
        }
        None => Err(CommunicationError::deref_err(
            ptr,
            "Could not dereference this pointer to a Region",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_region_passes_for_valid_region() {
        // empty
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 0,
        };
        validate_region(&region).unwrap();

        // half full
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 250,
        };
        validate_region(&region).unwrap();

        // full
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 500,
        };
        validate_region(&region).unwrap();

        // at end of linear memory (1)
        let region = Region {
            offset: u32::MAX,
            capacity: 0,
            length: 0,
        };
        validate_region(&region).unwrap();

        // at end of linear memory (2)
        let region = Region {
            offset: 1,
            capacity: u32::MAX - 1,
            length: 0,
        };
        validate_region(&region).unwrap();
    }

    #[test]
    fn validate_region_fails_for_zero_offset() {
        let region = Region {
            offset: 0,
            capacity: 500,
            length: 250,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::ZeroOffset { .. } => {}
            e => panic!("Got unexpected error: {:?}", e),
        }
    }

    #[test]
    fn validate_region_fails_for_length_exceeding_capacity() {
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 501,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::LengthExceedsCapacity {
                length, capacity, ..
            } => {
                assert_eq!(length, 501);
                assert_eq!(capacity, 500);
            }
            e => panic!("Got unexpected error: {:?}", e),
        }
    }

    #[test]
    fn validate_region_fails_when_exceeding_address_space() {
        let region = Region {
            offset: 23,
            capacity: u32::MAX,
            length: 501,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::OutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, 23);
                assert_eq!(capacity, u32::MAX);
            }
            e => panic!("Got unexpected error: {:?}", e),
        }

        let region = Region {
            offset: u32::MAX,
            capacity: 1,
            length: 0,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::OutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, u32::MAX);
                assert_eq!(capacity, 1);
            }
            e => panic!("Got unexpected error: {:?}", e),
        }
    }
}
