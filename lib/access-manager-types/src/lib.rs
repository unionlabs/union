use core::fmt;
use std::{borrow::Cow, ptr};

use cosmwasm_std::{StdError, StdResult};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::value::RawValue;
use unionlabs_primitives::Bytes;

use crate::time::Delay;

pub mod time;

pub mod managed;
pub mod manager;

#[cfg(doc)]
use crate::manager::msg::QueryMsg;

/// Structure that stores the details for a target contract.
///
/// ```solidity
/// struct TargetConfig {
///     mapping(bytes4 selector => uint64 roleId) allowedRoles;
///     Time.Delay adminDelay;
///     bool closed;
/// }
/// ```
///
/// Note that the `allowedRoles` field of the original solidity struct is implemented via the
/// `TargetAllowedRoles` store.
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct TargetConfig {
    pub admin_delay: Delay,
    pub closed: bool,
}

/// Structure that stores the details for a role/account pair.
///
/// ```solidity
/// struct Access {
///     uint48 since;
///     Time.Delay delay;
/// }
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Access {
    /// Timepoint at which the user gets the permission.
    ///
    /// If this is either 0 or in the future, then the role permission is not available.
    pub since: u64,
    /// Delay for execution. Only applies to `restricted()`/`execute()` calls.
    pub delay: Delay,
}

/// Structure that stores the details of a role.
///
/// ```solidity
/// struct Role {
///     mapping(address user => Access access) members;
///     uint64 admin;
///     uint64 guardian;
///     Time.Delay grantDelay;
/// }
/// ```
///
/// Note that the `members` field of the original solidity struct is implemented via the
/// `RoleMembers` store.
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Role {
    /// Admin who can grant or revoke permissions.
    pub admin: RoleId,
    /// Guardian who can cancel operations targeting functions that need this role.
    pub guardian: RoleId,
    /// Delay in which the role takes effect after being granted.
    pub grant_delay: Delay,
}

/// Structure that stores the details for a scheduled operation.
///
/// ```solidity
/// struct Schedule {
///     uint48 timepoint;
///     uint32 nonce;
/// }
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Schedule {
    /// Moment at which the operation can be executed.
    pub timepoint: u64,
    /// Operation nonce to allow third-party contracts to identify the operation.
    // TODO: Newtype
    pub nonce: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct RoleId(#[serde(with = "::serde_utils::string")] u64);

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl RoleId {
    /// See [`QueryMsg::AdminRole`].
    pub const ADMIN_ROLE: Self = Self(u64::MIN);

    /// See [`QueryMsg::PublicRole`].
    pub const PUBLIC_ROLE: Self = Self(u64::MAX);

    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(raw: [u8; 8]) -> StdResult<Self> {
        Ok(RoleId(u64::from_be_bytes(raw)))
    }

    pub fn try_from_be_bytes(raw: &Bytes) -> StdResult<Self> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 8 bytes, found {}: {raw}",
                    raw.len(),
                ))
            })
            .and_then(RoleId::from_be_bytes)
    }
}

#[derive(Debug, PartialEq, bincode::Encode)]
#[repr(transparent)]
pub struct Selector(str);

impl ToOwned for Selector {
    type Owned = Box<Self>;

    fn to_owned(&self) -> Self::Owned {
        unsafe { Box::from_raw(Box::into_raw(self.0.to_string().into_boxed_str()) as *mut Self) }
    }
}

impl Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Box<Selector> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Box<str>>::deserialize(deserializer).map(Selector::new_owned)
    }
}

impl<'de> Deserialize<'de> for &'de Selector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'de str>::deserialize(deserializer).map(Selector::new)
    }
}

impl<Context> bincode::Decode<Context> for Box<Selector> {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        <Box<str> as bincode::Decode<Context>>::decode(decoder).map(Selector::new_owned)
    }
}

impl Selector {
    pub fn new(selector: &str) -> &Self {
        unsafe { &*(ptr::from_ref::<str>(selector) as *const Self) }
    }

