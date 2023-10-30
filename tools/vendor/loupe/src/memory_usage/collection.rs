#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::collections::HashMap;
use std::mem;

impl<T> MemoryUsage for Vec<T>
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
mod test_vec_types {
    use super::*;

    #[test]
    fn test_vec() {
        let empty_vec_size = mem::size_of_val(&Vec::<i8>::new());

        let mut vec: Vec<i8> = Vec::new();
        assert_size_of_val_eq!(vec, empty_vec_size + 1 * 0);

        vec.push(1);
        assert_size_of_val_eq!(vec, empty_vec_size + 1 * 1);

        vec.push(2);
        assert_size_of_val_eq!(vec, empty_vec_size + 1 * 2);
    }

    #[test]
    fn test_vec_not_unique() {
        let empty_vec_size = mem::size_of_val(&Vec::<&i32>::new());

        let mut vec: Vec<&i32> = Vec::new();
        assert_size_of_val_eq!(vec, empty_vec_size);

        let one: i32 = 1;
        vec.push(&one);
        assert_size_of_val_eq!(vec, empty_vec_size + POINTER_BYTE_SIZE + 4);

        let two: i32 = 2;
        vec.push(&two);
        assert_size_of_val_eq!(
            vec,
            empty_vec_size + POINTER_BYTE_SIZE + 4 + POINTER_BYTE_SIZE + 4
        );

        // Push a reference to an item that already exists!
        vec.push(&one);
        assert_size_of_val_eq!(
            vec,
            empty_vec_size + POINTER_BYTE_SIZE + 4 + POINTER_BYTE_SIZE + 4 + POINTER_BYTE_SIZE + 0 /* no string content */
        );
    }
}

impl<K, V> MemoryUsage for HashMap<K, V>
where
    K: MemoryUsage,
    V: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .iter()
                .map(|(key, value)| key.size_of_val(tracker) + value.size_of_val(tracker))
                .sum::<usize>()
    }
}

#[cfg(test)]
mod test_collection_types {
    use super::*;

    #[test]
    fn test_hashmap() {
        let mut hashmap: HashMap<i8, i32> = HashMap::new();
        let empty_hashmap_size = mem::size_of_val(&hashmap);
        assert_size_of_val_eq!(hashmap, empty_hashmap_size + 1 * 0 + 4 * 0);

        hashmap.insert(1, 1);
        assert_size_of_val_eq!(hashmap, empty_hashmap_size + 1 * 1 + 4 * 1);

        hashmap.insert(2, 2);
        assert_size_of_val_eq!(hashmap, empty_hashmap_size + 1 * 2 + 4 * 2);
    }

    #[test]
    fn test_hashmap_not_unique() {
        let mut hashmap: HashMap<i8, &i32> = HashMap::new();
        let empty_hashmap_size = mem::size_of_val(&hashmap);
        assert_size_of_val_eq!(
            hashmap,
            empty_hashmap_size + 1 * 0 + (POINTER_BYTE_SIZE + 4) * 0
        );

        let one: i32 = 1;
        hashmap.insert(1, &one);
        assert_size_of_val_eq!(
            hashmap,
            empty_hashmap_size + 1 * 1 + (POINTER_BYTE_SIZE + 4) * 1
        );

        let two: i32 = 2;
        hashmap.insert(2, &two);
        assert_size_of_val_eq!(
            hashmap,
            empty_hashmap_size + 1 * 2 + (POINTER_BYTE_SIZE + 4) * 2
        );

        // Push a reference to an item that already exists!
        hashmap.insert(3, &one);
        assert_size_of_val_eq!(
            hashmap,
            empty_hashmap_size + 1 * 3 + (POINTER_BYTE_SIZE + 4) * 2 + POINTER_BYTE_SIZE + 0 /* no i32 */
        );
    }
}
