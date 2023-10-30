use std::collections::BTreeMap;
use std::fmt;
#[cfg(feature = "iterator")]
use std::iter;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

#[cfg(feature = "iterator")]
use crate::iterator::{Order, Record};
use crate::traits::Storage;

#[derive(Default)]
pub struct MemoryStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage::default()
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        if value.is_empty() {
            panic!("TL;DR: Value must not be empty in Storage::set but in most cases you can use Storage::remove instead. Long story: Getting empty values from storage is not well supported at the moment. Some of our internal interfaces cannot differentiate between a non-existent key and an empty value. Right now, you cannot rely on the behaviour of empty values. To protect you from trouble later on, we stop here. Sorry for the inconvenience! We highly welcome you to contribute to CosmWasm, making this more solid one way or the other.");
        }

        self.data.insert(key.to_vec(), value.to_vec());
    }

    fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        let bounds = range_bounds(start, end);

        // BTreeMap.range panics if range is start > end.
        // However, this cases represent just empty range and we treat it as such.
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                return Box::new(iter::empty());
            }
            _ => {}
        }

        let iter = self.data.range(bounds);
        match order {
            Order::Ascending => Box::new(iter.map(clone_item)),
            Order::Descending => Box::new(iter.rev().map(clone_item)),
        }
    }
}

/// This debug implementation is made for inspecting storages in unit testing.
/// It is made for human readability only and the output can change at any time.
impl fmt::Debug for MemoryStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryStorage ({} entries)", self.data.len())?;
        f.write_str(" {\n")?;
        for (key, value) in &self.data {
            f.write_str("  0x")?;
            for byte in key {
                write!(f, "{:02x}", byte)?;
            }
            f.write_str(": 0x")?;
            for byte in value {
                write!(f, "{:02x}", byte)?;
            }
            f.write_str("\n")?;
        }
        f.write_str("}")?;
        Ok(())
    }
}

#[cfg(feature = "iterator")]
fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

#[cfg(feature = "iterator")]
/// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, Vec<u8>>::range.
/// This is internal as it can change any time if the map implementation is swapped out.
type BTreeMapRecordRef<'a> = (&'a Vec<u8>, &'a Vec<u8>);

#[cfg(feature = "iterator")]
fn clone_item(item_ref: BTreeMapRecordRef) -> Record {
    let (key, value) = item_ref;
    (key.clone(), value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set() {
        let mut store = MemoryStorage::new();
        assert_eq!(store.get(b"foo"), None);
        store.set(b"foo", b"bar");
        assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
        assert_eq!(store.get(b"food"), None);
    }

    #[test]
    #[should_panic(
        expected = "Getting empty values from storage is not well supported at the moment."
    )]
    fn set_panics_for_empty() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"");
    }

    #[test]
    fn delete() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar");
        store.set(b"food", b"bank");
        store.remove(b"foo");

        assert_eq!(store.get(b"foo"), None);
        assert_eq!(store.get(b"food"), Some(b"bank".to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn iterator() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar");

        // ensure we had previously set "foo" = "bar"
        assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
        assert_eq!(store.range(None, None, Order::Ascending).count(), 1);

        // setup - add some data, and delete part of it as well
        store.set(b"ant", b"hill");
        store.set(b"ze", b"bra");

        // noise that should be ignored
        store.set(b"bye", b"bye");
        store.remove(b"bye");

        // unbounded
        {
            let iter = store.range(None, None, Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ant".to_vec(), b"hill".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ze".to_vec(), b"bra".to_vec()),
                ]
            );
        }

        // unbounded (descending)
        {
            let iter = store.range(None, None, Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ze".to_vec(), b"bra".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }

        // bounded
        {
            let iter = store.range(Some(b"f"), Some(b"n"), Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![(b"foo".to_vec(), b"bar".to_vec())]);
        }

        // bounded (descending)
        {
            let iter = store.range(Some(b"air"), Some(b"loop"), Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }

        // bounded empty [a, a)
        {
            let iter = store.range(Some(b"foo"), Some(b"foo"), Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, a) (descending)
        {
            let iter = store.range(Some(b"foo"), Some(b"foo"), Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a
        {
            let iter = store.range(Some(b"z"), Some(b"a"), Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a (descending)
        {
            let iter = store.range(Some(b"z"), Some(b"a"), Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // right unbounded
        {
            let iter = store.range(Some(b"f"), None, Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ze".to_vec(), b"bra".to_vec()),
                ]
            );
        }

        // right unbounded (descending)
        {
            let iter = store.range(Some(b"f"), None, Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ze".to_vec(), b"bra".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                ]
            );
        }

        // left unbounded
        {
            let iter = store.range(None, Some(b"f"), Order::Ascending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(elements, vec![(b"ant".to_vec(), b"hill".to_vec()),]);
        }

        // left unbounded (descending)
        {
            let iter = store.range(None, Some(b"no"), Order::Descending);
            let elements: Vec<Record> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }
    }

    #[test]
    fn memory_storage_implements_debug() {
        let store = MemoryStorage::new();
        assert_eq!(
            format!("{:?}", store),
            "MemoryStorage (0 entries) {\n\
            }"
        );

        // With one element
        let mut store = MemoryStorage::new();
        store.set(&[0x00, 0xAB, 0xDD], &[0xFF, 0xD5]);
        assert_eq!(
            format!("{:?}", store),
            "MemoryStorage (1 entries) {\n\
            \x20\x200x00abdd: 0xffd5\n\
            }"
        );

        // Sorted by key
        let mut store = MemoryStorage::new();
        store.set(&[0x00, 0xAB, 0xDD], &[0xFF, 0xD5]);
        store.set(&[0x00, 0xAB, 0xEE], &[0xFF, 0xD5]);
        store.set(&[0x00, 0xAB, 0xCC], &[0xFF, 0xD5]);
        assert_eq!(
            format!("{:?}", store),
            "MemoryStorage (3 entries) {\n\
            \x20\x200x00abcc: 0xffd5\n\
            \x20\x200x00abdd: 0xffd5\n\
            \x20\x200x00abee: 0xffd5\n\
            }"
        );

        // Different lengths
        let mut store = MemoryStorage::new();
        store.set(&[0xAA], &[0x11]);
        store.set(&[0xAA, 0xBB], &[0x11, 0x22]);
        store.set(&[0xAA, 0xBB, 0xCC], &[0x11, 0x22, 0x33]);
        store.set(&[0xAA, 0xBB, 0xCC, 0xDD], &[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            format!("{:?}", store),
            "MemoryStorage (4 entries) {\n\
            \x20\x200xaa: 0x11\n\
            \x20\x200xaabb: 0x1122\n\
            \x20\x200xaabbcc: 0x112233\n\
            \x20\x200xaabbccdd: 0x11223344\n\
            }"
        );
    }
}
