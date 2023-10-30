use crate::io;
use alloc::vec::Vec;

use super::{Deserialize, Error, Serialize, VarUint32};

use alloc::vec;
use core::{
	cmp::min,
	iter::{FromIterator, IntoIterator},
	mem, slice,
};

/// A map from non-contiguous `u32` keys to values of type `T`, which is
/// serialized and deserialized ascending order of the keys. Normally used for
/// relative dense maps with occasional "holes", and stored as an array.
///
/// **SECURITY WARNING:** This code is currently subject to a denial of service
/// attack if you create a map containing the key `u32::MAX`, which should never
/// happen in normal data. It would be pretty easy to provide a safe
/// deserializing mechanism which addressed this problem.
#[derive(Debug, Default)]
pub struct IndexMap<T> {
	/// The number of non-`None` entries in this map.
	len: usize,

	/// A vector of entries. Missing entries are represented as `None`.
	entries: Vec<Option<T>>,
}

impl<T> IndexMap<T> {
	/// Create an empty `IndexMap`, preallocating enough space to store
	/// `capacity` entries without needing to reallocate the underlying memory.
	pub fn with_capacity(capacity: usize) -> IndexMap<T> {
		IndexMap { len: 0, entries: Vec::with_capacity(capacity) }
	}

	/// Clear the map.
	pub fn clear(&mut self) {
		self.entries.clear();
		self.len = 0;
	}

	/// Return the name for the specified index, if it exists.
	pub fn get(&self, idx: u32) -> Option<&T> {
		match self.entries.get(idx as usize) {
			Some(&Some(ref value)) => Some(value),
			Some(&None) | None => None,
		}
	}

	/// Does the map contain an entry for the specified index?
	pub fn contains_key(&self, idx: u32) -> bool {
		match self.entries.get(idx as usize) {
			Some(&Some(_)) => true,
			Some(&None) | None => false,
		}
	}

	/// Insert a name into our map, returning the existing value if present.
	///
	/// Note: This API is designed for reasonably dense indices based on valid
	/// data. Inserting a huge `idx` will use up a lot of RAM, and this function
	/// will not try to protect you against that.
	pub fn insert(&mut self, idx: u32, value: T) -> Option<T> {
		let idx = idx as usize;
		let result = if idx >= self.entries.len() {
			// We need to grow the array, and add the new element at the end.
			for _ in 0..(idx - self.entries.len()) {
				// We can't use `extend(repeat(None)).take(n)`, because that
				// would require `T` to implement `Clone`.
				self.entries.push(None);
			}
			self.entries.push(Some(value));
			debug_assert_eq!(idx + 1, self.entries.len());
			self.len += 1;
			None
		} else {
			// We're either replacing an existing element, or filling in a
			// missing one.
			let existing = self.entries[idx].take();
			if existing.is_none() {
				self.len += 1;
			}
			self.entries[idx] = Some(value);
			existing
		};
		if mem::size_of::<usize>() > 4 {
			debug_assert!(self.entries.len() <= (u32::max_value() as usize) + 1);
		}
		#[cfg(slow_assertions)]
		debug_assert_eq!(self.len, self.slow_len());
		result
	}

	/// Remove an item if present and return it.
	pub fn remove(&mut self, idx: u32) -> Option<T> {
		let result = match self.entries.get_mut(idx as usize) {
			Some(value @ &mut Some(_)) => {
				self.len -= 1;
				value.take()
			},
			Some(&mut None) | None => None,
		};
		#[cfg(slow_assertions)]
		debug_assert_eq!(self.len, self.slow_len());
		result
	}

	/// The number of items in this map.
	pub fn len(&self) -> usize {
		#[cfg(slow_assertions)]
		debug_assert_eq!(self.len, self.slow_len());
		self.len
	}

