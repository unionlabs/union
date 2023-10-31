#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::any::Any;
use std::mem;

impl MemoryUsage for dyn Any {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
    }
}

#[cfg(test)]
mod test_any_types {
    use super::*;

    #[test]
    fn test_boxed_any() {
        let b: Box<dyn Any> = Box::new(1i8);
        assert_size_of_val_eq!(b, 2 * POINTER_BYTE_SIZE + 1);
    }
}
