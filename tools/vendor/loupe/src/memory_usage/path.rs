#[cfg(test)]
use crate::assert_size_of_val_eq;
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;
use std::path::PathBuf;

impl MemoryUsage for PathBuf {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.capacity()
    }
}

#[cfg(test)]
mod test_path_types {
    use super::*;

    #[test]
    fn test_pathbuf() {
        let mut path = PathBuf::new();
        let empty_path_size = mem::size_of_val(&path);

        path.push("foo");
        assert_size_of_val_eq!(path, empty_path_size + 8);

        path.push("foobar");
        assert_size_of_val_eq!(path, empty_path_size + 16);
    }
}
