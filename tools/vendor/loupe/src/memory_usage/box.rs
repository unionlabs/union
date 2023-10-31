#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

impl<T> MemoryUsage for Box<T>
where
    T: MemoryUsage + ?Sized,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        let reference = self.as_ref();

        mem::size_of_val(self)
            + if tracker.track(reference as *const _ as *const ()) {
                reference.size_of_val(tracker)
            } else {
                0
            }
    }
}

#[cfg(test)]
mod test_box_types {
    use super::*;

    #[test]
    fn test_box() {
        let b: Box<i8> = Box::new(1);
        assert_size_of_val_eq!(b, POINTER_BYTE_SIZE + 1);

        let b: Box<i32> = Box::new(1);
        assert_size_of_val_eq!(b, POINTER_BYTE_SIZE + 4);

        let b: Box<&str> = Box::new("abc");
        assert_size_of_val_eq!(b, POINTER_BYTE_SIZE + 2 * POINTER_BYTE_SIZE + 1 * 3);

        let b: Box<(i8, i16)> = Box::new((1, 2));
        assert_size_of_val_eq!(
            b,
            POINTER_BYTE_SIZE + 1 /* i8 */ + 2 /* i16 */ + 1, /* padding */
        );
    }

    #[test]
    fn test_boxed_slice() {
        let b: Box<[u8]> = vec![].into_boxed_slice();
        assert_size_of_val_eq!(b, 2 * POINTER_BYTE_SIZE);

        let b: Box<[u8]> = vec![1, 2, 3].into_boxed_slice();
        assert_size_of_val_eq!(b, 2 * POINTER_BYTE_SIZE + 1 * 3);
    }
}