    pub fn new_owned(selector: Box<str>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(selector) as *mut Self) }
    }

    pub fn extract_from_data(data: &str) -> Result<&Self, serde_json::Error> {
        struct ExtractSelector;

        impl<'de> Visitor<'de> for ExtractSelector {
            type Value = &'de Selector;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "json object with single top level key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                match map.next_entry::<&'de str, &'de RawValue>()? {
                    Some((selector, _)) => {
                        if map.next_key::<Cow<'de, str>>()?.is_some() {
                            Err(<A::Error as de::Error>::custom("multiple keys found"))
                        } else {
                            Ok(Selector::new(selector))
                        }
                    }
                    None => Err(<A::Error as de::Error>::custom("no key found")),
                }
            }
        }

        serde_json::de::Deserializer::from_str(data).deserialize_map(ExtractSelector)
    }

    pub fn extract_from_serialize(t: &impl Serialize) -> &'static Selector {
        struct ExtractSelector(Option<&'static Selector>);

        impl ExtractSelector {
            fn selector_or_error(&self) -> Result<&'static Selector, Error> {
                self.0.ok_or_else(|| Error("no selector".to_owned()))
            }
        }

        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        struct Error(String);

        impl serde::ser::Error for Error {
            fn custom<T>(msg: T) -> Self
            where
                T: std::fmt::Display,
            {
                Self(msg.to_string())
            }
        }

        macro_rules! simple_delegate {
            ($(fn $ident:ident(self $(, $arg:ident: $ty:ty)*);)*) => {
                $(
                    fn $ident(self $(, _:$ty)*) -> Result<Self::Ok, Self::Error> {
                        self.selector_or_error()
                        // Err(Error(concat!("unsupported: ", stringify!($ident)).to_string()))
                    }
                )*
            };
        }

        impl Serializer for ExtractSelector {
            type Ok = &'static Selector;

            type Error = Error;
            type SerializeSeq = Self;
            type SerializeTuple = Self;
            type SerializeTupleStruct = Self;
            type SerializeTupleVariant = Self;
            type SerializeMap = Self;
            type SerializeStruct = Self;
            type SerializeStructVariant = Self;

            simple_delegate! {
                fn serialize_bool(self, v: bool);
                fn serialize_i8(self, v: i8);
                fn serialize_i16(self, v: i16);
                fn serialize_i32(self, v: i32);
                fn serialize_i64(self, v: i64);
                fn serialize_u8(self, v: u8);
                fn serialize_u16(self, v: u16);
                fn serialize_u32(self, v: u32);
                fn serialize_u64(self, v: u64);
                fn serialize_f32(self, v: f32);
                fn serialize_f64(self, v: f64);
                fn serialize_char(self, v: char);
                fn serialize_str(self, v: &str);
                fn serialize_bytes(self, v: &[u8]);
                fn serialize_none(self);
                fn serialize_unit(self);
                fn serialize_unit_struct(self, name: &'static str);
                // we don't extract the variant from this method since unit variants serialize as "variant" in json, not {"variant":...}, so it is unsupported
                fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str);
            }

            fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + Serialize,
            {
                self.selector_or_error()
            }

            fn serialize_newtype_struct<T>(
                self,
                _: &'static str,
                _: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + Serialize,
            {
                self.selector_or_error()
            }

            fn serialize_newtype_variant<T>(
                self,
                _: &'static str,
                _: u32,
                variant: &'static str,
                _: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(Selector::new(variant))
            }

            fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
                Ok(self)
            }

            fn serialize_tuple_struct(
                self,
                _: &'static str,
                _: usize,
            ) -> Result<Self::SerializeTupleStruct, Self::Error> {
                Ok(self)
            }

            fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
                Ok(self)
            }

            fn serialize_tuple_variant(
                self,
                _: &'static str,
                _: u32,
                variant: &'static str,
                _: usize,
            ) -> Result<Self::SerializeTupleVariant, Self::Error> {
                Ok(Self(Some(Selector::new(variant))))
            }

            fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
                Ok(self)
            }

            fn serialize_struct(
                self,
                _: &'static str,
                _: usize,
            ) -> Result<Self::SerializeStruct, Self::Error> {
                Ok(self)
            }

            fn serialize_struct_variant(
                self,
                _: &'static str,
                _: u32,
                variant: &'static str,
                _: usize,
            ) -> Result<Self::SerializeStructVariant, Self::Error> {
                Ok(Self(Some(Selector::new(variant))))
            }
        }

        impl SerializeSeq for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_element<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeMap for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_key<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn serialize_value<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeStruct for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeTuple for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_element<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeTupleStruct for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_field<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeTupleVariant for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_field<T>(&mut self, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        impl SerializeStructVariant for ExtractSelector {
            type Ok = &'static Selector;
            type Error = Error;

            fn serialize_field<T>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                Ok(())
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.selector_or_error()
            }
        }

        t.serialize(ExtractSelector(None)).unwrap()
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanCall {
    // whether or not the caller can call the target *right now*, i.e. without any delay
    pub allowed: bool,
    pub delay: u32,
}

