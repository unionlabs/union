use std::fmt::Debug;

use serde::{de, Deserialize, Deserializer};

pub fn serde_as<'de, D, Src, Dst>(deserializer: D) -> Result<Dst, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    Src::deserialize(deserializer)?
        .try_into()
        .map_err(|e| de::Error::custom(format!("{e:?}")))
}

pub fn serde_as_opt<'de, D, Src, Dst>(deserializer: D) -> Result<Option<Dst>, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    <Option<Src>>::deserialize(deserializer)?
        .map(|src| {
            src.try_into()
                .map_err(|e| de::Error::custom(format!("{e:?}")))
        })
        .transpose()
}

pub fn serde_as_list<'de, D, Src, Dst>(deserializer: D) -> Result<Vec<Dst>, D::Error>
where
    D: Deserializer<'de>,
    Src: Deserialize<'de>,
    Src: TryInto<Dst, Error: Debug>,
{
    <Vec<Src>>::deserialize(deserializer)?
        .into_iter()
        .map(|x| {
            x.try_into()
                .map_err(|e| de::Error::custom(format!("{e:?}")))
        })
        .collect()
}
