use super::*;
use std::string::String;

#[test]
fn it_works() {
    let mut set = IndexSet::new();
    assert_eq!(set.is_empty(), true);
    set.insert(1);
    set.insert(1);
    assert_eq!(set.len(), 1);
    assert!(set.get(&1).is_some());
    assert_eq!(set.is_empty(), false);
}

#[test]
fn new() {
    let set = IndexSet::<String>::new();
    println!("{:?}", set);
    assert_eq!(set.capacity(), 0);
    assert_eq!(set.len(), 0);
    assert_eq!(set.is_empty(), true);
}

#[test]
fn insert() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5];
    let not_present = [1, 3, 6, 9, 10];
    let mut set = IndexSet::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(set.len(), i);
        set.insert(elt);
        assert_eq!(set.len(), i + 1);
        assert_eq!(set.get(&elt), Some(&elt));
    }
    println!("{:?}", set);

    for &elt in &not_present {
        assert!(set.get(&elt).is_none());
    }
}

#[test]
fn insert_full() {
    let insert = vec![9, 2, 7, 1, 4, 6, 13];
    let present = vec![1, 6, 2];
    let mut set = IndexSet::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(set.len(), i);
        let (index, success) = set.insert_full(elt);
        assert!(success);
        assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
        assert_eq!(set.len(), i + 1);
    }

    let len = set.len();
    for &elt in &present {
        let (index, success) = set.insert_full(elt);
        assert!(!success);
        assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
        assert_eq!(set.len(), len);
    }
}

#[test]
fn insert_2() {
    let mut set = IndexSet::with_capacity(16);

    let mut values = vec![];
    values.extend(0..16);
    values.extend(if cfg!(miri) { 32..64 } else { 128..267 });

    for &i in &values {
        let old_set = set.clone();
        set.insert(i);
        for value in old_set.iter() {
            if set.get(value).is_none() {
                println!("old_set: {:?}", old_set);
                println!("set: {:?}", set);
                panic!("did not find {} in set", value);
            }
        }
    }

    for &i in &values {
        assert!(set.get(&i).is_some(), "did not find {}", i);
    }
}

#[test]
fn insert_dup() {
    let mut elements = vec![0, 2, 4, 6, 8];
    let mut set: IndexSet<u8> = elements.drain(..).collect();
    {
        let (i, v) = set.get_full(&0).unwrap();
        assert_eq!(set.len(), 5);
        assert_eq!(i, 0);
        assert_eq!(*v, 0);
    }
    {
        let inserted = set.insert(0);
        let (i, v) = set.get_full(&0).unwrap();
        assert_eq!(set.len(), 5);
        assert_eq!(inserted, false);
        assert_eq!(i, 0);
        assert_eq!(*v, 0);
    }
}

#[test]
fn insert_order() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut set = IndexSet::new();

    for &elt in &insert {
        set.insert(elt);
    }

    assert_eq!(set.iter().count(), set.len());
    assert_eq!(set.iter().count(), insert.len());
    for (a, b) in insert.iter().zip(set.iter()) {
        assert_eq!(a, b);
    }
    for (i, v) in (0..insert.len()).zip(set.iter()) {
        assert_eq!(set.get_index(i).unwrap(), v);
    }
}

#[test]
fn replace() {
    let replace = [0, 4, 2, 12, 8, 7, 11, 5];
    let not_present = [1, 3, 6, 9, 10];
    let mut set = IndexSet::with_capacity(replace.len());

    for (i, &elt) in replace.iter().enumerate() {
        assert_eq!(set.len(), i);
        set.replace(elt);
        assert_eq!(set.len(), i + 1);
        assert_eq!(set.get(&elt), Some(&elt));
    }
    println!("{:?}", set);

    for &elt in &not_present {
        assert!(set.get(&elt).is_none());
    }
}

#[test]
fn replace_full() {
    let replace = vec![9, 2, 7, 1, 4, 6, 13];
    let present = vec![1, 6, 2];
    let mut set = IndexSet::with_capacity(replace.len());

    for (i, &elt) in replace.iter().enumerate() {
        assert_eq!(set.len(), i);
        let (index, replaced) = set.replace_full(elt);
        assert!(replaced.is_none());
        assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
        assert_eq!(set.len(), i + 1);
    }

    let len = set.len();
    for &elt in &present {
        let (index, replaced) = set.replace_full(elt);
        assert_eq!(Some(elt), replaced);
        assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
        assert_eq!(set.len(), len);
    }
}