// TODO: Make this an enum, execution_delay is only ever meaningful when is_member is true
// (i think)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HasRole {
    pub is_member: bool,
    pub execution_delay: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FullAccess {
    /// Timestamp at which the account membership becomes valid. 0 means role is not granted.
    pub since: u64,
    /// Current execution delay for the account.
    pub current_delay: u32,
    /// Pending execution delay for the account.
    pub pending_delay: u32,
    /// Timestamp at which the pending execution delay will become active. 0 means no delay update
    /// is scheduled.
    pub effect: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Msg {
        Unit,
        EmptyStruct {},
        Struct {
            field: u8,
        },
        EmptyTuple(),
        Tuple(u8, u8),
        NewType(NewType),
        #[serde(untagged)]
        Untagged(InnerMsg),
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct NewType {
        inner: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[expect(clippy::enum_variant_names, reason = "intentional")]
    enum InnerMsg {
        InnerUnit,
        InnerEmptyStruct {},
        InnerStruct { field: u8 },
        InnerEmptyTuple(),
        InnerTuple(u8, u8),
        InnerNewType(NewType),
    }

    #[test]
    fn selector_extract_from_serialize() {
        assert_eq!(
            Selector::extract_from_serialize(&Msg::EmptyStruct {}),
            Selector::new("empty_struct")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Struct { field: 1 }),
            Selector::new("struct")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::EmptyTuple()),
            Selector::new("empty_tuple")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Tuple(1, 2)),
            Selector::new("tuple")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Untagged(InnerMsg::InnerEmptyStruct {})),
            Selector::new("inner_empty_struct")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Untagged(InnerMsg::InnerStruct { field: 1 })),
            Selector::new("inner_struct")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Untagged(InnerMsg::InnerEmptyTuple())),
            Selector::new("inner_empty_tuple")
        );

        assert_eq!(
            Selector::extract_from_serialize(&Msg::Untagged(InnerMsg::InnerTuple(1, 2))),
            Selector::new("inner_tuple")
        );
    }

    #[test]
    fn selector_extract_from_data() {
        fn json(t: &impl Serialize) -> String {
            serde_json_wasm::to_string(t).unwrap()
        }

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::EmptyStruct {})).unwrap(),
            Selector::new("empty_struct")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Struct { field: 1 })).unwrap(),
            Selector::new("struct")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::EmptyTuple())).unwrap(),
            Selector::new("empty_tuple")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Tuple(1, 2))).unwrap(),
            Selector::new("tuple")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Untagged(InnerMsg::InnerEmptyStruct {})))
                .unwrap(),
            Selector::new("inner_empty_struct")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Untagged(InnerMsg::InnerStruct { field: 1 })))
                .unwrap(),
            Selector::new("inner_struct")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Untagged(InnerMsg::InnerEmptyTuple())))
                .unwrap(),
            Selector::new("inner_empty_tuple")
        );

        assert_eq!(
            Selector::extract_from_data(&json(&Msg::Untagged(InnerMsg::InnerTuple(1, 2)))).unwrap(),
            Selector::new("inner_tuple")
        );
    }
}
