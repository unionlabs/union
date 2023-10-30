#[cfg(feature = "vec_map")]
pub use vec_map::{Values, VecMap};

#[cfg(not(feature = "vec_map"))]
pub use self::vec_map::{Values, VecMap};

#[cfg(not(feature = "vec_map"))]
mod vec_map {
    use std::collections::btree_map;
    use std::collections::BTreeMap;
    use std::fmt::{self, Debug, Formatter};

    #[derive(Clone, Default, Debug)]
    pub struct VecMap<V> {
        inner: BTreeMap<usize, V>,
    }

    impl<V> VecMap<V> {
        pub fn new() -> Self {
            VecMap {
                inner: Default::default(),
            }
        }

        pub fn len(&self) -> usize {
            self.inner.len()
        }

        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }

        pub fn insert(&mut self, key: usize, value: V) -> Option<V> {
            self.inner.insert(key, value)
        }

        pub fn values(&self) -> Values<V> {
            self.inner.values()
        }

        pub fn iter(&self) -> Iter<V> {
            Iter {
                inner: self.inner.iter(),
            }
        }

        pub fn contains_key(&self, key: usize) -> bool {
            self.inner.contains_key(&key)
        }

        pub fn entry(&mut self, key: usize) -> Entry<V> {
            self.inner.entry(key)
        }

        pub fn get(&self, key: usize) -> Option<&V> {
            self.inner.get(&key)
        }
    }

    pub type Values<'a, V> = btree_map::Values<'a, usize, V>;

    pub type Entry<'a, V> = btree_map::Entry<'a, usize, V>;

    #[derive(Clone)]
    pub struct Iter<'a, V: 'a> {
        inner: btree_map::Iter<'a, usize, V>,
    }

    impl<'a, V: 'a + Debug> Debug for Iter<'a, V> {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            f.debug_list().entries(self.inner.clone()).finish()
        }
    }

    impl<'a, V: 'a> Iterator for Iter<'a, V> {
        type Item = (usize, &'a V);

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next().map(|(k, v)| (*k, v))
        }
    }

    impl<'a, V: 'a> DoubleEndedIterator for Iter<'a, V> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.inner.next_back().map(|(k, v)| (*k, v))
        }
    }
}