#[test]
fn replace_2() {
    let mut set = IndexSet::with_capacity(16);

    let mut values = vec![];
    values.extend(0..16);
    values.extend(if cfg!(miri) { 32..64 } else { 128..267 });

    for &i in &values {
        let old_set = set.clone();
        set.replace(i);
        for value in old_set.iter() {
            if set.get(value).is_none() {
                println!("old_set: {:?}", old_set);
                println!("set: {:?}", set);
                panic!("did not find {} in set", value);
            }
        }
    }

    for &i in &values {
        assert!(set.get(&i).is_some(), "did not find {}", i);
    }
}

#[test]
fn replace_dup() {
    let mut elements = vec![0, 2, 4, 6, 8];
    let mut set: IndexSet<u8> = elements.drain(..).collect();
    {
        let (i, v) = set.get_full(&0).unwrap();
        assert_eq!(set.len(), 5);
        assert_eq!(i, 0);
        assert_eq!(*v, 0);
    }
    {
        let replaced = set.replace(0);
        let (i, v) = set.get_full(&0).unwrap();
        assert_eq!(set.len(), 5);
        assert_eq!(replaced, Some(0));
        assert_eq!(i, 0);
        assert_eq!(*v, 0);
    }
}

#[test]
fn replace_order() {
    let replace = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut set = IndexSet::new();

    for &elt in &replace {
        set.replace(elt);
    }

    assert_eq!(set.iter().count(), set.len());
    assert_eq!(set.iter().count(), replace.len());
    for (a, b) in replace.iter().zip(set.iter()) {
        assert_eq!(a, b);
    }
    for (i, v) in (0..replace.len()).zip(set.iter()) {
        assert_eq!(set.get_index(i).unwrap(), v);
    }
}

#[test]
fn grow() {
    let insert = [0, 4, 2, 12, 8, 7, 11];
    let not_present = [1, 3, 6, 9, 10];
    let mut set = IndexSet::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(set.len(), i);
        set.insert(elt);
        assert_eq!(set.len(), i + 1);
        assert_eq!(set.get(&elt), Some(&elt));
    }

    println!("{:?}", set);
    for &elt in &insert {
        set.insert(elt * 10);
    }
    for &elt in &insert {
        set.insert(elt * 100);
    }
    for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
        set.insert(elt * 100 + i as i32);
    }
    println!("{:?}", set);
    for &elt in &not_present {
        assert!(set.get(&elt).is_none());
    }
}

#[test]
fn reserve() {
    let mut set = IndexSet::<usize>::new();
    assert_eq!(set.capacity(), 0);
    set.reserve(100);
    let capacity = set.capacity();
    assert!(capacity >= 100);
    for i in 0..capacity {
        assert_eq!(set.len(), i);
        set.insert(i);
        assert_eq!(set.len(), i + 1);
        assert_eq!(set.capacity(), capacity);
        assert_eq!(set.get(&i), Some(&i));
    }
    set.insert(capacity);
    assert_eq!(set.len(), capacity + 1);
    assert!(set.capacity() > capacity);
    assert_eq!(set.get(&capacity), Some(&capacity));
}

#[test]
fn try_reserve() {
    let mut set = IndexSet::<usize>::new();
    assert_eq!(set.capacity(), 0);
    assert_eq!(set.try_reserve(100), Ok(()));
    assert!(set.capacity() >= 100);
    assert!(set.try_reserve(usize::MAX).is_err());
}

#[test]
fn shrink_to_fit() {
    let mut set = IndexSet::<usize>::new();
    assert_eq!(set.capacity(), 0);
    for i in 0..100 {
        assert_eq!(set.len(), i);
        set.insert(i);
        assert_eq!(set.len(), i + 1);
        assert!(set.capacity() >= i + 1);
        assert_eq!(set.get(&i), Some(&i));
        set.shrink_to_fit();
        assert_eq!(set.len(), i + 1);
        assert_eq!(set.capacity(), i + 1);
        assert_eq!(set.get(&i), Some(&i));
    }
}

