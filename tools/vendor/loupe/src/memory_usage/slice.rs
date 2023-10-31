#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

impl<T> MemoryUsage for [T]
where
    T: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .iter()
                .map(|value| value.size_of_val(tracker) - mem::size_of_val(value))
                .sum::<usize>()
    }
}

impl<T> MemoryUsage for &[T]
where
    T: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + if tracker.track(*self as *const [T] as *const ()) {
                MemoryUsage::size_of_val(*self, tracker)
            } else {
                0
            }
    }
}

#[cfg(test)]
mod test_slice_types {
    use super::*;

    #[rustversion::any(stable(1.51), since(2021-02-01))]
    #[test]
    fn test_slice() {
        assert_size_of_val_eq!([1i16], 2 * 1);
        assert_size_of_val_eq!([1i16, 2], 2 * 2);
        assert_size_of_val_eq!([1i16, 2, 3], 2 * 3);
    }

    #[test]
    fn test_slice_dynamically_sized() {
        let slice: &[i16] = &[];
        assert_size_of_val_eq!(slice, 2 * POINTER_BYTE_SIZE + 2 * 0);

        let slice: &[i16] = &[1];
        assert_size_of_val_eq!(slice, 2 * POINTER_BYTE_SIZE + 2 * 1);

        let slice: &[i16] = &[1, 2];
        assert_size_of_val_eq!(slice, 2 * POINTER_BYTE_SIZE + 2 * 2);

        let slice: &[i16] = &[1, 2, 3];
        assert_size_of_val_eq!(slice, 2 * POINTER_BYTE_SIZE + 2 * 3);
    }
}
