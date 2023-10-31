use crate::{MemoryUsage, MemoryUsageTracker};
use std::marker::PhantomData;

impl<T> MemoryUsage for PhantomData<T> {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        0
    }
}
