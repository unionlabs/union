use super::*;
use std::string::String;

#[test]
fn it_works() {
    let mut map = IndexMap::new();
    assert_eq!(map.is_empty(), true);
    map.insert(1, ());
    map.insert(1, ());
    assert_eq!(map.len(), 1);
    assert!(map.get(&1).is_some());
    assert_eq!(map.is_empty(), false);
}

#[test]
fn new() {
    let map = IndexMap::<String, String>::new();
    println!("{:?}", map);
    assert_eq!(map.capacity(), 0);
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
}

#[test]
fn insert() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5];
    let not_present = [1, 3, 6, 9, 10];
    let mut map = IndexMap::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(map.len(), i);
        map.insert(elt, elt);
        assert_eq!(map.len(), i + 1);
        assert_eq!(map.get(&elt), Some(&elt));
        assert_eq!(map[&elt], elt);
    }
    println!("{:?}", map);

    for &elt in &not_present {
        assert!(map.get(&elt).is_none());
    }
}

#[test]
fn insert_full() {
    let insert = vec![9, 2, 7, 1, 4, 6, 13];
    let present = vec![1, 6, 2];
    let mut map = IndexMap::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(map.len(), i);
        let (index, existing) = map.insert_full(elt, elt);
        assert_eq!(existing, None);
        assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
        assert_eq!(map.len(), i + 1);
    }

    let len = map.len();
    for &elt in &present {
        let (index, existing) = map.insert_full(elt, elt);
        assert_eq!(existing, Some(elt));
        assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
        assert_eq!(map.len(), len);
    }
}

#[test]
fn insert_2() {
    let mut map = IndexMap::with_capacity(16);

    let mut keys = vec![];
    keys.extend(0..16);
    keys.extend(if cfg!(miri) { 32..64 } else { 128..267 });

    for &i in &keys {
        let old_map = map.clone();
        map.insert(i, ());
        for key in old_map.keys() {
            if map.get(key).is_none() {
                println!("old_map: {:?}", old_map);
                println!("map: {:?}", map);
                panic!("did not find {} in map", key);
            }
        }
    }

    for &i in &keys {
        assert!(map.get(&i).is_some(), "did not find {}", i);
    }
}

#[test]
fn insert_order() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut map = IndexMap::new();

    for &elt in &insert {
        map.insert(elt, ());
    }

    assert_eq!(map.keys().count(), map.len());
    assert_eq!(map.keys().count(), insert.len());
    for (a, b) in insert.iter().zip(map.keys()) {
        assert_eq!(a, b);
    }
    for (i, k) in (0..insert.len()).zip(map.keys()) {
        assert_eq!(map.get_index(i).unwrap().0, k);
    }
}

#[test]
fn grow() {
    let insert = [0, 4, 2, 12, 8, 7, 11];
    let not_present = [1, 3, 6, 9, 10];
    let mut map = IndexMap::with_capacity(insert.len());

    for (i, &elt) in insert.iter().enumerate() {
        assert_eq!(map.len(), i);
        map.insert(elt, elt);
        assert_eq!(map.len(), i + 1);
        assert_eq!(map.get(&elt), Some(&elt));
        assert_eq!(map[&elt], elt);
    }

    println!("{:?}", map);
    for &elt in &insert {
        map.insert(elt * 10, elt);
    }
    for &elt in &insert {
        map.insert(elt * 100, elt);
    }
    for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
        map.insert(elt * 100 + i as i32, elt);
    }
    println!("{:?}", map);
    for &elt in &not_present {
        assert!(map.get(&elt).is_none());
    }
}

#[test]
fn reserve() {
    let mut map = IndexMap::<usize, usize>::new();
    assert_eq!(map.capacity(), 0);
    map.reserve(100);
    let capacity = map.capacity();
    assert!(capacity >= 100);
    for i in 0..capacity {
        assert_eq!(map.len(), i);
        map.insert(i, i * i);
        assert_eq!(map.len(), i + 1);
        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.get(&i), Some(&(i * i)));
    }
    map.insert(capacity, std::usize::MAX);
    assert_eq!(map.len(), capacity + 1);
    assert!(map.capacity() > capacity);
    assert_eq!(map.get(&capacity), Some(&std::usize::MAX));
}

