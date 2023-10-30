#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::alloc::Layout;
use std::mem;

impl MemoryUsage for Layout {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
    }
}

#[cfg(test)]
mod test_alloc_types {
    use super::*;

    #[test]
    fn test_layout() {
        let layout = Layout::new::<i8>();
        assert_size_of_val_eq!(layout, 2 * POINTER_BYTE_SIZE);
    }
}
