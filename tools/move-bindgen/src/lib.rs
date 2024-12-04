use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

pub use aptos_rest_client;
pub use aptos_types;
pub use bcs;
pub use move_bindgen_derive::MoveOutputType;
pub use move_core_types;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    language_storage::{StructTag, TypeTag},
};
pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
pub use tracing;

/// Types that can be used as inputs to a move function.
pub trait MoveParamType {
    fn type_tag() -> TypeTag;
}

/// Types that can either be returned from #[view] functions or read from storage.
pub trait MoveOutputType {
    type Raw;

    fn from_raw(raw: Self::Raw) -> Self;

    fn into_raw(self) -> Self::Raw;
}

macro_rules! impl_param_and_output {
    (
        $T:ty,
        $type_tag:expr,
        $raw:ty,
        |$from_raw_param:ident| $from_raw:expr,
        |$into_raw_param:ident| $into_raw:expr
    ) => {
        impl MoveParamType for $T {
            fn type_tag() -> TypeTag {
                $type_tag
            }
        }

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

impl_param_and_output!(bool, TypeTag::Bool, bool, |raw| raw, |this| this);
impl_param_and_output!(u8, TypeTag::U8, u8, |raw| raw, |this| this);
impl_param_and_output!(u16, TypeTag::U16, u16, |raw| raw, |this| this);
impl_param_and_output!(u32, TypeTag::U32, u32, |raw| raw, |this| this);
impl_param_and_output!(
    u64,
    TypeTag::U64,
    aptos_rest_client::aptos_api_types::U64,
    |raw| raw.0,
    |this| this.into()
);
impl_param_and_output!(
    u128,
    TypeTag::U128,
    aptos_rest_client::aptos_api_types::U128,
    |raw| raw.0,
    |this| this.into()
);
impl_param_and_output!(
    String,
    TypeTag::Struct(Box::new(StructTag {
        address: AccountAddress::ONE,
        module: ident_str!("string").into(),
        name: ident_str!("String").into(),
        type_args: vec![],
    })),
    String,
    |raw| raw,
    |this| this
);
impl_param_and_output!(
    aptos_rest_client::aptos_api_types::Address,
    TypeTag::Address,
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

impl<T: MoveParamType> MoveParamType for Vec<T> {
    fn type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(T::type_tag()))
    }
}

impl<T: MoveOutputType> MoveOutputType for Vec<T> {
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
                // TODO: Figure out a way to not clone here
                aptos_rest_client::aptos_api_types::HexEncodedBytes(vec_u8.to_vec())
                    .serialize(serializer)
            }
            None => self.0.serialize(serializer),
        }
    }
}

// TODO: Figure out a way to not clone here
impl<'de, T: Clone + Deserialize<'de> + 'static> Deserialize<'de> for RawVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn is_u8<T: ?Sized + Any>() -> bool {
            TypeId::of::<u8>() == TypeId::of::<T>()
        }

        if is_u8::<T>() {
            aptos_rest_client::aptos_api_types::HexEncodedBytes::deserialize(deserializer)
                // TODO: Figure out a way to not clone here
                .map(|v| {
                    (&v.0 as &dyn Any)
                        .downcast_ref::<Vec<T>>()
                        .expect("T is u8; qed;")
                        .clone()
                })
                .map(Self)
        } else {
            <Vec<T>>::deserialize(deserializer).map(Self)
        }
    }
}

impl MoveParamType for aptos_rest_client::aptos_api_types::HexEncodedBytes {
    fn type_tag() -> TypeTag {
        TypeTag::Vector(Box::new(TypeTag::U8))
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

impl<T: MoveParamType> MoveParamType for MoveStruct<T> {
    fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: todo!(),
            module: todo!(),
            name: todo!(),
            type_args: todo!(),
        }))
    }
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
