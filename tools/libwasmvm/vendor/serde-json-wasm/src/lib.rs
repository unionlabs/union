//! [`serde-json`] for `wasm` programs
//!
//! [`serde-json`]: https://crates.io/crates/serde_json
//!
//! This version of [`serde-json`] is aimed at applications that run on resource constrained
//! devices.
//!
//! # Current features
//!
//! - The error type is a simple C like enum (less overhead, smaller memory footprint)
//! - (De)serialization doesn't require memory allocations
//! - Deserialization of integers doesn't go through `u64`; instead the string is directly parsed
//!   into the requested integer type. This avoids pulling in KBs of compiler intrinsics when
//!   targeting a non 64-bit architecture.
//! - Supports deserialization of:
//!   - `bool`
//!   - Integers
//!   - `str` (This is a zero copy operation.) (\*)
//!   - `Option`
//!   - Arrays
//!   - Tuples
//!   - Structs
//!   - C like enums
//! - Supports serialization (compact format only) of:
//!   - `bool`
//!   - Integers
//!   - `str`
//!   - `Option`
//!   - Arrays
//!   - Tuples
//!   - Structs
//!   - C like enums
//!
//! (\*) Deserialization of strings ignores escaped sequences. Escaped sequences might be supported
//! in the future using a different Serializer as this operation is not zero copy.
//!
//! # Planned features
//!
//! - (De)serialization from / into IO objects once `core::io::{Read,Write}` becomes a thing.
//!
//! # Non-features
//!
//! This is explicitly out of scope
//!
//! - Anything that involves dynamic memory allocation
//!   - Like the dynamic [`Value`](https://docs.rs/serde_json/1.0.11/serde_json/enum.Value.html)
//!     type
//!
//! # MSRV
//!
//! This crate is guaranteed to compile on stable Rust 1.31.0 and up. It *might* compile with older
//! versions but that may change in any new patch release.

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]

pub mod de;
pub mod ser;

#[doc(inline)]
pub use self::de::{from_slice, from_str};
#[doc(inline)]
pub use self::ser::{to_string, to_vec};

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Address(String);

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct CommentId(u32);

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    enum Model {
        Comment,
        Post { category: String, author: Address },
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Stats {
        views: u64,
        score: i64,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Item {
        model: Model,
        title: String,
        content: Option<String>,
        list: Vec<u32>,
        published: bool,
        comments: Vec<CommentId>,
        stats: Stats,
        balances: BTreeMap<String, u16>,
    }

    #[test]
    fn can_serde() {
        let min = Item {
            model: Model::Comment,
            title: "".to_string(),
            content: None,
            list: vec![],
            published: false,
            comments: vec![],
            stats: Stats { views: 0, score: 0 },
            balances: BTreeMap::new(),
        };
        let mut balances: BTreeMap<String, u16> = BTreeMap::new();
        balances.insert("chareen".into(), 347);
        let max = Item {
            model: Model::Post {
                category: "fun".to_string(),
                author: Address("sunnyboy85".to_string()),
            },
            title: "Nice message".to_string(),
            content: Some("Happy \"blogging\" 👏\n\n\tCheers, I'm out\0\0\0".to_string()),
            list: vec![0, 1, 2, 3, 42, 154841, std::u32::MAX],
            published: true,
            comments: vec![CommentId(2), CommentId(700)],
            stats: Stats {
                views: std::u64::MAX,
                score: std::i64::MIN,
            },
            balances,
        };

        // binary
        assert_eq!(from_slice::<Item>(&to_vec(&min).unwrap()).unwrap(), min);
        assert_eq!(from_slice::<Item>(&to_vec(&max).unwrap()).unwrap(), max);

        // string
        assert_eq!(from_str::<Item>(&to_string(&min).unwrap()).unwrap(), min);
        assert_eq!(from_str::<Item>(&to_string(&max).unwrap()).unwrap(), max);
    }

    #[test]
    fn untagged() {
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        #[serde(untagged)]
        enum UntaggedEnum {
            S(String),
            I(i64),
        }

        let s = UntaggedEnum::S("Some string".to_owned());
        let i = UntaggedEnum::I(32);

        assert_eq!(from_slice::<UntaggedEnum>(&to_vec(&s).unwrap()).unwrap(), s);
        assert_eq!(from_slice::<UntaggedEnum>(&to_vec(&i).unwrap()).unwrap(), i);

        assert_eq!(
            from_str::<UntaggedEnum>(&to_string(&s).unwrap()).unwrap(),
            s
        );
        assert_eq!(
            from_str::<UntaggedEnum>(&to_string(&i).unwrap()).unwrap(),
            i
        );
    }

    #[test]
    fn untagged_structures() {
        #[derive(Debug, Deserialize, Serialize, PartialEq)]
        #[serde(untagged)]
        enum ModelOrItem {
            Model(Model),
            Item(Item),
        }

        let model = ModelOrItem::Model(Model::Post {
            category: "Rust".to_owned(),
            author: Address("no-reply@domain.com".to_owned()),
        });

        let mut balances: BTreeMap<String, u16> = BTreeMap::new();
        balances.insert("chareen".into(), 347);

        let item = ModelOrItem::Item(Item {
            model: Model::Comment,
            title: "Title".to_owned(),
            content: None,
            list: vec![13, 14],
            published: true,
            comments: vec![],
            stats: Stats {
                views: 110,
                score: 12,
            },
            balances,
        });

        assert_eq!(
            from_slice::<ModelOrItem>(&to_vec(&model).unwrap()).unwrap(),
            model
        );
        assert_eq!(
            from_slice::<ModelOrItem>(&to_vec(&item).unwrap()).unwrap(),
            item
        );

        assert_eq!(
            from_str::<ModelOrItem>(&to_string(&model).unwrap()).unwrap(),
            model
        );
        assert_eq!(
            from_str::<ModelOrItem>(&to_string(&item).unwrap()).unwrap(),
            item
        );
    }
}
