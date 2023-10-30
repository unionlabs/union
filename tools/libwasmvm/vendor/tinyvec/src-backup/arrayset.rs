#![cfg(feature = "experimental_array_set")]

// This was contributed by user `dhardy`! Big thanks.

use super::{take, Array};
use core::{
  borrow::Borrow,
  fmt,
  mem::swap,
  ops::{AddAssign, SubAssign},
};

/// Error resulting from attempting to insert into a full array
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InsertError;

// TODO(when std): impl std::error::Error for InsertError {}

impl fmt::Display for InsertError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "ArraySet: insertion failed")
  }
}

/// An array-backed set
///
/// This set supports `O(n)` operations and has a fixed size, thus may fail to
/// insert items. The potential advantage is a *really* small size.
///
/// The set is backed by an array of type `A` and indexed by type `L`.
/// The item type must support `Default`.
/// Due to restrictions, `L` may be only `u8` or `u16`.
#[derive(Clone, Debug, Default)]
pub struct ArraySet<A: Array, L> {
  arr: A,
  len: L,
}

impl<A: Array + Default, L: From<u8>> ArraySet<A, L> {
  /// Constructs a new, empty, set
  #[inline]
  pub fn new() -> Self {
    ArraySet { arr: Default::default(), len: 0.into() }
  }
}

impl<A: Array, L: Copy + Into<usize>> ArraySet<A, L> {
  /// Constructs a new set from given inputs
  ///
  /// Panics if `len> arr.len()`.
  #[inline]
  pub fn from(arr: A, len: L) -> Self {
    if len.into() > A::CAPACITY {
      panic!("ArraySet::from(array, len): len > array.len()");
    }
    ArraySet { arr, len }
  }
}

impl<A: Array, L> ArraySet<A, L>
where
  L: Copy + PartialEq + From<u8> + Into<usize>,
{
  /// Returns the fixed capacity of the set
  #[inline]
  pub fn capacity(&self) -> usize {
    A::CAPACITY
  }

  /// Returns the number of elements in the set
  #[inline]
  pub fn len(&self) -> usize {
    self.len.into()
  }

  /// Returns true when the set contains no elements
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len == 0.into()
  }

  /// Removes all elements
  #[inline]
  pub fn clear(&mut self) {
    self.len = 0.into();
  }

  /// Iterate over all contents
  #[inline]
  pub fn iter(&self) -> Iter<A::Item> {
    Iter { a: self.arr.as_slice(), i: 0 }
  }
}

impl<A: Array, L> ArraySet<A, L>
where
  L: Copy + PartialOrd + AddAssign + SubAssign + From<u8> + Into<usize>,
{
  /// Check whether the set contains `elt`
  #[inline]
  pub fn contains<Q: Eq + ?Sized>(&self, elt: &Q) -> bool
  where
    A::Item: Borrow<Q>,
  {
    self.get(elt).is_some()
  }

  /// Get a reference to a contained item matching `elt`
  pub fn get<Q: Eq + ?Sized>(&self, elt: &Q) -> Option<&A::Item>
  where
    A::Item: Borrow<Q>,
  {
    let len: usize = self.len.into();
    let arr = self.arr.as_slice();
    for i in 0..len {
      if arr[i].borrow() == elt {
        return Some(&arr[i]);
      }
    }
    None
  }

  /// Remove an item matching `elt`, if any
  pub fn remove<Q: Eq + ?Sized>(&mut self, elt: &Q) -> Option<A::Item>
  where
    A::Item: Borrow<Q>,
  {
    let len: usize = self.len.into();
    let arr = self.arr.as_slice_mut();
    for i in 0..len {
      if arr[i].borrow() == elt {
        let l1 = len - 1;
        if i < l1 {
          arr.swap(i, l1);
        }
        self.len -= L::from(1);
        return Some(take(&mut arr[l1]));
      }
    }
    None
  }

  /// Remove any items for which `f(item) == false`
  pub fn retain<F>(&mut self, mut f: F)
  where
    F: FnMut(&A::Item) -> bool,
  {
    let mut len = self.len;
    let arr = self.arr.as_slice_mut();
    let mut i = 0;
    while i < len.into() {
      if !f(&arr[i]) {
        len -= L::from(1);
        if i < len.into() {
          arr.swap(i, len.into());
        }
      } else {
        i += 1;
      }
    }
    self.len = len;
  }
}