#[test]
fn try_reserve() {
    let mut map = IndexMap::<usize, usize>::new();
    assert_eq!(map.capacity(), 0);
    assert_eq!(map.try_reserve(100), Ok(()));
    assert!(map.capacity() >= 100);
    assert!(map.try_reserve(usize::MAX).is_err());
}

#[test]
fn shrink_to_fit() {
    let mut map = IndexMap::<usize, usize>::new();
    assert_eq!(map.capacity(), 0);
    for i in 0..100 {
        assert_eq!(map.len(), i);
        map.insert(i, i * i);
        assert_eq!(map.len(), i + 1);
        assert!(map.capacity() >= i + 1);
        assert_eq!(map.get(&i), Some(&(i * i)));
        map.shrink_to_fit();
        assert_eq!(map.len(), i + 1);
        assert_eq!(map.capacity(), i + 1);
        assert_eq!(map.get(&i), Some(&(i * i)));
    }
}

#[test]
fn remove() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut map = IndexMap::new();

    for &elt in &insert {
        map.insert(elt, elt);
    }

    assert_eq!(map.keys().count(), map.len());
    assert_eq!(map.keys().count(), insert.len());
    for (a, b) in insert.iter().zip(map.keys()) {
        assert_eq!(a, b);
    }

    let remove_fail = [99, 77];
    let remove = [4, 12, 8, 7];

    for &key in &remove_fail {
        assert!(map.swap_remove_full(&key).is_none());
    }
    println!("{:?}", map);
    for &key in &remove {
        //println!("{:?}", map);
        let index = map.get_full(&key).unwrap().0;
        assert_eq!(map.swap_remove_full(&key), Some((index, key, key)));
    }
    println!("{:?}", map);

    for key in &insert {
        assert_eq!(map.get(key).is_some(), !remove.contains(key));
    }
    assert_eq!(map.len(), insert.len() - remove.len());
    assert_eq!(map.keys().count(), insert.len() - remove.len());
}

#[test]
fn remove_to_empty() {
    let mut map = indexmap! { 0 => 0, 4 => 4, 5 => 5 };
    map.swap_remove(&5).unwrap();
    map.swap_remove(&4).unwrap();
    map.swap_remove(&0).unwrap();
    assert!(map.is_empty());
}

#[test]
fn swap_remove_index() {
    let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
    let mut map = IndexMap::new();

    for &elt in &insert {
        map.insert(elt, elt * 2);
    }

    let mut vector = insert.to_vec();
    let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];

    // check that the same swap remove sequence on vec and map
    // have the same result.
    for &rm in remove_sequence {
        let out_vec = vector.swap_remove(rm);
        let (out_map, _) = map.swap_remove_index(rm).unwrap();
        assert_eq!(out_vec, out_map);
    }
    assert_eq!(vector.len(), map.len());
    for (a, b) in vector.iter().zip(map.keys()) {
        assert_eq!(a, b);
    }
}

#[test]
fn partial_eq_and_eq() {
    let mut map_a = IndexMap::new();
    map_a.insert(1, "1");
    map_a.insert(2, "2");
    let mut map_b = map_a.clone();
    assert_eq!(map_a, map_b);
    map_b.swap_remove(&1);
    assert_ne!(map_a, map_b);

    let map_c: IndexMap<_, String> = map_b.into_iter().map(|(k, v)| (k, v.into())).collect();
    assert_ne!(map_a, map_c);
    assert_ne!(map_c, map_a);
}

#[test]
fn extend() {
    let mut map = IndexMap::new();
    map.extend(vec![(&1, &2), (&3, &4)]);
    map.extend(vec![(5, 6)]);
    assert_eq!(
        map.into_iter().collect::<Vec<_>>(),
        vec![(1, 2), (3, 4), (5, 6)]
    );
}

