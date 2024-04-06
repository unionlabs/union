//! Serialize `VariableList<VariableList<u8, M>, N>` as list of 0x-prefixed hex string.
use std::marker::PhantomData;

use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use typenum::Unsigned;

use crate::VariableList;

#[derive(Deserialize)]
#[serde(transparent)]
pub struct WrappedListOwned<N: Unsigned>(
    #[serde(with = "crate::serde_utils::hex_var_list")] VariableList<u8, N>,
);

#[derive(Serialize)]
#[serde(transparent)]
pub struct WrappedListRef<'a, N: Unsigned>(
    #[serde(with = "crate::serde_utils::hex_var_list")] &'a VariableList<u8, N>,
);

pub fn serialize<S, M, N>(
    list: &VariableList<VariableList<u8, M>, N>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    M: Unsigned,
    N: Unsigned,
{
    let mut seq = serializer.serialize_seq(Some(list.len()))?;
    for bytes in list {
        seq.serialize_element(&WrappedListRef(bytes))?;
    }
    seq.end()
}

#[derive(Default)]
pub struct Visitor<M, N> {
    _phantom_m: PhantomData<M>,
    _phantom_n: PhantomData<N>,
}

impl<'a, M, N> serde::de::Visitor<'a> for Visitor<M, N>
where
    M: Unsigned,
    N: Unsigned,
{
    type Value = VariableList<VariableList<u8, M>, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut list: VariableList<VariableList<u8, M>, N> = <_>::default();

        while let Some(val) = seq.next_element::<WrappedListOwned<M>>()? {
            list.push(val.0).map_err(|e| {
                serde::de::Error::custom(format!("failed to push value to list: {:?}.", e))
            })?;
        }

        Ok(list)
    }
}

pub fn deserialize<'de, D, M, N>(
    deserializer: D,
) -> Result<VariableList<VariableList<u8, M>, N>, D::Error>
where
    D: Deserializer<'de>,
    M: Unsigned,
    N: Unsigned,
{
    deserializer.deserialize_seq(Visitor::default())
}