	/// Is this map empty?
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	/// This function is only compiled when `--cfg slow_assertions` is enabled.
	/// It computes the `len` value using a slow algorithm.
	///
	/// WARNING: This turns a bunch of O(n) operations into O(n^2) operations.
	/// We may want to remove it once the code is tested, or to put it behind
	/// a feature flag named `slow_debug_checks`, or something like that.
	#[cfg(slow_assertions)]
	fn slow_len(&self) -> usize {
		self.entries.iter().filter(|entry| entry.is_some()).count()
	}

	/// Create a non-consuming iterator over this `IndexMap`'s keys and values.
	pub fn iter(&self) -> Iter<T> {
		// Note that this does the right thing because we use `&self`.
		self.into_iter()
	}

	/// Custom deserialization routine.
	///
	/// We will allocate an underlying array no larger than `max_entry_space` to
	/// hold the data, so the maximum index must be less than `max_entry_space`.
	/// This prevents mallicious *.wasm files from having a single entry with
	/// the index `u32::MAX`, which would consume far too much memory.
	///
	/// The `deserialize_value` function will be passed the index of the value
	/// being deserialized, and must deserialize the value.
	pub fn deserialize_with<R, F>(
		max_entry_space: usize,
		deserialize_value: &F,
		rdr: &mut R,
	) -> Result<IndexMap<T>, Error>
	where
		R: io::Read,
		F: Fn(u32, &mut R) -> Result<T, Error>,
	{
		let len: u32 = VarUint32::deserialize(rdr)?.into();
		let mut map = IndexMap::with_capacity(len as usize);
		let mut prev_idx = None;
		for _ in 0..len {
			let idx: u32 = VarUint32::deserialize(rdr)?.into();
			if idx as usize >= max_entry_space {
				return Err(Error::Other("index is larger than expected"))
			}
			match prev_idx {
				Some(prev) if prev >= idx => {
					// Supposedly these names must be "sorted by index", so
					// let's try enforcing that and seeing what happens.
					return Err(Error::Other("indices are out of order"))
				},
				_ => {
					prev_idx = Some(idx);
				},
			}
			let val = deserialize_value(idx, rdr)?;
			map.insert(idx, val);
		}
		Ok(map)
	}
}

impl<T: Clone> Clone for IndexMap<T> {
	fn clone(&self) -> IndexMap<T> {
		IndexMap { len: self.len, entries: self.entries.clone() }
	}
}

impl<T: PartialEq> PartialEq<IndexMap<T>> for IndexMap<T> {
	fn eq(&self, other: &IndexMap<T>) -> bool {
		if self.len() != other.len() {
			// If the number of non-`None` entries is different, we can't match.
			false
		} else {
			// This is tricky, because one `Vec` might have a bunch of empty
			// entries at the end which we want to ignore.
			let smallest_len = min(self.entries.len(), other.entries.len());
			self.entries[0..smallest_len].eq(&other.entries[0..smallest_len])
		}
	}
}

impl<T: Eq> Eq for IndexMap<T> {}

impl<T> FromIterator<(u32, T)> for IndexMap<T> {
	/// Create an `IndexMap` from an iterator.
	///
	/// Note: This API is designed for reasonably dense indices based on valid
	/// data. Inserting a huge `idx` will use up a lot of RAM, and this function
	/// will not try to protect you against that.
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = (u32, T)>,
	{
		let iter = iter.into_iter();
		let (lower, upper_opt) = iter.size_hint();
		let mut map = IndexMap::with_capacity(upper_opt.unwrap_or(lower));
		for (idx, value) in iter {
			map.insert(idx, value);
		}
		map
	}
}

/// An iterator over an `IndexMap` which takes ownership of it.
pub struct IntoIter<T> {
	next_idx: u32,
	remaining_len: usize,
	iter: vec::IntoIter<Option<T>>,
}