#[test]
fn entry() {
    let mut map = IndexMap::new();

    map.insert(1, "1");
    map.insert(2, "2");
    {
        let e = map.entry(3);
        assert_eq!(e.index(), 2);
        let e = e.or_insert("3");
        assert_eq!(e, &"3");
    }

    let e = map.entry(2);
    assert_eq!(e.index(), 1);
    assert_eq!(e.key(), &2);
    match e {
        Entry::Occupied(ref e) => assert_eq!(e.get(), &"2"),
        Entry::Vacant(_) => panic!(),
    }
    assert_eq!(e.or_insert("4"), &"2");
}

#[test]
fn entry_and_modify() {
    let mut map = IndexMap::new();

    map.insert(1, "1");
    map.entry(1).and_modify(|x| *x = "2");
    assert_eq!(Some(&"2"), map.get(&1));

    map.entry(2).and_modify(|x| *x = "doesn't exist");
    assert_eq!(None, map.get(&2));
}

#[test]
fn entry_or_default() {
    let mut map = IndexMap::new();

    #[derive(Debug, PartialEq)]
    enum TestEnum {
        DefaultValue,
        NonDefaultValue,
    }

    impl Default for TestEnum {
        fn default() -> Self {
            TestEnum::DefaultValue
        }
    }

    map.insert(1, TestEnum::NonDefaultValue);
    assert_eq!(&mut TestEnum::NonDefaultValue, map.entry(1).or_default());

    assert_eq!(&mut TestEnum::DefaultValue, map.entry(2).or_default());
}

#[test]
fn occupied_entry_key() {
    // These keys match hash and equality, but their addresses are distinct.
    let (k1, k2) = (&mut 1, &mut 1);
    let k1_ptr = k1 as *const i32;
    let k2_ptr = k2 as *const i32;
    assert_ne!(k1_ptr, k2_ptr);

    let mut map = IndexMap::new();
    map.insert(k1, "value");
    match map.entry(k2) {
        Entry::Occupied(ref e) => {
            // `OccupiedEntry::key` should reference the key in the map,
            // not the key that was used to find the entry.
            let ptr = *e.key() as *const i32;
            assert_eq!(ptr, k1_ptr);
            assert_ne!(ptr, k2_ptr);
        }
        Entry::Vacant(_) => panic!(),
    }
}

#[test]
fn keys() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: IndexMap<_, _> = vec.into_iter().collect();
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
    assert!(keys.contains(&3));
}

#[test]
fn into_keys() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: IndexMap<_, _> = vec.into_iter().collect();
    let keys: Vec<i32> = map.into_keys().collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
    assert!(keys.contains(&3));
}

#[test]
fn values() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: IndexMap<_, _> = vec.into_iter().collect();
    let values: Vec<_> = map.values().copied().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&'a'));
    assert!(values.contains(&'b'));
    assert!(values.contains(&'c'));
}

#[test]
fn values_mut() {
    let vec = vec![(1, 1), (2, 2), (3, 3)];
    let mut map: IndexMap<_, _> = vec.into_iter().collect();
    for value in map.values_mut() {
        *value *= 2
    }
    let values: Vec<_> = map.values().copied().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&2));
    assert!(values.contains(&4));
    assert!(values.contains(&6));
}

#[test]
fn into_values() {
    let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
    let map: IndexMap<_, _> = vec.into_iter().collect();
    let values: Vec<char> = map.into_values().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&'a'));
    assert!(values.contains(&'b'));
    assert!(values.contains(&'c'));
}

#[test]
#[cfg(feature = "std")]
fn from_array() {
    let map = IndexMap::from([(1, 2), (3, 4)]);
    let mut expected = IndexMap::new();
    expected.insert(1, 2);
    expected.insert(3, 4);

    assert_eq!(map, expected)
}

#[test]
fn iter_default() {
    struct K;
    struct V;
    fn assert_default<T>()
    where
        T: Default + Iterator,
    {
        assert!(T::default().next().is_none());
    }
    assert_default::<Iter<'static, K, V>>();
    assert_default::<IterMut<'static, K, V>>();
    assert_default::<IntoIter<K, V>>();
    assert_default::<Keys<'static, K, V>>();
    assert_default::<IntoKeys<K, V>>();
    assert_default::<Values<'static, K, V>>();
    assert_default::<ValuesMut<'static, K, V>>();
    assert_default::<IntoValues<K, V>>();
}
