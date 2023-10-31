#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

impl MemoryUsage for &str {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self) + self.as_bytes().size_of_val(tracker)
    }
}

impl MemoryUsage for String {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        self.as_str().size_of_val(tracker)
    }
}

#[cfg(test)]
mod test_string_types {
    use super::*;

    #[test]
    fn test_str() {
        let string: &str = "";
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 0);

        let string: &str = "a";
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 1);

        let string: &str = "ab";
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 2);

        let string: &str = "abc";
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 3);

        let string: &str = "…";
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 3);
    }

    #[test]
    fn test_string() {
        let string: String = "".to_string();
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 0);

        let string: String = "a".to_string();
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 1);

        let string: String = "ab".to_string();
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 2);

        let string: String = "abc".to_string();
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 3);

        let string: String = "…".to_string();
        assert_size_of_val_eq!(string, 2 * POINTER_BYTE_SIZE + 1 * 3);
    }
}
