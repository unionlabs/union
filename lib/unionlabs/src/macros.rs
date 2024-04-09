macro_rules! hex_string_array_wrapper {
    (
        $(
            pub struct $Struct:ident(pub [u8; $N:expr]);
        )+
    ) => {
        $(
            #[derive(
                Copy,
                Clone,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                ::ssz::Encode,
                ::ssz::Decode,
                ::serde::Serialize,
                ::serde::Deserialize,
                Hash
            )]
            #[ssz(struct_behaviour = "transparent")]
            #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
            pub struct $Struct(#[serde(with = "::serde_utils::hex_string")] pub [u8; $N]);

            impl $Struct {
                pub const BYTES_LEN: usize = $N;

                #[doc = concat!("The [`Display`](core::fmt::Display) impl for [`", stringify!($Struct), "`]")]
                /// prefixes the output with `0x`, which may not be desirable in all contexts.
                /// This fn serves as a convenience around [`hex::encode(&self)`](hex::encode).
                #[must_use]
                pub fn to_string_unprefixed(&self) -> String {
                    hex::encode(&self)
                }
            }

            impl core::str::FromStr for $Struct {
                type Err = serde_utils::FromHexStringError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    serde_utils::parse_hex(s)
                }
            }

            impl Default for $Struct {
                fn default() -> Self {
                    Self([0_u8; $N])
                }
            }

            impl TryFrom<Vec<u8>> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|invalid| crate::errors::InvalidLength {
                            expected: crate::errors::ExpectedLength::Exact($N),
                            found: invalid.len(),
                        })
                }
            }

            impl TryFrom<&[u8]> for $Struct {
                type Error = crate::errors::InvalidLength;

                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    value
                        .try_into()
                        .map(Self)
                        .map_err(|_| crate::errors::InvalidLength {
                            expected: crate::errors::ExpectedLength::Exact($N),
                            found: value.len(),
                        })
                }
            }

            impl From<$Struct> for Vec<u8> {
                fn from(value: $Struct) -> Self {
                    value.0.into()
                }
            }

            impl From<$Struct> for [u8; $N] {
                fn from(value: $Struct) -> Self {
                    value.0
                }
            }

            impl From<[u8; $N]> for $Struct {
                fn from(value: [u8; $N]) -> Self {
                    Self(value)
                }
            }

            impl ::core::fmt::Debug for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}({self})", stringify!($Struct))
                }
            }

            impl ::core::fmt::Display for $Struct {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "0x{}", hex::encode(self.0).as_str())
                }
            }

            // arrays and `FixedVector`s are effectively the exact same type, implement
            // the former in terms of the latter
            impl ::tree_hash::TreeHash for $Struct {
                fn tree_hash_type() -> tree_hash::TreeHashType {
                    ssz_types::FixedVector::<u8, ::typenum::U<$N>>::tree_hash_type()
                }

                fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
                    ssz_types::FixedVector::<u8, ::typenum::U<$N>>::tree_hash_packed_encoding(&self.0.into())
                }

                fn tree_hash_packing_factor() -> usize {
                    ssz_types::FixedVector::<u8, ::typenum::U<$N>>::tree_hash_packing_factor()
                }

                fn tree_hash_root(&self) -> tree_hash::Hash256 {
                    ssz_types::FixedVector::<u8, ::typenum::U<$N>>::tree_hash_root(&self.0.into())
                }
            }

            impl From<::generic_array::GenericArray<u8, ::typenum::U<$N>>> for $Struct {
                fn from(arr: ::generic_array::GenericArray<u8, ::typenum::U<$N>>) -> Self {
                    Self(arr.to_vec().try_into().expect("GenericArray has the correct length; qed;"))
                }
            }

            #[cfg(feature = "ethabi")]
            impl From<$Struct> for ::ethers_core::types::Bytes {
                fn from(value: $Struct) -> Self {
                    ::ethers_core::types::Bytes::from(value.0)
                }
            }

            #[cfg(feature = "ethabi")]
            impl TryFrom<::ethers_core::types::Bytes> for $Struct {
                type Error = <Self as TryFrom<Vec<u8>>>::Error;

                fn try_from(value: ::ethers_core::types::Bytes) -> Result<Self, Self::Error> {
                    Self::try_from(&value.0[..])
                }
            }

            #[cfg(feature = "ethabi")]
            impl TryFrom<&'_ ::ethers_core::types::Bytes> for $Struct {
                type Error = <Self as TryFrom<Vec<u8>>>::Error;

                fn try_from(value: &::ethers_core::types::Bytes) -> Result<Self, Self::Error> {
                    Self::try_from(&value.0[..])
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiType for $Struct {
                fn param_type() -> ::ethers_core::abi::ParamType {
                    ::ethers_core::abi::ParamType::FixedBytes($N)
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiArrayType for $Struct {}

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiEncode for $Struct {
                fn encode(self) -> Vec<u8> {
                    self.0.encode()
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::AbiDecode for $Struct {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ::ethers_core::abi::AbiError> {
                    <[u8; $N]>::decode(bytes).map(Self)
                }
            }

            #[cfg(feature = "ethabi")]
            impl ::ethers_core::abi::Tokenizable for $Struct {
                fn from_token(token: ::ethers_core::abi::Token) -> Result<Self, ::ethers_core::abi::InvalidOutputType> {
                    <[u8; $N]>::from_token(token).map(Self)
                }

                fn into_token(self) -> ::ethers_core::abi::Token {
                    self.0.into_token()
                }
            }

            impl AsRef<[u8]> for $Struct {
                fn as_ref(&self) -> &[u8] {
                    &self.0
                }
            }

            impl ::rlp::Encodable for $Struct {
                fn rlp_append(&self, s: &mut ::rlp::RlpStream) {
                    s.encoder().encode_value(self.as_ref());
                }
            }

            impl ::rlp::Decodable for $Struct {
                fn decode(rlp: &rlp::Rlp) -> Result<Self, ::rlp::DecoderError> {
                    rlp.decoder()
                        .decode_value(|bytes| match bytes.len().cmp(&$N) {
                            ::core::cmp::Ordering::Less => Err(::rlp::DecoderError::RlpIsTooShort),
                            ::core::cmp::Ordering::Greater => Err(::rlp::DecoderError::RlpIsTooBig),
                            ::core::cmp::Ordering::Equal => {
                                Ok($Struct(bytes.try_into().expect("size is checked; qed;")))
                            }
                        })
                }
            }
        )+
    };
}

pub(crate) use hex_string_array_wrapper;

macro_rules! wrapper_enum {
    (
        #[model(proto($Proto:ty))]
        $(#[$meta:meta])*
        // NOTE: Currently there are no ethabi generated enums; use this if/when there are any.
        // $(#[ethabi($EthAbi:ty)])?
        pub enum $Enum:ident {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident = $discriminant:literal,
            )+
        }
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        $(#[$meta])*
        #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
        pub enum $Enum {
            $(
                $(#[$inner_meta])*
                $Variant = $discriminant,
            )+
        }

        mod ensure_enum_values_are_same_as_proto {
            $(
                #[allow(non_upper_case_globals, dead_code)]
                const $Variant: () = assert!(
                    super::$Enum::$Variant as i32 == <$Proto>::$Variant as i32,
                );
            )+
        }

        impl core::str::FromStr for $Enum {
            type Err = crate::errors::UnknownEnumVariant<String>;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if s == <$Proto>::$Variant.as_str_name() {
                        Ok($Enum::$Variant)
                    } else
                )+
                {
                    Err(crate::errors::UnknownEnumVariant(s.to_string()))
                }
            }
        }

        impl From<$Enum> for &'static str {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => <$Proto>::$Variant.as_str_name(),
                    )+
                }
            }
        }

        impl core::fmt::Display for $Enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(<&'static str>::from(*self))
            }
        }

        impl From<$Enum> for u8 {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => $discriminant as u8,
                    )+
                }
            }
        }

        impl From<$Enum> for i32 {
            fn from(value: $Enum) -> Self {
                u8::from(value).into()
            }
        }

        impl TryFrom<u8> for $Enum {
            type Error = crate::errors::UnknownEnumVariant<u8>;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                i32::from(value)
                    .try_into()
                    .map_err(|_| crate::errors::UnknownEnumVariant(value))
            }
        }

        impl TryFrom<i32> for $Enum {
            type Error = crate::errors::UnknownEnumVariant<i32>;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                <$Proto>::try_from(value)
                    .map_err(|_| crate::errors::UnknownEnumVariant(value))
                    .map(Into::into)
            }
        }

        impl From<$Proto> for $Enum {
            fn from(value: $Proto) -> Self {
                match value {
                    $(
                        <$Proto>::$Variant => $Enum::$Variant,
                    )+
                }
            }
        }

        impl From<$Enum> for $Proto {
            fn from(value: $Enum) -> Self {
                match value {
                    $(
                        $Enum::$Variant => Self::$Variant,
                    )+
                }
            }
        }
    };
}

