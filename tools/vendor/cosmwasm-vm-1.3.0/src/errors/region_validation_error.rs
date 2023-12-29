use std::fmt::Debug;
use thiserror::Error;

/// An error validating a Region
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RegionValidationError {
    #[error(
        "Region length exceeds capacity. Length {}, capacity {}",
        length,
        capacity
    )]
    LengthExceedsCapacity { length: u32, capacity: u32 },
    #[error(
        "Region exceeds address space. Offset {}, capacity {}",
        offset,
        capacity
    )]
    OutOfRange { offset: u32, capacity: u32 },
    #[error("Got a zero Wasm address in the offset")]
    ZeroOffset {},
}

impl RegionValidationError {
    pub(crate) fn length_exceeds_capacity(length: u32, capacity: u32) -> Self {
        RegionValidationError::LengthExceedsCapacity { length, capacity }
    }

    pub(crate) fn out_of_range(offset: u32, capacity: u32) -> Self {
        RegionValidationError::OutOfRange { offset, capacity }
    }

    pub(crate) fn zero_offset() -> Self {
        RegionValidationError::ZeroOffset {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors

    #[test]
    fn length_exceeds_capacity_works() {
        let error = RegionValidationError::length_exceeds_capacity(50, 20);
        match error {
            RegionValidationError::LengthExceedsCapacity {
                length, capacity, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(capacity, 20);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn out_of_range_works() {
        let error = RegionValidationError::out_of_range(u32::MAX, 1);
        match error {
            RegionValidationError::OutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, u32::MAX);
                assert_eq!(capacity, 1);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn zero_offset() {
        let error = RegionValidationError::zero_offset();
        match error {
            RegionValidationError::ZeroOffset { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
