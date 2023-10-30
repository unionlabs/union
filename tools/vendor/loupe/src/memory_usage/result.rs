#[cfg(test)]
use crate::assert_size_of_val_eq;
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

impl<T, E> MemoryUsage for Result<T, E>
where
    T: MemoryUsage,
    E: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + match self.as_ref() {
                Ok(value) => value.size_of_val(tracker),
                Err(value) => value.size_of_val(tracker),
            }
    }
}

#[cfg(test)]
mod test_result_types {
    use super::*;

    #[test]
    fn test_result() {
        let result: Result<i8, i16> = Err(2);
        assert_size_of_val_eq!(result, 1 /* variant */ + 3 /* padding */ + 2 /* i16 */);

        let result: Result<i8, i16> = Ok(1);
        assert_size_of_val_eq!(result, 1 /* variant */ + 3 /* padding */ + 1 /* i8 */);

        let result: Result<i32, ()> = Ok(1);
        assert_size_of_val_eq!(result, 1 /* variant */ + 7 /* padding */ + 4 /* i32 */);
    }
}
