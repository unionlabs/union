use std::ops::Bound;

use cosmwasm_std::{
    Order,
    testing::{MockStorage, mock_dependencies},
};
use num_traits::ToBytes;
use unionlabs::primitives::ByteArrayExt;

use super::*;
use crate::value::{ValueCodecViaEncoding, ValueUnitEncoding};

enum TestStore {}

impl Store for TestStore {
    const PREFIX: Prefix = Prefix::new(b"test");

    type Key = u64;

    type Value = (u64, u64);
}

impl KeyCodec<u64> for TestStore {
    fn encode_key(key: &u64) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<u64> {
        raw.try_into().map(u64::from_be_bytes).map_err(|_| {
            StdError::generic_err(format!(
                "invalid key: expected 8 bytes, found {}: {raw}",
                raw.len()
            ))
        })
    }
}

impl ValueCodec<(u64, u64)> for TestStore {
    fn encode_value(value: &(u64, u64)) -> Bytes {
        [value.0.to_be_bytes(), value.1.to_be_bytes()]
            .into_iter()
            .flatten()
            .collect()
    }

    fn decode_value(raw: &Bytes) -> StdResult<(u64, u64)> {
        raw.try_into()
            .map(|arr: [u8; 16]| {
                (
                    u64::from_be_bytes(arr.array_slice::<0, 8>()),
                    u64::from_be_bytes(arr.array_slice::<8, 8>()),
                )
            })
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid value: expected 16 bytes, found {}: {raw}",
                    raw.len()
                ))
            })
    }
}

#[test]
fn read_write() {
    let mut storage = MockStorage::new();

    storage.write::<TestStore>(&1, &(1, 1));

    let value = storage.read::<TestStore>(&1).unwrap();

    assert_eq!(value, (1, 1));
}

#[cfg(test)]
mod iterator {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn init() -> (MockStorage, [(u64, (u64, u64)); 4]) {
        let mut storage = MockStorage::new();

        let kvs = [(1, (1, 1)), (2, (1, 2)), (3, (1, 3)), (u64::MAX, (0, 0))];

        // write additional values to storage to ensure only the prefixed store is iterated

        // b"test" - 1
        storage.set(b"tess".as_slice(), &[0]);
        // b"test" + 1
        storage.set(b"tesu".as_slice(), &[0]);

        for (k, v) in &kvs {
            storage.write::<TestStore>(k, v);
        }

        (storage, kvs)
    }

    #[test]
    fn iter() {
        let (storage, kvs) = init();

        let res = storage
            .iter::<TestStore>(Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, kvs);
    }

    #[test]
    fn iter_range_full() {
        let (storage, kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, ..)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, kvs);
    }

    #[test]
    fn iter_range_both_inclusive() {
        let (storage, kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, 0..=u64::MAX)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, kvs);
    }

    #[test]
    fn iter_range_start_inclusive_end_exclusive() {
        let (storage, _kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, 1..3)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, [(1, (1, 1)), (2, (1, 2))]);
    }

    #[test]
    fn iter_range_start_unbounded_end_exclusive() {
        let (storage, _kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, ..3)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, [(1, (1, 1)), (2, (1, 2))]);
    }

    #[test]
    fn iter_range_start_unbounded_end_inclusive() {
        let (storage, kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, ..=u64::MAX)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, kvs);
    }

    #[test]
    fn iter_range_start_inclusive_end_unbounded() {
        let (storage, kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, 1..)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, kvs);
    }

    #[test]
    fn iter_range_start_exclusive_end_unbounded() {
        let (storage, _kvs) = init();

        let res = storage
            .iter_range::<TestStore>(Order::Ascending, (Bound::Excluded(1), Bound::Unbounded))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, [(2, (1, 2)), (3, (1, 3)), (u64::MAX, (0, 0))]);
    }

    #[test]
    fn iter_range_start_exclusive_end_inclusive() {
        let (storage, _kvs) = init();

        let res = storage
            .iter_range::<TestStore>(
                Order::Ascending,
                (Bound::Excluded(1), Bound::Included(u64::MAX)),
            )
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, [(2, (1, 2)), (3, (1, 3)), (u64::MAX, (0, 0))]);
    }

    #[test]
    fn iter_range_both_exclusive() {
        let (storage, _kvs) = init();

        let res = storage
            .iter_range::<TestStore>(
                Order::Ascending,
                (Bound::Excluded(1), Bound::Excluded(u64::MAX)),
            )
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(res, [(2, (1, 2)), (3, (1, 3))]);
    }
}

