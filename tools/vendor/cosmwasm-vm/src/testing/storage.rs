use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::collections::HashMap;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, Record};

#[cfg(feature = "iterator")]
use crate::BackendError;
use crate::{BackendResult, GasInfo, Storage};

#[cfg(feature = "iterator")]
const GAS_COST_LAST_ITERATION: u64 = 37;

#[cfg(feature = "iterator")]
const GAS_COST_RANGE: u64 = 11;

#[cfg(feature = "iterator")]
#[derive(Default, Debug)]
struct Iter {
    data: Vec<Record>,
    position: usize,
}

#[derive(Default, Debug)]
pub struct MockStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    #[cfg(feature = "iterator")]
    iterators: HashMap<u32, Iter>,
}

impl MockStorage {
    pub fn new() -> Self {
        MockStorage::default()
    }

    #[cfg(feature = "iterator")]
    pub fn all(&mut self, iterator_id: u32) -> BackendResult<Vec<Record>> {
        let mut out: Vec<Record> = Vec::new();
        let mut total = GasInfo::free();
        loop {
            let (result, info) = self.next(iterator_id);
            total += info;
            match result {
                Err(err) => return (Err(err), total),
                Ok(ok) => {
                    if let Some(v) = ok {
                        out.push(v);
                    } else {
                        break;
                    }
                }
            }
        }
        (Ok(out), total)
    }
}

impl Storage for MockStorage {
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>> {
        let gas_info = GasInfo::with_externally_used(key.len() as u64);
        (Ok(self.data.get(key).cloned()), gas_info)
    }

    #[cfg(feature = "iterator")]
    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32> {
        let gas_info = GasInfo::with_externally_used(GAS_COST_RANGE);
        let bounds = range_bounds(start, end);

        let values: Vec<Record> = match (bounds.start_bound(), bounds.end_bound()) {
            // BTreeMap.range panics if range is start > end.
            // However, this cases represent just empty range and we treat it as such.
            (Bound::Included(start), Bound::Excluded(end)) if start > end => Vec::new(),
            _ => match order {
                Order::Ascending => self.data.range(bounds).map(clone_item).collect(),
                Order::Descending => self.data.range(bounds).rev().map(clone_item).collect(),
            },
        };

        let last_id: u32 = self
            .iterators
            .len()
            .try_into()
            .expect("Found more iterator IDs than supported");
        let new_id = last_id + 1;
        let iter = Iter {
            data: values,
            position: 0,
        };
        self.iterators.insert(new_id, iter);

        (Ok(new_id), gas_info)
    }

    #[cfg(feature = "iterator")]
    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<Record>> {
        let iterator = match self.iterators.get_mut(&iterator_id) {
            Some(i) => i,
            None => {
                return (
                    Err(BackendError::iterator_does_not_exist(iterator_id)),
                    GasInfo::free(),
                )
            }
        };

        let (value, gas_info): (Option<Record>, GasInfo) =
            if iterator.data.len() > iterator.position {
                let item = iterator.data[iterator.position].clone();
                iterator.position += 1;
                let gas_cost = (item.0.len() + item.1.len()) as u64;
                (Some(item), GasInfo::with_cost(gas_cost))
            } else {
                (None, GasInfo::with_externally_used(GAS_COST_LAST_ITERATION))
            };

        (Ok(value), gas_info)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        let gas_info = GasInfo::with_externally_used((key.len() + value.len()) as u64);
        (Ok(()), gas_info)
    }

    fn remove(&mut self, key: &[u8]) -> BackendResult<()> {
        self.data.remove(key);
        let gas_info = GasInfo::with_externally_used(key.len() as u64);
        (Ok(()), gas_info)
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
        let mut store = MockStorage::new();
        assert_eq!(None, store.get(b"foo").0.unwrap());
        store.set(b"foo", b"bar").0.unwrap();
        assert_eq!(Some(b"bar".to_vec()), store.get(b"foo").0.unwrap());
        assert_eq!(None, store.get(b"food").0.unwrap());
    }

    #[test]
    fn delete() {
        let mut store = MockStorage::new();
        store.set(b"foo", b"bar").0.unwrap();
        store.set(b"food", b"bank").0.unwrap();
        store.remove(b"foo").0.unwrap();

        assert_eq!(None, store.get(b"foo").0.unwrap());
        assert_eq!(Some(b"bank".to_vec()), store.get(b"food").0.unwrap());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn iterator() {
        let mut store = MockStorage::new();
        store.set(b"foo", b"bar").0.expect("error setting value");

        // ensure we had previously set "foo" = "bar"
        assert_eq!(store.get(b"foo").0.unwrap(), Some(b"bar".to_vec()));
        let iter_id = store.scan(None, None, Order::Ascending).0.unwrap();
        assert_eq!(store.all(iter_id).0.unwrap().len(), 1);

        // setup - add some data, and delete part of it as well
        store.set(b"ant", b"hill").0.expect("error setting value");
        store.set(b"ze", b"bra").0.expect("error setting value");

        // noise that should be ignored
        store.set(b"bye", b"bye").0.expect("error setting value");
        store.remove(b"bye").0.expect("error removing key");

        // unbounded
        {
            let iter_id = store.scan(None, None, Order::Ascending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
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
            let iter_id = store.scan(None, None, Order::Descending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
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
            let iter_id = store
                .scan(Some(b"f"), Some(b"n"), Order::Ascending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![(b"foo".to_vec(), b"bar".to_vec())]);
        }

        // bounded (descending)
        {
            let iter_id = store
                .scan(Some(b"air"), Some(b"loop"), Order::Descending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
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
            let iter_id = store
                .scan(Some(b"foo"), Some(b"foo"), Order::Ascending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, a) (descending)
        {
            let iter_id = store
                .scan(Some(b"foo"), Some(b"foo"), Order::Descending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a
        {
            let iter_id = store
                .scan(Some(b"z"), Some(b"a"), Order::Ascending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a (descending)
        {
            let iter_id = store
                .scan(Some(b"z"), Some(b"a"), Order::Descending)
                .0
                .unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![]);
        }

        // right unbounded
        {
            let iter_id = store.scan(Some(b"f"), None, Order::Ascending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
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
            let iter_id = store.scan(Some(b"f"), None, Order::Descending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
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
            let iter_id = store.scan(None, Some(b"f"), Order::Ascending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(elements, vec![(b"ant".to_vec(), b"hill".to_vec()),]);
        }

        // left unbounded (descending)
        {
            let iter_id = store.scan(None, Some(b"no"), Order::Descending).0.unwrap();
            let elements = store.all(iter_id).0.unwrap();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }
    }
}