#[test]
fn remove() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut set = IndexSet::new();

    for &elt in &insert {
        set.insert(elt);
    }

    assert_eq!(set.iter().count(), set.len());
    assert_eq!(set.iter().count(), insert.len());
    for (a, b) in insert.iter().zip(set.iter()) {
        assert_eq!(a, b);
    }

    let remove_fail = [99, 77];
    let remove = [4, 12, 8, 7];

    for &value in &remove_fail {
        assert!(set.swap_remove_full(&value).is_none());
    }
    println!("{:?}", set);
    for &value in &remove {
        //println!("{:?}", set);
        let index = set.get_full(&value).unwrap().0;
        assert_eq!(set.swap_remove_full(&value), Some((index, value)));
    }
    println!("{:?}", set);

    for value in &insert {
        assert_eq!(set.get(value).is_some(), !remove.contains(value));
    }
    assert_eq!(set.len(), insert.len() - remove.len());
    assert_eq!(set.iter().count(), insert.len() - remove.len());
}

#[test]
fn swap_remove_index() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut set = IndexSet::new();

    for &elt in &insert {
        set.insert(elt);
    }

    let mut vector = insert.to_vec();
    let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];

    // check that the same swap remove sequence on vec and set
    // have the same result.
    for &rm in remove_sequence {
        let out_vec = vector.swap_remove(rm);
        let out_set = set.swap_remove_index(rm).unwrap();
        assert_eq!(out_vec, out_set);
    }
    assert_eq!(vector.len(), set.len());
    for (a, b) in vector.iter().zip(set.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn partial_eq_and_eq() {
    let mut set_a = IndexSet::new();
    set_a.insert(1);
    set_a.insert(2);
    let mut set_b = set_a.clone();
    assert_eq!(set_a, set_b);
    set_b.swap_remove(&1);
    assert_ne!(set_a, set_b);

    let set_c: IndexSet<_> = set_b.into_iter().collect();
    assert_ne!(set_a, set_c);
    assert_ne!(set_c, set_a);
}

#[test]
fn extend() {
    let mut set = IndexSet::new();
    set.extend(vec![&1, &2, &3, &4]);
    set.extend(vec![5, 6]);
    assert_eq!(set.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn comparisons() {
    let set_a: IndexSet<_> = (0..3).collect();
    let set_b: IndexSet<_> = (3..6).collect();
    let set_c: IndexSet<_> = (0..6).collect();
    let set_d: IndexSet<_> = (3..9).collect();

    assert!(!set_a.is_disjoint(&set_a));
    assert!(set_a.is_subset(&set_a));
    assert!(set_a.is_superset(&set_a));

    assert!(set_a.is_disjoint(&set_b));
    assert!(set_b.is_disjoint(&set_a));
    assert!(!set_a.is_subset(&set_b));
    assert!(!set_b.is_subset(&set_a));
    assert!(!set_a.is_superset(&set_b));
    assert!(!set_b.is_superset(&set_a));

    assert!(!set_a.is_disjoint(&set_c));
    assert!(!set_c.is_disjoint(&set_a));
    assert!(set_a.is_subset(&set_c));
    assert!(!set_c.is_subset(&set_a));
    assert!(!set_a.is_superset(&set_c));
    assert!(set_c.is_superset(&set_a));

    assert!(!set_c.is_disjoint(&set_d));
    assert!(!set_d.is_disjoint(&set_c));
    assert!(!set_c.is_subset(&set_d));
    assert!(!set_d.is_subset(&set_c));
    assert!(!set_c.is_superset(&set_d));
    assert!(!set_d.is_superset(&set_c));
}

#[test]
fn iter_comparisons() {
    use std::iter::empty;

    fn check<'a, I1, I2>(iter1: I1, iter2: I2)
    where
        I1: Iterator<Item = &'a i32>,
        I2: Iterator<Item = i32>,
    {
        assert!(iter1.copied().eq(iter2));
    }

    let set_a: IndexSet<_> = (0..3).collect();
    let set_b: IndexSet<_> = (3..6).collect();
    let set_c: IndexSet<_> = (0..6).collect();
    let set_d: IndexSet<_> = (3..9).rev().collect();

    check(set_a.difference(&set_a), empty());
    check(set_a.symmetric_difference(&set_a), empty());
    check(set_a.intersection(&set_a), 0..3);
    check(set_a.union(&set_a), 0..3);

    check(set_a.difference(&set_b), 0..3);
    check(set_b.difference(&set_a), 3..6);
    check(set_a.symmetric_difference(&set_b), 0..6);
    check(set_b.symmetric_difference(&set_a), (3..6).chain(0..3));
    check(set_a.intersection(&set_b), empty());
    check(set_b.intersection(&set_a), empty());
    check(set_a.union(&set_b), 0..6);
    check(set_b.union(&set_a), (3..6).chain(0..3));

    check(set_a.difference(&set_c), empty());
    check(set_c.difference(&set_a), 3..6);
    check(set_a.symmetric_difference(&set_c), 3..6);
    check(set_c.symmetric_difference(&set_a), 3..6);
    check(set_a.intersection(&set_c), 0..3);
    check(set_c.intersection(&set_a), 0..3);
    check(set_a.union(&set_c), 0..6);
    check(set_c.union(&set_a), 0..6);

    check(set_c.difference(&set_d), 0..3);
    check(set_d.difference(&set_c), (6..9).rev());
    check(
        set_c.symmetric_difference(&set_d),
        (0..3).chain((6..9).rev()),
    );
    check(set_d.symmetric_difference(&set_c), (6..9).rev().chain(0..3));
    check(set_c.intersection(&set_d), 3..6);
    check(set_d.intersection(&set_c), (3..6).rev());
    check(set_c.union(&set_d), (0..6).chain((6..9).rev()));
    check(set_d.union(&set_c), (3..9).rev().chain(0..3));
}

#[test]
fn ops() {
    let empty = IndexSet::<i32>::new();
    let set_a: IndexSet<_> = (0..3).collect();
    let set_b: IndexSet<_> = (3..6).collect();
    let set_c: IndexSet<_> = (0..6).collect();
    let set_d: IndexSet<_> = (3..9).rev().collect();

    #[allow(clippy::eq_op)]
    {
        assert_eq!(&set_a & &set_a, set_a);
        assert_eq!(&set_a | &set_a, set_a);
        assert_eq!(&set_a ^ &set_a, empty);
        assert_eq!(&set_a - &set_a, empty);
    }

    assert_eq!(&set_a & &set_b, empty);
    assert_eq!(&set_b & &set_a, empty);
    assert_eq!(&set_a | &set_b, set_c);
    assert_eq!(&set_b | &set_a, set_c);
    assert_eq!(&set_a ^ &set_b, set_c);
    assert_eq!(&set_b ^ &set_a, set_c);
    assert_eq!(&set_a - &set_b, set_a);
    assert_eq!(&set_b - &set_a, set_b);

    assert_eq!(&set_a & &set_c, set_a);
    assert_eq!(&set_c & &set_a, set_a);
    assert_eq!(&set_a | &set_c, set_c);
    assert_eq!(&set_c | &set_a, set_c);
    assert_eq!(&set_a ^ &set_c, set_b);
    assert_eq!(&set_c ^ &set_a, set_b);
    assert_eq!(&set_a - &set_c, empty);
    assert_eq!(&set_c - &set_a, set_b);

    assert_eq!(&set_c & &set_d, set_b);
    assert_eq!(&set_d & &set_c, set_b);
    assert_eq!(&set_c | &set_d, &set_a | &set_d);
    assert_eq!(&set_d | &set_c, &set_a | &set_d);
    assert_eq!(&set_c ^ &set_d, &set_a | &(&set_d - &set_b));
    assert_eq!(&set_d ^ &set_c, &set_a | &(&set_d - &set_b));
    assert_eq!(&set_c - &set_d, set_a);
    assert_eq!(&set_d - &set_c, &set_d - &set_b);
}

#[test]
#[cfg(feature = "std")]
fn from_array() {
    let set1 = IndexSet::from([1, 2, 3, 4]);
    let set2: IndexSet<_> = [1, 2, 3, 4].into();

    assert_eq!(set1, set2);
}

#[test]
fn iter_default() {
    struct Item;
    fn assert_default<T>()
    where
        T: Default + Iterator,
    {
        assert!(T::default().next().is_none());
    }
    assert_default::<Iter<'static, Item>>();
    assert_default::<IntoIter<Item>>();
}
