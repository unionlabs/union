use cosmwasm_std::testing::MockStorage;
use unionlabs::primitives::ByteArrayExt;

use super::*;

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

#[test]
fn iterator() {
    let mut storage = MockStorage::new();

    let kvs = [(1, (1, 1)), (2, (1, 2)), (3, (1, 3))];

    for (ref k, ref v) in &kvs {
        storage.write::<TestStore>(k, v);
    }

    let iter_kvs = storage
        .iter::<TestStore>(cosmwasm_std::Order::Ascending)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(iter_kvs, kvs);
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

    enum B {}
    impl Store for B {
        const PREFIX: Prefix = Prefix::new(&[1, 1]);
        type Key = Bytes;
        type Value = Bytes;
    }

    trait BytesStore {}
    impl BytesStore for A {}
    impl BytesStore for B {}

    impl<T: BytesStore> KeyCodec<Bytes> for T {
        fn encode_key(key: &Bytes) -> Bytes {
            key.clone()
        }

        fn decode_key(raw: &Bytes) -> StdResult<Bytes> {
            Ok(raw.clone())
        }
    }

    impl<T: BytesStore> ValueCodec<Bytes> for T {
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
