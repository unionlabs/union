#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

impl<T> MemoryUsage for Option<T>
where
    T: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .iter()
                .map(|value| value.size_of_val(tracker))
                .sum::<usize>()
    }
}

#[cfg(test)]
mod test_option_types {
    use super::*;

    #[test]
    fn test_option() {
        let option: Option<i8> = None;
        assert_size_of_val_eq!(option, 1 /* variant */ + 1 /* padding */);

        let option: Option<i8> = Some(1);
        assert_size_of_val_eq!(option, 1 /* variant */ + 1 /* padding */ + 1 /* i8 */);

        let option: Option<i32> = None;
        assert_size_of_val_eq!(option, 1 /* variant */ + 7 /* padding */);

        let option: Option<i32> = Some(1);
        assert_size_of_val_eq!(option, 1 /* variant */ + 7 /* padding */ + 4 /* i32 */);

        let option: Option<&str> = None;
        assert_size_of_val_eq!(option, 1 /* variant */ + 15 /* padding */);

        let option: Option<&str> = Some("abc");
        assert_size_of_val_eq!(
            option,
            1 /* variant */ + 15 /* padding */ + 2 * POINTER_BYTE_SIZE + 1 * 3, /* &str */
        );
    }
}
