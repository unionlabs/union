use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

pub use aptos_rest_client;
use aptos_rest_client::aptos_api_types::HexEncodedBytes;
pub use aptos_types;
pub use bcs;
pub use move_bindgen_derive::MoveOutputType;
pub use move_core_types;
use move_core_types::account_address::AccountAddress;
pub use serde;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use serde_json;
pub use tracing;

/// Types that can either be returned from `#[view]` functions or read from storage.
pub trait MoveOutputType {
    type Raw: DeserializeOwned;

    fn from_raw(raw: Self::Raw) -> Self;

    fn into_raw(self) -> Self::Raw;
}

macro_rules! impl_param_and_output {
    (
        $T:ty,
        $raw:ty,
        |$from_raw_param:ident| $from_raw:expr,
        |$into_raw_param:ident| $into_raw:expr
    ) => {
        impl MoveOutputType for $T {
            type Raw = $raw;

            fn from_raw($from_raw_param: Self::Raw) -> Self {
                $from_raw
            }

            fn into_raw(self) -> Self::Raw {
                let $into_raw_param = self;
                $into_raw
            }
        }
    };
}

impl_param_and_output!(bool, bool, |raw| raw, |this| this);
impl_param_and_output!(u8, u8, |raw| raw, |this| this);
impl_param_and_output!(u16, u16, |raw| raw, |this| this);
impl_param_and_output!(u32, u32, |raw| raw, |this| this);
impl_param_and_output!(
    u64,
    aptos_rest_client::aptos_api_types::U64,
    |raw| raw.0,
    |this| this.into()
);
impl_param_and_output!(
    u128,
    aptos_rest_client::aptos_api_types::U128,
    |raw| raw.0,
    |this| this.into()
);
impl_param_and_output!(String, String, |raw| raw, |this| this);
impl_param_and_output!(
    aptos_rest_client::aptos_api_types::Address,
    aptos_rest_client::aptos_api_types::Address,
    |raw| raw,
    |this| this
);

// TODO: impl this in unionlabs
// impl TypeTagged for U256 {
//     fn type_tag() -> TypeTag {
//         TypeTag::U256
//     }
// }

impl<T: MoveOutputType + 'static> MoveOutputType for Vec<T> {
    type Raw = RawVec<T::Raw>;

    fn from_raw(raw: Self::Raw) -> Self {
        raw.0.into_iter().map(T::from_raw).collect()
    }

    fn into_raw(self) -> Self::Raw {
        RawVec(self.into_iter().map(T::into_raw).collect())
    }
}

#[derive(Debug, Clone)]
pub struct RawVec<T>(pub Vec<T>);

impl<T: Serialize + 'static> Serialize for RawVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match (&self.0 as &dyn Any).downcast_ref::<Vec<u8>>() {
            Some(vec_u8) => {
                if serializer.is_human_readable() {
                    // TODO: Figure out a way to not clone here
                    aptos_rest_client::aptos_api_types::HexEncodedBytes(vec_u8.to_vec())
                        .serialize(serializer)
                } else {
                    self.0.serialize(serializer)
                }
            }
            None => self.0.serialize(serializer),
        }
    }
}

