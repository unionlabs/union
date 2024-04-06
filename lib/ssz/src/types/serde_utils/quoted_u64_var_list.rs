//! Formats `VariableList<u64,N>` and similar types using quotes.
//!
//! E.g., `VariableList::from(vec![0, 1, 2])` serializes as `["0", "1", "2"]`.
//!
//! Quotes can be optional during decoding. If the length of the `Vec` is greater than `N`, deserialization fails.

use std::{iter, marker::PhantomData};

use itertools::process_results;
use serde::{de::Error, ser::SerializeSeq, Deserializer, Serializer};
use serde_utils::quoted_u64_vec::QuotedIntWrapper;
use ssz::TryFromIter;

pub struct QuotedIntVarListVisitor<C> {
    _phantom: PhantomData<C>,
}

impl<'a, C> serde::de::Visitor<'a> for QuotedIntVarListVisitor<C>
where
    C: TryFromIter<u64>,
{
    type Value = C;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of quoted or unquoted integers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        process_results(iter::from_fn(|| seq.next_element().transpose()), |iter| {
            C::try_from_iter(iter.map(|QuotedIntWrapper { int }| int))
                .map_err(|e| A::Error::custom(format!("{e:?}")))
        })?
    }
}

pub fn serialize<'a, C, I, S>(value: &'a C, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    &'a C: IntoIterator<IntoIter = I>,
    I: Iterator<Item = &'a u64> + ExactSizeIterator,
{
    let iter = value.into_iter();
    let mut seq = serializer.serialize_seq(Some(iter.len()))?;
    for &int in iter {
        seq.serialize_element(&QuotedIntWrapper { int })?;
    }
    seq.end()
}

pub fn deserialize<'de, D, C>(deserializer: D) -> Result<C, D::Error>
where
    D: Deserializer<'de>,
    C: TryFromIter<u64>,
{
    deserializer.deserialize_any(QuotedIntVarListVisitor {
        _phantom: PhantomData,
    })
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use typenum::U4;

    use crate::VariableList;

    #[derive(Debug, Serialize, Deserialize)]
    struct Obj {
        #[serde(with = "crate::serde_utils::quoted_u64_var_list")]
        values: VariableList<u64, U4>,
    }

    #[test]
    fn quoted_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": ["1", "2", "3", "4"] }"#).unwrap();
        let expected: VariableList<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn unquoted_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": [1, 2, 3, 4] }"#).unwrap();
        let expected: VariableList<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn mixed_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": ["1", 2, "3", "4"] }"#).unwrap();
        let expected: VariableList<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn empty_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": [] }"#).unwrap();
        assert!(obj.values.is_empty());
    }

    #[test]
    fn short_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": [1, 2] }"#).unwrap();
        let expected: VariableList<u64, U4> = vec![1, 2].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn long_list_err() {
        serde_json::from_str::<Obj>(r#"{ "values": [1, 2, 3, 4, 5] }"#).unwrap_err();
    }

    #[test]
    fn whole_list_quoted_err() {
        serde_json::from_str::<Obj>(r#"{ "values": "[1, 2, 3, 4]" }"#).unwrap_err();
    }
}