impl<T> Iterator for IntoIter<T> {
	type Item = (u32, T);

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.remaining_len, Some(self.remaining_len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		// Bail early if we know there are no more items. This also keeps us
		// from repeatedly calling `self.iter.next()` once it has been
		// exhausted, which is not guaranteed to keep returning `None`.
		if self.remaining_len == 0 {
			return None
		}
		for value_opt in &mut self.iter {
			let idx = self.next_idx;
			self.next_idx += 1;
			if let Some(value) = value_opt {
				self.remaining_len -= 1;
				return Some((idx, value))
			}
		}
		debug_assert_eq!(self.remaining_len, 0);
		None
	}
}

impl<T> IntoIterator for IndexMap<T> {
	type Item = (u32, T);
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> IntoIter<T> {
		IntoIter { next_idx: 0, remaining_len: self.len, iter: self.entries.into_iter() }
	}
}

/// An iterator over a borrowed `IndexMap`.
pub struct Iter<'a, T: 'static> {
	next_idx: u32,
	remaining_len: usize,
	iter: slice::Iter<'a, Option<T>>,
}

impl<'a, T: 'static> Iterator for Iter<'a, T> {
	type Item = (u32, &'a T);

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.remaining_len, Some(self.remaining_len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		// Bail early if we know there are no more items. This also keeps us
		// from repeatedly calling `self.iter.next()` once it has been
		// exhausted, which is not guaranteed to keep returning `None`.
		if self.remaining_len == 0 {
			return None
		}
		for value_opt in &mut self.iter {
			let idx = self.next_idx;
			self.next_idx += 1;
			if let Some(ref value) = *value_opt {
				self.remaining_len -= 1;
				return Some((idx, value))
			}
		}
		debug_assert_eq!(self.remaining_len, 0);
		None
	}
}

impl<'a, T: 'static> IntoIterator for &'a IndexMap<T> {
	type Item = (u32, &'a T);
	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Iter<'a, T> {
		Iter { next_idx: 0, remaining_len: self.len, iter: self.entries.iter() }
	}
}

impl<T> Serialize for IndexMap<T>
where
	T: Serialize,
	Error: From<<T as Serialize>::Error>,
{
	type Error = Error;

	fn serialize<W: io::Write>(self, wtr: &mut W) -> Result<(), Self::Error> {
		VarUint32::from(self.len()).serialize(wtr)?;
		for (idx, value) in self {
			VarUint32::from(idx).serialize(wtr)?;
			value.serialize(wtr)?;
		}
		Ok(())
	}
}

