use std::any::type_name;

use crate::errors::{VmError, VmResult};

/// Safely converts input of type T to u32.
/// Errors with a cosmwasm_vm::errors::VmError::ConversionErr if conversion cannot be done.
pub fn to_u32<T: TryInto<u32> + ToString + Copy>(input: T) -> VmResult<u32> {
    input.try_into().map_err(|_| {
        VmError::conversion_err(type_name::<T>(), type_name::<u32>(), input.to_string())
    })
}

/// Safely converts input of type &T to u32.
/// Errors with a cosmwasm_vm::errors::VmError::ConversionErr if conversion cannot be done.
pub fn ref_to_u32<T: TryInto<u32> + ToString + Clone>(input: &T) -> VmResult<u32> {
    input.clone().try_into().map_err(|_| {
        VmError::conversion_err(type_name::<T>(), type_name::<u32>(), input.to_string())
    })
}

/// Safely converts input of type T to i32.
/// Errors with a cosmwasm_vm::errors::VmError::ConversionErr if conversion cannot be done.
///
/// Used in tests and in iterator, but not with default build
#[allow(dead_code)]
pub fn to_i32<T: TryInto<i32> + ToString + Copy>(input: T) -> VmResult<i32> {
    input.try_into().map_err(|_| {
        VmError::conversion_err(type_name::<T>(), type_name::<i32>(), input.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::VmError;

    #[test]
    fn to_u32_works_for_usize() {
        assert_eq!(to_u32(0usize).unwrap(), 0);
        assert_eq!(to_u32(1usize).unwrap(), 1);
        assert_eq!(to_u32(2147483647usize).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648usize).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295usize).unwrap(), 4294967295);

        match to_u32(4294967296usize) {
            Err(VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "usize");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_u32_works_for_u64() {
        assert_eq!(to_u32(0u64).unwrap(), 0);
        assert_eq!(to_u32(1u64).unwrap(), 1);
        assert_eq!(to_u32(2147483647u64).unwrap(), 2147483647);
        assert_eq!(to_u32(2147483648u64).unwrap(), 2147483648);
        assert_eq!(to_u32(4294967295u64).unwrap(), 4294967295);

        match to_u32(4294967296u64) {
            Err(VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "u64");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_u32_works_for_i32() {
        assert_eq!(to_u32(0i32).unwrap(), 0);
        assert_eq!(to_u32(1i32).unwrap(), 1);
        assert_eq!(to_u32(2147483647i32).unwrap(), 2147483647);

        match to_u32(-1i32) {
            Err(VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-1");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn ref_to_u32_works() {
        // i32
        assert_eq!(ref_to_u32(&0i32).unwrap(), 0);
        assert_eq!(ref_to_u32(&1i32).unwrap(), 1);
        assert_eq!(ref_to_u32(&2147483647i32).unwrap(), 2147483647);
        match ref_to_u32(&-1i32).unwrap_err() {
            VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            } => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-1");
            }
            err => panic!("Unexpected error: {:?}", err),
        };

        // usize
        assert_eq!(ref_to_u32(&0usize).unwrap(), 0);
        assert_eq!(ref_to_u32(&1usize).unwrap(), 1);
        assert_eq!(ref_to_u32(&2147483647usize).unwrap(), 2147483647);
        assert_eq!(ref_to_u32(&2147483648usize).unwrap(), 2147483648);
        assert_eq!(ref_to_u32(&4294967295usize).unwrap(), 4294967295);
        match ref_to_u32(&4294967296usize).unwrap_err() {
            VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            } => {
                assert_eq!(from_type, "usize");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "4294967296");
            }
            err => panic!("Unexpected error: {:?}", err),
        };
    }

    #[test]
    fn to_i32_works_for_usize() {
        assert_eq!(to_i32(0usize).unwrap(), 0);
        assert_eq!(to_i32(1usize).unwrap(), 1);
        assert_eq!(to_i32(2147483647usize).unwrap(), 2147483647);

        match to_i32(2147483648usize) {
            Err(VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "usize");
                assert_eq!(to_type, "i32");
                assert_eq!(input, "2147483648");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }

    #[test]
    fn to_i32_works_for_i64() {
        assert_eq!(to_i32(0i64).unwrap(), 0);
        assert_eq!(to_i32(1i64).unwrap(), 1);
        assert_eq!(to_i32(2147483647i64).unwrap(), 2147483647);

        assert_eq!(to_i32(-1i64).unwrap(), -1);
        assert_eq!(to_i32(-2147483647i64).unwrap(), -2147483647);
        assert_eq!(to_i32(-2147483648i64).unwrap(), -2147483648);

        match to_i32(-2147483649i64) {
            Err(VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            }) => {
                assert_eq!(from_type, "i64");
                assert_eq!(to_type, "i32");
                assert_eq!(input, "-2147483649");
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };
    }
}