pub(crate) use wrapper_enum;

// Useful in const contexts in place of `?`. Uncomment if you need it!
// macro_rules! option_try {
//     ($expr:expr) => {
//         match $expr {
//             Some(some) => some,
//             None => return None,
//         }
//     };
// }

// pub(crate) use option_try;

macro_rules! result_try {
    ($expr:expr$(, $map_err:expr)?) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => return Err($($map_err)?(err)),
        }
    };
}

pub(crate) use result_try;

// Useful in const contexts in place of `.unwrap()`
#[macro_export]
macro_rules! result_unwrap {
    ($expr:expr) => {{
        // assign to a const here so this can't be called in non-const contexts
        const _: () = match $expr {
            Ok(_) => {}
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        };

        match $expr {
            Ok(ok) => ok,
            Err(err) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }};
}

// Useful in const contexts in place of `.unwrap()`
#[macro_export]
macro_rules! option_unwrap {
    ($expr:expr) => {{
        // assign to a const here so this can't be called in non-const contexts
        const _: () = match $expr {
            Some(_) => {}
            None => panic!("called `Option::unwrap()` on an `None` value"),
        };

        match $expr {
            Some(some) => some,
            None => panic!("called `Option::unwrap()` on an `None` value"),
        }
    }};
}

// Useful in const contexts to promote a value to a const when const promotion fails with "temporary value dropped while borrowed"
#[macro_export]
macro_rules! promote {
    ($ty:ty: $expr:expr) => {{
        const PROMOTED: $ty = $expr;
        PROMOTED
    }};
}

/// Assert some expression with the provided const variables.
///
/// Note that if this is called within a function, then the function must be called in order for the
/// generated constant to be evaluated to cause a compilation error.
///
/// # Example
///
/// ```rust
/// # use unionlabs::const_assert;
///
/// struct Bounded<const MIN: u8, const MAX: u8>(u8);
///
/// impl<const MIN: u8, const MAX: u8> Bounded<MIN, MAX> {
///     fn new(n: u8) -> Option<Self> {
///         const_assert!(MIN: u8, MAX: u8 => MIN < MAX);
///         (MIN..=MAX).contains(&n).then_some(Self(n))
///     }
/// }
///
/// // anywhere that `Bounded::new` is called will fail to compile if MIN >= MAX.
/// ```
#[macro_export]
macro_rules! const_assert {
    ($($list:ident: $ty:ty),* => $expr:expr) => {{
        struct Assert<$(const $list: $ty,)*>;
        impl<$(const $list: $ty,)*> Assert<$($list,)*> {
            const OK: () = assert!($expr);
        }
        let _t = Assert::<$($list,)*>::OK;
    }};
}