#[test]
fn compound_key_iter_range() {
    enum A {}
    impl Store for A {
        const PREFIX: Prefix = Prefix::new(&[1]);
        type Key = (Bytes, u32);
        type Value = ();
    }
    impl KeyCodec<(Bytes, u32)> for A {
        fn encode_key((bz, n): &(Bytes, u32)) -> Bytes {
            bz.iter().copied().chain(n.to_be_bytes()).collect()
        }

        fn decode_key(raw: &Bytes) -> StdResult<(Bytes, u32)> {
            let (bz, n) = raw.split_at(raw.len() - 4);
            Ok((
                bz.into(),
                u32::from_be_bytes(n.try_into().expect("is 4 bytes; qed;")),
            ))
        }
    }
    impl ValueCodecViaEncoding for A {
        type Encoding = ValueUnitEncoding;
    }

    let mut deps = mock_dependencies();

    deps.storage.write::<A>(&([0x00].into(), u32::MAX), &());
    deps.storage.write::<A>(&([0x01].into(), 0), &());

    let range = deps
        .storage
        .iter_range::<A>(
            Order::Ascending,
            ([0x00].into(), 0)..=([0x00].into(), u32::MAX),
        )
        .map(|r| r.map(|(k, ())| k))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // this does NOT include ([0x01], 0)
    assert_eq!(range, [([0x00].into(), u32::MAX)]);
}

#[test]
#[allow(non_local_definitions)]
fn no_overlap() {
    enum A {}
    impl Store for A {
        const PREFIX: Prefix = Prefix::new(&[1]);
        type Key = Bytes;
        type Value = Bytes;
    }

    impl KeyCodec<Bytes> for A {
        fn encode_key(key: &Bytes) -> Bytes {
            key.clone()
        }

        fn decode_key(raw: &Bytes) -> StdResult<Bytes> {
            Ok(raw.clone())
        }
    }

    impl ValueCodec<Bytes> for A {
        fn encode_value(value: &Bytes) -> Bytes {
            value.clone()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Bytes> {
            Ok(raw.clone())
        }
    }

    enum B {}
    impl Store for B {
        const PREFIX: Prefix = Prefix::new(&[1, 1]);
        type Key = Bytes;
        type Value = Bytes;
    }

    impl KeyCodec<Bytes> for B {
        fn encode_key(key: &Bytes) -> Bytes {
            key.clone()
        }

        fn decode_key(raw: &Bytes) -> StdResult<Bytes> {
            Ok(raw.clone())
        }
    }

    impl ValueCodec<Bytes> for B {
        fn encode_value(value: &Bytes) -> Bytes {
            value.clone()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Bytes> {
            Ok(raw.clone())
        }
    }

    let mut storage = MockStorage::new();

    let a_value = Bytes::new(b"a");
    let b_value = Bytes::new(b"b");

    // this will be stored under 01 00 01
    storage.write::<A>(&Bytes::new(&[1]), &a_value);
    // this will be stored under 01 01 00
    storage.write::<B>(&Bytes::new(&[]), &b_value);

    let value = storage.read::<A>(&Bytes::new(&[1])).unwrap();
    assert_eq!(value, a_value);

    let value = storage.read::<B>(&Bytes::new(&[])).unwrap();
    assert_eq!(value, b_value);
}

#[test]
fn prefix_new() {
    let _ = Prefix::new(b"");
    let _ = Prefix::new(b"a");
    let _ = Prefix::new(b"aa");
    let _ = Prefix::new(b"aaa");
    let _ = Prefix::new(b"aaaa");
    let _ = Prefix::new(b"aaaaa");
}

macro_rules! prefix_panic_tests {
    ($($test:ident = $expr:expr;)*) => {
        $(
            #[test]
            #[should_panic = "prefix cannot contain the prefix separator byte 0x00"]
            fn $test() {
                let _ = Prefix::new($expr);
            }
        )*
    };
}

prefix_panic_tests! {
    prefix_new_0 = b"\0";

    prefix_new_a0 = b"a\0";
    prefix_new_aa0 = b"aa\0";

    prefix_new_0a = b"\0a";
    prefix_new_0aa = b"\0aa";

    prefix_new_a0a = b"a\0a";
    prefix_new_aa0aa = b"aa\0aa";

    prefix_new_a0aa = b"a\0aa";
}