impl<A: Array, L> ArraySet<A, L>
where
  A::Item: Eq,
  L: Copy + PartialOrd + AddAssign + SubAssign + From<u8> + Into<usize>,
{
  /// Insert an item
  ///
  /// Due to the fixed size of the backing array, insertion may fail.
  #[inline]
  pub fn insert(&mut self, elt: A::Item) -> Result<bool, InsertError> {
    if self.contains(&elt) {
      return Ok(false);
    }

    let len = self.len.into();
    let arr = self.arr.as_slice_mut();
    if len >= arr.len() {
      return Err(InsertError);
    }
    arr[len] = elt;
    self.len += L::from(1);
    Ok(true)
  }

  /* Hits borrow checker
  pub fn get_or_insert(&mut self, elt: A::Item) -> Result<&A::Item, InsertError> {
      if let Some(r) = self.get(&elt) {
          return Ok(r);
      }
      self.insert(elt)?;
      let len: usize = self.len.into();
      Ok(&self.arr.as_slice()[len - 1])
  }
  */

  /// Replace an item matching `elt` with `elt`, or insert `elt`
  ///
  /// Returns the replaced item, if any. Fails when there is no matching item
  /// and the backing array is full, preventing insertion.
  pub fn replace(
    &mut self,
    mut elt: A::Item,
  ) -> Result<Option<A::Item>, InsertError> {
    let len: usize = self.len.into();
    let arr = self.arr.as_slice_mut();
    for i in 0..len {
      if arr[i] == elt {
        swap(&mut arr[i], &mut elt);
        return Ok(Some(elt));
      }
    }

    if len >= arr.len() {
      return Err(InsertError);
    }
    arr[len] = elt;
    self.len += L::from(1);
    Ok(None)
  }
}

/// Type returned by [`ArraySet::iter`]
pub struct Iter<'a, T> {
  a: &'a [T],
  i: usize,
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
  #[inline]
  fn len(&self) -> usize {
    self.a.len() - self.i
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.i < self.a.len() {
      let i = self.i;
      self.i += 1;
      Some(&self.a[i])
    } else {
      None
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.len();
    (len, Some(len))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use core::mem::size_of;

  #[test]
  fn test_size() {
    assert_eq!(size_of::<ArraySet<[i8; 7], u8>>(), 8);
  }

  #[test]
  fn test() {
    let mut set: ArraySet<[i8; 7], u8> = ArraySet::new();
    assert_eq!(set.capacity(), 7);

    assert_eq!(set.insert(1), Ok(true));
    assert_eq!(set.insert(5), Ok(true));
    assert_eq!(set.insert(6), Ok(true));
    assert_eq!(set.len(), 3);

    assert_eq!(set.insert(5), Ok(false));
    assert_eq!(set.len(), 3);

    assert_eq!(set.replace(1), Ok(Some(1)));
    assert_eq!(set.replace(2), Ok(None));
    assert_eq!(set.len(), 4);

    assert_eq!(set.insert(3), Ok(true));
    assert_eq!(set.insert(4), Ok(true));
    assert_eq!(set.insert(7), Ok(true));
    assert_eq!(set.insert(8), Err(InsertError));
    assert_eq!(set.len(), 7);

    assert_eq!(set.replace(9), Err(InsertError));

    assert_eq!(set.remove(&3), Some(3));
    assert_eq!(set.len(), 6);

    set.retain(|x| *x == 3 || *x == 6);
    assert_eq!(set.len(), 1);
    assert!(!set.contains(&3));
    assert!(set.contains(&6));
  }
}