impl<T: Deserialize> IndexMap<T>
where
	T: Deserialize,
	Error: From<<T as Deserialize>::Error>,
{
	/// Deserialize a map containing simple values that support `Deserialize`.
	/// We will allocate an underlying array no larger than `max_entry_space` to
	/// hold the data, so the maximum index must be less than `max_entry_space`.
	pub fn deserialize<R: io::Read>(max_entry_space: usize, rdr: &mut R) -> Result<Self, Error> {
		let deserialize_value: fn(u32, &mut R) -> Result<T, Error> =
			|_idx, rdr| T::deserialize(rdr).map_err(Error::from);
		Self::deserialize_with(max_entry_space, &deserialize_value, rdr)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::io;

	#[test]
	fn default_is_empty_no_matter_how_we_look_at_it() {
		let map = IndexMap::<String>::default();
		assert_eq!(map.len(), 0);
		assert!(map.is_empty());
		assert_eq!(map.iter().count(), 0);
		assert_eq!(map.into_iter().count(), 0);
	}

	#[test]
	fn with_capacity_creates_empty_map() {
		let map = IndexMap::<String>::with_capacity(10);
		assert!(map.is_empty());
	}

	#[test]
	fn clear_removes_all_values() {
		let mut map = IndexMap::<String>::default();
		map.insert(0, "sample value".to_string());
		assert_eq!(map.len(), 1);
		map.clear();
		assert_eq!(map.len(), 0);
	}

	#[test]
	fn get_returns_elements_that_are_there_but_nothing_else() {
		let mut map = IndexMap::<String>::default();
		map.insert(1, "sample value".to_string());
		assert_eq!(map.len(), 1);
		assert_eq!(map.get(0), None);
		assert_eq!(map.get(1), Some(&"sample value".to_string()));
		assert_eq!(map.get(2), None);
	}

	#[test]
	fn contains_key_returns_true_when_a_key_is_present() {
		let mut map = IndexMap::<String>::default();
		map.insert(1, "sample value".to_string());
		assert!(!map.contains_key(0));
		assert!(map.contains_key(1));
		assert!(!map.contains_key(2));
	}

	#[test]
	fn insert_behaves_like_other_maps() {
		let mut map = IndexMap::<String>::default();

		// Insert a key which requires extending our storage.
		assert_eq!(map.insert(1, "val 1".to_string()), None);
		assert_eq!(map.len(), 1);
		assert!(map.contains_key(1));

		// Insert a key which requires filling in a hole.
		assert_eq!(map.insert(0, "val 0".to_string()), None);
		assert_eq!(map.len(), 2);
		assert!(map.contains_key(0));

		// Insert a key which replaces an existing key.
		assert_eq!(map.insert(1, "val 1.1".to_string()), Some("val 1".to_string()));
		assert_eq!(map.len(), 2);
		assert!(map.contains_key(1));
		assert_eq!(map.get(1), Some(&"val 1.1".to_string()));
	}

	#[test]
	fn remove_behaves_like_other_maps() {
		let mut map = IndexMap::<String>::default();
		assert_eq!(map.insert(1, "val 1".to_string()), None);

		// Remove an out-of-bounds element.
		assert_eq!(map.remove(2), None);
		assert_eq!(map.len(), 1);

		// Remove an in-bounds but missing element.
		assert_eq!(map.remove(0), None);
		assert_eq!(map.len(), 1);

		// Remove an existing element.
		assert_eq!(map.remove(1), Some("val 1".to_string()));
		assert_eq!(map.len(), 0);
	}

	#[test]
	fn partial_eq_works_as_expected_in_simple_cases() {
		let mut map1 = IndexMap::<String>::default();
		let mut map2 = IndexMap::<String>::default();
		assert_eq!(map1, map2);

		map1.insert(1, "a".to_string());
		map2.insert(1, "a".to_string());
		assert_eq!(map1, map2);

		map1.insert(0, "b".to_string());
		assert_ne!(map1, map2);
		map1.remove(0);
		assert_eq!(map1, map2);

		map1.insert(1, "not a".to_string());
		assert_ne!(map1, map2);
	}

	#[test]
	fn partial_eq_is_smart_about_none_values_at_the_end() {
		let mut map1 = IndexMap::<String>::default();
		let mut map2 = IndexMap::<String>::default();

		map1.insert(1, "a".to_string());
		map2.insert(1, "a".to_string());

		// Both maps have the same (idx, value) pairs, but map2 has extra space.
		map2.insert(10, "b".to_string());
		map2.remove(10);
		assert_eq!(map1, map2);

		// Both maps have the same (idx, value) pairs, but map1 has extra space.
		map1.insert(100, "b".to_string());
		map1.remove(100);
		assert_eq!(map1, map2);

		// Let's be paranoid.
		map2.insert(1, "b".to_string());
		assert_ne!(map1, map2);
	}

	#[test]
	fn from_iterator_builds_a_map() {
		let data = &[
			// We support out-of-order values here!
			(3, "val 3"),
			(2, "val 2"),
			(5, "val 5"),
		];
		let iter = data.iter().map(|&(idx, val)| (idx, val.to_string()));
		let map = iter.collect::<IndexMap<_>>();
		assert_eq!(map.len(), 3);
		assert_eq!(map.get(2), Some(&"val 2".to_string()));
		assert_eq!(map.get(3), Some(&"val 3".to_string()));
		assert_eq!(map.get(5), Some(&"val 5".to_string()));
	}

	#[test]
	fn iterators_are_well_behaved() {
		// Create a map with reasonably complex internal structure, making
		// sure that we have both internal missing elements, and a bunch of
		// missing elements at the end.
		let data = &[(3, "val 3"), (2, "val 2"), (5, "val 5")];
		let src_iter = data.iter().map(|&(idx, val)| (idx, val.to_string()));
		let mut map = src_iter.collect::<IndexMap<_>>();
		map.remove(5);

		// Make sure `size_hint` and `next` behave as we expect at each step.
		{
			let mut iter1 = map.iter();
			assert_eq!(iter1.size_hint(), (2, Some(2)));
			assert_eq!(iter1.next(), Some((2, &"val 2".to_string())));
			assert_eq!(iter1.size_hint(), (1, Some(1)));
			assert_eq!(iter1.next(), Some((3, &"val 3".to_string())));
			assert_eq!(iter1.size_hint(), (0, Some(0)));
			assert_eq!(iter1.next(), None);
			assert_eq!(iter1.size_hint(), (0, Some(0)));
			assert_eq!(iter1.next(), None);
			assert_eq!(iter1.size_hint(), (0, Some(0)));
		}

		// Now do the same for a consuming iterator.
		let mut iter2 = map.into_iter();
		assert_eq!(iter2.size_hint(), (2, Some(2)));
		assert_eq!(iter2.next(), Some((2, "val 2".to_string())));
		assert_eq!(iter2.size_hint(), (1, Some(1)));
		assert_eq!(iter2.next(), Some((3, "val 3".to_string())));
		assert_eq!(iter2.size_hint(), (0, Some(0)));
		assert_eq!(iter2.next(), None);
		assert_eq!(iter2.size_hint(), (0, Some(0)));
		assert_eq!(iter2.next(), None);
		assert_eq!(iter2.size_hint(), (0, Some(0)));
	}

	#[test]
	fn serialize_and_deserialize() {
		let mut map = IndexMap::<String>::default();
		map.insert(1, "val 1".to_string());

		let mut output = vec![];
		map.clone().serialize(&mut output).expect("serialize failed");

		let mut input = io::Cursor::new(&output);
		let deserialized = IndexMap::deserialize(2, &mut input).expect("deserialize failed");

		assert_eq!(deserialized, map);
	}

	#[test]
	fn deserialize_requires_elements_to_be_in_order() {
		// Build a in-order example by hand.
		let mut valid = vec![];
		VarUint32::from(2u32).serialize(&mut valid).unwrap();
		VarUint32::from(0u32).serialize(&mut valid).unwrap();
		"val 0".to_string().serialize(&mut valid).unwrap();
		VarUint32::from(1u32).serialize(&mut valid).unwrap();
		"val 1".to_string().serialize(&mut valid).unwrap();
		let map = IndexMap::<String>::deserialize(2, &mut io::Cursor::new(valid))
			.expect("unexpected error deserializing");
		assert_eq!(map.len(), 2);

		// Build an out-of-order example by hand.
		let mut invalid = vec![];
		VarUint32::from(2u32).serialize(&mut invalid).unwrap();
		VarUint32::from(1u32).serialize(&mut invalid).unwrap();
		"val 1".to_string().serialize(&mut invalid).unwrap();
		VarUint32::from(0u32).serialize(&mut invalid).unwrap();
		"val 0".to_string().serialize(&mut invalid).unwrap();
		let res = IndexMap::<String>::deserialize(2, &mut io::Cursor::new(invalid));
		assert!(res.is_err());
	}

	#[test]
	fn deserialize_enforces_max_idx() {
		// Build an example with an out-of-bounds index by hand.
		let mut invalid = vec![];
		VarUint32::from(1u32).serialize(&mut invalid).unwrap();
		VarUint32::from(5u32).serialize(&mut invalid).unwrap();
		"val 5".to_string().serialize(&mut invalid).unwrap();
		let res = IndexMap::<String>::deserialize(1, &mut io::Cursor::new(invalid));
		assert!(res.is_err());
	}
}
