#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use indexmap::IndexMap;
use std::mem;

impl<K, V> MemoryUsage for IndexMap<K, V>
where
    V: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .into_iter()
                .map(|(_key, value)| value.size_of_val(tracker))
                .sum::<usize>()
    }
}

#[cfg(test)]
mod test_indexmap_types {
    use super::*;

    #[test]
    fn test_indexmap() {
        let mut map = IndexMap::with_capacity(1);
        let empty_map_size = mem::size_of_val(&map);

        let one = 1i8;
        map.insert("a", &one);
        assert_size_of_val_eq!(map, empty_map_size + (POINTER_BYTE_SIZE + 1) * 1);

        map.insert("b", &2i8);
        map.insert("c", &3i8);
        assert_size_of_val_eq!(map, empty_map_size + (POINTER_BYTE_SIZE + 1) * 3);

        map.insert("d", &one);
        assert_size_of_val_eq!(
            map,
            empty_map_size + (POINTER_BYTE_SIZE + 1) * 3 + POINTER_BYTE_SIZE + 0
        );
    }
}