// TODO: Figure out a way to not clone here
impl<'de, T: Deserialize<'de> + 'static> Deserialize<'de> for RawVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn is_u8<T: ?Sized + Any>() -> bool {
            TypeId::of::<u8>() == TypeId::of::<T>()
        }

        if is_u8::<T>() {
            aptos_rest_client::aptos_api_types::HexEncodedBytes::deserialize(deserializer)
                .map(|v| unsafe { std::mem::transmute::<Vec<u8>, Vec<T>>(v.0) })
                .map(Self)
        } else {
            <Vec<T>>::deserialize(deserializer).map(Self)
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MoveOption<T> {
    // TODO: Add verification here that this contains at most one item, maybe through a custom `SingleItemVec`?
    pub vec: Vec<T>,
}

impl<T: MoveOutputType> MoveOutputType for Option<T> {
    type Raw = MoveOption<T::Raw>;

    fn from_raw(mut raw: Self::Raw) -> Self {
        raw.vec.pop().map(T::from_raw)
    }

    fn into_raw(self) -> Self::Raw {
        Self::Raw {
            vec: self.into_iter().map(T::into_raw).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeInfo {
    pub account_address: AccountAddress,
    pub module_name: Vec<u8>,
    pub struct_name: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTypeInfo {
    pub account_address: AccountAddress,
    pub module_name: HexEncodedBytes,
    pub struct_name: HexEncodedBytes,
}

impl MoveOutputType for TypeInfo {
    type Raw = RawTypeInfo;

    fn from_raw(raw: Self::Raw) -> Self {
        Self {
            account_address: raw.account_address,
            module_name: raw.module_name.0,
            struct_name: raw.struct_name.0,
        }
    }

    fn into_raw(self) -> Self::Raw {
        Self::Raw {
            account_address: self.account_address,
            module_name: HexEncodedBytes(self.module_name),
            struct_name: HexEncodedBytes(self.struct_name),
        }
    }
}

pub mod fungible_asset {
    use serde::{Deserialize, Serialize};

    use crate::MoveOutputType;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Metadata {
        name: String,
        symbol: String,
        decimals: u8,
        icon_uri: String,
        project_uri: String,
    }

    impl MoveOutputType for Metadata {
        type Raw = Metadata;

        fn from_raw(raw: Self::Raw) -> Self {
            raw
        }

        fn into_raw(self) -> Self::Raw {
            self
        }
    }
}

pub mod object {
    use std::marker::PhantomData;

    use move_core_types::account_address::AccountAddress;
    use serde::{Deserialize, Serialize};

    use crate::MoveOutputType;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Object<T> {
        inner: AccountAddress,
        #[serde(skip)]
        __marker: PhantomData<fn() -> T>,
    }

    impl<T> MoveOutputType for Object<T> {
        type Raw = Object<T>;

        fn from_raw(raw: Self::Raw) -> Self {
            raw
        }

        fn into_raw(self) -> Self::Raw {
            self
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SimpleMap<K, V> {
    pub data: Vec<SimpleMapEntry<K, V>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SimpleMapEntry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Ord + MoveOutputType, V: MoveOutputType> MoveOutputType for BTreeMap<K, V> {
    type Raw = SimpleMap<K::Raw, V::Raw>;

    fn from_raw(raw: Self::Raw) -> Self {
        raw.data
            .into_iter()
            .map(|entry| (K::from_raw(entry.key), V::from_raw(entry.value)))
            .collect()
    }

    fn into_raw(self) -> Self::Raw {
        Self::Raw {
            data: self
                .into_iter()
                .map(|(key, value)| SimpleMapEntry {
                    key: key.into_raw(),
                    value: value.into_raw(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveStruct<T> {
    pub address: AccountAddress,
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    type RawOf<T> = <T as MoveOutputType>::Raw;

    #[test]
    fn raw_vec_u8() {
        let json = r#""0x010203""#;

        let de = <Vec<u8>>::from_raw(serde_json::from_str::<RawOf<Vec<u8>>>(json).unwrap());

        assert_eq!(de, [1, 2, 3].to_vec());

        let ser = serde_json::to_string(&de.into_raw()).unwrap();

        assert_eq!(json, ser);
    }

    #[test]
    fn raw_vec_u64() {
        let json = r#"["1","2","3"]"#;

        let de = <Vec<u64>>::from_raw(serde_json::from_str::<RawOf<Vec<u64>>>(json).unwrap());

        assert_eq!(de, [1, 2, 3]);

        let ser = serde_json::to_string(&de.into_raw()).unwrap();

        assert_eq!(json, ser);
    }

    #[test]
    fn raw_vec_bool() {
        let json = r#"[true,false,true]"#;

        let de = <Vec<bool>>::from_raw(serde_json::from_str::<RawOf<Vec<bool>>>(json).unwrap());

        assert_eq!(de, [true, false, true]);

        let ser = serde_json::to_string(&de.into_raw()).unwrap();

        assert_eq!(json, ser);
    }

    #[test]
    fn raw_vec_vec_u8() {
        let json = r#"["0x010203","0x010203","0x010203"]"#;

        let de =
            <Vec<Vec<u8>>>::from_raw(serde_json::from_str::<RawOf<Vec<Vec<u8>>>>(json).unwrap());

        assert_eq!(de, [[1, 2, 3], [1, 2, 3], [1, 2, 3]]);

        let ser = serde_json::to_string(&de.into_raw()).unwrap();

        assert_eq!(json, ser);
    }
}
